#![feature(link_args)]
#![feature(box_syntax)]

extern crate cgmath;
extern crate game_platforms;
extern crate gl;
extern crate image;
extern crate sdl2;
extern crate synth;
extern crate serialize;

mod game;
#[allow(dead_code)]
mod opengl_util;
mod util;

// Statically link SDL2 (libSDL2.a)
// Link the required Windows dependencies
#[cfg(target_os="windows")]
#[cfg(feature="statically-link-sdl2")]
#[link_args = "-lwinmm -lole32 -lgdi32 -limm32 -lversion -loleaut32 -luuid"]
extern {}

#[cfg(target_os="windows")]
#[cfg(feature="no-console")]
#[link_args = "-Wl,--subsystem,windows"]
extern {}

fn main() {
    use game_platforms::sdl2_opengl::{Platform, RenderContext};

    let game = game::Game::new();

    let init_renderer = |:| {
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
        (w, h) => (w as int * scale, h as int * scale)
    };

    let render_ctx = match RenderContext::new("Mr. Scroll", size, (3, 0), init_renderer) {
        Ok(ctx) => ctx,
        Err(e) => panic!("{}", e)
    };

    let platform = match Platform::new(game, render_ctx) {
        Ok(ctx) => ctx,
        Err(e) => panic!("{}", e)
    };

    let _game = platform.run();
}
