pub struct Poof {
    pub x: f32,
    pub y: f32,
    pub phase: f32
}

pub struct DynamicItems {
    pub poofs: Vec<Poof>
}

impl DynamicItems {
    pub fn new() -> DynamicItems {
        DynamicItems {
            poofs: vec![]
        }
    }

    pub fn trigger(&mut self, id: u8) {

    }

    pub fn add_poof(&mut self, x: f32, y: f32) {
        self.poofs.push(Poof {
            x: x,
            y: y,
            phase: 0.0
        });
    }

    pub fn step_poofs(&mut self) {
        let new_poofs = self.poofs.iter().filter_map(|poof| {
            let phase = poof.phase + 0.05;
            if phase >= 1.0 {
                None
            } else {
                Some(Poof {
                    x: poof.x,
                    y: poof.y,
                    phase: phase
                })
            }
        }).collect();

        self.poofs = new_poofs;
    }
}
