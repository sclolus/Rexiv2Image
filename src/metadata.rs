use rexiv2::*;
use std::fs::File;
use std::path::Path;
use std::convert::From;
use std::result::Result;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std;
use self::png::*;
use self::png;
use self::jpeg::*;
use self::jpeg;
use self::pnm::*;
use self::pnm;
use self::ico::*;
use self::ico;
use self::tiff::*;
use self::tiff;
use self::tga::*;
use self::tga;
use self::bmp::*;
use self::bmp;
use self::gif::*;
use self::gif;
use image::*;
use image::ColorType;

#[derive(Debug)]
pub enum Rexiv2ImageError {
    //Error from rexiv2 crate
    MetadataError(Rexiv2Error),
    //Error from image crate
    DecoderError(ImageError),
    //Internal error: described by String
    Internal(String),
}

pub enum DecoderType {
    PNG(PNGDecoder<File>),
    JPEG(JPEGDecoder<File>),
    PNM(PNMDecoder<File>),
    ICO(ICODecoder<File>),
    TIFF(TIFFDecoder<File>),
    TGA(TGADecoder<File>),
    BMP(BMPDecoder<File>),
    GIF(Decoder<File>),
}

pub struct DecoderWithMetadata {
    //Could be private but would force to implement as the methods of the Metadata type to this container
    pub metadata: Metadata,
    decoder: DecoderType,
}

impl DecoderWithMetadata {
    pub fn new(path: &Path, format: ImageFormat)
                                        -> Result<DecoderWithMetadata, Rexiv2ImageError> {
        let metadata = Metadata::new_from_path(path)?;
        let input_file = File::open(path)?;
        
        Ok(DecoderWithMetadata {
            metadata,
            decoder: DecoderWithMetadata::get_new_decoder(format, input_file)?,
        })
    }
    
    pub fn save_metadata(&self, path: &Path) -> Result<(), Rexiv2ImageError> {
        Ok(self.metadata.save_to_file(path)?)
    }
    
    fn get_new_decoder(format: ImageFormat, input_file: File) -> Result<DecoderType, Rexiv2ImageError> {
        Ok(match format {
            ImageFormat::PNG => DecoderType::PNG(png::PNGDecoder::new(input_file)),
            ImageFormat::JPEG => DecoderType::JPEG(jpeg::JPEGDecoder::new(input_file)),
            ImageFormat::PNM => DecoderType::PNM(pnm::PNMDecoder::new(input_file)?),
            ImageFormat::ICO => DecoderType::ICO(ico::ICODecoder::new(input_file)?),
            ImageFormat::TIFF => DecoderType::TIFF(tiff::TIFFDecoder::new(input_file)?),
            ImageFormat::TGA => DecoderType::TGA(tga::TGADecoder::new(input_file)),
            ImageFormat::BMP => DecoderType::BMP(bmp::BMPDecoder::new(input_file)),
            ImageFormat::GIF => DecoderType::GIF(gif::Decoder::new(input_file)),
            _ => return Err(Rexiv2ImageError::Internal("Unsupported file format".to_string())),
        })
    }
}

macro_rules! select_decoder_variant {
    (*$enumeration:expr, $method:ident) => {
        match *$enumeration {
            DecoderType::PNG(ref mut decoder) => decoder.$method(),
            DecoderType::JPEG(ref mut decoder) => decoder.$method(),
            _ => Err(ImageError::FormatError("Unsupported file format".to_string())),
        }
    };
    (*$enumeration:expr, $method:ident, $($args:expr),* ) => {
        match *$enumeration {
            DecoderType::PNG(ref mut decoder) => decoder.$method($($args),*),
            DecoderType::JPEG(ref mut decoder) => decoder.$method($($args),*),
            _ => Err(ImageError::FormatError("Unsupported file format".to_string())),
        }
    };
    ($enumeration:expr, $method:ident) => {
        match $enumeration {
            DecoderType::PNG(decoder) => decoder.$method(),
            DecoderType::JPEG(decoder) => decoder.$method(),
            _ => Err(ImageError::FormatError("Unsupported file format".to_string())),
        }
    };
}

impl ImageDecoder for DecoderType {
    fn dimensions(&mut self) -> ImageResult<(u32, u32)> {
        select_decoder_variant!(*self, dimensions)
    }
    
    fn colortype(&mut self) -> ImageResult<ColorType> {
        select_decoder_variant!(*self, colortype)
    }
    
    fn row_len(&mut self) -> ImageResult<usize> {
        select_decoder_variant!(*self, row_len)
    }
    
    fn read_scanline(&mut self, buf: &mut [u8]) -> ImageResult<u32> {
        select_decoder_variant!(*self, read_scanline, buf)
    }
    
    fn read_image(&mut self) -> ImageResult<DecodingResult> {
        select_decoder_variant!(*self, read_image)
    }

    fn is_animated(&mut self) -> ImageResult<bool> {
        select_decoder_variant!(*self, is_animated)
    }
    fn into_frames(self) -> ImageResult<Frames> {
        select_decoder_variant!(self, into_frames)
    }
    fn load_rect(&mut self, x: u32, y: u32, length: u32, width: u32) -> ImageResult<Vec<u8>> {
        select_decoder_variant!(*self, load_rect, x, y, length, width)
    }    
}

impl ImageDecoder for DecoderWithMetadata {
    fn dimensions(&mut self) -> ImageResult<(u32, u32)> {
        self.decoder.dimensions()
    }
    
    fn colortype(&mut self) -> ImageResult<ColorType> {
        self.decoder.colortype()
    }
    
    fn row_len(&mut self) -> ImageResult<usize> {
        self.decoder.row_len()
    }
    
    fn read_scanline(&mut self, buf: &mut [u8]) -> ImageResult<u32> {
        self.decoder.read_scanline(buf)
    }
    
    fn read_image(&mut self) -> ImageResult<DecodingResult> {
        self.decoder.read_image()
    }
    
    fn is_animated(&mut self) -> ImageResult<bool> {
        self.decoder.is_animated()
    }
    
    fn into_frames(self) -> ImageResult<Frames> {
        self.decoder.into_frames()
    }
    
    fn load_rect(&mut self, x: u32, y: u32, length: u32, width: u32) -> ImageResult<Vec<u8>> {
        self.decoder.load_rect(x, y, length, width)
    }
}

impl From<Rexiv2Error> for Rexiv2ImageError {
    fn from(rexiv2error: Rexiv2Error) -> Rexiv2ImageError {
        Rexiv2ImageError::MetadataError(rexiv2error)
    }
}

impl From<ImageError> for Rexiv2ImageError {
    fn from(error: ImageError) -> Rexiv2ImageError {
        Rexiv2ImageError::DecoderError(error)
    }
}

impl From<std::io::Error> for Rexiv2ImageError {
    fn from(error: std::io::Error) -> Rexiv2ImageError {
        Rexiv2ImageError::Internal(error.description().to_string())
    }
}

impl Display for Rexiv2ImageError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Rexiv2ImageError::Internal(ref err_string) => write!(f, "{}", err_string),
            Rexiv2ImageError::MetadataError(ref err) => err.fmt(f),
            Rexiv2ImageError::DecoderError(ref err) => err.fmt(f),
        }
    }
}

impl Error for Rexiv2ImageError {
    fn description(&self) -> &str {
        match *self {
            Rexiv2ImageError::MetadataError(ref err) => err.description(),
            Rexiv2ImageError::DecoderError(ref err) => err.description(),
            Rexiv2ImageError::Internal(ref err) => err.as_str(),
        }
    }
    fn cause(&self) -> Option<&Error> {
        match *self {
            Rexiv2ImageError::MetadataError(ref err) => Some(err),
            Rexiv2ImageError::DecoderError(ref err) => Some(err),
            Rexiv2ImageError::Internal(_) => None,
        }
    }
}
