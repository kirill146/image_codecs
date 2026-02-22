
use crate::DecodingError;
use crate::Image;
use std::cmp::max;

pub const PNG_SIGNATURE: &[u8] = b"\x89\x50\x4e\x47\x0d\x0a\x1a\x0a";

const CRC_TABLE: [u32; 256] = [
    0x00000000, 0x77073096, 0xee0e612c, 0x990951ba, 0x076dc419, 0x706af48f, 0xe963a535, 0x9e6495a3, 
    0x0edb8832, 0x79dcb8a4, 0xe0d5e91e, 0x97d2d988, 0x09b64c2b, 0x7eb17cbd, 0xe7b82d07, 0x90bf1d91, 
    0x1db71064, 0x6ab020f2, 0xf3b97148, 0x84be41de, 0x1adad47d, 0x6ddde4eb, 0xf4d4b551, 0x83d385c7, 
    0x136c9856, 0x646ba8c0, 0xfd62f97a, 0x8a65c9ec, 0x14015c4f, 0x63066cd9, 0xfa0f3d63, 0x8d080df5, 
    0x3b6e20c8, 0x4c69105e, 0xd56041e4, 0xa2677172, 0x3c03e4d1, 0x4b04d447, 0xd20d85fd, 0xa50ab56b, 
    0x35b5a8fa, 0x42b2986c, 0xdbbbc9d6, 0xacbcf940, 0x32d86ce3, 0x45df5c75, 0xdcd60dcf, 0xabd13d59, 
    0x26d930ac, 0x51de003a, 0xc8d75180, 0xbfd06116, 0x21b4f4b5, 0x56b3c423, 0xcfba9599, 0xb8bda50f, 
    0x2802b89e, 0x5f058808, 0xc60cd9b2, 0xb10be924, 0x2f6f7c87, 0x58684c11, 0xc1611dab, 0xb6662d3d, 
    0x76dc4190, 0x01db7106, 0x98d220bc, 0xefd5102a, 0x71b18589, 0x06b6b51f, 0x9fbfe4a5, 0xe8b8d433, 
    0x7807c9a2, 0x0f00f934, 0x9609a88e, 0xe10e9818, 0x7f6a0dbb, 0x086d3d2d, 0x91646c97, 0xe6635c01, 
    0x6b6b51f4, 0x1c6c6162, 0x856530d8, 0xf262004e, 0x6c0695ed, 0x1b01a57b, 0x8208f4c1, 0xf50fc457, 
    0x65b0d9c6, 0x12b7e950, 0x8bbeb8ea, 0xfcb9887c, 0x62dd1ddf, 0x15da2d49, 0x8cd37cf3, 0xfbd44c65, 
    0x4db26158, 0x3ab551ce, 0xa3bc0074, 0xd4bb30e2, 0x4adfa541, 0x3dd895d7, 0xa4d1c46d, 0xd3d6f4fb, 
    0x4369e96a, 0x346ed9fc, 0xad678846, 0xda60b8d0, 0x44042d73, 0x33031de5, 0xaa0a4c5f, 0xdd0d7cc9, 
    0x5005713c, 0x270241aa, 0xbe0b1010, 0xc90c2086, 0x5768b525, 0x206f85b3, 0xb966d409, 0xce61e49f, 
    0x5edef90e, 0x29d9c998, 0xb0d09822, 0xc7d7a8b4, 0x59b33d17, 0x2eb40d81, 0xb7bd5c3b, 0xc0ba6cad, 
    0xedb88320, 0x9abfb3b6, 0x03b6e20c, 0x74b1d29a, 0xead54739, 0x9dd277af, 0x04db2615, 0x73dc1683, 
    0xe3630b12, 0x94643b84, 0x0d6d6a3e, 0x7a6a5aa8, 0xe40ecf0b, 0x9309ff9d, 0x0a00ae27, 0x7d079eb1, 
    0xf00f9344, 0x8708a3d2, 0x1e01f268, 0x6906c2fe, 0xf762575d, 0x806567cb, 0x196c3671, 0x6e6b06e7, 
    0xfed41b76, 0x89d32be0, 0x10da7a5a, 0x67dd4acc, 0xf9b9df6f, 0x8ebeeff9, 0x17b7be43, 0x60b08ed5, 
    0xd6d6a3e8, 0xa1d1937e, 0x38d8c2c4, 0x4fdff252, 0xd1bb67f1, 0xa6bc5767, 0x3fb506dd, 0x48b2364b, 
    0xd80d2bda, 0xaf0a1b4c, 0x36034af6, 0x41047a60, 0xdf60efc3, 0xa867df55, 0x316e8eef, 0x4669be79, 
    0xcb61b38c, 0xbc66831a, 0x256fd2a0, 0x5268e236, 0xcc0c7795, 0xbb0b4703, 0x220216b9, 0x5505262f, 
    0xc5ba3bbe, 0xb2bd0b28, 0x2bb45a92, 0x5cb36a04, 0xc2d7ffa7, 0xb5d0cf31, 0x2cd99e8b, 0x5bdeae1d,
    0x9b64c2b0, 0xec63f226, 0x756aa39c, 0x026d930a, 0x9c0906a9, 0xeb0e363f, 0x72076785, 0x05005713,
    0x95bf4a82, 0xe2b87a14, 0x7bb12bae, 0x0cb61b38, 0x92d28e9b, 0xe5d5be0d, 0x7cdcefb7, 0x0bdbdf21,
    0x86d3d2d4, 0xf1d4e242, 0x68ddb3f8, 0x1fda836e, 0x81be16cd, 0xf6b9265b, 0x6fb077e1, 0x18b74777,
    0x88085ae6, 0xff0f6a70, 0x66063bca, 0x11010b5c, 0x8f659eff, 0xf862ae69, 0x616bffd3, 0x166ccf45,
    0xa00ae278, 0xd70dd2ee, 0x4e048354, 0x3903b3c2, 0xa7672661, 0xd06016f7, 0x4969474d, 0x3e6e77db,
    0xaed16a4a, 0xd9d65adc, 0x40df0b66, 0x37d83bf0, 0xa9bcae53, 0xdebb9ec5, 0x47b2cf7f, 0x30b5ffe9,
    0xbdbdf21c, 0xcabac28a, 0x53b39330, 0x24b4a3a6, 0xbad03605, 0xcdd70693, 0x54de5729, 0x23d967bf,
    0xb3667a2e, 0xc4614ab8, 0x5d681b02, 0x2a6f2b94, 0xb40bbe37, 0xc30c8ea1, 0x5a05df1b, 0x2d02ef8d,
];


pub struct PNGImage {
    pub image: Image,
    palette: Option<Palette>,
    trns_alpha: Option<[u16; 3]>,
    color_type: u8,
    gama: Option<f32>,
    interlaced: bool,
}

pub struct Palette {
    values: [[u8; 256]; 4],
    len: usize
}

struct PNGDatastream<'a> {
    buf: &'a [u8],
    cursor: usize,
    crc: u32,
}

struct BitStream {
    buf: u64,
    bits_left: u32,
    chunk_bytes_left: u32,
}

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
enum Filter {
    None,
    Sub,
    Up,
    Average,
    Paeth
}

#[derive(Default)]
struct PNGReconstructor {
    y: u32,
    filter: Option<Filter>,
    pass_id: usize, // 0: non-interlaced, 1..7: interlaced
    scanline_bufs: [Vec<u8>; 2], // 0 - prev, 1 - cur
    cur_consumable_bytes: usize,
    cur_scanline_cursor: usize,
}

impl Palette {
    fn new(len: usize) -> Palette {
        Palette {
            values: [[0; 256], [0; 256], [0; 256], [255; 256]],
            len
        }
    }
}

impl<'a> PNGDatastream<'a> {
    fn new(buf: &'a [u8]) -> PNGDatastream<'a> {
        PNGDatastream {
            buf,
            cursor: 0,
            crc: 0,
        }
    }

    fn len(&self) -> usize {
        self.buf.len()
    }

    fn eof(&self) -> bool {
        self.cursor == self.buf.len()
    }

    fn read_u32_unchecked(&mut self) -> Result<u32, DecodingError> {
        if self.cursor + 4 <= self.buf.len() {
            self.update_crc(&self.buf[self.cursor..self.cursor + 4]);
            let val = u32::from_be_bytes([
                self.buf[self.cursor], self.buf[self.cursor + 1], self.buf[self.cursor + 2], self.buf[self.cursor + 3]
            ]);
            self.cursor += 4;
            Ok(val)
        } else {
            Err(DecodingError::MalformedImage)
        }
    }

    fn read_u32(&mut self) -> Result<u32, DecodingError> {
        let val = self.read_u32_unchecked()?;
        if val < 0x80000000 {
            Ok(val)
        } else {
            Err(DecodingError::MalformedImage)
        }
    }

    fn read_u16(&mut self) -> Result<u16, DecodingError> {
        if self.cursor + 2 <= self.buf.len() {
            self.update_crc(&self.buf[self.cursor..self.cursor + 2]);
            let val = u16::from_be_bytes([
                self.buf[self.cursor], self.buf[self.cursor + 1]
            ]);
            self.cursor += 2;
            Ok(val)
        } else {
            Err(DecodingError::MalformedImage)
        }
    }

    // TODO: maybe some trait can avoid this duplication?
    // fn read_i32(&mut self) -> Result<i32, DecodingError> {
    //     if self.cursor + 4 <= self.buf.len() {
    //         self.update_crc(&self.buf[self.cursor..self.cursor + 4]);
    //         let val = i32::from_be_bytes([
    //             self.buf[self.cursor], self.buf[self.cursor + 1], self.buf[self.cursor + 2], self.buf[self.cursor + 3]
    //         ]);
    //         if val != std::i32::MIN {
    //             self.cursor += 4;
    //             Ok(val)
    //         } else {
    //             Err(DecodingError::MalformedImage)
    //         }
    //     } else {
    //         Err(DecodingError::MalformedImage)
    //     }
    // }

    fn read_u8(&mut self) -> Result<u8, DecodingError> {
        if self.cursor + 1 <= self.buf.len() {
            self.update_crc(&self.buf[self.cursor..self.cursor + 1]);
            let val = self.buf[self.cursor];
            self.cursor += 1;
            Ok(val)
        } else {
            Err(DecodingError::MalformedImage)
        }
    }

    fn consume(&mut self, pattern: &[u8]) -> Result<(), DecodingError> {
        if self.buf[self.cursor..self.buf.len()].starts_with(pattern) {
            self.update_crc(&self.buf[self.cursor..self.cursor + pattern.len()]);
            self.cursor += pattern.len();
            Ok(())
        } else {
            Err(DecodingError::MalformedImage)
        }
    }

    fn read_chunk_name(&mut self) -> Result<[u8; 4], DecodingError> {
        if self.cursor + 4 < self.buf.len() {
            self.update_crc(&self.buf[self.cursor..self.cursor + 4]);
            let val = [
                self.buf[self.cursor + 0],
                self.buf[self.cursor + 1],
                self.buf[self.cursor + 2],
                self.buf[self.cursor + 3],
            ];
            self.cursor += 4;
            Ok(val)
        } else {
            Err(DecodingError::MalformedImage)
        }
    }

    fn skip(&mut self, count: usize) -> Result<(), DecodingError> {
        if self.cursor + count <= self.buf.len() {
            self.update_crc(&self.buf[self.cursor..self.cursor + count]);
            self.cursor += count;
            Ok(())
        } else {
            Err(DecodingError::MalformedImage)
        }
    }

    fn reset_crc(&mut self) {
        self.crc = 0xffffffff;
    }

    fn update_crc(&mut self, buf: &[u8]) {
        for elem in buf {
            self.crc = CRC_TABLE[(self.crc as u8 ^ elem) as usize] ^ (self.crc >> 8);
        }
    }

    fn consume_crc(&mut self) -> Result<(), DecodingError> {
        let crc = self.crc ^ 0xffffffff;
        let crc_check = self.read_u32_unchecked()?;
        if crc != crc_check {
            // println!("{:08x} != {:08x}", crc, crc_check);
            Err(DecodingError::MalformedImage)
        } else {
            Ok(())
        }
    }
}


fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let a32 = a as i32;
    let b32 = b as i32;
    let c32 = c as i32;

    let p = a32 + b32 - c32;
    let pa = (p - a32).abs();
    let pb = (p - b32).abs();
    let pc = (p - c32).abs();

    // let pa = (b32 - c32).abs();
    // let pb = (a32 - c32).abs();
    // let pc = (a32 + b32 - c32 * 2).abs();

    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

fn defilter(filter: Filter, a: u8, b: u8, c: u8, filtered: u8) -> u8 {
    match filter {
        Filter::None => filtered,
        Filter::Sub => filtered.wrapping_add(a),
        Filter::Up => filtered.wrapping_add(b),
        Filter::Average => {
            let avg = (a as u32 + b as u32) / 2;
            filtered.wrapping_add(avg as u8)
        },
        Filter::Paeth => filtered.wrapping_add(paeth_predictor(a, b, c)),
    }
}

fn defilter_sub<const BPP: usize>(cur: &mut [u8]) {
    for i in BPP..cur.len() {
        cur[i] = cur[i].wrapping_add(cur[i - BPP]);
    }
}

fn defilter_up(bpp: usize, prev: &[u8], cur: &mut [u8]) {
    for i in bpp..cur.len() {
        cur[i] = cur[i].wrapping_add(prev[i]);
    }
}

fn defilter_avg<const BPP: usize>(prev: &[u8], cur: &mut [u8]) {
    for i in BPP..cur.len() {
        let avg = (cur[i - BPP] as u32 + prev[i] as u32) / 2;
        cur[i] = cur[i].wrapping_add(avg as u8);
    }
}

fn defilter_paeth<const BPP: usize>(prev: &[u8], cur: &mut [u8]) {
    for i in BPP..cur.len() {
        cur[i] = cur[i].wrapping_add(paeth_predictor(cur[i - BPP], prev[i], prev[i - BPP]));
    }
}


fn defilter_sub_3(cur: &mut [u8]) {
    for i in 3..cur.len() {
        cur[i] = cur[i].wrapping_add(cur[i - 3]);
    }
}

fn defilter_avg_3(prev: &[u8], cur: &mut [u8]) {
    for i in 3..cur.len() {
        let avg = (cur[i - 3] as u32 + prev[i] as u32) / 2;
        cur[i] = cur[i].wrapping_add(avg as u8);
    }
}

fn defilter_paeth_3(prev: &[u8], cur: &mut [u8]) {
    for i in 3..cur.len() {
        cur[i] = cur[i].wrapping_add(paeth_predictor(cur[i - 3], prev[i], prev[i - 3]));
    }
}

fn defilter_sub_4(cur: &mut [u8]) {
    for i in 4..cur.len() {
        cur[i] = cur[i].wrapping_add(cur[i - 4]);
    }
}

fn defilter_avg_4(prev: &[u8], cur: &mut [u8]) {
    for i in 4..cur.len() {
        let avg = (cur[i - 4] as u32 + prev[i] as u32) / 2;
        cur[i] = cur[i].wrapping_add(avg as u8);
    }
}

fn defilter_paeth_4(prev: &[u8], cur: &mut [u8]) {
    for i in 4..cur.len() {
        cur[i] = cur[i].wrapping_add(paeth_predictor(cur[i - 4], prev[i], prev[i - 4]));
    }
}


fn defilter_scanline<const BPP: usize>(filter: Filter, prev: &[u8], cur: &mut [u8]) {
    match filter {
        Filter::None => (),
        Filter::Sub => defilter_sub::<BPP>(cur),
        Filter::Up => defilter_up(BPP, prev, cur),
        Filter::Average => defilter_avg::<BPP>(prev, cur),
        Filter::Paeth => defilter_paeth::<BPP>(prev, cur),
    }
}

fn defilter_scanline_3(filter: Filter, prev: &[u8], cur: &mut [u8]) {
    match filter {
        Filter::None => (),
        Filter::Sub => defilter_sub_3(cur),
        Filter::Up => defilter_up(3, prev, cur),
        Filter::Average => defilter_avg_3(prev, cur),
        Filter::Paeth => defilter_paeth_3(prev, cur),
    }
}

fn defilter_scanline_4(filter: Filter, prev: &[u8], cur: &mut [u8]) {
    match filter {
        Filter::None => (),
        Filter::Sub => defilter_sub_4(cur),
        Filter::Up => defilter_up(4, prev, cur),
        Filter::Average => defilter_avg_4(prev, cur),
        Filter::Paeth => defilter_paeth_4(prev, cur),
    }
}

// index 0 is for regular non-interlaced images
// indices 1..7 are for interlacing passes
const START_X: [u32; 8] = [ 0, 0, 4, 0, 2, 0, 1, 0 ];
const START_Y: [u32; 8] = [ 0, 0, 0, 4, 0, 2, 0, 1 ];
const STEP_X:  [u32; 8] = [ 1, 8, 8, 4, 4, 2, 2, 1 ];
const STEP_Y:  [u32; 8] = [ 1, 8, 8, 8, 4, 4, 2, 2 ];

impl PNGReconstructor {
    #[inline(never)]
    fn process_scanline(&mut self, png_image: &mut PNGImage) -> Result<(), DecodingError> {
        if self.y >= png_image.image.h || self.pass_id >= 8 {
            return Err(DecodingError::MalformedImage);
        }

        let png_channels =
            if png_image.color_type == 3 {
                1
            } else {
                png_image.image.channels as usize - png_image.trns_alpha.is_some() as usize
            };
        if self.filter == None {
            match self.scanline_bufs[1][0] {
                0 => self.filter = Some(Filter::None),
                1 => self.filter = Some(Filter::Sub),
                2 => self.filter = Some(Filter::Up),
                3 => self.filter = Some(Filter::Average),
                4 => self.filter = Some(Filter::Paeth),
                _ => return Err(DecodingError::MalformedImage)
            }
            // println!("filter: {}", byte);
            
            let scanline_pixels = (png_image.image.w + STEP_X[self.pass_id] - START_X[self.pass_id] - 1) / STEP_X[self.pass_id];
            self.cur_consumable_bytes = (scanline_pixels as usize * png_channels * png_image.image.depth as usize + 7) / 8;
            self.cur_scanline_cursor = 0;

            return Ok(());
        }

        let bpp_out = (png_image.image.channels * max(png_image.image.depth, 8) as u32 / 8) as usize; // bytes per pixel
        let bpp_in = if png_image.image.depth < 8 { 1 } else { bpp_out };
        let bpc = (png_image.image.depth / 8) as usize; // bytes per channel

        for i in 0..bpp_in {
            let a = 0;
            let b = self.scanline_bufs[0][i];
            let c = 0;
            self.scanline_bufs[1][i] =
                defilter(self.filter.unwrap(), a, b, c, self.scanline_bufs[1][i]);
        }

        let (prev, cur) = self.scanline_bufs.split_at_mut(1);
        let prev = &prev[0][0..self.cur_consumable_bytes];
        let cur = &mut cur[0][0..self.cur_consumable_bytes];
        
        match bpp_in {
            1 => defilter_scanline::<1>(self.filter.unwrap(), prev, cur),
            2 => defilter_scanline::<2>(self.filter.unwrap(), prev, cur),
            // 3 => defilter_scanline::<3>(self.filter.unwrap(), prev, cur),
            3 => defilter_scanline_3(self.filter.unwrap(), prev, cur),
            // 4 => defilter_scanline::<4>(self.filter.unwrap(), prev, cur),
            4 => defilter_scanline_4(self.filter.unwrap(), prev, cur),
            6 => defilter_scanline::<6>(self.filter.unwrap(), prev, cur),
            8 => defilter_scanline::<8>(self.filter.unwrap(), prev, cur),
            _ => panic!("Unreachable")
        }

        if let Some(palette) = &png_image.palette {
            let base_idx = (self.y as usize * png_image.image.w as usize + START_X[self.pass_id] as usize) * bpp_out;
            let mut idx = 0;
            for i in 0..self.cur_consumable_bytes {
                let byte = self.scanline_bufs[1][i];
                for j in (0..8).step_by(png_image.image.depth as usize) {
                    let idx_in_palette = ((byte >> (8 - png_image.image.depth - j)) & (((1 as u32) << png_image.image.depth) - 1) as u8) as usize;
                    for c in 0..png_image.image.channels {
                        png_image.image.buf[base_idx + idx + c as usize] = palette.values[c as usize][idx_in_palette];
                    }
                    idx += STEP_X[self.pass_id] as usize * bpp_out;
                    if idx + START_X[self.pass_id] as usize * bpp_out >= png_image.image.w as usize * bpp_out as usize {
                        break;
                    }
                }
            }
        } else { // no palette
            if png_image.image.depth < 8 {
                let max_val = (1 << png_image.image.depth) - 1;
                let base_idx = (self.y as usize * png_image.image.w as usize + START_X[self.pass_id] as usize) * bpp_out;
                let mut idx = 0;
                for i in 0..self.cur_consumable_bytes {
                    let byte = self.scanline_bufs[1][i];
                    for j in (0..8).step_by(png_image.image.depth as usize).rev() {
                        let val = byte >> j & max_val;
                        png_image.image.buf[base_idx + idx] = val * (255 / max_val) as u8;
                        if let Some(trns_alpha) = png_image.trns_alpha {
                            let alpha = if val as u16 == trns_alpha[0] { 0 } else { 255 };
                            png_image.image.buf[base_idx + idx + 1] = alpha;
                        }
                        idx += STEP_X[self.pass_id] as usize * bpp_out as usize;
                        if idx + START_X[self.pass_id] as usize * bpp_out >= png_image.image.w as usize * bpp_out as usize {
                            break;
                        }
                    }
                }
            } else { // depth >= 8
                let mut idx = (self.y as usize * png_image.image.w as usize + START_X[self.pass_id] as usize) * bpp_out as usize;
                let pix_size = png_channels * bpc;
                for i in (0..self.cur_consumable_bytes).step_by(pix_size) {
                    png_image.image.buf[idx..idx + pix_size].copy_from_slice(&self.scanline_bufs[1][i..i + pix_size]);
                    if let Some(trns_alpha) = png_image.trns_alpha {
                        if png_image.image.depth == 8 {
                            let alpha =
                                if self.scanline_bufs[1][i..i + png_channels]
                                    .iter().enumerate().all(|it| *it.1 as u16 == trns_alpha[it.0])
                                { 0 } else { 255 };
                            png_image.image.buf[idx + png_channels * bpc] = alpha;
                        } else { // depth == 16
                            let trns_alpha = unsafe { trns_alpha.align_to::<u8>().1 };
                            let alpha =
                                if self.scanline_bufs[1][i..i + png_channels * 2]
                                    .iter().enumerate().all(|it| *it.1 == trns_alpha[it.0])
                                { 0 } else { 255 };
                            png_image.image.buf[idx + png_channels * 2 + 0] = alpha; // lo
                            png_image.image.buf[idx + png_channels * 2 + 1] = alpha; // hi
                        }
                    }
                    idx += STEP_X[self.pass_id] as usize * bpp_out as usize;
                }
            }
        }

        self.y += STEP_Y[self.pass_id];
        self.filter = None;
        self.cur_scanline_cursor = 0;
        self.cur_consumable_bytes = 1; // filter type
        self.scanline_bufs.swap(0, 1);

        if png_image.interlaced {
            while START_X[self.pass_id] >= png_image.image.w || self.y >= png_image.image.h {
                self.pass_id += 1;
                if self.pass_id < 8 {
                    self.y = START_Y[self.pass_id];
                } else {
                    break;
                }
            }
            if self.pass_id < 8 {
                let scanline_pixels = (png_image.image.w + STEP_X[self.pass_id] - START_X[self.pass_id] - 1) / STEP_X[self.pass_id];
                self.cur_consumable_bytes = (scanline_pixels as usize * png_channels * png_image.image.depth as usize + 7) / 8;
                self.scanline_bufs[0][0..self.cur_consumable_bytes].fill(0);
            }
        }

        Ok(())
    }

    fn consume_decoded_byte(&mut self, png_image: &mut PNGImage, byte: u8) -> Result<(), DecodingError> {
        // if self.y == 0 {
        //     println!("byte: {byte}");
        // }

        self.scanline_bufs[1][self.cur_scanline_cursor] = byte;
        self.cur_scanline_cursor += 1;

        if self.cur_scanline_cursor != self.cur_consumable_bytes {
            Ok(()) // fast path
        } else {
            self.process_scanline(png_image) // slow path
        }
    }
}

impl BitStream {
    fn new(chunk_bytes_left: u32) -> BitStream {
        BitStream {
            buf: 0,
            bits_left: 0,
            chunk_bytes_left,
        }
    }

    fn ensure(&mut self, datastream: &mut PNGDatastream, len: u32) -> Result<(), DecodingError> {
        assert!(len <= 57, "");
        if len <= self.bits_left {
            return Ok(());
        }
        let mut req_bytes = (len - self.bits_left + 7) / 8;
        loop {
            let mut available_bytes = std::cmp::min(self.chunk_bytes_left, req_bytes);
            req_bytes -= available_bytes;
            self.chunk_bytes_left -= available_bytes;
            while available_bytes > 0 {
                self.buf |= (datastream.read_u8()? as u64) << self.bits_left;
                self.bits_left += 8;
                available_bytes -= 1;
            }
            if req_bytes != 0 {
                datastream.consume_crc()?;
                self.chunk_bytes_left = datastream.read_u32()?;
                datastream.reset_crc();
                let name = datastream.read_chunk_name()?;
                if &name != b"IDAT" {
                    return Err(DecodingError::MalformedImage);
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn read(&mut self, count: u32) -> u64 {
        assert!(self.bits_left >= count);
        self.bits_left -= count;
        let res = self.buf & ((1 << count) - 1);
        self.buf >>= count;
        res
    }

    fn peek(&mut self, count: u32) -> u64 {
        assert!(self.bits_left >= count);
        self.buf & ((1 << count) - 1)
    }

    fn skip(&mut self, count: u8) {
        assert!(self.bits_left >= count as u32);
        self.bits_left -= count as u32;
        self.buf >>= count;
    }
}

fn decode_ihdr(stream: &mut PNGDatastream, png_image: &mut PNGImage) -> Result<(), DecodingError> {
    if stream.len() != 13 {
        return Err(DecodingError::MalformedImage);
    }

    let w = stream.read_u32()?;
    let h = stream.read_u32()?;
    if w == 0 || h == 0 {
        return Err(DecodingError::MalformedImage);
    }
    // println!("{w} x {h}");

    png_image.image.depth = stream.read_u8()? as u8;
    if !([1, 2, 4, 8, 16] as [u8; 5]).contains(&png_image.image.depth) {
        return Err(DecodingError::MalformedImage);
    }
    // if png_image.image.depth != 8 && png_image.image.depth != 16 {
    //     println!("depth == {}", png_image.image.depth);
    // }

    png_image.color_type = stream.read_u8()?;
    match png_image.color_type {
        0 => png_image.image.channels = 1, // grayscale
        2 => png_image.image.channels = 3, // rgb
        3 => png_image.image.channels = 3, // indexed color, channels might be promoted to 4 later
        4 => png_image.image.channels = 2, // grayscale with alpha
        6 => png_image.image.channels = 4, // rgba
        _ => return Err(DecodingError::MalformedImage),
    }
    // println!("chans: {}", png_image.image.channels);

    // check allowed combinations of color_type and depth
    if png_image.color_type == 0 && !([1, 2, 4, 8, 16] as [u8; 5]).contains(&png_image.image.depth)
        || png_image.color_type == 3 && !([1, 2, 4, 8] as [u8; 4]).contains(&png_image.image.depth)
        || png_image.color_type != 0 && png_image.color_type != 3 && png_image.image.depth != 8 && png_image.image.depth != 16
    {
        return Err(DecodingError::MalformedImage);
    }
    // println!("color type: {}", png_image.color_type);

    let compression_method = stream.read_u8()?;
    if compression_method != 0 {
        return Err(DecodingError::MalformedImage);
    }

    let filter_method = stream.read_u8()?;
    if filter_method != 0 {
        return Err(DecodingError::MalformedImage);
    }

    let interlace = stream.read_u8()?;
    if interlace > 1 {
        return Err(DecodingError::MalformedImage);
    }
    png_image.interlaced = interlace == 1;
    // if png_image.interlaced {
    //     println!("Interlaced");
    // }

    png_image.image.w = w;
    png_image.image.h = h;

    Ok(())
}

fn build_huffman_lut<const N: usize, const M: usize>(cls: &[u8]) -> [u16; M] {
    let mut bl_count: [u16; N] = [0; N];
    cls.iter()
        .for_each(|val| bl_count[*val as usize] += 1);
    bl_count[0] = 0;
    let mut next_code: [u16; N] = [0; N];
    for bits in 1..N {
        next_code[bits] = (next_code[bits - 1] + bl_count[bits - 1] as u16) << 1;
        if next_code[bits] + bl_count[bits] > (1 << bits) {
            panic!("Malformed");
        }
    }

    assert_eq!(M, 1 << (N - 1)); // TODO: wait for #![feature(generic_const_exprs)]
    let mut huff = [0; M];
    for n in 0..cls.len() {
        let len = cls[n];
        if len != 0 {
            let code = (next_code[len as usize].reverse_bits() >> (16 - len)) as usize;
            for c in (code..M).step_by(1 << len) {
                huff[c] = n as u16;
            }
            next_code[len as usize] += 1;
        }
    }

    return huff;
}

fn decode_cls<const N_MAX: usize, const N: u32>(bs: &mut BitStream, stream: &mut PNGDatastream, huff: &[u16], huff_cls: &[u8], max_symbol: u32) -> Result<[u8; N_MAX], DecodingError> {
    // N_MAX is 286 (or maybe 288) or 32
    let mut cls: [u8; N_MAX] = [0; N_MAX];
    const OFFSETS: [u32; 3] = [3, 3, 11];
    const EXTRA_BITS: [u32; 3] = [2, 3, 7];
    let mut sym: u32 = 0;
    while sym < max_symbol {
        bs.ensure(stream, N - 1)?;
        let code = bs.peek(N - 1); // TODO: skip code_len bits
        let cl = huff[code as usize] as u8;
        bs.skip(huff_cls[cl as usize]);
        if cl >= 16 {
            let idx = cl as usize - 16;
            let offset = OFFSETS[idx];
            let extra_bits = EXTRA_BITS[idx];
            bs.ensure(stream, extra_bits)?;
            let reps = bs.read(extra_bits) as u32 + offset;
            let till = sym + reps;
            while sym != till {
                cls[sym as usize] = if cl == 16 { cls[sym as usize - 1] } else { 0 };
                sym += 1;
            }
        } else {
            cls[sym as usize] = cl;
            sym += 1;
        }
    }

    Ok(cls)
}

#[inline(never)]
fn decode_idat(stream: &mut PNGDatastream, chunk_bytes_left: u32, png_image: &mut PNGImage) -> Result<(), DecodingError> {
    use std::cmp::max;
    let sz = png_image.image.w as usize * png_image.image.h as usize * png_image.image.channels as usize
        * max((png_image.image.depth / 8) as usize, 1);
    png_image.image.buf.resize(sz, 0);
    let mut bs = BitStream::new(chunk_bytes_left);
    let mut reconstructor: PNGReconstructor = Default::default();
    if png_image.interlaced {
        reconstructor.pass_id = 1;
    }
    let bytes_per_scanline =
        if png_image.color_type != 3 {
            (png_image.image.depth as usize * png_image.image.channels as usize * png_image.image.w as usize + 7) / 8
        } else {
            (png_image.image.depth as usize * png_image.image.w as usize + 7) / 8
        };
    reconstructor.scanline_bufs = [ vec![0; bytes_per_scanline], vec![0; bytes_per_scanline] ];
    reconstructor.cur_consumable_bytes = 1; // filter type

    bs.ensure(stream, 16)?;
    let cmf = bs.read(8);
    let cm = cmf & 0xf;
    let cinfo = cmf >> 4;
    if cm != 8 || cinfo > 7 {
        return Err(DecodingError::MalformedImage);
    }
    let window_size = 1usize << (cinfo + 8);
    let flg = bs.read(8);
    if (((cmf as u32) << 8 | flg as u32) & 0x1f) % 31 == 0 || flg & 0x20 != 0 {
        return Err(DecodingError::MalformedImage);
    }
    // let flevel = flg >> 6; // ignore compression level

    // compressed data
    let mut dec_buf: [u8; 32768] = [0; 32768];
    let mut dec_cursor = 0;
    loop {
        bs.ensure(stream, 3)?;
        let header = bs.read(3);
        let bfinal = header & 1;
        let btype = header >> 1;
        match btype {
            3 => return Err(DecodingError::MalformedImage),
            1 | 2 => {
                let mut cls_dist: [u8; 32] = [5; 32];
                let cls_lit_len = if btype == 2 {
                    // parse literal/length codes
                    bs.ensure(stream, 14)?;
                    let hlit = bs.read(5);
                    let hdist = bs.read(5);
                    let hclen = bs.read(4) as u32;

                    // cls is code lengths
                    bs.ensure(stream, (hclen + 4) * 3)?; // conveniently no more than 57 bits
                    let mut cls_of_cls: [u8; 19] = [0; 19];
                    const INDEX_ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
                    for i in 0..hclen + 4 {
                        cls_of_cls[INDEX_ORDER[i as usize]] = bs.read(3) as u8;
                    }

                    let huff: [u16; 128] = build_huffman_lut::<8, 128>(&cls_of_cls);
                    let cls_lit_len = decode_cls::<288, 8>(&mut bs, stream, &huff, &cls_of_cls, 257 + hlit as u32)?;
                    cls_dist = decode_cls::<32, 8>(&mut bs, stream, &huff, &cls_of_cls, 1 + hdist as u32)?;
                    cls_lit_len
                } else {
                    core::array::from_fn(|i|{
                        match i {
                            _ if i <= 143 => 8,
                            _ if i <= 255 => 9,
                            _ if i <= 279 => 7,
                            _ => 8,
                        }
                    })
                };

                let huff_lit_len: [u16; 32768] = build_huffman_lut::<16, 32768>(&cls_lit_len);
                let huff_dist:    [u16; 32768] = build_huffman_lut::<16, 32768>(&cls_dist);

                // the actual compressed data starts here
                const LEN_OFFSETS: [u8; 20] = [11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227];
                const DIST_OFFSETS: [u16; 26] = [5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577];
                loop {
                    bs.ensure(stream, 15)?;
                    let code = bs.peek(15) as usize;
                    let sym = huff_lit_len[code];
                    bs.skip(cls_lit_len[sym as usize]);
                    if sym < 256 {
                        let byte = sym as u8;
                        reconstructor.consume_decoded_byte(png_image, byte)?;
                        dec_buf[dec_cursor] = byte;
                        dec_cursor += 1;
                        dec_cursor &= window_size - 1;
                        // println!("lit {sym}");
                    } else if sym == 256 { // end of block
                        // println!("EOB");
                        break;
                    } else {
                        if sym > 285 {
                            return Err(DecodingError::MalformedImage);
                        }
                        let mut len =
                            if sym < 265 {
                                sym - 254
                            } else if sym == 285 {
                                258
                            } else {
                                let extra_bits = (sym - 261) / 4;
                                bs.ensure(stream, extra_bits as u32)?;
                                let len = bs.read(extra_bits as u32) as u16;
                                len + LEN_OFFSETS[(sym - 265) as usize] as u16
                            };
                        bs.ensure(stream, 15)?;
                        let code = bs.peek(15) as usize;
                        let dist_code = huff_dist[code];
                        bs.skip(cls_dist[dist_code as usize]);
                        if dist_code > 29 {
                            return Err(DecodingError::MalformedImage);
                        }
                        let dist =
                            if dist_code < 4 {
                                dist_code as usize + 1
                            } else {
                                let extra_bits = (dist_code - 2) / 2;
                                bs.ensure(stream, extra_bits as u32)?;
                                let len = bs.read(extra_bits as u32) as u32;
                                len as usize + DIST_OFFSETS[(dist_code - 4) as usize] as usize
                            };
                        let mut p = (dec_cursor + 32768 - dist) & 0x7fff;
                        // println!("len {len} dist {dist}");
                        while len > 0 {
                            let byte = dec_buf[p];
                            reconstructor.consume_decoded_byte(png_image, byte)?;
                            dec_buf[dec_cursor] = byte;
                            dec_cursor += 1;
                            dec_cursor &= window_size - 1;
                            p += 1;
                            p &= window_size - 1;
                            len -= 1;
                        }
                    }
                }
            },
            _ => { // 0
                bs.skip(bs.bits_left as u8 % 8);
                bs.ensure(stream, 32)?;
                let len_nlen = bs.read(32) as u32;
                let mut len = len_nlen as u16;
                let nlen = (len_nlen >> 16) as u16;
                if nlen != !len {
                    return Err(DecodingError::MalformedImage);
                }
                while len > 0 {
                    bs.ensure(stream, 8)?;
                    let byte = bs.read(8) as u8;
                    reconstructor.consume_decoded_byte(png_image, byte)?;
                    len -= 1;
                }
            }
        }
        if bfinal != 0 {
            break;
        }
    }

    // skip 4 bytes of zlib's ADLER32 checksum
    let skip_bits = 32 + bs.bits_left % 8; // ignore bs padding bits in the last byte
    bs.ensure(stream, skip_bits)?;
    bs.skip(skip_bits as u8);
    assert_eq!(bs.bits_left, 0); // TODO: is it always true?

    stream.consume_crc()?;

    if png_image.image.depth == 16 {
        for i in (0..png_image.image.buf.len()).step_by(2) {
            png_image.image.buf.swap(i, i + 1); // make it little-endian
        }
    }

    Ok(())
}

fn decode_plte(stream: &mut PNGDatastream, png_image: &mut PNGImage) -> Result<(), DecodingError> {
    if stream.len() % 3 != 0 {
        return Err(DecodingError::MalformedImage);
    }

    if png_image.color_type == 3 {
        let palette_len = stream.len() / 3;
        if palette_len > (1 << png_image.image.depth) {
            return Err(DecodingError::MalformedImage);
        }

        let mut palette = Palette::new(palette_len);

        for i in 0..palette.len {
            palette.values[0][i] = stream.read_u8()?; // r
            palette.values[1][i] = stream.read_u8()?; // g
            palette.values[2][i] = stream.read_u8()?; // b
        }
        png_image.palette = Some(palette);
    } else {
        stream.cursor = stream.len();
    }

    Ok(())
}

fn decode_trns(stream: &mut PNGDatastream, png_image: &mut PNGImage) -> Result<(), DecodingError> {
    let len = stream.len();

    match png_image.color_type {
        0 | 2 => {
            assert!(png_image.image.channels == 1 || png_image.image.channels == 3);
            let mut trns_alpha: [u16; 3] = [0; 3];
            for i in 0..png_image.image.channels {
                trns_alpha[i as usize] = stream.read_u16()?;
            }
            // println!("trns_alpha: {:?}", trns_alpha);
            png_image.trns_alpha = Some(trns_alpha);
            png_image.image.channels += 1; // + alpha
        }
        3 => {
            assert!(png_image.image.channels == 3);
            assert!(png_image.palette.is_some());
            png_image.image.channels = 4;


            let palette = png_image.palette.as_mut().unwrap();
            if len > palette.len {
                return Err(DecodingError::MalformedImage);
            }

            for i in 0..len {
                palette.values[3][i] = stream.read_u8()?;
            }
        },
        _ => return Err(DecodingError::MalformedImage)
    }

    Ok(())
}

#[allow(dead_code)]
fn decode_phys(stream: &mut PNGDatastream) -> Result<(), DecodingError> {
    if stream.len() != 9 {
        return Err(DecodingError::MalformedImage);
    }

    let _x = stream.read_u32()?;
    let _y = stream.read_u32()?;
    let _unit = stream.read_u8()?;
    // println!("pHYs: {_x} {_y} {_unit}");

    Ok(())
}

fn decode_gama(stream: &mut PNGDatastream, png_image: &mut PNGImage) -> Result<(), DecodingError> {
    let raw_gama = stream.read_u32()?;
    // println!("raw_gama: {raw_gama}");
    if raw_gama != 0 {
        png_image.gama = Some((raw_gama as f32) / 100000.);
    }
    Ok(())
}

pub fn decode(buf: &[u8]) -> Result<PNGImage, DecodingError> {
    let mut stream = PNGDatastream::new(buf);
    stream.consume(PNG_SIGNATURE)?;

    let mut png_image = PNGImage {
        image: Image {
            w: 0,
            h: 0,
            channels: 0,
            buf: vec![],
            depth: 0,
        },
        palette: None,
        trns_alpha: None,
        color_type: 0,
        gama: None,
        interlaced: false,
    };

    loop {
        let len = stream.read_u32()? as usize;
        stream.reset_crc();
        let chunk_name = stream.read_chunk_name()?;
        if stream.cursor + len >= buf.len() {
            println!("Chunk length is out of bounds");
            return Err(DecodingError::MalformedImage);
        }
        let mut chunk_stream = PNGDatastream::new(&buf[stream.cursor..stream.cursor + len]);

        if chunk_name != *b"IDAT" {
            stream.skip(len)?;
            stream.consume_crc()?;
        }

        match &chunk_name {
            b"IHDR" => decode_ihdr(&mut chunk_stream, &mut png_image)?,
            b"PLTE" => decode_plte(&mut chunk_stream, &mut png_image)?,
            b"tRNS" => decode_trns(&mut chunk_stream, &mut png_image)?,
            b"pHYs" => decode_phys(&mut chunk_stream)?,
            b"gAMA" => decode_gama(&mut chunk_stream, &mut png_image)?,
            b"IDAT" => decode_idat(&mut stream, len as u32, &mut png_image)?,
            b"IEND" => break,
            _ => {
                println!("skipping {}", String::from_utf8_lossy(&chunk_name));
                chunk_stream.skip(len)?;
            }
        }

        if chunk_name != *b"IDAT" && !chunk_stream.eof() {
            return Err(DecodingError::MalformedImage);
        }
    }

    if stream.eof() {
        if png_image.image.buf.len() != 0 {
            Ok(png_image)
        } else {
            Err(DecodingError::MalformedImage) // missing IDAT chunk?
        }
    } else {
        Err(DecodingError::MalformedImage)
    }
}

#[cfg(test)]
mod tests {
    use crate::png_decoder::CRC_TABLE;

    #[test]
    fn test_crc_table() {
        let expected_table: [u32; 256] = core::array::from_fn(|i| {
            let mut c = i as u32;
            for _ in 0..8 {
                c = if c & 1 > 0 { 0xedb88320 ^ (c >> 1) } else { c >> 1 };
            }
            c
        });
        assert_eq!(CRC_TABLE, expected_table);
    }
}
