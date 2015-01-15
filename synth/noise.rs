//! Essentially a pulse wave with pseudo-random periods (NES style)
//!
//! The PRNG is meant to be lousy (bad entropy), as we want a distinguishable
//! pattern for the noise.

use super::{Channel, ChannelEffectOut};

pub struct Noise {
    random_iterator: RepeatingIterator<f32>,
    base_phase_inc: f32,
    phase: f32,
    volume: f32
}

impl Noise {
    pub fn new() -> Noise {
        use std::rand::{Rng, XorShiftRng};

        let mut rng = XorShiftRng::new_unseeded();

        let iter = rng.gen_iter().take(10000).map(|v: f32| {
            v*2.0 - 1.0
        });

        Noise {
            random_iterator: RepeatingIterator::new(iter.collect()),
            base_phase_inc: 0.0,
            phase: 0.0,
            volume: 1.0
        }
    }
}

impl Channel for Noise {
    fn generate(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x += *self.random_iterator.current() * self.volume;
            self.phase += self.base_phase_inc;
            while self.phase >= 1.0 {
                self.random_iterator.next();
                self.phase -= 1.0;
            }
        }
    }

    fn apply(&mut self, o: ChannelEffectOut, sink_freq: f32) {
        self.volume = o.volume;
        self.base_phase_inc = o.freq / sink_freq;
    }

    fn silence(&mut self) {
        self.volume = 0.0;
        self.base_phase_inc = 0.0;
    }
}

struct RepeatingIterator<T> {
    vec: Vec<T>,
    index: uint
}

impl<T> RepeatingIterator<T> {
    pub fn new(vec: Vec<T>) -> RepeatingIterator<T> {
        assert!(vec.len() > 0);
        RepeatingIterator {
            vec: vec,
            index: 0
        }
    }

    pub fn current(&self) -> &T { self.vec.get(self.index).unwrap() }
}

impl<T: Clone> Iterator for RepeatingIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let value = self.vec.get(self.index).unwrap();
        self.index = (self.index + 1) % self.vec.len();
        Some(value.clone())
    }
}
