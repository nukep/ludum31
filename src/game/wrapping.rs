#[deriving(Copy)]
pub struct Screen {
    pub width: f32,
    pub height: f32
}

impl Screen {
    pub fn new(width: f32, height: f32) -> Screen {
        Screen {
            width: width,
            height: height
        }
    }

    pub fn size(&self) -> (f32, f32) { (self.width, self.height) }

    pub fn wrap_coord(&self, coord: (f32, f32)) -> (f32, f32) {
        let (x, y) = coord;
        ((x + self.width) % self.width, (y + self.height) % self.height)
    }

    pub fn relative_wrap(&self, origin: (f32, f32), coord: (f32, f32)) -> (f32, f32) {
        let (width, height) = self.size();
        let (origin_x, origin_y) = origin;
        let (coord_x, coord_y) = coord;
        let (mut new_coord_x, mut new_coord_y) = (coord_x - origin_x, coord_y - origin_y);

        if new_coord_x < -width/2.0 { new_coord_x += width }
        if new_coord_x >= width/2.0 { new_coord_x -= width }
        if new_coord_y < -height/2.0 { new_coord_y += height }
        if new_coord_y >= height/2.0 { new_coord_y -= height }

        (new_coord_x, new_coord_y)
    }
}
