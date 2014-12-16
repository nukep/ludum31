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

pub trait GameRenderer<GameStepper, StepResult> {
    fn render(&mut self, game: &GameStepper, &step_result: &StepResult);

    fn frame_limit(&self) -> Option<u32> { None }
}
