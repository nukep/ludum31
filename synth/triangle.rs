use super::{Channel, ChannelEffectOut};
use std::default::Default;

pub struct Triangle {
    pub volume: f32,

    phase: f32,
    phase_inc: f32
}

impl Default for Triangle {
    fn default() -> Triangle {
        Triangle {
            volume: 0.0,
            phase: 0.0,
            phase_inc: 0.0
        }
    }
}

impl Channel for Triangle {
    fn generate(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x += match self.phase {
                0.0...0.5 => 4.0*self.phase - 1.0,
                _ => 3.0 - 4.0*self.phase
            } * self.volume;
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }

    fn apply(&mut self, o: ChannelEffectOut, sink_freq: f32) {
        self.volume = o.volume;
        self.phase_inc = o.freq / sink_freq;
    }

    fn silence(&mut self) {
        self.volume = 0.0;
        self.phase_inc = 0.0;
    }
}
