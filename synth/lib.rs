#![feature(if_let)]

use noise::Noise;
use pulse::Pulse;
use triangle::Triangle;

pub mod effect;

mod noise;
mod pulse;
mod triangle;

struct Generator {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise
}

#[deriving(Clone, Copy)]
pub struct ChannelEffectOut {
    pub freq: f32,
    pub volume: f32,
    pub duty: f32
}

trait Channel {
    fn generate(&mut self, out: &mut [f32]);
    fn apply(&mut self, o: ChannelEffectOut, sink_freq: f32);
    fn silence(&mut self);
}

impl Generator {
    pub fn new() -> Generator {
        use std::default::Default;

        Generator {
            pulse1: Default::default(),
            pulse2: Default::default(),
            triangle: Default::default(),
            noise: Noise::new(),
        }
    }

    pub fn generate(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() { *x = 0.0; }

        self.pulse1.generate(out);
        self.pulse2.generate(out);
        self.triangle.generate(out);
        self.noise.generate(out);
    }
}

pub struct Controller<'a> {
    pub volume: f32,

    generator: Generator,

    sink_freq: f32,

    /// How long each engine tick should be, in samples
    tick_length: uint,
    /// How many samples are left. Decrements and wraps as generation occurs.
    /// Maximum is tick_length.
    tick_remainder: uint,

    channel_effects: Vec<[Option<Box<Iterator<ChannelEffectOut> + 'a>>, ..4]>
}

impl<'a> Controller<'a> {
    pub fn new(sink_freq: f32, tick_length_s: f32, layers: uint) -> Controller<'a> {
        use std::default::Default;

        let tick_length = (tick_length_s * sink_freq) as uint;

        Controller {
            volume: 1.0,
            generator: Generator::new(),
            sink_freq: sink_freq,
            tick_length: tick_length,
            tick_remainder: tick_length,
            channel_effects: Vec::from_fn(layers, |_| [None, None, None, None])
        }
    }

    pub fn generate(&mut self, out: &mut [f32]) {
        let mut offset = 0;
        while offset < out.len() {
            offset += self.generate_tick(out, offset);
        }

        for x in out.iter_mut() { *x *= self.volume; }
    }

    fn get_layer(&mut self, layer: uint) -> &mut [Option<Box<Iterator<ChannelEffectOut> + 'a>>, ..4] {
        self.channel_effects.index_mut(&layer)
    }

    pub fn set_effect(&mut self, layer: uint, channel: uint, effect: Box<Iterator<ChannelEffectOut> + 'a>) {
        let mut layer = self.get_layer(layer);
        layer[channel] = Some(effect);
    }

    pub fn clear_effect(&mut self, layer: uint, channel: uint) {
        let mut layer = self.get_layer(layer);
        layer[channel] = None;
    }

    fn generate_tick(&mut self, out: &mut [f32], offset: uint) -> uint {
        use std::cmp::min;

        if self.tick_remainder == self.tick_length {
            // First sample of the tick - iterate the tick.
            self.tick();
        }

        let samples = min(out.len() - offset, self.tick_remainder);
        self.tick_remainder = match self.tick_remainder - samples {
            // Tick ended - wrap around
            0 => self.tick_length,
            r => r
        };

        let o = out.slice_mut(offset, offset + samples);
        self.generator.generate(o);

        samples
    }

    fn tick(&mut self) {
        fn tick_channel<'a, T: Channel>(c: &mut T, current_effect: &mut Option<Box<Iterator<ChannelEffectOut> + 'a>>, sink_freq: f32) {
            let finished = if let &Some(ref mut effect) = current_effect {
                match effect.next() {
                    Some(effect_out) => {
                        c.apply(effect_out, sink_freq);
                        false
                    },
                    None => true
                }
            } else {
                false
            };

            if finished {
                *current_effect = None;
            }
        }

        self.generator.pulse1.silence();
        self.generator.pulse2.silence();
        self.generator.triangle.silence();
        self.generator.noise.silence();

        for layer in self.channel_effects.iter_mut() {
            tick_channel(&mut self.generator.pulse1, &mut layer[0], self.sink_freq);
            tick_channel(&mut self.generator.pulse2, &mut layer[1], self.sink_freq);
            tick_channel(&mut self.generator.triangle, &mut layer[2], self.sink_freq);
            tick_channel(&mut self.generator.noise, &mut layer[3], self.sink_freq);
        }
    }
}
