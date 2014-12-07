//! Effects are based on iterators.

use super::{ChannelEffectOut};

use std::iter::{Take, Repeat};
use std::rand::{XorShiftRng};

pub struct RandomEffect {
    pub freq: (f32, f32),
    pub volume: (f32, f32),
    pub duty: (f32, f32)
}

impl RandomEffect {
    pub fn iter(&self) -> RandomEffectIterator {
        use std::num::Float;

        fn sort(r: (f32, f32)) -> (f32, f32) {
            if r.val0() > r.val1() { (r.val1(), r.val0()) }
            else { r }
        }

        let (freq_a, freq_b) = self.freq;

        RandomEffectIterator {
            rng: XorShiftRng::new_unseeded(),
            freq: sort((freq_a.log2(), freq_b.log2())),
            volume: sort(self.volume),
            duty: sort(self.duty)
        }
    }
}

pub struct RandomEffectIterator {
    rng: XorShiftRng,
    freq: (f32, f32),
    volume: (f32, f32),
    duty: (f32, f32)
}

impl Iterator<ChannelEffectOut> for RandomEffectIterator {
    fn next(&mut self) -> Option<ChannelEffectOut> {
        use std::rand::Rng;

        fn gen(rng: &mut XorShiftRng, r: (f32, f32)) -> f32 {
            let (a, b) = r;
            if a == b { a }
            else { rng.gen_range(a, b) }
        }

        fn freq_log_gen(rng: &mut XorShiftRng, r: (f32, f32)) -> f32 {
            use std::num::Float;
            Float::powf(2.0, gen(rng, r))
        }

        Some(ChannelEffectOut {
            freq: freq_log_gen(&mut self.rng, self.freq),
            volume: gen(&mut self.rng, self.volume),
            duty: gen(&mut self.rng, self.duty)
        })
    }
}

#[deriving(Copy, Clone)]
pub struct SweepEffect {
    pub freq: (f32, f32),
    pub volume: (f32, f32),
    pub duty: (f32, f32),
    pub ticks: uint,
    pub quantize: uint
}

impl SweepEffect {
    pub fn reverse(&self) -> SweepEffect {
        fn rev(input: (f32, f32)) -> (f32, f32) {
            let (a,b) = input;
            (b,a)
        }

        SweepEffect {
            freq: rev(self.freq),
            volume: rev(self.volume),
            duty: rev(self.duty),
            ticks: self.ticks,
            quantize: self.quantize
        }
    }

    pub fn iter(&self) -> SweepEffectIterator {
        use std::num::Float;

        let (freq_a, freq_b) = self.freq;

        SweepEffectIterator {
            freq_log: (freq_a.log2(), freq_b.log2()),
            volume: self.volume,
            duty: self.duty,
            ticks: self.ticks,
            quantize: self.quantize,
            tick: 0
        }
    }

    /// Iterate iter(), then reverse().iter()
    pub fn iter_triangle(&self) -> SweepEffectTriangleIterator {
        SweepEffectTriangleIterator {
            forward: self.iter(),
            reverse: self.reverse().iter(),
            on_reverse: false
        }
    }
}

#[deriving(Copy, Clone)]
pub struct SweepEffectIterator {
    freq_log: (f32, f32),
    volume: (f32, f32),
    duty: (f32, f32),
    ticks: uint,
    quantize: uint,
    tick: uint
}

impl Iterator<ChannelEffectOut> for SweepEffectIterator {
    fn next(&mut self) -> Option<ChannelEffectOut> {
        fn lerp(ab: (f32, f32), p: f32) -> f32 {
            let (a, b) = ab;
            (b-a)*p + a
        }

        fn freq_log_lerp(ab: (f32, f32), p: f32) -> f32 {
            use std::num::Float;
            Float::powf(2.0, lerp(ab, p))
        }

        if self.tick >= self.ticks {
            None
        } else {
            // Truncate the tick down to a multiple of quantize.
            let quantized_tick = (self.tick / self.quantize) * self.quantize;
            // Progress of the sweep is 0 to 1
            let progress = quantized_tick as f32 / (self.ticks-1) as f32;
            self.tick += 1;
            Some(ChannelEffectOut {
                freq: freq_log_lerp(self.freq_log, progress),
                volume: lerp(self.volume, progress),
                duty: lerp(self.duty, progress)
            })
        }
    }
}



#[deriving(Copy, Clone)]
pub struct SweepEffectTriangleIterator {
    forward: SweepEffectIterator,
    reverse: SweepEffectIterator,
    on_reverse: bool
}

impl Iterator<ChannelEffectOut> for SweepEffectTriangleIterator {
    fn next(&mut self) -> Option<ChannelEffectOut> {
        if self.on_reverse {
            self.reverse.next()
        } else {
            match self.forward.next() {
                Some(v) => Some(v),
                None => {
                    self.on_reverse = true;
                    self.next()
                }
            }
        }
    }
}
