use super::wrapping::Screen;

/// Should've used something like this at the beginning
// pub struct Rect<S> {
//     x: S,
//     y: S,
//     w: S,
//     h: S
// }
//
// pub struct Point<S> {
//     x: S,
//     y: S
// }
//
// impl Rect<f32> {
//     pub fn new_with_size(level: &Level, x: f32, y: f32, w: f32, h: f32) {
//         let (nx, ny) = level.wrap_coordinates((x, y));
//         Rect {
//             x: nx,
//             y: ny,
//             w: w,
//             h: h
//         }
//     }
// }
//
// impl RectExt<f32, Point<f32> for Rect<f32> {
//     fn offset(&self, level: &Level, x: f32, y: f32) -> Rect<f32> {
//
//     }
// }

pub trait RectExt<S, P> {
    fn offset(&self, screen: &Screen, x: S, y: S) -> Self;
    fn set_x(&self, screen: &Screen, x: S) -> Self;
    fn set_y(&self, screen: &Screen, y: S) -> Self;
    fn set_width(&self, width: S) -> Self;
    fn set_height(&self, height: S) -> Self;
    fn x(&self) -> S;
    fn y(&self) -> S;
    fn width(&self) -> S;
    fn height(&self) -> S;

    fn left_top(&self) -> P;
    fn size(&self) -> P;
}

impl RectExt<f32, (f32, f32)> for (f32, f32, f32, f32) {
    fn offset(&self, screen: &Screen, x: f32, y: f32) -> (f32, f32, f32, f32) {
        let (x1, y1, x2, y2) = *self;
        let (w, h) = (x2 - x1, y2 - y1);
        let (nx1, ny1) = screen.wrap_coord((x1+x, y1+y));
        let (nx2, ny2) = (nx1 + w, ny1 + h);

        (nx1, ny1, nx2, ny2)
    }

    fn set_x(&self, screen: &Screen, x: f32) -> (f32, f32, f32, f32) {
        self.offset(screen, x - self.x(), 0.0)
    }

    fn set_y(&self, screen: &Screen, y: f32) -> (f32, f32, f32, f32) {
        self.offset(screen, 0.0, y - self.y())
    }

    fn set_width(&self, width: f32) -> (f32, f32, f32, f32) {
        let (x1, y1, _, y2) = *self;
        (x1, y1, x1+width, y2)
    }

    fn set_height(&self, height: f32) -> (f32, f32, f32, f32) {
        let (x1, y1, x2, _) = *self;
        (x1, y1, x2, y1 + height)
    }

    fn x(&self) -> f32 { self.0 }
    fn y(&self) -> f32 { self.1 }
    fn width(&self) -> f32 { self.2 - self.0 }
    fn height(&self) -> f32 { self.3 - self.1 }

    fn left_top(&self) -> (f32, f32) { (self.x(), self.y()) }
    fn size(&self) -> (f32, f32) { (self.width(), self.height()) }
}
