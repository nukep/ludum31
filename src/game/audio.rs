use sdl2;
use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioDevice};
use synth::Controller;
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

static MUSIC: uint = 0;
static NOISE_FX: uint = 1;
static PRIMARY_FX: uint = 2;
static LAYERS: uint = 3;

impl Audio {
    pub fn new() -> Result<Audio, String> {
        let subsystem = AudioSubsystem::init();
        let freq = 44100;

        let mut controller = Controller::new(freq as f32, 1.0/60.0, LAYERS);
        controller.volume = 0.5;

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

        controller.set_effect(PRIMARY_FX, 0, box SweepEffect {
            freq: (200.0, 1000.0),
            volume: (0.5, 0.0),
            duty: (0.25, 0.25),
            ticks: 35,
            quantize: 2
        }.iter());
    }

    pub fn poof(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(PRIMARY_FX, 2, box SweepEffect {
            freq: (400.0, 10.0),
            volume: (0.3, 0.5),
            duty: (0.5, 0.5),
            ticks: 16,
            quantize: 1
        }.iter());

        controller.set_effect(PRIMARY_FX, 3, box SweepEffect {
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

        controller.set_effect(PRIMARY_FX, 3, box SweepEffect {
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

        controller.set_effect(PRIMARY_FX, 0, box SweepEffect {
            freq: (100.0, 500.0),
            volume: (0.4, 0.7),
            duty: (0.25, 0.5),
            ticks: 20,
            quantize: 2
        }.iter());
    }

    pub fn key_get(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(PRIMARY_FX, 0, box vec![800.0, 1200.0].into_iter().flat_map(|freq| {
            SweepEffect {
                freq: (freq, freq),
                volume: (0.5, 0.0),
                duty: (0.5, 0.5),
                ticks: 6,
                quantize: 1
            }.iter()
        }));
    }

    pub fn unlock(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(PRIMARY_FX, 3, box  vec![400.0, 1200.0].into_iter().flat_map(|freq| {
            SweepEffect {
                freq: (freq, freq),
                volume: (0.5, 0.0),
                duty: (0.5, 0.5),
                ticks: 6,
                quantize: 1
            }.iter()
        }));

        controller.set_effect(PRIMARY_FX, 2, box vec![400.0, 1200.0].into_iter().flat_map(|freq| {
            SweepEffect {
                freq: (freq, freq*0.1),
                volume: (0.5, 0.0),
                duty: (0.5, 0.5),
                ticks: 6,
                quantize: 1
            }.iter()
        }));
    }

    pub fn coin(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(PRIMARY_FX, 0, box vec![(493.0*2.0, 1, 0.3), (659.0*2.0, 12, 0.0)].into_iter().flat_map(|(freq, length, to_volume)| {
            SweepEffect {
                freq: (freq*0.95, freq*0.95),
                volume: (0.5, to_volume),
                duty: (0.5, 0.5),
                ticks: 5*length,
                quantize: 2
            }.iter()
        }));
    }

    pub fn nothing(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(PRIMARY_FX, 0, box vec![2, 12].into_iter().flat_map(|length| {
            let freq = 50.0;
            SweepEffect {
                freq: (freq, freq),
                volume: (0.5, 0.5),
                duty: (0.25, 0.25),
                ticks: 2*length,
                quantize: 2
            }.iter().chain(
                SweepEffect {
                    freq: (1.0, 1.0),
                    volume: (0.0, 0.0),
                    duty: (0.5, 0.5),
                    ticks: 5,
                    quantize: 1
                }.iter()
            )
        }));
    }

    pub fn fire(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(PRIMARY_FX, 0, box SweepEffect {
            freq: (200.0, 10.0),
            volume: (1.0, 0.8),
            duty: (0.5, 0.5),
            ticks: 8,
            quantize: 1
        }.iter());
        controller.set_effect(PRIMARY_FX, 2, box SweepEffect {
            freq: (2000.0, 200.0),
            volume: (1.0, 0.8),
            duty: (0.5, 0.5),
            ticks: 8,
            quantize: 1
        }.iter());
        controller.set_effect(PRIMARY_FX, 3, box SweepEffect {
            freq: (4000.0, 200.0),
            volume: (0.5, 0.2),
            duty: (0.5, 0.5),
            ticks: 16,
            quantize: 4
        }.iter());
    }

    pub fn die(&mut self) {
        self.explode();
    }

    pub fn start_drilling(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(NOISE_FX, 3, box RandomEffect {
            freq: (900.0, 950.0),
            volume: (0.3, 0.05),
            duty: (0.5, 0.5),
        }.iter().flat_map(|e| {
            SweepEffect {
                freq: (e.freq, e.freq),
                volume: (0.0, e.volume),
                duty: (0.5, 0.5),
                ticks: 4,
                quantize: 1
            }.iter()
        }));
    }

    pub fn stop_drilling(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);
        controller.clear_effect(NOISE_FX, 3);
    }

    pub fn start_walking(&mut self) {
        use std::iter::repeat;

        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);

        controller.set_effect(NOISE_FX, 3, box repeat(0u).flat_map(|_| {
            SweepEffect {
                freq: (500.0, 100.0),
                volume: (0.3, 0.0),
                duty: (0.5, 0.5),
                ticks: 6,
                quantize: 1
            }.iter().chain(SweepEffect {
                freq: (2000.0, 100.0),
                volume: (0.0, 0.0),
                duty: (0.5, 0.5),
                ticks: 8,
                quantize: 1
            }.iter())
        }));
    }

    pub fn stop_walking(&mut self) {
        let mut lock = self.device.lock();
        let mut controller = &mut ((*lock).controller);
        controller.clear_effect(NOISE_FX, 3);
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
