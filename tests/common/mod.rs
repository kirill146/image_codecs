use image::ImageFormat;
use png::BitDepth;
use png::Transformations;
use walkdir::WalkDir;
use image::ImageReader;
use image::ColorType;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use std::io::Cursor;
use std::env;
use std::fs;

use image_codecs::DecodingError;
use image_codecs::Image;

pub fn read_all_images(root: &str, extension: &str) -> Vec<(PathBuf, Vec<u8>)> {
    WalkDir::new(root)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|entry|
        entry.path()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension)))
    .filter_map(|entry| {
        let path = entry.into_path();
        fs::read(&path).ok().map(|buf| (path, buf))
    })
    .collect()
}

#[allow(dead_code)]
fn reinterpret(v: Vec<u16>) -> Vec<u8> {
    let len = v.len() * 2;
    let cap = v.capacity() * 2;
    let ptr = v.as_ptr() as *mut u8;

    std::mem::forget(v);

    unsafe { Vec::from_raw_parts(ptr, len, cap) }
}

#[allow(dead_code)]
fn decode_with_image_rs(bytes: &Vec<u8>, image_type: &str) -> Result<Vec<u8>, image::error::ImageError> {
    let mut image_reader = ImageReader::new(Cursor::new(bytes));
    let image_reader =
        if image_type == "tga" {
            image_reader.set_format(ImageFormat::Tga);
            image_reader
        } else {
            image_reader.with_guessed_format().expect("Reference decoder failed, can't guess format")
        };

    let img = image_reader.decode()?;
    let col = img.color();
    let vec16: Vec<u16>;
    let decoded_bytes =
        match col {
            ColorType::L8 => img.to_luma8().into_raw(),
            ColorType::La8 => img.to_luma_alpha8().into_raw(),
            ColorType::Rgb8 => img.to_rgb8().into_raw(),
            ColorType::Rgba8 => img.to_rgba8().into_raw(),
            ColorType::L16 | ColorType::La16 | ColorType::Rgb16 | ColorType::Rgba16 => {
                match col {
                    ColorType::L16 => vec16 = img.to_luma16().into_raw(),
                    ColorType::La16 => vec16 = img.to_luma_alpha16().into_raw(),
                    ColorType::Rgb16 => vec16 = img.to_rgb16().into_raw(),
                    ColorType::Rgba16 => vec16 = img.to_rgba16().into_raw(),
                    _ => panic!("unreachable"),
                }
                reinterpret(vec16)
            }
            _ => panic!("Unexpected ColorType: {:?}", img.color())
        };

    Ok(decoded_bytes)
}

fn decode_with_png_rs(bytes: &Vec<u8>) -> Result<Vec<u8>, png::DecodingError> {
    let mut options = png::DecodeOptions::default();
    options.set_ignore_checksums(false);
    options.set_ignore_iccp_chunk(false);
    options.set_ignore_text_chunk(false);

    let mut decoder = png::Decoder::new_with_options(Cursor::new(bytes), options);
    decoder.set_transformations(Transformations::EXPAND);
    let mut reader = decoder.read_info()?;

    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf)?; // actual decoding
    assert_eq!(info.buffer_size(), buf.len());

    if info.bit_depth == BitDepth::Sixteen {
        // PNGs are big endian, so let's fix it:
        for i in (0..buf.len()).step_by(2) {
            buf.swap(i, i + 1);
        }
    }

    Ok(buf)
}

pub fn test_decoder(image_type: &str) {
    let root = env::var("TEST_IMAGES_ROOT").expect("TEST_IMAGES_ROOT env var not found");
    println!("Scanning {root} for {image_type}s");
    let imgs = read_all_images(&root, image_type);
    assert_ne!(imgs.len(), 0);

    let mut n_bad = 0;
    let mut n_ok = 0;
    let mut n_skipped = 0;
    let mut total_time = Duration::default();
    let mut total_time_ref = Duration::default();
    let mut total_bytes_decoded = 0;
    let mut total_pixels_decoded = 0;
    for (path, bytes) in imgs {
        // if path.file_name().unwrap().display().to_string() != "WaterBottle_Occlusion.png" {
        //     n_skipped += 1;
        //     continue;
        // }
        let should_fail = path.to_string_lossy().to_lowercase().contains("pngsuite")
            && path.file_name().unwrap().to_string_lossy().starts_with('x');
        println!("decoding {}", path.strip_prefix(&root).unwrap().display());
        let start = Instant::now();
        let image = Image::new(&bytes); 
        let elapsed = start.elapsed();
        let image = match image {
            Err(DecodingError::NotImplemented) => {
                println!("Decoding failed: not implemented. Skipping");
                n_skipped += 1;
                continue;
            },
            Err(err) => {
                if !should_fail {
                    panic!("Decoding failed: {:?}", err)
                }
                continue;
            },
            Ok(image) => image,
        };
        total_time += elapsed;
        total_bytes_decoded += image.buf.len();
        total_pixels_decoded += image.w as usize * image.h as usize;
        if should_fail {
            panic!("Should have failed");
        }
        // println!("decoded: w={}, h={}, channels={}", image.w, image.h, image.channels);
        assert!(image.channels > 0);
        let sz = image.w as usize * image.h as usize * image.channels as usize * image.depth as usize / 8;
        assert_eq!(image.buf.len(), sz);

        let start = Instant::now();
        let expected_buf = match decode_with_png_rs(&bytes) {
            Err(err) => {
                println!("Reference decoder failed: {err}, skipping");
                n_skipped += 1;
                continue;
            },
            Ok(expected_buf) => expected_buf
        };
        let elapsed = start.elapsed();
        total_time_ref += elapsed;

        assert_eq!(image.buf.len(), expected_buf.len());
        for i in 0..image.buf.len() {
            if image.buf[i] != expected_buf[i] {
                println!("our[{i}] != expected[{i}]: {} != {}", image.buf[i], expected_buf[i]);
                break;
            }
        }
        // assert!(image.buf == expected_bytes); // assert_eq prints entire vectors on failure

        if image.buf != expected_buf {
            println!("BAD: Comparison failed");
            n_bad += 1;
        } else {
            // println!("OK");
            n_ok += 1;
        }
    }
    let rel_perf = total_time.as_nanos() * 100 / total_time_ref.as_nanos();
    let mb_per_sec = total_bytes_decoded as u64 * 1_000_000_000 / (1024 * 1024 * total_time.as_nanos() as u64);
    let mpix_per_sec = total_pixels_decoded as u64 * 1000 / (total_time.as_nanos() as u64);
    println!("ok: {n_ok}, bad: {n_bad}, skipped: {n_skipped}");
    println!("total: {:?}, total_ref: {:?} ({rel_perf}%), {mb_per_sec} Mb/s, {mpix_per_sec} MP/s", total_time, total_time_ref);
    assert_ne!(n_ok, 0);
    assert_eq!(n_bad, 0);
    assert_eq!(n_skipped, 0);
}
