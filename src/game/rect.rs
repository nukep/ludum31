use super::level::Level;

pub trait RectExt {
    fn offset(&self, level: &Level, x: f32, y: f32) -> Self;
}

impl RectExt for (f32, f32, f32, f32) {
    fn offset(&self, level: &Level, x: f32, y: f32) -> (f32, f32, f32, f32) {
        let (x1, y1, x2, y2) = *self;
        let (w, h) = (x2 - x1, y2 - y1);
        let (nx1, ny1) = level.wrap_coordinates((x1+x, y1+y));
        let (nx2, ny2) = (nx1 + w, ny1 + h);

        (nx1, ny1, nx2, ny2)
    }
}
