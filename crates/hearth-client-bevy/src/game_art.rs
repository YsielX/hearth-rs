use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
};

pub(crate) struct GameArtPlugin;

impl Plugin for GameArtPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "../assets/ui/tavern-board.png");
        embedded_asset!(app, "../assets/ui/arcane-card-back.png");
    }
}

#[derive(Resource)]
pub(crate) struct GameArt {
    pub(crate) tavern_board: Handle<Image>,
    pub(crate) card_back: Handle<Image>,
}

impl GameArt {
    pub(crate) fn load(asset_server: &AssetServer) -> Self {
        Self {
            tavern_board: load_embedded_asset!(asset_server, "../assets/ui/tavern-board.png"),
            card_back: load_embedded_asset!(asset_server, "../assets/ui/arcane-card-back.png"),
        }
    }
}
