use bevy::{
    asset::{embedded_asset, load_embedded_asset},
    prelude::*,
};

const CARD_ART_BASE: &str = "https://art.hearthstonejson.com/v1/512x";

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

    /// Loads official card artwork through HearthstoneJSON. Bevy owns the
    /// request and stores successful downloads in its local web-asset cache.
    /// Invalid or unavailable IDs simply leave the colored frame visible.
    pub(crate) fn card(&self, asset_server: &AssetServer, card_id: &str) -> Handle<Image> {
        asset_server.load(card_art_url(card_id))
    }

    pub(crate) fn hero(
        &self,
        asset_server: &AssetServer,
        card_id: &str,
        class: &str,
    ) -> Handle<Image> {
        let art_id = self.hero_card_id(card_id, class);
        self.card(asset_server, art_id)
    }

    pub(crate) fn hero_card_id<'a>(&self, card_id: &'a str, class: &str) -> &'a str {
        if card_id == "builtin_hero" {
            basic_hero_card_id(class)
        } else {
            card_id
        }
    }
}

fn card_art_url(card_id: &str) -> String {
    // Card IDs from the trusted local data pack are ASCII identifiers. Keeping
    // only their documented alphabet prevents them from changing the host or
    // path used by the web asset loader.
    let safe_id = card_id
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
        .collect::<String>();
    format!("{CARD_ART_BASE}/{safe_id}.jpg")
}

fn basic_hero_card_id(class: &str) -> &'static str {
    match class {
        "warrior" => "HERO_01",
        "shaman" => "HERO_02",
        "rogue" => "HERO_03",
        "paladin" => "HERO_04",
        "hunter" => "HERO_05",
        "druid" => "HERO_06",
        "warlock" => "HERO_07",
        "mage" => "HERO_08",
        "priest" => "HERO_09",
        "demon_hunter" => "HERO_10",
        "death_knight" => "HERO_11",
        _ => "HERO_08",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_art_url_cannot_escape_the_fixed_host() {
        assert_eq!(
            card_art_url("EX1_001?x=https://evil.invalid"),
            "https://art.hearthstonejson.com/v1/512x/EX1_001xhttpsevilinvalid.jpg"
        );
    }

    #[test]
    fn builtin_heroes_use_their_class_portraits() {
        assert_eq!(basic_hero_card_id("mage"), "HERO_08");
        assert_eq!(basic_hero_card_id("death_knight"), "HERO_11");
    }
}
