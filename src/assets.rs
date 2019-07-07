use crate::{
    Result,
    tileset::*,
    level::*,
};
use piston_window::{G2dTextureContext, G2dTexture};

pub struct Assets {
    levels: Levels,
    tiles: TileSet,
}

impl Assets {
    pub fn init(context: G2dTextureContext) -> Result<Assets> {
        Ok(Assets {
            levels: load_levels()?,
            tiles: load_tileset(context)?,
        })
    }

    pub fn get_level(&self, id: LevelId) -> &Level {
        &self.levels[id]
    }

    pub fn get_tile(&self, id: TileId) -> &G2dTexture {
        &self.tiles[id]
    }
}