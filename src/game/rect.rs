use super::wrapping::Screen;

#[deriving(Copy, Clone)]
pub struct Rect<S> {
    xy: Point<S>,
    w: S,
    h: S
}

#[deriving(Copy, Clone)]
pub struct Point<S> {
    x: S,
    y: S
}

impl Point<f32> {
    pub fn new(screen: &Screen, xy: (f32, f32)) -> Point<f32> {
        let (nx, ny) = screen.wrap_coord(xy);
        Point {
            x: nx,
            y: ny
        }
    }

    pub fn offset(&self, screen: &Screen, x: f32, y: f32) -> Point<f32> {
        Point::new(screen, (self.x + x, self.y + y))
    }

    pub fn set_x(&self, screen: &Screen, x: f32) -> Point<f32> {
        Point::new(screen, (x, self.y))
    }

    pub fn set_y(&self, screen: &Screen, y: f32) -> Point<f32> {
        Point::new(screen, (self.x, y))
    }

    pub fn x(&self) -> f32 { self.x }
    pub fn y(&self) -> f32 { self.y }
    pub fn xy(&self) -> (f32, f32) { (self.x, self.y) }

    pub fn floor(&self, screen: &Screen, multiple: f32) -> Point<f32> {
        use std::num::Float;

        let x = Float::floor(self.x / multiple) * multiple;
        let y = Float::floor(self.y / multiple) * multiple;

        Point::new(screen, (x, y))
    }
}

impl Rect<f32> {
    pub fn from_xywh(screen: &Screen, x: f32, y: f32, w: f32, h: f32) -> Rect<f32> {
        Rect {
            xy: Point::new(screen, (x, y)),
            w: w,
            h: h
        }
    }

    pub fn new(pos: Point<f32>, (w, h): (f32, f32)) -> Rect<f32> {
        Rect {
            xy: pos,
            w: w,
            h: h
        }
    }

    pub fn offset(&self, screen: &Screen, x: f32, y: f32) -> Rect<f32> {
        Rect::new(self.xy.offset(screen, x, y), self.size())
    }

    pub fn set_x(&self, screen: &Screen, x: f32) -> Rect<f32> {
        Rect::new(Point::new(screen, (x, self.xy.y)), self.size())
    }

    pub fn set_y(&self, screen: &Screen, y: f32) -> Rect<f32> {
        Rect::new(Point::new(screen, (self.xy.x, y)), self.size())
    }

    pub fn set_width(&self, width: f32) -> Rect<f32> {
        Rect::new(self.xy, (width, self.height()))
    }

    pub fn set_height(&self, height: f32) -> Rect<f32> {
        Rect::new(self.xy, (self.width(), height))
    }

    pub fn x(&self) -> f32 { self.xy.x }
    pub fn y(&self) -> f32 { self.xy.y }
    pub fn width(&self) -> f32 { self.w }
    pub fn height(&self) -> f32 { self.h }

    pub fn left_top(&self) -> Point<f32> { self.xy }
    pub fn ltrb(&self) -> (f32, f32, f32, f32) {
        (self.xy.x, self.xy.y, self.xy.x + self.w, self.xy.y + self.h)
    }
    pub fn size(&self) -> (f32, f32) { (self.w, self.h) }
}
