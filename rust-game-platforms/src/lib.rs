extern crate sdl2;
extern crate time;

pub mod sdl2_opengl;
mod fps_meter;

pub enum PlatformStepResult<StepResult> {
    Continue(StepResult),
    Exit
}

pub trait GameStepper<Input> {
    type StepResult;

    fn steps_per_second(&self) -> u32;
    fn step(&mut self, input: &Input) -> PlatformStepResult<Self::StepResult>;
}

pub trait GameRenderer<GameStepper, StepResult> {
    fn render(&mut self, game: &GameStepper, &step_result: &StepResult);

    fn frame_limit(&self) -> Option<u32> { None }
}
