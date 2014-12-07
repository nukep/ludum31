use super::{Channel, ChannelEffectOut};
use std::default::Default;

pub struct Pulse {
    pub duty: f32,
    pub volume: f32,
    pub phase_inc: f32,

    phase: f32,
}

impl Default for Pulse {
    fn default() -> Pulse {
        Pulse {
            duty: 0.5,
            volume: 0.0,
            phase_inc: 0.0,
            phase: 0.0
        }
    }
}

impl Channel for Pulse {
    fn generate(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x += if self.phase <= self.duty { 1.0 }
                else { -1.0 } * self.volume;
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }

    fn apply(&mut self, o: ChannelEffectOut, sink_freq: f32) {
        self.duty = o.duty;
        self.volume = o.volume;
        self.phase_inc = o.freq / sink_freq;
    }

    fn silence(&mut self) {
        self.volume = 0.0;
        self.phase_inc = 0.0;
    }
}
