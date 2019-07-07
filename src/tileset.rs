use std::{
    fmt::Write,
    ops::Index,
};

use image::{FilterType, GenericImageView};
use piston_window::{Texture, TextureSettings, G2dTextureContext, G2dTexture};

use crate::{SCALE, Result};

const NUM_TILES: u8 = 158;

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
    pub fn new(id: u8) -> Result<TileId> {
        if id < NUM_TILES {
            Ok(TileId(id))
        } else {
            Err(format!("Invalid tile id: {}", id))?
        }
    }
}

pub fn load_tileset(mut context: G2dTextureContext) -> Result<TileSet> {
    let mut tiles = Vec::new();

    let mut name_buf = String::new();
    for i in 0..NUM_TILES {
        name_buf.clear();
        write!(&mut name_buf, "tiles/tile{}.bmp", i)?;

        let image = image::open(&name_buf)?;

        // Because we're doing a scaling here, and the rendering backend doesn't support it,
        // we need to resize the tile image.
        let image = image.resize(
            image.width() * SCALE,
            image.height()* SCALE,
            FilterType::Nearest,
        ).to_rgba();

        let texture = Texture::from_image(
            &mut context,
            &image,
            &TextureSettings::new(),
        )?;

        tiles.push(texture);
    }

    Ok(TileSet(tiles))
}