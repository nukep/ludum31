extern crate cgmath;
extern crate game_platforms;
extern crate gl;
extern crate image;
extern crate rustc_serialize;
extern crate sdl2;
extern crate synth;

mod game;
#[allow(dead_code)] mod opengl_util;
mod util;

fn main() {
    use game_platforms::sdl2_opengl::{Platform, RenderContext};

    let sdl_context = sdl2::init(sdl2::INIT_VIDEO).unwrap();

    let game = game::Game::new(&sdl_context);

    let init_renderer = || {
        gl::load_with(|s: &str| unsafe {
            use std;
            match sdl2::video::gl_get_proc_address(s) {
                Some(ptr) => std::mem::transmute(ptr),
                None => std::ptr::null()
            }
        });

        game::render::Renderer::new()
    };

    let scale = 2;

    let size = match game.level.level_size_as_u32() {
        (w, h) => (w as u16 * scale, h as u16 * scale)
    };

    let render_ctx = match RenderContext::new("Mr. Scroll", size, (3, 0), init_renderer) {
        Ok(ctx) => ctx,
        Err(e) => panic!("{}", e)
    };

    let platform = match Platform::new(&sdl_context, game, render_ctx) {
        Ok(ctx) => ctx,
        Err(e) => panic!("{}", e)
    };

    let _game = platform.run();
}
