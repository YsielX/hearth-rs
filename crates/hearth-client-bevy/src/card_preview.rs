use bevy::prelude::*;
use hearth_app::CardCatalogEntry;
use hearth_core::CardKind;

use crate::frontend::ClientCatalog;
use crate::frontend::FrontendState;
use crate::game_art::GameArt;
use crate::i18n::{class_label, kind_label, pick, rarity_label};
use crate::{CARD_SELECTED, MUTED_TEXT, TEXT, text_font};

const PREVIEW_WIDTH: f32 = 320.0;
const PREVIEW_HEIGHT: f32 = 460.0;
const CURSOR_GAP: f32 = 24.0;
const WINDOW_MARGIN: f32 = 8.0;

#[derive(Component, Clone)]
pub struct InspectableCard(pub String);

#[derive(Resource, Default)]
pub struct CardPreviewState {
    owner: Option<Entity>,
    card_id: Option<String>,
}

#[derive(Component)]
pub struct CardPreviewRoot;

#[derive(Component)]
pub(crate) struct CardPreviewImage;

#[derive(Component, Clone, Copy)]
pub(crate) enum CardPreviewText {
    Title,
    Meta,
    Body,
    Keywords,
}

pub fn spawn_card_preview(commands: &mut Commands) {
    commands
        .spawn((
            CardPreviewRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(WINDOW_MARGIN),
                top: px(WINDOW_MARGIN),
                width: px(PREVIEW_WIDTH),
                min_height: px(220),
                max_height: px(PREVIEW_HEIGHT),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                padding: UiRect::all(px(14)),
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(px(14)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.055, 0.075, 0.105, 0.98)),
            BorderColor::all(CARD_SELECTED),
            GlobalZIndex(220),
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_children(|preview| {
            preview.spawn((
                CardPreviewImage,
                ImageNode::default().with_mode(NodeImageMode::Stretch),
                Node {
                    width: percent(100),
                    height: px(175),
                    flex_shrink: 0.0,
                    border_radius: BorderRadius::all(px(9)),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            preview.spawn((
                CardPreviewText::Title,
                Text::new(""),
                text_font(24.0),
                TextColor(CARD_SELECTED),
                Pickable::IGNORE,
            ));
            preview.spawn((
                CardPreviewText::Meta,
                Text::new(""),
                text_font(12.0),
                TextColor(MUTED_TEXT),
                Pickable::IGNORE,
            ));
            preview.spawn((
                CardPreviewText::Body,
                Text::new(""),
                text_font(15.0),
                TextColor(TEXT),
                Pickable::IGNORE,
            ));
            preview.spawn((
                CardPreviewText::Keywords,
                Text::new(""),
                text_font(12.0),
                TextColor(Color::srgb(0.51, 0.82, 0.95)),
                Pickable::IGNORE,
            ));
        });
}

pub fn show_card_preview(
    event: On<Pointer<Over>>,
    cards: Query<&InspectableCard>,
    mut state: ResMut<CardPreviewState>,
) {
    let target = event.event_target();
    let Ok(card) = cards.get(target) else {
        return;
    };
    state.owner = Some(target);
    state.card_id = Some(card.0.clone());
}

pub fn hide_card_preview(event: On<Pointer<Out>>, mut state: ResMut<CardPreviewState>) {
    if state.owner == Some(event.event_target()) {
        state.owner = None;
        state.card_id = None;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_card_preview(
    windows: Query<&Window>,
    catalog: Res<ClientCatalog>,
    frontend: Res<FrontendState>,
    art: Res<GameArt>,
    asset_server: Res<AssetServer>,
    cards: Query<(), With<InspectableCard>>,
    mut state: ResMut<CardPreviewState>,
    mut preview: Query<(&mut Node, &mut BorderColor, &mut Visibility), With<CardPreviewRoot>>,
    mut text: Query<(&CardPreviewText, &mut Text)>,
    mut image: Query<&mut ImageNode, With<CardPreviewImage>>,
) {
    if frontend.handoff_player.is_some() {
        state.owner = None;
        state.card_id = None;
        if let Ok((_, _, mut visibility)) = preview.single_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }
    if state.owner.is_some_and(|owner| cards.get(owner).is_err()) {
        state.owner = None;
        state.card_id = None;
    }
    let Ok((mut node, mut border, mut visibility)) = preview.single_mut() else {
        return;
    };
    let Some(definition) = state
        .card_id
        .as_deref()
        .and_then(|card_id| catalog.0.definition(card_id))
    else {
        *visibility = Visibility::Hidden;
        return;
    };
    let Ok(window) = windows.single() else {
        *visibility = Visibility::Hidden;
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        *visibility = Visibility::Hidden;
        return;
    };

    position_preview(&mut node, window, cursor);
    border.set_all(kind_color(definition.kind));
    let title_text = format!("[{}] {}", definition.cost, definition.name);
    let locale = frontend.config.locale;
    let meta_text = preview_meta(definition, locale);
    let body_text = plain_card_text(&definition.text);
    let keywords_text = if definition.keywords.is_empty() {
        String::new()
    } else {
        format!(
            "{}: {}",
            pick(locale, "Keywords", "关键词", "關鍵字"),
            definition.keywords.join(", "),
        )
    };
    if let Ok(mut preview_image) = image.single_mut() {
        preview_image.image = art.card(&asset_server, &definition.id);
    }
    for (slot, mut value) in &mut text {
        **value = match slot {
            CardPreviewText::Title => title_text.clone(),
            CardPreviewText::Meta => meta_text.clone(),
            CardPreviewText::Body => body_text.clone(),
            CardPreviewText::Keywords => keywords_text.clone(),
        };
    }
    *visibility = Visibility::Inherited;
}

fn position_preview(node: &mut Node, window: &Window, cursor: Vec2) {
    let window_width = window.resolution.width();
    let window_height = window.resolution.height();
    let left = if cursor.x + CURSOR_GAP + PREVIEW_WIDTH + WINDOW_MARGIN <= window_width {
        cursor.x + CURSOR_GAP
    } else {
        (cursor.x - PREVIEW_WIDTH - CURSOR_GAP).max(WINDOW_MARGIN)
    };
    let maximum_top = (window_height - PREVIEW_HEIGHT - WINDOW_MARGIN).max(WINDOW_MARGIN);
    node.left = px(left);
    node.top = px((cursor.y - 72.0).clamp(WINDOW_MARGIN, maximum_top));
}

fn preview_meta(card: &CardCatalogEntry, locale: hearth_core::Locale) -> String {
    let class = if card.classes.is_empty() {
        class_label(locale, &card.class).to_owned()
    } else {
        card.classes
            .iter()
            .map(|class| class_label(locale, class))
            .collect::<Vec<_>>()
            .join(" / ")
    };
    let rarity = card
        .rarity
        .as_deref()
        .map(|rarity| rarity_label(locale, rarity))
        .unwrap_or(if card.collectible {
            pick(locale, "no rarity", "无稀有度", "無稀有度")
        } else {
            pick(locale, "uncollectible", "不可收藏", "不可收藏")
        });
    let stats = match card.kind {
        CardKind::Minion => format!(" · {}/{}", card.attack, card.health),
        CardKind::Weapon => format!(
            " · {} {} / {} {}",
            card.attack,
            pick(locale, "Attack", "攻击力", "攻擊力"),
            card.health,
            pick(locale, "Durability", "耐久度", "耐久度")
        ),
        CardKind::Location => format!(
            " · {} {}",
            card.health,
            pick(locale, "Durability", "耐久度", "耐久度")
        ),
        CardKind::Hero => format!(
            " · {} {}",
            card.armor,
            pick(locale, "Armor", "护甲", "護甲")
        ),
        CardKind::Spell | CardKind::HeroPower => String::new(),
    };
    format!(
        "{} · {} · {} · {}{}",
        kind_label(locale, card.kind),
        class,
        card.set,
        rarity,
        stats
    )
}

fn kind_color(kind: CardKind) -> Color {
    match kind {
        CardKind::Minion => CARD_SELECTED,
        CardKind::Spell => Color::srgb(0.35, 0.58, 0.95),
        CardKind::Weapon => Color::srgb(0.78, 0.36, 0.22),
        CardKind::Location => Color::srgb(0.55, 0.38, 0.80),
        CardKind::Hero | CardKind::HeroPower => Color::srgb(0.35, 0.75, 0.52),
    }
}

fn plain_card_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut in_tag = false;
    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            '$' if !in_tag => {}
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_card() -> CardCatalogEntry {
        CardCatalogEntry {
            id: "card".to_owned(),
            name: "Card".to_owned(),
            text: String::new(),
            set: "TEST".to_owned(),
            kind: CardKind::Minion,
            collectible: true,
            class: "neutral".to_owned(),
            classes: Vec::new(),
            sideboard_size: 0,
            deck_size: None,
            starting_health: None,
            rune_cost: hearth_core::RuneCost::default(),
            rarity: None,
            cost: 1,
            attack: 1,
            health: 1,
            armor: 0,
            keywords: Vec::new(),
        }
    }

    #[test]
    fn markup_is_removed_without_losing_visible_text() {
        assert_eq!(
            plain_card_text("<b>Battlecry:</b> Deal $3 damage.\n<i>Fast</i>"),
            "Battlecry: Deal 3 damage.\nFast"
        );
    }

    #[test]
    fn preview_moves_left_when_the_cursor_is_near_the_right_edge() {
        let mut node = Node::default();
        let window = Window {
            resolution: bevy::window::WindowResolution::new(1_000, 700),
            ..default()
        };
        position_preview(&mut node, &window, Vec2::new(980.0, 350.0));
        assert_eq!(node.left, px(636.0));
    }

    #[test]
    fn metadata_distinguishes_missing_rarity_from_uncollectible() {
        let mut card = sample_card();
        assert!(preview_meta(&card, hearth_core::Locale::EnUs).contains("no rarity"));
        card.collectible = false;
        assert!(preview_meta(&card, hearth_core::Locale::EnUs).contains("uncollectible"));
    }

    #[test]
    fn hero_metadata_uses_printed_armor_instead_of_health() {
        let mut card = sample_card();
        card.kind = CardKind::Hero;
        card.health = 30;
        card.armor = 7;
        let meta = preview_meta(&card, hearth_core::Locale::EnUs);
        assert!(meta.contains("7 Armor"));
        assert!(!meta.contains("30 Armor"));
    }
}
