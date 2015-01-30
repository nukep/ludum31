//! SDL2 platform with OpenGL/OpenGL ES context.
//! Used on Windows, OS X and Linux

use sdl2;
use time;
use std::collections::HashSet;
use std::num::cast;
use super::{GameStepper, GameRenderer, PlatformStepResult};
use super::fps_meter::{FPSMeter, ValueOnChange};

#[derive(Clone)]
struct SDLInputFrame {
    /// sdl2::keycode::KeyCode doesn't implement Clone, so we need to store an integer representation
    keyboard: HashSet<u32>,
    mouse: Option<(sdl2::mouse::MouseState, i32, i32)>,
    mouse_in_focus: bool,
    mouse_wheel_absolute: (i32, i32),
    viewport: (i32, i32),
    exit_request: bool,
    fps: Option<f64>
}

impl SDLInputFrame {
    fn new(initial_viewport: (i32, i32)) -> SDLInputFrame {
        SDLInputFrame {
            keyboard: HashSet::new(),
            mouse: None,
            mouse_in_focus: false,
            mouse_wheel_absolute: (0, 0),
            viewport: initial_viewport,
            exit_request: false,
            fps: None
        }
    }

    fn is_mouse_button_down(&self, button: sdl2::mouse::MouseState) -> bool {
        match self.mouse {
            Some((state, _, _)) => state.intersects(button),
            None => false
        }
    }

    fn is_keycode_down(&self, keycode: sdl2::keycode::KeyCode) -> bool {
        use std::num::ToPrimitive;

        let keycode_int = keycode.to_u32().expect("Could not convert keycode to u32");
        self.keyboard.contains(&keycode_int)
    }

    fn get_mouse_position_if_focused(&self) -> Option<(i32, i32)> {
        match self.mouse {
            Some((_, x, y)) if self.mouse_in_focus => Some((x, y)),
            _ => None
        }
    }

    fn get_mouse_position_if_button(&self, button: sdl2::mouse::MouseState) -> Option<(i32, i32)> {
        match self.mouse {
            Some((mouse_state, x, y)) if mouse_state.intersects(button) => {
                Some((x, y))
            },
            _ => None
        }
    }
}

pub struct Input {
    last_frame: SDLInputFrame,
    current_frame: SDLInputFrame
}

impl Input {
    fn new(initial_viewport: (i32, i32)) -> Input {
        Input {
            last_frame: SDLInputFrame::new(initial_viewport),
            current_frame: SDLInputFrame::new(initial_viewport)
        }
    }

    fn push_new_frame(&mut self, frame: SDLInputFrame) {
        use std::mem::replace;
        self.last_frame = replace(&mut self.current_frame, frame);
    }

    fn push_current_frame(&mut self) {
        self.last_frame = self.current_frame.clone();
    }

    pub fn is_mouse_button_down(&self, button: sdl2::mouse::MouseState) -> bool {
        self.current_frame.is_mouse_button_down(button)
    }

    pub fn is_mouse_button_newly_down(&self, button: sdl2::mouse::MouseState) -> bool {
        !self.last_frame.is_mouse_button_down(button) && self.current_frame.is_mouse_button_down(button)
    }

    pub fn get_mouse_position_if_focused(&self) -> Option<(i32, i32)> {
        self.current_frame.get_mouse_position_if_focused()
    }

    pub fn get_mouse_position_if_button(&self, button: sdl2::mouse::MouseState) -> Option<(i32, i32)> {
        self.current_frame.get_mouse_position_if_button(button)
    }

    pub fn get_mouse_wheel_delta(&self) -> (i32, i32) {
        match (self.current_frame.mouse_wheel_absolute, self.last_frame.mouse_wheel_absolute) {
            ((cx, cy), (lx, ly)) => (cx-lx, cy-ly)
        }
    }

    /// Get the mouse delta since the last frame if the specified button was
    /// down for both the last and current frame.
    pub fn get_mouse_drag_delta(&self, button: sdl2::mouse::MouseState) -> Option<(i32, i32)> {
        match (self.last_frame.get_mouse_position_if_button(button), self.current_frame.get_mouse_position_if_button(button)) {
            (Some((o_x, o_y)), Some((n_x, n_y))) => {
                match (n_x - o_x, n_y - o_y) {
                    // A delta of (0, 0) means there was no change
                    (0, 0) => None,
                    delta => Some(delta)
                }
            },
            _ => None
        }
    }

    pub fn is_keycode_down(&self, keycode: sdl2::keycode::KeyCode) -> bool {
        self.current_frame.is_keycode_down(keycode)
    }

    pub fn is_keycode_newly_down(&self, keycode: sdl2::keycode::KeyCode) -> bool {
        !self.last_frame.is_keycode_down(keycode) && self.current_frame.is_keycode_down(keycode)
    }

    pub fn get_viewport(&self) -> (i32, i32) { self.current_frame.viewport }

    pub fn exit_request(&self) -> bool { self.current_frame.exit_request }

    pub fn get_fps(&self) -> Option<f64> { self.current_frame.fps }
}

pub struct RenderContext<Renderer> {
    _rs: RenderSubsystem,
    window: sdl2::video::Window,
    _gl_context: sdl2::video::GLContext,
    pub renderer: Renderer
}

/// An empty struct that initializes and quits the SDL subsystems in RAII fashion
struct RenderSubsystem;

impl RenderSubsystem {
    fn init() -> RenderSubsystem { sdl2::init_subsystem(sdl2::INIT_VIDEO); RenderSubsystem }
}

impl Drop for RenderSubsystem {
    fn drop(&mut self) { sdl2::quit_subsystem(sdl2::INIT_VIDEO); }
}

impl<Renderer> RenderContext<Renderer> {
    pub fn new<F: FnOnce() -> Renderer>(title: &str, (width, height): (u16, u16), gl_version: (u8, u8), init_renderer: F)
    -> Result<RenderContext<Renderer>, String> {
        let rs = RenderSubsystem::init();

        match gl_version {
            (major, minor) => {
                sdl2::video::gl_set_attribute(sdl2::video::GLAttr::GLContextMajorVersion, major as i32);
                sdl2::video::gl_set_attribute(sdl2::video::GLAttr::GLContextMinorVersion, minor as i32);
            }
        }

        sdl2::video::gl_set_attribute(sdl2::video::GLAttr::GLDepthSize, 24);
        sdl2::video::gl_set_attribute(sdl2::video::GLAttr::GLDoubleBuffer, 1);
        sdl2::video::gl_set_attribute(
            sdl2::video::GLAttr::GLContextProfileMask,
            sdl2::video::GLProfile::GLCoreProfile as i32
        );

        let window = match sdl2::video::Window::new(title, sdl2::video::WindowPos::PosCentered, sdl2::video::WindowPos::PosCentered, width as i32, height as i32, sdl2::video::OPENGL | sdl2::video::SHOWN) {
            Ok(window) => window,
            Err(err) => return Err(format!("failed to create window: {}", err))
        };

        let gl_context = match window.gl_create_context() {
            Ok(context) => context,
            Err(err) => return Err(format!("failed to create context: {}", err))
        };

        let renderer = init_renderer();

        Ok(RenderContext {
            _rs: rs,
            window: window,
            _gl_context: gl_context,
            renderer: renderer
        })
    }

    pub fn get_viewport(&self) -> (i32, i32) {
        match self.window.get_size() {
            (w, h) => (w as i32, h as i32)
        }
    }
}

pub struct Platform<Renderer, Stepper> {
    render_ctx: RenderContext<Renderer>,
    game: Stepper,

    mouse_wheel_absolute: (i32, i32)
}

impl<Stepper, Renderer> Platform<Renderer, Stepper> where
    Stepper: GameStepper<Input>, Renderer: GameRenderer<Stepper, <Stepper as GameStepper<Input>>::StepResult>
{
    pub fn new(game: Stepper, render_ctx: RenderContext<Renderer>) -> Result<Platform<Renderer, Stepper>, String> {
        Ok(Platform {
            game: game,
            render_ctx: render_ctx,
            mouse_wheel_absolute: (0, 0)
        })
    }

    pub fn run(mut self) -> Result<Stepper, String> {
        let step_interval: f64 = 1.0/(self.game.steps_per_second() as f64);

        let mut last_time: f64 = time::precise_time_s();

        let mut input = Input::new(self.render_ctx.get_viewport());

        let mut last_step_result = match self.game.step(&input) {
            PlatformStepResult::Continue(result) => result,
            PlatformStepResult::Exit => panic!("Game exited suddenly")
        };

        let mut step_error: f64 = 0.0;

        let mut fps_meter = FPSMeter::new(1.0);
        let mut fps_meter_change = ValueOnChange::new();

        // Run subsequent frames in a loop
        // The loop always has a "last frame" to refer to
        'main: loop {
            let current_time: f64 = time::precise_time_s();

            let delta: f64 = current_time - last_time;

            step_error += delta;

            if step_error >= step_interval {
                input.push_new_frame(self.event_loop(fps_meter.get_fps()));
            }

            while step_error >= step_interval {
                let result = match self.game.step(&input) {
                    PlatformStepResult::Continue(result) => result,
                    PlatformStepResult::Exit => break 'main
                };

                step_error -= step_interval;
                last_step_result = result;
                input.push_current_frame();
            }

            self.render_ctx.renderer.render(&self.game, &last_step_result);

            self.render_ctx.window.gl_swap_window();

            match self.render_ctx.renderer.frame_limit() {
                Some(fps) => {
                    let d = time::precise_time_s() - current_time;
                    let ms = 1000/fps as i32 - (d*1000.0) as i32;
                    if ms > 0 {
                        sdl2::timer::delay(cast(ms).expect("Overflow"))
                    }
                },
                None => ()
            }

            // Update FPS
            fps_meter.meter_frame();

            // Show FPS when it changes
            if cfg!(feature="print-fps") {
                match fps_meter_change.value(fps_meter.get_fps()) {
                    Some(fps) => match fps {
                        Some(fps) => println!("{} FPS", fps),
                        None => ()  // no FPS recorded
                    },
                    None => ()      //no change
                }
            }

            last_time = current_time;
        }

        Ok(self.game)
    }

    fn event_loop(&mut self, fps: Option<f64>) -> SDLInputFrame {
        let mut exit_request = false;

        'event: loop {
            use sdl2::event::Event;
            use sdl2::keycode::KeyCode;

            match sdl2::event::poll_event() {
                Event::Quit {..} => { exit_request = true; },
                Event::KeyDown { keycode: KeyCode::Escape, .. } => {
                    exit_request = true;
                },
                Event::MouseWheel { x, y, .. } => {
                    let (abs_x, abs_y) = self.mouse_wheel_absolute;
                    self.mouse_wheel_absolute = (abs_x + x, abs_y + y);
                },
                Event::None => { break 'event; },
                _ => ()
            }
        }

        let mouse = match sdl2::mouse::get_mouse_state() {
            (mouse_state, x, y) => (mouse_state, x as i32, y as i32)
        };
        let keys = sdl2::keyboard::get_keyboard_state();

        let mouse_in_focus = match sdl2::mouse::get_mouse_focus() {
            Some(_window) => true,
            None => false
        };

        let mut keyboard = HashSet::new();
        for (scancode, pressed) in keys.iter() {
            if *pressed {
                use std::num::ToPrimitive;

                let keycode = sdl2::keyboard::get_key_from_scancode(*scancode);
                keyboard.insert(keycode.to_u32().expect("Could not convert keycode to u32"));
            }
        }

        SDLInputFrame {
            keyboard: keyboard,
            mouse: Some(mouse),
            mouse_in_focus: mouse_in_focus,
            mouse_wheel_absolute: self.mouse_wheel_absolute,
            viewport: self.render_ctx.get_viewport(),
            exit_request: exit_request,
            fps: fps
        }
    }
}
