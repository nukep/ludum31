extern crate sdl2;
extern crate time;

pub mod sdl2_opengl;
mod fps_meter;

pub enum PlatformStepResult<SR> {
    Continue(SR),
    Exit
}

pub trait GameStepper<I, SR> {
    fn steps_per_second() -> u32;
    fn step(&mut self, input: &I) -> PlatformStepResult<SR>;
}

pub trait GameRenderer<R, SR> {
    fn render(&self, step_result: &SR, ctx: &mut R);

    fn frame_limit(&self) -> Option<u32> { None }
}
