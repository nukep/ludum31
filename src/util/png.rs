use gl;
use gl::types::{GLint, GLuint, GLsizei};
use lodepng;
use opengl_util::texture::Texture2D;

pub fn load_png32_data_and_upload(png_data: &[u8]) -> Result<Texture2D, String> {
    use std::mem::transmute;

    let img = match lodepng::decode32(png_data) {
        Ok(img) => img,
        Err(e) => return Err(format!("LodePNG decoding error: {}", e))
    };

    let tex_id: GLuint = unsafe {
        let mut id = 0;
        gl::GenTextures(1, &mut id);
        gl::BindTexture(gl::TEXTURE_2D, id);
        id
    };

    unsafe {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
    }

    unsafe {
        let ptr = transmute(img.buffer.get(0).unwrap());
        let internal = gl::RGBA8 as GLint;
        let format = gl::RGBA;
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            internal,
            img.width as GLsizei,
            img.height as GLsizei,
            0,
            format,
            gl::UNSIGNED_BYTE,
            ptr
        );
    }

    Ok(Texture2D { id: tex_id })
}
