#![feature(link_args)]

extern crate cgmath;
extern crate game_platforms;
extern crate gl;
extern crate sdl2;
extern crate synth;
extern crate lodepng;
extern crate serialize;

use std::sync::{Arc, Mutex};

mod game;
#[allow(dead_code)]
mod opengl_util;
mod util;

// Statically link SDL2 (libSDL2.a)
// Link the required Windows dependencies
#[cfg(target_os="windows")]
#[link_args = "-lwinmm -lole32 -lgdi32 -limm32 -lversion -loleaut32 -luuid -Wl,--subsystem,windows"]
extern {}

/// Read and lock a mutex value infinitely
pub struct MutexIterator<T: Send> {
    mutex: Arc<Mutex<T>>
}

impl<T: Clone + Send> Iterator<T> for MutexIterator<T> {
    fn next(&mut self) -> Option<T> {
        let value: T = {
            let lock = (*self.mutex).lock();
            lock.clone()
        };
        Some(value)
    }
}

fn main() {
    use game_platforms::sdl2_opengl::{Platform, RenderContext};

    let game = game::Game::new();

    let load_gl = || {
        gl::load_with(|s: &str| unsafe {
            use std;
            match sdl2::video::gl_get_proc_address(s) {
                Some(ptr) => std::mem::transmute(ptr),
                None => std::ptr::null()
            }
            });
        ((), game::render::GameRenderState::new())
    };

    let scale = 2;

    let size = match game.level.level_size_as_u32() {
        (w, h) => (w as int * scale, h as int * scale)
    };

    let render_ctx = match RenderContext::new("Mr. Scroll", size, (3, 0), load_gl) {
        Ok(ctx) => ctx,
        Err(e) => panic!("{}", e)
    };

    let platform = match Platform::new(game, render_ctx) {
        Ok(ctx) => ctx,
        Err(e) => panic!("{}", e)
    };

    let _game = platform.run();
}
