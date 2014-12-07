use sdl2;
use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioDevice};
use synth::Controller;
use synth::ChannelEffectOut;
use synth::effect::{SweepEffect, RandomEffect};

/// An empty struct that initializes and quits the SDL subsystems in RAII fashion
struct AudioSubsystem;

impl AudioSubsystem {
    fn init() -> AudioSubsystem { sdl2::init_subsystem(sdl2::INIT_AUDIO); AudioSubsystem }
}

impl Drop for AudioSubsystem {
    fn drop(&mut self) { sdl2::quit_subsystem(sdl2::INIT_AUDIO); }
}

pub struct Audio {
    _subsystem: AudioSubsystem,
    device: AudioDevice<MyCallback>
}

impl Audio {
    pub fn new() -> Result<Audio, String> {
        let subsystem = AudioSubsystem::init();
        let freq = 44100;

        let mut controller = Controller::new(freq as f32, 1.0/60.0);

        let desired_spec = AudioSpecDesired {
            freq: freq,
            channels: 1,
            callback: MyCallback { controller: controller }
        };

        match desired_spec.open_audio_device(None, false) {
            Ok(device) => {
                device.resume();
                Ok(Audio {
                    _subsystem: subsystem,
                    device: device
                })
            },
            Err(s) => Err(format!("Error initializing audio: {}", s))
        }
    }

    // TODO: Don't lock the audio thread, enqueue instead

    pub fn jump(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(0, box SweepEffect {
            freq: (2000.0, 50.0),
            volume: (0.4, 0.5),
            duty: (0.5, 0.25),
            ticks: 30,
            quantize: 1
        }.iter());
    }

    pub fn poof(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(2, box SweepEffect {
            freq: (400.0, 10.0),
            volume: (0.3, 0.5),
            duty: (0.5, 0.5),
            ticks: 16,
            quantize: 1
        }.iter());

        controller.set_effect(3, box SweepEffect {
                freq: (50.0, 2000.0),
                volume: (0.3, 0.5),
                duty: (0.5, 0.5),
                ticks: 16,
                quantize: 1
        }.iter());
    }

    pub fn explode(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(3, box SweepEffect {
            freq: (1000.0, 50.0),
            volume: (0.7, 0.2),
            duty: (0.5, 0.5),
            ticks: 60,
            quantize: 2
        }.iter());
    }

    pub fn item_get(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(0, box SweepEffect {
            freq: (100.0, 500.0),
            volume: (0.4, 0.7),
            duty: (0.25, 0.5),
            ticks: 20,
            quantize: 2
        }.iter());
    }

    pub fn start_walking(&mut self) {
        use std::iter::repeat;

        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(3, box repeat(0u).flat_map(|_| {
            SweepEffect {
                freq: (2000.0, 100.0),
                volume: (0.3, 0.0),
                duty: (0.5, 0.5),
                ticks: 8,
                quantize: 1
            }.iter()
        }));
    }

    pub fn stop_walking(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);
        controller.clear_effect(3);
    }
}

struct MyCallback {
    controller: Controller<'static>
}
impl AudioCallback<f32> for MyCallback {
    fn callback(&mut self, out: &mut [f32]) {
        self.controller.generate(out);
    }
}
