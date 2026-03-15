
use crate::DecodingError;
use crate::Image;

pub struct TGAImage {
    pub image: Image,
    _image_type: u8,
    color_map_len: u16,
    color_map_entry_size: u8,
}

struct TGADatastream<'a> {
    buf: &'a [u8],
    cursor: usize,
}

impl<'a> TGADatastream<'a> {
    fn new(buf: &'a [u8]) -> TGADatastream<'a> {
        TGADatastream {
            buf,
            cursor: 0,
        }
    }

    fn read_u8(&mut self) -> Result<u8, DecodingError> {
        if self.cursor + 1 <= self.buf.len() {
            let val = self.buf[self.cursor];
            self.cursor += 1;
            Ok(val)
        } else {
            Err(DecodingError::MalformedImage)
        }
    }

    fn read_u16(&mut self) -> Result<u16, DecodingError> {
        if self.cursor + 2 <= self.buf.len() {
            let val = u16::from_le_bytes([
                self.buf[self.cursor], self.buf[self.cursor + 1]
            ]);
            self.cursor += 2;
            Ok(val)
        } else {
            Err(DecodingError::MalformedImage)
        }
    }

    fn read_u32(&mut self) -> Result<u32, DecodingError> {
        if self.cursor + 4 <= self.buf.len() {
            let val = u32::from_be_bytes([
                self.buf[self.cursor], self.buf[self.cursor + 1], self.buf[self.cursor + 2], self.buf[self.cursor + 3]
            ]);
            self.cursor += 4;
            Ok(val)
        } else {
            Err(DecodingError::MalformedImage)
        }
    }

    fn skip(&mut self, count: usize) -> Result<(), DecodingError> {
        if self.cursor + count <= self.buf.len() {
            self.cursor += count;
            Ok(())
        } else {
            Err(DecodingError::MalformedImage)
        }
    }
}

pub fn is_tga(_buf: &[u8]) -> bool {
    true // TODO
}

fn decode_header(stream: &mut TGADatastream) -> Result<TGAImage, DecodingError> {
    let id_len = stream.read_u8()?;
    if id_len != 0 {
        println!("Non-empty identification field");
    }

    let color_map_type = stream.read_u8()?;
    if color_map_type != 0 && color_map_type != 1 {
        println!("color map type: {color_map_type}");
        return Err(DecodingError::MalformedImage);
    }
    if color_map_type != 0 {
        println!("color map type 1, Not implemented");
        return Err(DecodingError::NotImplemented);
    }

    let image_type = stream.read_u8()?;
    if image_type != 2 && image_type != 3 {
        println!("image type: {image_type}");
    }

    // color map specification
    let _first_entry_idx = stream.read_u16()?;
    let color_map_len = stream.read_u16()?;
    let color_map_entry_size = stream.read_u8()?;
    if !([0, 16, 24, 32] as [u8; 4]).contains(&color_map_entry_size) {
        return Err(DecodingError::MalformedImage);
    }

    // image specification
    let _x_orig = stream.read_u16()?;
    let _y_orig = stream.read_u16()?;
    let w = stream.read_u16()?;
    let h = stream.read_u16()?;
    let depth = stream.read_u8()?;
    let image_descriptor = stream.read_u8()?;
    if image_descriptor != 0 {
        println!("image descriptor: {image_descriptor}");
    }
    // println!("{w} x {h}, depth: {depth}");
    if !([8, 16, 24, 32] as [u8; 4]).contains(&depth) {
        return Err(DecodingError::MalformedImage);
    }

    if image_type != 2 && image_type != 3 {
        return Err(DecodingError::NotImplemented);
    }

    stream.skip(id_len as usize)?;

    Ok(TGAImage {
        image: Image {
            w: w as u32,
            h: h as u32,
            channels: (depth / 8) as u32,
            buf: vec![],
            depth: 8,
        },
        _image_type: image_type,
        color_map_len,
        color_map_entry_size,
    })
}

fn decode_color_map(stream: &mut TGADatastream, tga_image: &TGAImage) -> Result<(), DecodingError> {
    stream.skip(tga_image.color_map_len as usize * tga_image.color_map_entry_size as usize / 8)?;
    Ok(())
}

fn decode_1_channel(input: &[u8], output: &mut Image) {
    let scanline_size = output.w as usize * output.channels as usize;
    for i in 0..output.h as usize {
        let out_base = scanline_size as usize * (output.h as usize - i - 1) as usize;
        let in_base = i * scanline_size;
        output.buf[out_base..out_base + output.w as usize].copy_from_slice(&input[in_base..in_base + output.w as usize]);
    }
}

fn decode_2_channels(input: &[u8], output: &mut Image) {
    assert_eq!(output.channels, 4);
    let scanline_size = output.w as usize * 2;
    for i in 0..output.h as usize {
        let out_base = scanline_size as usize * (output.h as usize - i - 1) as usize;
        let in_base = i * scanline_size;
        for j in 0..output.w as usize {
            let out_pix_base = out_base + j * output.channels as usize;
            let in_pix_base = in_base + j * 2;
            let lo = input[in_pix_base + 0];
            let hi = input[in_pix_base + 1];
            output.buf[out_pix_base + 0] = (hi >> 2) & 0x1f; // r
            output.buf[out_pix_base + 1] = ((hi & 3) << 3) | (lo >> 5); // g
            output.buf[out_pix_base + 2] = lo & 0x1f; // b
            output.buf[out_pix_base + 3] = hi >> 7; // a
        }
    }
}

fn decode_3_channels(input: &[u8], output: &mut Image) {
    let scanline_size = output.w as usize * output.channels as usize;
    for i in 0..output.h as usize {
        let out_base = scanline_size as usize * (output.h as usize - i - 1) as usize;
        let in_base = i * scanline_size;
        for j in 0..output.w as usize {
            let out_pix_base = out_base + j * output.channels as usize;
            let in_pix_base = in_base + j * output.channels as usize;
            output.buf[out_pix_base + 0] = input[in_pix_base + 2]; // r
            output.buf[out_pix_base + 1] = input[in_pix_base + 1]; // g
            output.buf[out_pix_base + 2] = input[in_pix_base + 0]; // b
        }
    }
}

fn decode_4_channels(input: &[u8], output: &mut Image) {
    let scanline_size = output.w as usize * output.channels as usize;
    for i in 0..output.h as usize {
        let out_base = scanline_size as usize * (output.h as usize - i - 1) as usize;
        let in_base = i * scanline_size;
        for j in 0..output.w as usize {
            let out_pix_base = out_base + j * output.channels as usize;
            let in_pix_base = in_base + j * output.channels as usize;
            output.buf[out_pix_base + 0] = input[in_pix_base + 2]; // r
            output.buf[out_pix_base + 1] = input[in_pix_base + 1]; // g
            output.buf[out_pix_base + 2] = input[in_pix_base + 0]; // b
            output.buf[out_pix_base + 3] = input[in_pix_base + 3]; // a
        }
    }
}

fn decode_image_data(stream: &mut TGADatastream, tga_image: &mut TGAImage) -> Result<(), DecodingError> {
    let len = tga_image.image.w as usize * tga_image.image.h as usize * tga_image.image.channels as usize;
    if stream.cursor + len > stream.buf.len() {
        return Err(DecodingError::MalformedImage);
    }

    tga_image.image.buf = vec![0; len];
    match tga_image.image.channels {
        1 => decode_1_channel(&stream.buf[stream.cursor..stream.cursor + len], &mut tga_image.image),
        2 => decode_2_channels(&stream.buf[stream.cursor..stream.cursor + len], &mut tga_image.image),
        3 => decode_3_channels(&stream.buf[stream.cursor..stream.cursor + len], &mut tga_image.image),
        _ => decode_4_channels(&stream.buf[stream.cursor..stream.cursor + len], &mut tga_image.image),
    }

    stream.cursor += len;
    Ok(())
}

fn decode_footer(stream: &mut TGADatastream, _tga_image: &mut TGAImage) -> Result<(), DecodingError> {
    const FOOTER_SIZE: usize = 26;
    if stream.cursor + FOOTER_SIZE > stream.buf.len() {
        return Err(DecodingError::MalformedImage);
    }
    let gap = stream.buf.len() - FOOTER_SIZE - stream.cursor;
    if gap != 0 {
        println!("Gap between image data and footer: {gap} bytes");
    }

    let _prev_cursor = stream.cursor;
    stream.cursor = stream.buf.len() - FOOTER_SIZE;

    let extension_area_offset = stream.read_u32()?;
    let developer_directory_offset = stream.read_u32()?;
    if extension_area_offset != 0 {
        println!("has extension area");
    }
    if developer_directory_offset != 0 {
        println!("has developer directory");
    }
    const SIGNATURE: &[u8] = b"\x54\x52\x55\x45\x56\x49\x53\x49\x4F\x4E\x2D\x58\x46\x49\x4C\x45\x2E\x00";
    if &stream.buf[stream.cursor..stream.cursor + SIGNATURE.len()] != SIGNATURE {
        return Err(DecodingError::MalformedImage);
    }

    // TODO

    Ok(())
}

pub fn decode(buf: &[u8]) -> Result<TGAImage, DecodingError> {
    let mut stream = TGADatastream::new(buf);
    let mut tga_image = decode_header(&mut stream)?;
    decode_color_map(&mut stream, &tga_image)?;
    decode_image_data(&mut stream, &mut tga_image)?;
    if stream.cursor != stream.buf.len() {
        decode_footer(&mut stream, &mut tga_image)?;
    }
    Ok(tga_image)
}
