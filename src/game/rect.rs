use super::level::Level;

pub trait RectExt<T> {
    fn offset(&self, level: &Level, x: f32, y: f32) -> (T, T, T, T);
    fn set_x(&self, level: &Level, x: f32) -> (T, T, T, T);
    fn set_y(&self, level: &Level, y: f32) -> (T, T, T, T);
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn width(&self) -> T;
    fn height(&self) -> T;

    fn left_top(&self) -> (T, T) { (self.x(), self.y()) }
    fn size(&self) -> (T, T) { (self.width(), self.height()) }
}

impl RectExt<f32> for (f32, f32, f32, f32) {
    fn offset(&self, level: &Level, x: f32, y: f32) -> (f32, f32, f32, f32) {
        let (x1, y1, x2, y2) = *self;
        let (w, h) = (x2 - x1, y2 - y1);
        let (nx1, ny1) = level.wrap_coordinates((x1+x, y1+y));
        let (nx2, ny2) = (nx1 + w, ny1 + h);

        (nx1, ny1, nx2, ny2)
    }

    fn set_x(&self, level: &Level, x: f32) -> (f32, f32, f32, f32) {
        self.offset(level, x - self.x(), 0.0)
    }

    fn set_y(&self, level: &Level, y: f32) -> (f32, f32, f32, f32) {
        self.offset(level, 0.0, y - self.y())
    }

    fn x(&self) -> f32 { self.val0() }
    fn y(&self) -> f32 { self.val1() }
    fn width(&self) -> f32 { self.val2() - self.val0() }
    fn height(&self) -> f32 { self.val3() - self.val1() }
}
