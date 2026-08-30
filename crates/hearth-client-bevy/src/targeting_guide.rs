use bevy::prelude::*;
use hearth_app::GameSession;
use hearth_core::EntityId;

use crate::frontend::{ClientScene, FrontendState};
use crate::interaction::{ActionSource, command_source, command_target, is_candidate_target};
use crate::{GameEntity, UiState};

const SHAFT_THICKNESS: f32 = 9.0;
const HEAD_LENGTH: f32 = 28.0;
const HEAD_SPREAD: f32 = 0.58;
const SOURCE_CLEARANCE: f32 = 28.0;
const CURSOR_CLEARANCE: f32 = 12.0;
const TARGET_CLEARANCE: f32 = 28.0;
const MIN_DISTANCE: f32 = 54.0;

#[derive(Component)]
pub(crate) struct HeroPowerTargetingSource;

#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub(crate) enum TargetingGuidePart {
    Shaft,
    HeadLeft,
    HeadRight,
    OriginRing,
    TargetRing,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LineLayout {
    center: Vec2,
    size: Vec2,
    rotation: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TargetingLayout {
    shaft: LineLayout,
    head_left: LineLayout,
    head_right: LineLayout,
    origin: Vec2,
    target: Vec2,
}

pub(crate) fn spawn_targeting_guide(commands: &mut Commands) {
    for part in [
        TargetingGuidePart::Shaft,
        TargetingGuidePart::HeadLeft,
        TargetingGuidePart::HeadRight,
        TargetingGuidePart::OriginRing,
        TargetingGuidePart::TargetRing,
    ] {
        let ring = matches!(
            part,
            TargetingGuidePart::OriginRing | TargetingGuidePart::TargetRing
        );
        commands.spawn((
            part,
            Node {
                position_type: PositionType::Absolute,
                width: px(if ring { 30.0 } else { 1.0 }),
                height: px(if ring { 30.0 } else { SHAFT_THICKNESS }),
                border: UiRect::all(px(if ring { 3.0 } else { 0.0 })),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            UiTransform::default(),
            GlobalZIndex(180),
            Visibility::Hidden,
            Pickable::IGNORE,
        ));
    }
}

pub(crate) fn update_targeting_guide(
    windows: Query<&Window>,
    session: NonSend<GameSession>,
    frontend: Res<FrontendState>,
    mut ui: ResMut<UiState>,
    entities: Query<(&GameEntity, &ComputedNode, &UiGlobalTransform, &Interaction)>,
    hero_power: Query<(&ComputedNode, &UiGlobalTransform), With<HeroPowerTargetingSource>>,
    mut parts: Query<(
        &TargetingGuidePart,
        &mut Node,
        &mut UiTransform,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut Visibility,
    )>,
) {
    let view = session.view();
    if frontend.scene != ClientScene::Match
        || frontend.handoff_player.is_some()
        || frontend.match_menu_open
        || view.pending_input.is_some()
        || view.outcome.is_some()
    {
        ui.dragged = None;
        ui.drag_origin = None;
        hide_all(&mut parts);
        return;
    }

    if ui
        .dragged
        .is_some_and(|dragged| !entities.iter().any(|(game, _, _, _)| game.0 == dragged))
    {
        ui.dragged = None;
        ui.drag_origin = None;
    }
    let source = ui
        .dragged
        .map(ActionSource::Entity)
        .or(ui.interaction.source);
    let Ok(legal) = session.legal_actions() else {
        hide_all(&mut parts);
        return;
    };
    let Some(source) = source.filter(|source| {
        legal.iter().any(|action| {
            command_source(&action.command) == Some(*source)
                && command_target(&action.command).is_some()
        })
    }) else {
        hide_all(&mut parts);
        return;
    };

    let from = if ui.dragged.is_some() {
        ui.drag_origin
    } else {
        match source {
            ActionSource::Entity(source) => {
                entities.iter().find_map(|(game, computed, global, _)| {
                    (game.0 == source)
                        .then(|| screen_center(computed, global))
                        .flatten()
                })
            }
            ActionSource::HeroPower => hero_power
                .iter()
                .find_map(|(computed, global)| screen_center(computed, global)),
        }
    };
    let Some(from) = from else {
        hide_all(&mut parts);
        return;
    };

    let cursor = windows
        .single()
        .ok()
        .and_then(|window| window.cursor_position());
    let selected = ui
        .interaction
        .target
        .filter(|target| is_candidate_target(&legal, Some(source), *target));
    let snapped = selected
        .and_then(|target| center_for(target, &entities))
        .or_else(|| {
            entities
                .iter()
                .find(|(game, _, _, interaction)| {
                    matches!(interaction, Interaction::Hovered | Interaction::Pressed)
                        && is_candidate_target(&legal, Some(source), game.0)
                })
                .and_then(|(game, _, _, _)| center_for(game.0, &entities))
        })
        .or_else(|| {
            cursor.and_then(|cursor| {
                entities
                    .iter()
                    .find(|(game, computed, global, _)| {
                        is_candidate_target(&legal, Some(source), game.0)
                            && cursor_over_node(cursor, computed, global)
                    })
                    .and_then(|(_, computed, global, _)| screen_center(computed, global))
            })
        });
    let Some(to) = snapped.or(cursor) else {
        hide_all(&mut parts);
        return;
    };
    let Some(layout) = targeting_layout(
        from,
        to,
        if snapped.is_some() {
            TARGET_CLEARANCE
        } else {
            CURSOR_CLEARANCE
        },
    ) else {
        hide_all(&mut parts);
        return;
    };

    let color = if snapped.is_some() {
        Color::srgba(1.0, 0.24, 0.12, 0.96)
    } else {
        Color::srgba(1.0, 0.66, 0.12, 0.92)
    };
    for (part, mut node, mut transform, mut background, mut border, mut visibility) in &mut parts {
        let line = match part {
            TargetingGuidePart::Shaft => Some(layout.shaft),
            TargetingGuidePart::HeadLeft => Some(layout.head_left),
            TargetingGuidePart::HeadRight => Some(layout.head_right),
            TargetingGuidePart::OriginRing | TargetingGuidePart::TargetRing => None,
        };
        if let Some(line) = line {
            set_line(&mut node, &mut transform, line);
            background.0 = color;
            border.set_all(Color::NONE);
            *visibility = Visibility::Visible;
            continue;
        }

        if *part == TargetingGuidePart::TargetRing && snapped.is_none() {
            *visibility = Visibility::Hidden;
            continue;
        }
        let center = if *part == TargetingGuidePart::OriginRing {
            layout.origin
        } else {
            layout.target
        };
        let diameter = if *part == TargetingGuidePart::OriginRing {
            30.0
        } else {
            38.0
        };
        node.left = px(center.x - diameter * 0.5);
        node.top = px(center.y - diameter * 0.5);
        node.width = px(diameter);
        node.height = px(diameter);
        transform.rotation = Rot2::IDENTITY;
        background.0 = Color::NONE;
        border.set_all(color);
        *visibility = Visibility::Visible;
    }
}

fn hide_all(
    parts: &mut Query<(
        &TargetingGuidePart,
        &mut Node,
        &mut UiTransform,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut Visibility,
    )>,
) {
    for (_, _, _, _, _, mut visibility) in parts {
        *visibility = Visibility::Hidden;
    }
}

fn screen_center(computed: &ComputedNode, global: &UiGlobalTransform) -> Option<Vec2> {
    (computed.size().min_element() > 2.0)
        .then(|| global.affine().translation * computed.inverse_scale_factor)
}

fn cursor_over_node(cursor: Vec2, computed: &ComputedNode, global: &UiGlobalTransform) -> bool {
    let Some(center) = screen_center(computed, global) else {
        return false;
    };
    let size = computed.size() * computed.inverse_scale_factor;
    point_in_rect(cursor, center, size + Vec2::splat(12.0))
}

fn point_in_rect(point: Vec2, center: Vec2, size: Vec2) -> bool {
    let offset = (point - center).abs();
    offset.cmple(size * 0.5).all()
}

fn center_for(
    target: EntityId,
    entities: &Query<(&GameEntity, &ComputedNode, &UiGlobalTransform, &Interaction)>,
) -> Option<Vec2> {
    entities.iter().find_map(|(game, computed, global, _)| {
        (game.0 == target)
            .then(|| screen_center(computed, global))
            .flatten()
    })
}

fn targeting_layout(from: Vec2, to: Vec2, target_clearance: f32) -> Option<TargetingLayout> {
    let delta = to - from;
    let distance = delta.length();
    if distance < MIN_DISTANCE + target_clearance {
        return None;
    }
    let direction = delta / distance;
    let shaft_start = from + direction * SOURCE_CLEARANCE;
    let tip = to - direction * target_clearance;
    let shaft_end = tip - direction * (HEAD_LENGTH * 0.68);
    let backward = -direction;
    let left_end = tip + Rot2::radians(HEAD_SPREAD) * backward * HEAD_LENGTH;
    let right_end = tip + Rot2::radians(-HEAD_SPREAD) * backward * HEAD_LENGTH;
    Some(TargetingLayout {
        shaft: line_layout(shaft_start, shaft_end, SHAFT_THICKNESS),
        head_left: line_layout(tip, left_end, SHAFT_THICKNESS),
        head_right: line_layout(tip, right_end, SHAFT_THICKNESS),
        origin: from,
        target: to,
    })
}

fn line_layout(from: Vec2, to: Vec2, thickness: f32) -> LineLayout {
    let delta = to - from;
    LineLayout {
        center: from.midpoint(to),
        size: Vec2::new(delta.length(), thickness),
        rotation: delta.y.atan2(delta.x),
    }
}

fn set_line(node: &mut Node, transform: &mut UiTransform, line: LineLayout) {
    node.left = px(line.center.x - line.size.x * 0.5);
    node.top = px(line.center.y - line.size.y * 0.5);
    node.width = px(line.size.x);
    node.height = px(line.size.y);
    transform.rotation = Rot2::radians(line.rotation);
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;

    #[test]
    fn horizontal_arrow_leaves_source_and_target_clearance() {
        let layout = targeting_layout(Vec2::ZERO, Vec2::new(240.0, 0.0), TARGET_CLEARANCE)
            .expect("long path");
        assert_eq!(layout.origin, Vec2::ZERO);
        assert_eq!(layout.target, Vec2::new(240.0, 0.0));
        assert_eq!(layout.shaft.rotation, 0.0);
        assert!(layout.shaft.center.x > SOURCE_CLEARANCE);
        assert!(layout.shaft.center.x < 240.0 - TARGET_CLEARANCE);
        assert!(layout.head_left.rotation.abs() > PI * 0.5);
        assert!(layout.head_right.rotation.abs() > PI * 0.5);
    }

    #[test]
    fn vertical_arrow_rotates_its_shaft_and_heads() {
        let layout = targeting_layout(Vec2::ZERO, Vec2::new(0.0, 240.0), CURSOR_CLEARANCE)
            .expect("long path");
        assert!((layout.shaft.rotation - PI * 0.5).abs() < 0.001);
        assert!((layout.head_left.size.x - HEAD_LENGTH).abs() < 0.001);
        assert!((layout.head_right.size.x - HEAD_LENGTH).abs() < 0.001);
    }

    #[test]
    fn short_pointer_motion_does_not_cover_the_source() {
        assert!(targeting_layout(Vec2::ZERO, Vec2::new(40.0, 0.0), CURSOR_CLEARANCE).is_none());
    }

    #[test]
    fn target_hit_test_includes_edges_but_rejects_outside_points() {
        let center = Vec2::new(100.0, 80.0);
        let size = Vec2::new(40.0, 20.0);
        assert!(point_in_rect(center, center, size));
        assert!(point_in_rect(Vec2::new(120.0, 90.0), center, size));
        assert!(!point_in_rect(Vec2::new(120.1, 90.0), center, size));
        assert!(!point_in_rect(Vec2::new(100.0, 90.1), center, size));
    }
}
