use std::fs;
use std::cmp::max;
use std::io::Cursor;
use std::time::Duration;
use std::time::Instant;
use std::env;
use walkdir::WalkDir;
use image::ImageReader;
use image::ColorType;

use image_codecs::DecodingError;
use image_codecs::Image;

#[test]
fn test_png_decoder() {
    let root = env::var("TEST_IMAGES_ROOT").expect("TEST_IMAGES_ROOT env var not found");
    let pngs = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry|
            entry.path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
        ).map(walkdir::DirEntry::into_path);

    let mut n_bad = 0;
    let mut n_ok = 0;
    let mut n_skipped = 0;
    let mut total = Duration::default();
    let mut total_ref = Duration::default();
    let mut total_bytes_decoded = 0;
    let mut total_pixels_decoded = 0;
    for path in pngs {
        // if path.file_name().unwrap().display().to_string() != "tm3n3p02.png" {
        //     n_skipped += 1;
        //     continue;
        // }
        let should_fail = path.to_string_lossy().to_lowercase().contains("pngsuite")
            && path.file_name().unwrap().to_string_lossy().starts_with('x');
            // && !path.file_name().unwrap().to_string_lossy().starts_with("xcs"); // TODO: handle invalid crc
            // && !path.file_name().unwrap().to_string_lossy().starts_with("xhd"); // TODO: handle invalid crc
        println!("decoding {}", path.display());
        match fs::read(path) {
            Err(e) => {
                println!("Can't read file: {}", e.to_string());
                n_skipped += 1;
            }
            Ok(bytes) => {
                let start = Instant::now();
                let image = Image::new(&bytes); 
                let elapsed = start.elapsed();
                match image {
                    Err(DecodingError::NotImplemented) => {
                        println!("Decoding failed: not implemented. Skipping");
                        n_skipped += 1;
                    },
                    Err(err) => if !should_fail { panic!("Decoding failed: {:?}", err) },
                    Ok(image) => {
                        total += elapsed;
                        total_bytes_decoded += image.buf.len();
                        total_pixels_decoded += image.w as usize * image.h as usize;
                        if should_fail {
                            panic!("Should have failed");
                        }
                        // println!("decoded: w={}, h={}, channels={}", image.w, image.h, image.channels);
                        assert!(image.channels > 0);
                        let sz = image.w as usize * image.h as usize * image.channels as usize * max((image.depth / 8) as usize, 1);
                        assert_eq!(image.buf.len(), sz);

                        let mut image_reader = ImageReader::new(Cursor::new(bytes));
                        image_reader.set_format(image::ImageFormat::Png);
                        let start = Instant::now();
                        match image_reader.decode() {
                            Err(err) => {
                                println!("Reference decoder failed: {err}");
                                n_skipped += 1;
                            },
                            Ok(img) => {
                                let vec16: Vec<u16>;
                                let col = img.color();
                                let expected_bytes =
                                    match col {
                                        ColorType::L8 => &img.to_luma8().into_raw(),
                                        ColorType::La8 => &img.to_luma_alpha8().into_raw(),
                                        ColorType::Rgb8 => &img.to_rgb8().into_raw(),
                                        ColorType::Rgba8 => &img.to_rgba8().into_raw(),
                                        ColorType::L16 | ColorType::La16 | ColorType::Rgb16 | ColorType::Rgba16 => {
                                            match col {
                                                ColorType::L16 => vec16 = img.to_luma16().into_raw(),
                                                ColorType::La16 => vec16 = img.to_luma_alpha16().into_raw(),
                                                ColorType::Rgb16 => vec16 = img.to_rgb16().into_raw(),
                                                ColorType::Rgba16 => vec16 = img.to_rgba16().into_raw(),
                                                _ => panic!("unreachable"),
                                            }
                                            unsafe { vec16.align_to::<u8>().1 }
                                        }
                                        _ => panic!("Unexpected ColorType: {:?}", img.color())
                                    };
                                let elapsed = start.elapsed();
                                total_ref += elapsed;
                                assert!(image.buf.len() == expected_bytes.len());

                                for i in 0..image.buf.len() {
                                    if image.buf[i] != expected_bytes[i] {
                                        println!("our[{i}] != expected[{i}]: {} != {}", image.buf[i], expected_bytes[i]);
                                        break;
                                    }
                                }
                                // assert!(image.buf == expected_bytes); // assert_eq prints entire vectors on failure
                                if image.buf != expected_bytes {
                                    println!("BAD: Comparison failed");
                                    n_bad += 1;
                                } else {
                                    // println!("OK");
                                    n_ok += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let rel_perf = total.as_nanos() * 100 / total_ref.as_nanos();
    let mb_per_sec = total_bytes_decoded as u64 / (1024 * 1024) / total.as_secs();
    let mpix_per_sec = total_pixels_decoded as u64 / 1_000_000 / total.as_secs();
    println!("ok: {n_ok}, bad: {n_bad}, skipped: {n_skipped}");
    println!("total: {:?}, total_ref: {:?} ({rel_perf}%), {mb_per_sec} Mb/s, {mpix_per_sec} MP/s", total, total_ref);
    assert_ne!(n_ok, 0);
    assert_eq!(n_bad, 0);
    assert_eq!(n_skipped, 0);
}
