use time;

pub struct FPSMeter {
    interval: f64,
    time_measure_begin: f64,
    frames_since: u32,
    last_fps: Option<f64>
}
impl FPSMeter {
    pub fn new(interval: f64) -> FPSMeter {
        FPSMeter {
            interval: interval,
            time_measure_begin: time::precise_time_s(),
            frames_since: 0,
            last_fps: None
        }
    }
    pub fn meter_frame(&mut self) {
        let time = time::precise_time_s();
        let delta = time - self.time_measure_begin;

        if delta >= self.interval {
            self.last_fps = Some(self.frames_since as f64 / self.interval);
            self.time_measure_begin += self.interval;
            self.frames_since = 0;
        }
        self.frames_since += 1;
    }
    pub fn get_fps(&self) -> Option<f64> { self.last_fps }
}

pub struct ValueOnChange<T> {
    old: Option<T>
}
impl<T: Copy + PartialEq> ValueOnChange<T> {
    pub fn new() -> ValueOnChange<T> {
        ValueOnChange { old: None }
    }

    pub fn value(&mut self, value: T) -> Option<T> {
        if Some(value) != self.old {
            // changed
            self.old = Some(value);
            Some(value)
        } else {
            None
        }
    }
}
