use std::collections::{BTreeMap, BTreeSet};
use std::f32::consts::PI;
use std::time::Duration;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use hearth_app::GameSession;
use hearth_core::{EntityId, Locale, PublicEvent};

use crate::frontend::{ClientScene, FrontendState};
use crate::{GameEntity, GameUiRoot, text_font};

const PULSE_SECONDS: f32 = 0.48;
const FLOAT_SECONDS: f32 = 0.82;
const TRAJECTORY_SECONDS: f32 = 0.58;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PulseKind {
    Attack,
    Damage,
    Healing,
    Summon,
    Blocked,
}

#[derive(Clone, Copy, Debug)]
struct ActivePulse {
    kind: PulseKind,
    elapsed: f32,
}

#[derive(Resource, Default)]
pub struct CombatFeedbackState {
    match_number: Option<u64>,
    cursor: usize,
    pulses: BTreeMap<EntityId, ActivePulse>,
    pending_trajectories: Vec<TrajectorySpec>,
}

#[derive(Component)]
pub(crate) struct CombatFloat {
    timer: Timer,
    color: Color,
    start_y: f32,
}

#[derive(Component)]
pub(crate) struct CombatTrajectory {
    timer: Timer,
    from: Vec2,
    to: Vec2,
    color: Color,
    kind: TrajectoryKind,
}

#[derive(Clone, Debug, PartialEq)]
struct FloatSpec {
    target: EntityId,
    label: String,
    color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PulseSpec {
    target: EntityId,
    kind: PulseKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrajectoryKind {
    Attack,
    Damage,
    Healing,
    Arcane,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TrajectorySpec {
    source: EntityId,
    target: EntityId,
    kind: TrajectoryKind,
}

#[derive(Debug, Default, PartialEq)]
struct EventFeedback {
    floats: Vec<FloatSpec>,
    pulses: Vec<PulseSpec>,
    trajectories: Vec<TrajectorySpec>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ScreenEntity {
    node: Entity,
    center: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TrajectoryLayout {
    center: Vec2,
    size: Vec2,
    rotation: f32,
    alpha: f32,
}

type GameEntityQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GameEntity,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static mut UiTransform,
    ),
    (Without<CombatFloat>, Without<CombatTrajectory>),
>;

type CombatFloatQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut CombatFloat,
        &'static mut Node,
        &'static mut TextColor,
        &'static mut UiTransform,
    ),
    (Without<GameEntity>, Without<CombatTrajectory>),
>;

type CombatTrajectoryQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut CombatTrajectory,
        &'static mut Node,
        &'static mut BackgroundColor,
        &'static mut UiTransform,
    ),
    (Without<GameEntity>, Without<CombatFloat>),
>;

#[derive(SystemParam)]
pub(crate) struct CombatFeedbackTargets<'w, 's> {
    game_entities: GameEntityQuery<'w, 's>,
    roots: Query<
        'w,
        's,
        (Entity, &'static ComputedNode, &'static UiGlobalTransform),
        With<GameUiRoot>,
    >,
    floats: CombatFloatQuery<'w, 's>,
    trajectories: CombatTrajectoryQuery<'w, 's>,
}

pub(crate) fn update_combat_feedback(
    mut commands: Commands,
    time: Res<Time>,
    session: NonSend<GameSession>,
    frontend: Res<FrontendState>,
    mut state: ResMut<CombatFeedbackState>,
    mut targets: CombatFeedbackTargets,
) {
    let view = session.view();
    if state.match_number != Some(frontend.match_number) {
        state.match_number = Some(frontend.match_number);
        state.cursor = 0;
        state.pulses.clear();
        state.pending_trajectories.clear();
    }
    if frontend.match_menu_open {
        return;
    }
    if frontend.scene != ClientScene::Match || frontend.handoff_player.is_some() {
        state.cursor = view.history.len();
        state.pulses.clear();
        state.pending_trajectories.clear();
        for (entity, _, _, _, _) in &mut targets.floats {
            commands.entity(entity).despawn();
        }
        for (entity, _, _, _, _) in &mut targets.trajectories {
            commands.entity(entity).despawn();
        }
        return;
    }
    if state.cursor > view.history.len() {
        state.cursor = 0;
        state.pulses.clear();
        state.pending_trajectories.clear();
    }

    let visible_targets = targets
        .game_entities
        .iter_mut()
        .map(|(entity, game, computed, global, _)| {
            (
                game.0,
                ScreenEntity {
                    node: entity,
                    center: screen_center(computed, global),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let fallback_root = targets
        .roots
        .iter()
        .next()
        .map(|(entity, computed, global)| ScreenEntity {
            node: entity,
            center: screen_center(computed, global),
        });
    let mut float_lanes = BTreeMap::<EntityId, usize>::new();
    let mut fallback_lane = 0usize;
    let mut trajectory_pairs = state
        .pending_trajectories
        .iter()
        .map(|spec| (spec.source, spec.target))
        .collect::<BTreeSet<_>>();
    for record in view.history.iter().skip(state.cursor) {
        let feedback = feedback_for_event(&record.event, session.locale());
        for pulse in feedback.pulses {
            state.pulses.insert(
                pulse.target,
                ActivePulse {
                    kind: pulse.kind,
                    elapsed: 0.0,
                },
            );
        }
        for spec in feedback.trajectories {
            if trajectory_pairs.insert((spec.source, spec.target)) {
                state.pending_trajectories.push(spec);
            }
        }
        for spec in feedback.floats {
            let visible_parent = visible_targets.get(&spec.target).map(|target| target.node);
            let fallback = visible_parent.is_none();
            let lane = if fallback {
                let lane = fallback_lane;
                fallback_lane += 1;
                lane
            } else {
                let lane = float_lanes.entry(spec.target).or_default();
                let current = *lane;
                *lane += 1;
                current
            };
            let parent = visible_parent.or(fallback_root.map(|root| root.node));
            if let Some(parent) = parent {
                spawn_float(&mut commands, parent, &spec, lane, fallback);
            }
        }
    }
    state.cursor = view.history.len();

    // Actions rebuild the match UI before this system runs. Fresh nodes receive
    // their computed layout on the following frame, so retain paths until the
    // full-screen root has valid coordinates.
    if let Some(root) = fallback_root.filter(|root| root.center.length_squared() > 16.0) {
        for spec in std::mem::take(&mut state.pending_trajectories) {
            let Some(from) = resolve_origin(spec.source, &visible_targets, &view) else {
                continue;
            };
            let to = visible_targets
                .get(&spec.target)
                .map_or(root.center, |target| target.center);
            if from.distance_squared(to) > 16.0 {
                spawn_trajectory(&mut commands, root.node, from, to, spec.kind);
            }
        }
    }

    let delta = time.delta_secs();
    for pulse in state.pulses.values_mut() {
        pulse.elapsed += delta;
    }
    state
        .pulses
        .retain(|_, pulse| pulse.elapsed < PULSE_SECONDS);
    for (_, game, _, _, mut transform) in &mut targets.game_entities {
        let Some(pulse) = state.pulses.get(&game.0) else {
            transform.scale = Vec2::ONE;
            transform.rotation = Rot2::IDENTITY;
            continue;
        };
        let progress = (pulse.elapsed / PULSE_SECONDS).clamp(0.0, 1.0);
        let (scale, rotation) = pulse_transform(pulse.kind, progress);
        transform.scale = Vec2::splat(scale);
        transform.rotation = Rot2::radians(rotation);
    }

    for (entity, mut effect, mut node, mut color, mut transform) in &mut targets.floats {
        effect.timer.tick(time.delta());
        let progress = effect.timer.fraction();
        node.top = px(effect.start_y - progress * 58.0);
        let pop = 0.86 + (progress * PI).sin() * 0.32;
        transform.scale = Vec2::splat(pop);
        let alpha = if progress < 0.62 {
            1.0
        } else {
            ((1.0 - progress) / 0.38).clamp(0.0, 1.0)
        };
        color.0 = effect.color.with_alpha(alpha);
        if effect.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut effect, mut node, mut background, mut transform) in &mut targets.trajectories {
        effect.timer.tick(time.delta());
        let layout =
            trajectory_layout(effect.from, effect.to, effect.timer.fraction(), effect.kind);
        node.left = px(layout.center.x - layout.size.x * 0.5);
        node.top = px(layout.center.y - layout.size.y * 0.5);
        node.width = px(layout.size.x);
        node.height = px(layout.size.y);
        transform.rotation = Rot2::radians(layout.rotation);
        background.0 = effect.color.with_alpha(layout.alpha);
        if effect.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn screen_center(computed: &ComputedNode, global: &UiGlobalTransform) -> Vec2 {
    global.affine().translation * computed.inverse_scale_factor
}

fn resolve_origin(
    source: EntityId,
    visible: &BTreeMap<EntityId, ScreenEntity>,
    view: &hearth_core::PlayerView,
) -> Option<Vec2> {
    visible
        .get(&source)
        .map(|entity| entity.center)
        .or_else(|| {
            let controller = view.entity(source)?.controller;
            visible
                .get(&view.player(controller).hero)
                .map(|hero| hero.center)
        })
}

fn spawn_float(
    commands: &mut Commands,
    parent: Entity,
    spec: &FloatSpec,
    lane: usize,
    fallback: bool,
) {
    let start_y = if fallback {
        390.0 - lane as f32 * 24.0
    } else {
        28.0 - lane as f32 * 21.0
    };
    let child = commands
        .spawn((
            CombatFloat {
                timer: Timer::new(Duration::from_secs_f32(FLOAT_SECONDS), TimerMode::Once),
                color: spec.color,
                start_y,
            },
            Text::new(spec.label.clone()),
            text_font(if fallback { 32.0 } else { 30.0 }),
            TextColor(spec.color),
            TextLayout::justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                left: if fallback { percent(42) } else { percent(38) },
                top: px(start_y),
                min_width: px(70),
                ..default()
            },
            UiTransform::from_scale(Vec2::splat(0.86)),
            ZIndex(40),
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(parent).add_child(child);
}

fn spawn_trajectory(
    commands: &mut Commands,
    parent: Entity,
    from: Vec2,
    to: Vec2,
    kind: TrajectoryKind,
) {
    let color = trajectory_color(kind);
    let child = commands
        .spawn((
            CombatTrajectory {
                timer: Timer::new(Duration::from_secs_f32(TRAJECTORY_SECONDS), TimerMode::Once),
                from,
                to,
                color,
                kind,
            },
            Node {
                position_type: PositionType::Absolute,
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BackgroundColor(color),
            UiTransform::default(),
            ZIndex(35),
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(parent).add_child(child);
}

fn trajectory_layout(
    from: Vec2,
    to: Vec2,
    progress: f32,
    kind: TrajectoryKind,
) -> TrajectoryLayout {
    let delta = to - from;
    let distance = delta.length().max(1.0);
    let direction = delta / distance;
    let eased = 1.0 - (1.0 - progress.clamp(0.0, 1.0)).powi(3);
    let head = from.lerp(to, eased);
    let (maximum_length, thickness) = match kind {
        TrajectoryKind::Attack => (96.0, 13.0),
        TrajectoryKind::Damage => (64.0, 9.0),
        TrajectoryKind::Healing => (52.0, 11.0),
        TrajectoryKind::Arcane => (58.0, 12.0),
        TrajectoryKind::Blocked => (44.0, 14.0),
    };
    let length = (distance * 0.24).clamp(28.0, maximum_length);
    let center = head - direction * length * 0.42;
    let fade_in = (progress / 0.08).clamp(0.0, 1.0);
    let fade_out = ((1.0 - progress) / 0.28).clamp(0.0, 1.0);
    TrajectoryLayout {
        center,
        size: Vec2::new(length, thickness),
        rotation: delta.y.atan2(delta.x),
        alpha: fade_in.min(fade_out),
    }
}

fn trajectory_color(kind: TrajectoryKind) -> Color {
    match kind {
        TrajectoryKind::Attack => Color::srgb(1.0, 0.68, 0.12),
        TrajectoryKind::Damage => Color::srgb(1.0, 0.22, 0.12),
        TrajectoryKind::Healing => Color::srgb(0.20, 1.0, 0.46),
        TrajectoryKind::Arcane => Color::srgb(0.28, 0.76, 1.0),
        TrajectoryKind::Blocked => Color::srgb(1.0, 0.88, 0.30),
    }
}

fn pulse_transform(kind: PulseKind, progress: f32) -> (f32, f32) {
    let envelope = 1.0 - progress;
    match kind {
        PulseKind::Attack => (
            1.0 + (progress * PI).sin() * 0.14,
            (progress * PI * 2.0).sin() * 0.035 * envelope,
        ),
        PulseKind::Damage => (
            1.0 + (progress * PI * 4.0).sin().abs() * 0.12 * envelope,
            (progress * PI * 8.0).sin() * 0.065 * envelope,
        ),
        PulseKind::Healing => (1.0 + (progress * PI).sin() * 0.11, 0.0),
        PulseKind::Summon => (0.82 + progress.min(0.45) / 0.45 * 0.18, 0.0),
        PulseKind::Blocked => (
            1.0 + (progress * PI * 2.0).sin().abs() * 0.07 * envelope,
            0.0,
        ),
    }
}

fn feedback_for_event(event: &PublicEvent, locale: Locale) -> EventFeedback {
    let damage = Color::srgb(1.0, 0.26, 0.16);
    let healing = Color::srgb(0.25, 1.0, 0.43);
    let armor = Color::srgb(0.35, 0.78, 1.0);
    let blocked = Color::srgb(0.95, 0.78, 0.25);
    match event {
        PublicEvent::SpellCast {
            spell,
            target: Some(target),
            ..
        }
        | PublicEvent::SpellTargeted { spell, target, .. } => EventFeedback {
            trajectories: vec![TrajectorySpec {
                source: spell.id,
                target: target.id,
                kind: TrajectoryKind::Arcane,
            }],
            ..default()
        },
        PublicEvent::HeroPowerUsed {
            hero_power,
            target: Some(target),
            ..
        } => EventFeedback {
            trajectories: vec![TrajectorySpec {
                source: hero_power.id,
                target: target.id,
                kind: TrajectoryKind::Arcane,
            }],
            ..default()
        },
        PublicEvent::LocationUsed {
            location,
            target: Some(target),
            ..
        } => EventFeedback {
            trajectories: vec![TrajectorySpec {
                source: location.id,
                target: target.id,
                kind: TrajectoryKind::Arcane,
            }],
            ..default()
        },
        PublicEvent::Magnetized {
            attachment, target, ..
        } => EventFeedback {
            trajectories: vec![TrajectorySpec {
                source: attachment.id,
                target: target.id,
                kind: TrajectoryKind::Arcane,
            }],
            ..default()
        },
        PublicEvent::Attack {
            attacker, defender, ..
        } => EventFeedback {
            pulses: vec![
                PulseSpec {
                    target: attacker.id,
                    kind: PulseKind::Attack,
                },
                PulseSpec {
                    target: defender.id,
                    kind: PulseKind::Blocked,
                },
            ],
            trajectories: vec![TrajectorySpec {
                source: attacker.id,
                target: defender.id,
                kind: TrajectoryKind::Attack,
            }],
            ..default()
        },
        PublicEvent::Damaged {
            source,
            target,
            amount,
        } => EventFeedback {
            floats: vec![FloatSpec {
                target: target.id,
                label: format!("-{amount}"),
                color: damage,
            }],
            pulses: vec![PulseSpec {
                target: target.id,
                kind: PulseKind::Damage,
            }],
            trajectories: optional_trajectory(source, target.id, TrajectoryKind::Damage),
        },
        PublicEvent::Healed {
            source,
            target,
            amount,
        } => EventFeedback {
            floats: vec![FloatSpec {
                target: target.id,
                label: format!("+{amount}"),
                color: healing,
            }],
            pulses: vec![PulseSpec {
                target: target.id,
                kind: PulseKind::Healing,
            }],
            trajectories: optional_trajectory(source, target.id, TrajectoryKind::Healing),
        },
        PublicEvent::ArmorGained {
            source,
            target,
            amount,
        } => EventFeedback {
            floats: vec![FloatSpec {
                target: target.id,
                label: format!("+{amount}"),
                color: armor,
            }],
            pulses: vec![PulseSpec {
                target: target.id,
                kind: PulseKind::Healing,
            }],
            trajectories: optional_trajectory(source, target.id, TrajectoryKind::Healing),
        },
        PublicEvent::DamagePrevented { source, target, .. } => EventFeedback {
            floats: vec![FloatSpec {
                target: target.id,
                label: match locale {
                    Locale::EnUs => "BLOCKED",
                    Locale::ZhCn => "已格挡",
                    Locale::ZhTw => "已格擋",
                }
                .to_owned(),
                color: blocked,
            }],
            pulses: vec![PulseSpec {
                target: target.id,
                kind: PulseKind::Blocked,
            }],
            trajectories: optional_trajectory(source, target.id, TrajectoryKind::Blocked),
        },
        PublicEvent::Frozen { source, target } => EventFeedback {
            floats: vec![FloatSpec {
                target: target.id,
                label: match locale {
                    Locale::EnUs => "FROZEN",
                    Locale::ZhCn => "冻结",
                    Locale::ZhTw => "凍結",
                }
                .to_owned(),
                color: armor,
            }],
            pulses: vec![PulseSpec {
                target: target.id,
                kind: PulseKind::Blocked,
            }],
            trajectories: optional_trajectory(source, target.id, TrajectoryKind::Arcane),
        },
        PublicEvent::MinionSummoned { entity, .. } => EventFeedback {
            pulses: vec![PulseSpec {
                target: entity.id,
                kind: PulseKind::Summon,
            }],
            ..default()
        },
        PublicEvent::Transformed { entity, .. } => EventFeedback {
            pulses: vec![PulseSpec {
                target: entity.id,
                kind: PulseKind::Summon,
            }],
            ..default()
        },
        PublicEvent::EntityDied { entity, .. } => EventFeedback {
            floats: vec![FloatSpec {
                target: entity.id,
                label: match locale {
                    Locale::EnUs => "DESTROYED",
                    Locale::ZhCn => "已消灭",
                    Locale::ZhTw => "已消滅",
                }
                .to_owned(),
                color: damage,
            }],
            ..default()
        },
        _ => EventFeedback::default(),
    }
}

fn optional_trajectory(
    source: &Option<hearth_core::PublicEntity>,
    target: EntityId,
    kind: TrajectoryKind,
) -> Vec<TrajectorySpec> {
    source
        .as_ref()
        .filter(|source| source.id != target)
        .map(|source| TrajectorySpec {
            source: source.id,
            target,
            kind,
        })
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use hearth_core::PublicEntity;

    use super::*;

    fn entity(id: u64) -> PublicEntity {
        PublicEntity {
            id: EntityId(id),
            card_id: format!("card-{id}"),
        }
    }

    #[test]
    fn damage_healing_and_attack_create_targeted_feedback() {
        let damaged = feedback_for_event(
            &PublicEvent::Damaged {
                source: None,
                target: entity(2),
                amount: 4,
            },
            Locale::EnUs,
        );
        assert_eq!(damaged.floats[0].label, "-4");
        assert_eq!(damaged.pulses[0].kind, PulseKind::Damage);

        let healed = feedback_for_event(
            &PublicEvent::Healed {
                source: None,
                target: entity(2),
                amount: 3,
            },
            Locale::EnUs,
        );
        assert_eq!(healed.floats[0].label, "+3");
        assert_eq!(healed.pulses[0].kind, PulseKind::Healing);

        let attack = feedback_for_event(
            &PublicEvent::Attack {
                attacker: entity(1),
                defender: entity(2),
                collateral: Vec::new(),
            },
            Locale::EnUs,
        );
        assert_eq!(attack.pulses.len(), 2);
        assert_eq!(attack.pulses[0].target, EntityId(1));
        assert_eq!(attack.pulses[1].target, EntityId(2));
        assert_eq!(
            attack.trajectories,
            vec![TrajectorySpec {
                source: EntityId(1),
                target: EntityId(2),
                kind: TrajectoryKind::Attack,
            }]
        );
    }

    #[test]
    fn status_feedback_is_localized() {
        let blocked = feedback_for_event(
            &PublicEvent::DamagePrevented {
                source: None,
                target: entity(9),
                reason: "divine_shield".to_owned(),
            },
            Locale::ZhCn,
        );
        assert_eq!(blocked.floats[0].label, "已格挡");

        let frozen = feedback_for_event(
            &PublicEvent::Frozen {
                source: None,
                target: entity(9),
            },
            Locale::ZhTw,
        );
        assert_eq!(frozen.floats[0].label, "凍結");
    }

    #[test]
    fn targeted_effects_route_from_source_to_target() {
        let feedback = feedback_for_event(
            &PublicEvent::HeroPowerUsed {
                player: hearth_core::PlayerId::ONE,
                hero_power: entity(5),
                target: Some(entity(9)),
            },
            Locale::EnUs,
        );
        assert_eq!(
            feedback.trajectories,
            vec![TrajectorySpec {
                source: EntityId(5),
                target: EntityId(9),
                kind: TrajectoryKind::Arcane,
            }]
        );
    }

    #[test]
    fn trajectory_geometry_moves_toward_target_and_preserves_direction() {
        let start = trajectory_layout(
            Vec2::ZERO,
            Vec2::new(100.0, 0.0),
            0.0,
            TrajectoryKind::Attack,
        );
        let middle = trajectory_layout(
            Vec2::ZERO,
            Vec2::new(100.0, 0.0),
            0.5,
            TrajectoryKind::Attack,
        );
        assert!(middle.center.x > start.center.x);
        assert_eq!(middle.rotation, 0.0);
        assert!(middle.alpha > 0.9);

        let vertical = trajectory_layout(
            Vec2::ZERO,
            Vec2::new(0.0, 100.0),
            0.5,
            TrajectoryKind::Arcane,
        );
        assert!((vertical.rotation - PI * 0.5).abs() < 0.0001);
        assert!(vertical.size.x > vertical.size.y);
    }
}
