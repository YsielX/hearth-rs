use bevy::prelude::*;
use hearth_core::Locale;

use crate::{CARD_SELECTED, MUTED_TEXT, PANEL, text_font};

const MAX_HAND_SIZE: usize = 10;
const CARD_BACK: Color = Color::srgb(0.11, 0.24, 0.43);

#[derive(Component)]
pub struct OpponentHandRoot;

#[derive(Component)]
pub struct HiddenHandCard;

pub fn spawn_opponent_hand(
    parent: &mut ChildSpawnerCommands,
    hand_size: usize,
    locale: Locale,
    card_back: &Handle<Image>,
) {
    parent
        .spawn((
            OpponentHandRoot,
            Node {
                width: percent(100),
                min_height: px(58),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(5),
                padding: UiRect::vertical(px(3)),
                ..default()
            },
            BackgroundColor(PANEL.with_alpha(0.72)),
        ))
        .with_children(|hand| {
            hand.spawn((
                Text::new(match locale {
                    Locale::EnUs => format!("OPPONENT HAND  {hand_size}"),
                    Locale::ZhCn => format!("对手手牌  {hand_size}"),
                    Locale::ZhTw => format!("對手手牌  {hand_size}"),
                }),
                text_font(11.0),
                TextColor(MUTED_TEXT),
                Node {
                    margin: UiRect::right(px(8)),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            for _ in 0..visible_card_backs(hand_size) {
                hand.spawn((
                    HiddenHandCard,
                    Node {
                        width: px(36),
                        height: px(50),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(7)),
                        ..default()
                    },
                    BorderColor::all(CARD_SELECTED),
                    BackgroundColor(CARD_BACK),
                    ImageNode::new(card_back.clone()).with_mode(NodeImageMode::Stretch),
                    Pickable::IGNORE,
                ));
            }
        });
}

fn visible_card_backs(hand_size: usize) -> usize {
    hand_size.min(MAX_HAND_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_back_count_tracks_only_public_hand_size_and_stays_bounded() {
        assert_eq!(visible_card_backs(0), 0);
        assert_eq!(visible_card_backs(7), 7);
        assert_eq!(visible_card_backs(10), 10);
        assert_eq!(visible_card_backs(usize::MAX), 10);
    }
}
