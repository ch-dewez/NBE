use thiserror::Error;
use std::{io::{self, Cursor}, path::Path};

use glow::{HasContext};
use image::{EncodableLayout, ImageBuffer, ImageError, ImageReader};

use crate::context::OpenGlContext;

/// represent a texutre 2D, only supports 2D rgb8 textures, mabye others in the future
pub struct Texture {
    texture:glow::Texture 
}

#[derive(Error, Debug)]
pub enum TextureFromPathError{
    #[error("Failed to load image, io error, maybe wrong path")]
    IoError(#[from] io::Error),
    #[error("Failed to decode the image")]
    ImageForm(#[from] ImageError),
}

impl Texture {
    pub fn new_from_path(&self, gl:&OpenGlContext, path: &Path) -> Result<Self, TextureFromPathError>{
        let image: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageReader::open(path)?.decode()?.to_rgb8();
        Ok(Self::new_from_image(gl, image))
    }

    pub fn new_from_undecoded_bytes(gl:&OpenGlContext, bytes: &[u8]) -> Result<Self, TextureFromPathError>{
        let image = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?.decode()?.to_rgb8();
        Ok(Self::new_from_image(gl, image))
    }

    pub fn new_from_image(gl:&OpenGlContext, image: ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Self {
        unsafe {
            let texture = gl.0.create_texture().expect("Couldn't create texture");
            gl.0.bind_texture(glow::TEXTURE_2D, Some(texture));

            let pixels = glow::PixelUnpackData::Slice(Some(image.as_bytes()));

            gl.0.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.0.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);

            gl.0.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR_MIPMAP_LINEAR as i32);
            gl.0.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

            gl.0.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGB as i32, image.width() as i32, image.height() as i32, 0, glow::RGB, glow::UNSIGNED_BYTE, pixels);
            gl.0.generate_mipmap(glow::TEXTURE_2D);

            Self{
                texture
            }
        }
    }

    pub fn bind(&self, gl:&OpenGlContext, slot: u32){
        unsafe {
            gl.0.active_texture(glow::TEXTURE0 + slot);
            gl.0.bind_texture(glow::TEXTURE_2D, Some(self.texture));
        }
    }
}

