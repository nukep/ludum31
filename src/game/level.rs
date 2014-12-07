use std::num::FromStrRadix;
use serialize;

pub struct Tile {
    pub tile_type: TileType,
    pub flip_x: bool,
    pub flip_y: bool
}

pub struct TileType {
    pub id: u16,
    pub is_blocking: bool
}

impl TileType {
    pub fn from_id(id: u16) -> TileType {
        let is_blocking = match id {
            0 => false,
            // Brown treasure
            5 => false,
            // Blue treasure
            10 => false,
            _ => true
        };

        TileType {
            id: id,
            is_blocking: is_blocking
        }
    }
}

pub struct Switch {
    pub x: f32,
    pub y: f32,
    pub trigger: u8
}

pub struct Chest {
    pub x: f32,
    pub y: f32,
    pub trigger: Option<u8>,
    pub triggered_by: Option<u8>,
    pub poof: bool,
    pub is_static: bool,
    pub fall_distance: f32
}

pub struct Beanstalk {
    pub x: f32,
    pub y: f32,
    pub height: f32,
    pub triggered_by: Option<u8>,
    pub poof: bool,
}

pub struct Level {
    pub width: u8,
    pub height: u8,
    pub player_start_pos: (f32, f32),
    tiles: Vec<Tile>,
    pub switches: Vec<Switch>,
    pub chests: Vec<Chest>,
    pub beanstalks: Vec<Beanstalk>
}

impl Level {
    pub fn load() -> Level {
        let level_data = include_str!("../../assets/level.json");

        parse_from_json(level_data)
    }

    pub fn get_tile(&self, x: u8, y: u8) -> &Tile {
        let offset = y as uint * self.width as uint + x as uint;
        self.tiles.index(&offset)
    }

    pub fn iter(&self) -> LevelTileIterator {
        LevelTileIterator {
            tiles: &self.tiles,
            width: self.width as uint,
            index: 0
        }
    }

    /// Direction: (right?, down?)
    /// Returns: new top_left
    pub fn collision_tile(&self, left_top: (f32, f32), direction: (Option<bool>, Option<bool>)) -> Option<(f32, f32)> {
        use std::num::Float;

        let (x, y) = left_top;
        let (left, top) = {
            (Float::floor(x / 16.0) as int, Float::floor(y / 16.0) as int)
        };
        let (right, bottom) = {
            let (mut r, mut b) = (Float::floor((x + 15.0) / 16.0) as int, Float::floor((y + 15.0) / 16.0) as int);

            // Wrapping
            if r >= self.width as int { r -= self.width as int }
            if b >= self.height as int { b -= self.height as int }

            (r, b)
        };

        let tiles: [&Tile, ..4] = [
            self.get_tile(left as u8, top as u8),
            self.get_tile(right as u8, top as u8),
            self.get_tile(left as u8, bottom as u8),
            self.get_tile(right as u8, bottom as u8),
        ];

        let mut nudge = false;

        for t in tiles.iter() {
            if t.tile_type.is_blocking {
                nudge = true;
            }
        }

        if nudge {
            let (nudge_right, nudge_bottom) = direction;
            let new_left = match nudge_right {
                None => x,
                Some(false) => right as f32 * 16.0,
                Some(true) => left as f32 * 16.0
            };
            let new_top = match nudge_bottom {
                None => y,
                Some(false) => bottom as f32 * 16.0,
                Some(true) => top as f32 * 16.0
            };
            Some((new_left, new_top))
        } else {
            None
        }
    }

    pub fn level_size_as_u32(&self) -> (u32, u32) {
        (self.width as u32 * 16, self.height as u32 * 16)
    }

    pub fn level_size_as_f32(&self) -> (f32, f32) {
        (self.width as f32 * 16.0, self.height as f32 * 16.0)
    }

    pub fn wrap_coordinates(&self, coord: (f32, f32)) -> (f32, f32) {
        let (x, y) = coord;
        let (w, h) = self.level_size_as_f32();
        ((x+w) % w, (y+h) % h)
    }
}

pub struct LevelTileIterator<'a> {
    tiles: &'a Vec<Tile>,
    width: uint,
    index: uint
}

impl<'a> Iterator<(u8, u8, &'a Tile)> for LevelTileIterator<'a> {
    fn next(&mut self) -> Option<(u8, u8, &'a Tile)> {
        if self.index >= self.tiles.len() {
            None
        } else {
            let tile = self.tiles.index(&self.index);
            let (x, y) = (self.index % self.width, self.index / self.width);
            self.index += 1;

            Some((x as u8, y as u8, tile))
        }
    }
}

fn parse_from_json(input: &str) -> Level {
    use std::str::{from_utf8, FromStr};
    use serialize::json::Json;


    let (width, height) = (28, 16);

    let json = match FromStr::from_str(input) {
        Some(Json::Object(obj)) => obj,
        _ => panic!("Not a JSON object")
    };

    let layers = json.get("layers").unwrap().as_array().expect("Not a JSON array");

    // First layer is the level data
    let level_data_json = {
        let layer = layers[0].as_object().expect("Not a JSON object");
        layer.get("data").unwrap().as_array().expect("Not a JSON array")
    };

    let tiles: Vec<Tile> = level_data_json.iter().map(|num_json| {
        let value = num_json.as_u64().expect("Not a JSON number") as u32;

        let (id, flip_x, flip_y) = (value & 0x3FFFFFFF, (value & 0x80000000) != 0, (value & 0x40000000) != 0);

        Tile {
            tile_type: TileType::from_id(id as u16),
            flip_x: flip_x,
            flip_y: flip_y
        }
    }).collect();

    assert_eq!(tiles.len(), width*height);

    let objects_json = {
        let layer = layers[1].as_object().expect("Not a JSON object");
        layer.get("objects").unwrap().as_array().expect("Not a JSON array")
    };

    let mut player_start_pos = (0.0, 0.0);
    let mut switches: Vec<Switch> = Vec::new();
    let mut chests: Vec<Chest> = Vec::new();
    let mut beanstalks: Vec<Beanstalk> = Vec::new();

    for x in objects_json.iter() {

        let object = x.as_object().expect("Not a JSON object");
        let x = object.get("x").unwrap().as_f64().unwrap() as f32;
        let y = object.get("y").unwrap().as_f64().unwrap() as f32;
        let height = object.get("height").unwrap().as_f64().unwrap() as f32;
        let properties = object.get("properties").unwrap().as_object().expect("Not a JSON object");
        let typ = object.get("type").unwrap().as_string().unwrap();

        match typ {
            "player" => { player_start_pos = (x, y); },
            "switch" => {
                let trigger = parse_property_as_number(properties, "trigger").expect("Requires 'trigger'");

                switches.push(Switch {
                    x: x,
                    y: y,
                    trigger: trigger
                });
            },
            "chest" => {
                let trigger = parse_property_as_number(properties, "trigger");
                let triggered_by = parse_property_as_number(properties, "triggered_by");
                let poof = parse_property_as_boolean(properties, "poof");
                let is_static = parse_property_as_boolean(properties, "static");

                chests.push(Chest {
                    x: x,
                    y: y,
                    trigger: trigger,
                    triggered_by: triggered_by,
                    poof: poof,
                    is_static: is_static,
                    fall_distance: height - 16.0
                });
            },
            "beanstalk" => {
                let triggered_by = parse_property_as_number(properties, "triggered_by");
                let poof = parse_property_as_boolean(properties, "poof");

                beanstalks.push(Beanstalk {
                    x: x,
                    y: y,
                    height: height,
                    triggered_by: triggered_by,
                    poof: poof,
                })
            },
            "monster1" => {

            },
            _ => panic!("Unknown type: {}", typ)
        };
    }

    Level {
        width: width as u8,
        height: height as u8,
        player_start_pos: player_start_pos,
        tiles: tiles,
        switches: switches,
        chests: chests,
        beanstalks: beanstalks
    }
}

fn parse_property_as_boolean(properties: &serialize::json::Object, key: &str) -> bool {
    match properties.get(key) {
        Some(j) => {
            let value_str = j.as_string().expect("Not a JSON string");
            match value_str {
                "true" => true,
                _ => false
            }
        },
        None => false
    }
}

fn parse_property_as_number<T: FromStrRadix>(properties: &serialize::json::Object, key: &str) -> Option<T> {
    match properties.get(key) {
        Some(j) => {
            let value_str = j.as_string().expect("Not a JSON string");
            Some(FromStrRadix::from_str_radix(value_str, 10).expect("Not a base 10 number"))
        },
        None => None
    }
}
