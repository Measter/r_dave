use std::{
    fmt::Write,
    ops::Index,
};

use image::{FilterType, Rgba};
use piston_window::{Texture, TextureSettings, G2dTextureContext, G2dTexture};

use crate::{SCALE, Result};

const NUM_TILES: u8 = 158;

// See the level.rs file comment for the reason behind this data structure.

#[derive(Debug)]
pub struct TileSet(Vec<G2dTexture>);

impl Index<TileId> for TileSet {
    type Output = G2dTexture;
    fn index(&self, idx: TileId) -> &Self::Output {
        // Because we're indexing through a type that can only be constructed
        // by going through validation, we can skip the bounds check here.
        unsafe{ self.0.get_unchecked(idx.0 as usize) }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TileId(u8);

impl TileId {
    pub const TILE_DAVE_RIGHT1: TileId = TileId(53);
    pub const TILE_DAVE_RIGHT2: TileId = TileId(54);
    pub const TILE_DAVE_RIGHT3: TileId = TileId(55);
    pub const TILE_DAVE_BASIC: TileId = TileId(56);
    pub const TILE_DAVE_LEFT1: TileId = TileId(57);
    pub const TILE_DAVE_LEFT2: TileId = TileId(58);
    pub const TILE_DAVE_LEFT3: TileId = TileId(59);
    pub const TILE_DAVE_JETPACK_RIGHT1: TileId = TileId(77);
    pub const TILE_DAVE_JETPACK_RIGHT2: TileId = TileId(78);
    pub const TILE_DAVE_JETPACK_RIGHT3: TileId = TileId(79);
    pub const TILE_DAVE_JETPACK_LEFT1: TileId = TileId(80);
    pub const TILE_DAVE_JETPACK_LEFT2: TileId = TileId(81);
    pub const TILE_DAVE_JETPACK_LEFT3: TileId = TileId(81);
    pub const TILE_DAVE_JUMP_RIGHT: TileId = TileId(67);
    pub const TILE_DAVE_JUMP_LEFT: TileId = TileId(68);

    pub const TILE_BLANK: TileId = TileId(0);
    pub const TILE_GUN: TileId = TileId(20);
    pub const TILE_JETPACK: TileId = TileId(4);
    pub const TILE_BULLET_LEFT: TileId = TileId(128);
    pub const TILE_BULLET_RIGHT: TileId = TileId(127);
}

impl TileId {
    pub fn new(id: u8) -> Result<TileId> {
        if id < NUM_TILES {
            Ok(TileId(id))
        } else {
            Err(format!("Invalid tile id: {}", id))?
        }
    }

    pub fn is_collidable(self) -> bool {
        match self.0 {
            1 | 3 | 5 | 15..=19 | 21..=24 | 29 | 30 => true,
            _ => false,
        }
    }

    pub fn is_pickup(self) -> bool {
        match self.0 {
            4 | 10..=14 | 20 | 47..=52 => true,
            _ => false,
        }
    }

    pub fn is_door(self) -> bool {
        self.0 == 2
    }

    pub fn is_trophy(self) -> bool {
        match self.0 {
            10..=14 => true,
            _ => false,
        }
    }

    fn is_dave(id: u8) -> bool {
        match id {
            53..=59| 67 | 68 | 71..=73 | 77..=82 => true,
            _ => false,
        }
    }

    fn get_dave_mask(id: u8) -> u8 {
        match id {
            53..=59 => id + 7,
            67 | 68 => id + 2,
            71..=73 => id + 3,
            77..=82 => id + 6,
            _ => panic!("Invalid Dave tile!"),
        }
    }
}

pub fn load_tileset(mut context: G2dTextureContext) -> Result<TileSet> {
    let mut tiles = Vec::new();

    let mut name_buf = String::new();
    for i in 0..NUM_TILES {
        name_buf.clear();
        write!(&mut name_buf, "tiles/tile{}.bmp", i)?;

        let mut tile = image::open(&name_buf)?.to_rgba();

        // Now we apply the alpha mask to the dave tile.
        if TileId::is_dave(i) {
            let mask_id = TileId::get_dave_mask(i);
            name_buf.clear();
            write!(&mut name_buf, "tiles/tile{}.bmp", mask_id)?;
            let mask = image::open(&name_buf)?.to_rgba();

            for ((_, _ , tp), (_, _, mp)) in tile.enumerate_pixels_mut().zip(mask.enumerate_pixels()) {
                if mp == &Rgba([0xfc, 0xfc, 0xfc, 0xff]) {
                    tp[3] = 0;
                }
            }
        }

        // Because we're doing a scaling here, and the rendering backend doesn't support it,
        // we need to resize the tile image.
        let image = image::imageops::resize(
            &tile,
            tile.width() * SCALE,
            tile.height()* SCALE,
            FilterType::Nearest,
        );

        let texture = Texture::from_image(
            &mut context,
            &image,
            &TextureSettings::new(),
        )?;

        tiles.push(texture);
    }

    Ok(TileSet(tiles))
}