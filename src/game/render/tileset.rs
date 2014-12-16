use cgmath::Matrix4;

pub struct TilesetDrawer<'a, F: Fn(u16, &[[f32, ..4], ..4]) + 'a> {
    pub screen_size: (f32, f32),
    pub tile_size: f32,
    pub draw: F,
}

impl<'a, F: Fn(u16, &[[f32, ..4], ..4])> TilesetDrawer<'a, F> {
    pub fn draw(&self, (x, y): (f32, f32), id: u16, flip: (bool, bool), rotate_90: bool) {
        let (width, height) = self.screen_size;
        let model = tile_model(self.tile_size, flip, rotate_90);

        self.draw_tile_offset(&model, id, x - width, y);
        self.draw_tile_offset(&model, id, x, y);
        self.draw_tile_offset(&model, id, x + width, y);
        self.draw_tile_offset(&model, id, x - width, y + height);
        self.draw_tile_offset(&model, id, x, y + height);
        self.draw_tile_offset(&model, id, x + width, y + height);
    }

    fn draw_tile_offset(&self, model: &Matrix4<f32>, id: u16, x: f32, y: f32) {
        use cgmath::{Matrix, Vector3, FixedArray};

        let m = Matrix4::from_translation(&Vector3::new(x, y, 0.0)).mul_m(model);
        (self.draw)(id, m.as_fixed());
    }
}


fn tile_model(tile_size: f32, (flip_x, flip_y): (bool, bool), rotate_90: bool) -> Matrix4<f32> {
    use util::matrix::MatrixBuilder;

    let mut model = Matrix4::identity()
        .scale(tile_size, tile_size, tile_size);

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

    model
}
