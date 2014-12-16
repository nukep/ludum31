pub struct TilesetDrawer<'a, F: Fn(u16, &[[f32, ..4], ..4]) + 'a> {
    pub screen_size: (f32, f32),
    pub tile_size: f32,
    pub draw: F,
}

impl<'a, F: Fn(u16, &[[f32, ..4], ..4])> TilesetDrawer<'a, F> {
    pub fn draw_tile(&self, (x, y): (f32, f32), id: u16, flip: (bool, bool), rotate_90: bool) {
        let (width, height) = self.screen_size;

        self.draw_tile_single(x - width, y - height, id, flip, rotate_90);
        self.draw_tile_single(x, y - height, id, flip, rotate_90);
        self.draw_tile_single(x + width, y - height, id, flip, rotate_90);
        self.draw_tile_single(x - width, y, id, flip, rotate_90);
        self.draw_tile_single(x, y, id, flip, rotate_90);
        self.draw_tile_single(x + width, y, id, flip, rotate_90);
        self.draw_tile_single(x - width, y + height, id, flip, rotate_90);
        self.draw_tile_single(x, y + height, id, flip, rotate_90);
        self.draw_tile_single(x + width, y + height, id, flip, rotate_90);
    }

    fn draw_tile_single(&self, x: f32, y: f32, id: u16, (flip_x, flip_y): (bool, bool), rotate_90: bool) {
        use cgmath::{Matrix4, FixedArray};
        use util::matrix::MatrixBuilder;

        let mut model = Matrix4::identity()
            .translate(x, y, 0.0)
            .scale(self.tile_size, self.tile_size, self.tile_size);

        if flip_x {
            model = model
                .translate(1.0, 0.0, 0.0)
                .scale(-1.0, 1.0, 1.0);
        }
        if flip_y {
            model = model
                .translate(0.0, 1.0, 0.0)
                .scale(1.0, -1.0, 1.0);
        }
        if rotate_90 {
            use std::f32::consts::FRAC_PI_2;
            model = model
                .translate(1.0, 0.0, 0.0)
                .rotate_z(FRAC_PI_2);
        }

        (self.draw)(id, model.as_fixed());
    }
}
