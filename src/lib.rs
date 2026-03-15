
pub mod png_decoder;
pub mod tga_decoder;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum DecodingError {
    UnknownFormat,
    MalformedImage,
    NotImplemented,
}

pub struct Image {
    pub w: u32,
    pub h: u32,
    pub channels: u32,
    pub buf: Vec<u8>,
    pub depth: u8,
}

impl Image {
    pub fn new(buf: &[u8]) -> Result<Image, DecodingError> {
        if buf.starts_with(png_decoder::PNG_SIGNATURE) {
            png_decoder::decode(buf).map(|png_image| png_image.image)
        } else if tga_decoder::is_tga(buf) {
            tga_decoder::decode(buf).map(|tga_image| tga_image.image)
        } else {
            Err(DecodingError::UnknownFormat)
        }
    }
}
