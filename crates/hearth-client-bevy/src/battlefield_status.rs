use bevy::prelude::*;
use hearth_app::GameSession;
use hearth_core::{EntityId, PlayerId, PlayerStateView, PlayerView};

use crate::card_preview::{InspectableCard, hide_card_preview, show_card_preview};
use crate::i18n::pick;
use crate::{ACTION_HOVER, ButtonColors, CARD_SELECTED, MUTED_TEXT, TEXT, text_font};

const WEAPON_COLOR: Color = Color::srgb(0.48, 0.20, 0.12);
const SECRET_COLOR: Color = Color::srgb(0.14, 0.30, 0.54);
const OBJECTIVE_COLOR: Color = Color::srgb(0.39, 0.25, 0.56);
const PLAGUE_COLOR: Color = Color::srgb(0.18, 0.38, 0.22);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BattlefieldItem {
    Weapon(EntityId),
    KnownSecret(EntityId),
    HiddenSecret,
    PublicObjective(EntityId),
    UnendingPlagues,
}

pub fn spawn_battlefield_status(
    parent: &mut ChildSpawnerCommands,
    session: &GameSession,
    view: &PlayerView,
    player: PlayerId,
) {
    let items = battlefield_items(view.viewer, view.player(player));
    if items.is_empty() {
        return;
    }

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(4),
            row_gap: px(4),
            max_width: px(390),
            ..default()
        })
        .with_children(|status| {
            for item in items {
                match item {
                    BattlefieldItem::Weapon(entity_id) => {
                        let Some(entity) = view.entity(entity_id) else {
                            continue;
                        };
                        spawn_known_badge(
                            status,
                            &format!(
                                "{}\n{} / {}",
                                pick(session.locale(), "WEAPON", "武器", "武器"),
                                entity.attack,
                                entity.health()
                            ),
                            &entity.card_id,
                            WEAPON_COLOR,
                            74.0,
                        );
                    }
                    BattlefieldItem::KnownSecret(entity_id) => {
                        let Some(entity) = view.entity(entity_id) else {
                            continue;
                        };
                        spawn_known_badge(
                            status,
                            &format!(
                                "{}\n{}",
                                pick(session.locale(), "SECRET", "奥秘", "秘密"),
                                crate::shorten(&session.card_name(&entity.card_id), 11)
                            ),
                            &entity.card_id,
                            SECRET_COLOR,
                            92.0,
                        );
                    }
                    BattlefieldItem::HiddenSecret => {
                        spawn_hidden_secret(status, session.locale());
                    }
                    BattlefieldItem::PublicObjective(entity_id) => {
                        let Some(entity) = view.entity(entity_id) else {
                            continue;
                        };
                        spawn_known_badge(
                            status,
                            &format!(
                                "{}\n{}",
                                pick(session.locale(), "OBJECTIVE", "任务", "任務"),
                                crate::shorten(&session.card_name(&entity.card_id), 13)
                            ),
                            &entity.card_id,
                            OBJECTIVE_COLOR,
                            108.0,
                        );
                    }
                    BattlefieldItem::UnendingPlagues => {
                        spawn_status_badge(
                            status,
                            &format!(
                                "{}\n{}",
                                pick(session.locale(), "PLAGUES", "疫病", "瘟疫"),
                                pick(session.locale(), "UNENDING", "无尽", "無盡")
                            ),
                            PLAGUE_COLOR,
                            76.0,
                        );
                    }
                }
            }
        });
}

fn battlefield_items(viewer: PlayerId, player: &PlayerStateView) -> Vec<BattlefieldItem> {
    let mut items = Vec::new();
    if let Some(weapon) = player.weapon {
        items.push(BattlefieldItem::Weapon(weapon));
    }

    if viewer == player.id {
        items.extend(
            player
                .secrets
                .iter()
                .copied()
                .filter(|secret| !player.public_objectives.contains(secret))
                .map(BattlefieldItem::KnownSecret),
        );
    } else {
        items.extend((0..player.secrets_count).map(|_| BattlefieldItem::HiddenSecret));
    }
    items.extend(
        player
            .public_objectives
            .iter()
            .copied()
            .map(BattlefieldItem::PublicObjective),
    );
    if player
        .public_keywords
        .iter()
        .any(|keyword| keyword == "unending_plagues")
    {
        items.push(BattlefieldItem::UnendingPlagues);
    }
    items
}

fn badge_node(width: f32) -> Node {
    Node {
        width: px(width),
        min_height: px(44),
        padding: UiRect::all(px(4)),
        border: UiRect::all(px(2)),
        border_radius: BorderRadius::all(px(14)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

fn spawn_known_badge(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    card_id: &str,
    normal: Color,
    width: f32,
) {
    parent
        .spawn((
            Button,
            InspectableCard(card_id.to_owned()),
            ButtonColors {
                normal,
                hovered: ACTION_HOVER,
                pressed: CARD_SELECTED,
            },
            badge_node(width),
            BorderColor::all(CARD_SELECTED),
            BackgroundColor(normal),
        ))
        .observe(show_card_preview)
        .observe(hide_card_preview)
        .with_child((
            Text::new(label),
            text_font(10.0),
            TextColor(TEXT),
            TextLayout::justify(Justify::Center),
            Pickable::IGNORE,
        ));
}

fn spawn_hidden_secret(parent: &mut ChildSpawnerCommands, locale: hearth_core::Locale) {
    parent
        .spawn((
            badge_node(52.0),
            BorderColor::all(Color::srgb(0.45, 0.67, 0.94)),
            BackgroundColor(SECRET_COLOR),
        ))
        .with_child((
            Text::new(format!("{}\n?", pick(locale, "SECRET", "奥秘", "秘密"))),
            text_font(10.0),
            TextColor(MUTED_TEXT),
            TextLayout::justify(Justify::Center),
            Pickable::IGNORE,
        ));
}

fn spawn_status_badge(parent: &mut ChildSpawnerCommands, label: &str, color: Color, width: f32) {
    parent
        .spawn((
            badge_node(width),
            BorderColor::all(CARD_SELECTED),
            BackgroundColor(color),
            Pickable::IGNORE,
        ))
        .with_child((
            Text::new(label),
            text_font(10.0),
            TextColor(TEXT),
            TextLayout::justify(Justify::Center),
            Pickable::IGNORE,
        ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(id: PlayerId) -> PlayerStateView {
        PlayerStateView {
            id,
            class: "mage".to_owned(),
            hero: EntityId(1),
            deck_size: 20,
            hand_size: 5,
            hand: Vec::new(),
            sideboards: Default::default(),
            board: Vec::new(),
            weapon: None,
            hero_power: EntityId(2),
            hero_power_used: false,
            hero_power_uses_this_turn: 0,
            secrets_count: 0,
            secrets: Vec::new(),
            public_objectives: Vec::new(),
            mana: 3,
            max_mana: 3,
            temporary_mana: 0,
            corpses: 0,
            corpses_spent: 0,
            public_keywords: Vec::new(),
            overload_pending: 0,
            overloaded_mana: 0,
            fatigue: 0,
            cards_played_this_turn: 0,
        }
    }

    #[test]
    fn own_secret_identity_is_kept_and_public_objective_is_not_duplicated() {
        let mut state = player(PlayerId::ONE);
        state.weapon = Some(EntityId(3));
        state.secrets = vec![EntityId(4), EntityId(5)];
        state.public_objectives = vec![EntityId(5)];
        state.secrets_count = 1;

        assert_eq!(
            battlefield_items(PlayerId::ONE, &state),
            vec![
                BattlefieldItem::Weapon(EntityId(3)),
                BattlefieldItem::KnownSecret(EntityId(4)),
                BattlefieldItem::PublicObjective(EntityId(5)),
            ]
        );
    }

    #[test]
    fn opponent_secret_slots_never_require_hidden_entity_identities() {
        let mut state = player(PlayerId::TWO);
        state.secrets_count = 2;
        state.secrets = Vec::new();
        state.public_objectives = vec![EntityId(8)];

        assert_eq!(
            battlefield_items(PlayerId::ONE, &state),
            vec![
                BattlefieldItem::HiddenSecret,
                BattlefieldItem::HiddenSecret,
                BattlefieldItem::PublicObjective(EntityId(8)),
            ]
        );
    }

    #[test]
    fn unending_plagues_are_visible_to_both_players() {
        let mut state = player(PlayerId::TWO);
        state.public_keywords = vec!["unending_plagues".to_owned()];

        assert_eq!(
            battlefield_items(PlayerId::ONE, &state),
            vec![BattlefieldItem::UnendingPlagues]
        );
        assert_eq!(
            battlefield_items(PlayerId::TWO, &state),
            vec![BattlefieldItem::UnendingPlagues]
        );
    }
}
