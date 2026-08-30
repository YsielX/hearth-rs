use hearth_core::{
    GameOutcome, PlayerId, PlayerView, PublicEntity, PublicEvent, PublicEventRecord, Zone,
};
use serde::{Deserialize, Serialize};

use crate::entity_refs::EpisodeRefs;
use crate::{EntityRef, EnvError, RelativePlayer};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicHistory {
    pub cards_played: Vec<String>,
    pub spells_cast: Vec<String>,
    pub minions_played: Vec<String>,
    pub weapons_played: Vec<String>,
    pub locations_played: Vec<String>,
    pub discarded_cards: Vec<String>,
    pub minions_died: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    GameStarted,
    TurnStarted,
    CardDrawn,
    CardBurned,
    CardCreated,
    Fatigue,
    CardPlayed,
    SpellCast,
    SpellTargeted,
    MinionPlayed,
    WeaponPlayed,
    LocationPlayed,
    CardCountered,
    CardDiscarded,
    CardTraded,
    TradeDraw,
    MinionSummoned,
    Magnetized,
    WeaponEquipped,
    WeaponDestroyed,
    LocationUsed,
    LocationDestroyed,
    HeroPowerUsed,
    HeroPowerReplaced,
    HeroReplaced,
    SecretPlayed,
    SecretRevealed,
    ZoneChanged,
    ControllerChanged,
    Transformed,
    Attack,
    Damaged,
    DamagePrevented,
    Healed,
    ArmorGained,
    OverloadQueued,
    ManaLocked,
    ManaUnlocked,
    OverloadCleared,
    TemporaryManaGained,
    TemporaryManaExpired,
    ManaCrystalsGained,
    ManaCrystalsDestroyed,
    ManaSpent,
    PlayerResourceGained,
    PlayerResourceSpent,
    KeywordDisabled,
    Frozen,
    EntityDied,
    TurnEnded,
    Conceded,
    GameEnded,
    ChoiceRequested,
    ChoiceMade,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventEntityRole {
    Card,
    Source,
    Spell,
    Target,
    Minion,
    Weapon,
    Location,
    HeroPower,
    Old,
    New,
    Secret,
    Entity,
    Attachment,
    Attacker,
    Defender,
    Collateral,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventEntityObservation {
    pub role: EventEntityRole,
    pub entity: EntityRef,
    /// Identity frozen at event time rather than read from the current entity.
    pub card_id: String,
    /// Role-specific scalar, currently used for collateral damage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeObservation {
    Winner(RelativePlayer),
    Draw,
}

/// Framework-neutral event envelope. `kind` determines which optional fields
/// are meaningful; absent information was either irrelevant or not visible to
/// this player in the core `PublicEvent` stream.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventObservation {
    pub kind: EventKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player: Option<RelativePlayer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_player: Option<RelativePlayer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_player: Option<RelativePlayer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<EventEntityObservation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporary: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_turn: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_zone: Option<Zone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_zone: Option<Zone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_card_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_card_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<OutcomeObservation>,
}

impl EventObservation {
    fn new(kind: EventKind) -> Self {
        Self {
            kind,
            player: None,
            from_player: None,
            to_player: None,
            entities: Vec::new(),
            amount: None,
            cost: None,
            pending: None,
            locked: None,
            temporary: None,
            position: None,
            option_count: None,
            choice_index: None,
            event_turn: None,
            from_zone: None,
            to_zone: None,
            from_card_id: None,
            to_card_id: None,
            keyword: None,
            resource: None,
            reason: None,
            filled: None,
            outcome: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRecordObservation {
    /// Position in this viewer's public stream. This deliberately does not
    /// expose the core event-log sequence, whose gaps describe hidden events.
    pub cursor: u64,
    pub turn: u32,
    pub event: EventObservation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventWindow {
    /// Offset in this player's public event stream, independent of hidden gaps.
    pub start_cursor: u64,
    pub next_cursor: u64,
    pub has_earlier_events: bool,
    pub events: Vec<EventRecordObservation>,
}

#[derive(Default)]
pub(crate) struct ViewerMemory {
    pub(crate) refs: EpisodeRefs,
    histories: [PublicHistory; 2],
    events: Vec<EventRecordObservation>,
    processed_events: usize,
}

impl ViewerMemory {
    pub(crate) fn sync(&mut self, view: &PlayerView) -> Result<(), EnvError> {
        if view.history.len() < self.processed_events {
            return Err(EnvError::PublicHistoryRewound {
                processed: self.processed_events,
                available: view.history.len(),
            });
        }
        for record in &view.history[self.processed_events..] {
            apply_aggregate(&mut self.histories, &record.event);
            let cursor =
                u64::try_from(self.events.len()).map_err(|_| EnvError::PublicEventValueTooLarge)?;
            self.events
                .push(encode_record(record, cursor, view.viewer, &mut self.refs)?);
        }
        self.processed_events = view.history.len();
        Ok(())
    }

    pub(crate) fn histories(&self) -> &[PublicHistory; 2] {
        &self.histories
    }

    pub(crate) fn window(&self, limit: Option<usize>) -> EventWindow {
        let start = limit
            .map(|limit| self.events.len().saturating_sub(limit))
            .unwrap_or(0);
        EventWindow {
            start_cursor: start as u64,
            next_cursor: self.events.len() as u64,
            has_earlier_events: start > 0,
            events: self.events[start..].to_vec(),
        }
    }
}

fn apply_aggregate(histories: &mut [PublicHistory; 2], event: &PublicEvent) {
    match event {
        PublicEvent::CardPlayed { player, card, .. }
        | PublicEvent::CardCountered { player, card } => histories[player.index()]
            .cards_played
            .push(card.card_id.clone()),
        PublicEvent::SpellCast { player, spell, .. } => histories[player.index()]
            .spells_cast
            .push(spell.card_id.clone()),
        PublicEvent::MinionPlayed { player, minion } => histories[player.index()]
            .minions_played
            .push(minion.card_id.clone()),
        PublicEvent::WeaponPlayed { player, weapon } => histories[player.index()]
            .weapons_played
            .push(weapon.card_id.clone()),
        PublicEvent::LocationPlayed { player, location } => histories[player.index()]
            .locations_played
            .push(location.card_id.clone()),
        PublicEvent::CardDiscarded { player, card, .. } => histories[player.index()]
            .discarded_cards
            .push(card.card_id.clone()),
        PublicEvent::EntityDied { player, entity, .. } => histories[player.index()]
            .minions_died
            .push(entity.card_id.clone()),
        _ => {}
    }
}

fn relative(player: PlayerId, viewer: PlayerId) -> RelativePlayer {
    RelativePlayer::from_player(player, viewer)
}

fn push_entity(
    output: &mut EventObservation,
    refs: &mut EpisodeRefs,
    role: EventEntityRole,
    entity: &PublicEntity,
    value: Option<i64>,
) -> Result<(), EnvError> {
    output.entities.push(EventEntityObservation {
        role,
        entity: refs.observe_public(entity)?,
        card_id: entity.card_id.clone(),
        value,
    });
    Ok(())
}

fn push_optional(
    output: &mut EventObservation,
    refs: &mut EpisodeRefs,
    role: EventEntityRole,
    entity: Option<&PublicEntity>,
) -> Result<(), EnvError> {
    if let Some(entity) = entity {
        push_entity(output, refs, role, entity, None)?;
    }
    Ok(())
}

fn encode_record(
    record: &PublicEventRecord,
    cursor: u64,
    viewer: PlayerId,
    refs: &mut EpisodeRefs,
) -> Result<EventRecordObservation, EnvError> {
    let event = match &record.event {
        PublicEvent::GameStarted => EventObservation::new(EventKind::GameStarted),
        PublicEvent::TurnStarted { player, turn } => {
            let mut output = EventObservation::new(EventKind::TurnStarted);
            output.player = Some(relative(*player, viewer));
            output.event_turn = Some(*turn);
            output
        }
        PublicEvent::CardDrawn {
            player,
            card,
            source,
        } => {
            let mut output = EventObservation::new(EventKind::CardDrawn);
            output.player = Some(relative(*player, viewer));
            push_optional(&mut output, refs, EventEntityRole::Card, card.as_ref())?;
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::CardBurned {
            player,
            card,
            source,
        } => {
            let mut output = EventObservation::new(EventKind::CardBurned);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Card, card, None)?;
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::CardCreated {
            player,
            card,
            source,
        } => {
            let mut output = EventObservation::new(EventKind::CardCreated);
            output.player = Some(relative(*player, viewer));
            push_optional(&mut output, refs, EventEntityRole::Card, card.as_ref())?;
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::Fatigue { player, amount } => {
            let mut output = EventObservation::new(EventKind::Fatigue);
            output.player = Some(relative(*player, viewer));
            output.amount = Some(i64::from(*amount));
            output
        }
        PublicEvent::CardPlayed { player, card, cost } => {
            let mut output = EventObservation::new(EventKind::CardPlayed);
            output.player = Some(relative(*player, viewer));
            output.cost = Some(*cost);
            push_entity(&mut output, refs, EventEntityRole::Card, card, None)?;
            output
        }
        PublicEvent::SpellCast {
            player,
            spell,
            generated_by,
            target,
            cost,
        } => {
            let mut output = EventObservation::new(EventKind::SpellCast);
            output.player = Some(relative(*player, viewer));
            output.cost = Some(*cost);
            push_entity(&mut output, refs, EventEntityRole::Spell, spell, None)?;
            push_optional(
                &mut output,
                refs,
                EventEntityRole::Source,
                generated_by.as_ref(),
            )?;
            push_optional(&mut output, refs, EventEntityRole::Target, target.as_ref())?;
            output
        }
        PublicEvent::SpellTargeted {
            player,
            spell,
            target,
            generated_by,
        } => {
            let mut output = EventObservation::new(EventKind::SpellTargeted);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Spell, spell, None)?;
            push_entity(&mut output, refs, EventEntityRole::Target, target, None)?;
            push_optional(
                &mut output,
                refs,
                EventEntityRole::Source,
                generated_by.as_ref(),
            )?;
            output
        }
        PublicEvent::MinionPlayed { player, minion } => {
            let mut output = EventObservation::new(EventKind::MinionPlayed);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Minion, minion, None)?;
            output
        }
        PublicEvent::WeaponPlayed { player, weapon } => {
            let mut output = EventObservation::new(EventKind::WeaponPlayed);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Weapon, weapon, None)?;
            output
        }
        PublicEvent::LocationPlayed { player, location } => {
            let mut output = EventObservation::new(EventKind::LocationPlayed);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Location, location, None)?;
            output
        }
        PublicEvent::CardCountered { player, card } => {
            let mut output = EventObservation::new(EventKind::CardCountered);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Card, card, None)?;
            output
        }
        PublicEvent::CardDiscarded {
            player,
            card,
            source,
        } => {
            let mut output = EventObservation::new(EventKind::CardDiscarded);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Card, card, None)?;
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::CardTraded { player, card } => {
            let mut output = EventObservation::new(EventKind::CardTraded);
            output.player = Some(relative(*player, viewer));
            push_optional(&mut output, refs, EventEntityRole::Card, card.as_ref())?;
            output
        }
        PublicEvent::TradeDraw { player } => {
            let mut output = EventObservation::new(EventKind::TradeDraw);
            output.player = Some(relative(*player, viewer));
            output
        }
        PublicEvent::MinionSummoned { player, entity } => {
            let mut output = EventObservation::new(EventKind::MinionSummoned);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Entity, entity, None)?;
            output
        }
        PublicEvent::Magnetized {
            player,
            attachment,
            target,
        } => {
            let mut output = EventObservation::new(EventKind::Magnetized);
            output.player = Some(relative(*player, viewer));
            push_entity(
                &mut output,
                refs,
                EventEntityRole::Attachment,
                attachment,
                None,
            )?;
            push_entity(&mut output, refs, EventEntityRole::Target, target, None)?;
            output
        }
        PublicEvent::WeaponEquipped { player, weapon } => {
            let mut output = EventObservation::new(EventKind::WeaponEquipped);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Weapon, weapon, None)?;
            output
        }
        PublicEvent::WeaponDestroyed { player, weapon } => {
            let mut output = EventObservation::new(EventKind::WeaponDestroyed);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Weapon, weapon, None)?;
            output
        }
        PublicEvent::LocationUsed {
            player,
            location,
            target,
        } => {
            let mut output = EventObservation::new(EventKind::LocationUsed);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Location, location, None)?;
            push_optional(&mut output, refs, EventEntityRole::Target, target.as_ref())?;
            output
        }
        PublicEvent::LocationDestroyed { player, location } => {
            let mut output = EventObservation::new(EventKind::LocationDestroyed);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Location, location, None)?;
            output
        }
        PublicEvent::HeroPowerUsed {
            player,
            hero_power,
            target,
        } => {
            let mut output = EventObservation::new(EventKind::HeroPowerUsed);
            output.player = Some(relative(*player, viewer));
            push_entity(
                &mut output,
                refs,
                EventEntityRole::HeroPower,
                hero_power,
                None,
            )?;
            push_optional(&mut output, refs, EventEntityRole::Target, target.as_ref())?;
            output
        }
        PublicEvent::HeroPowerReplaced {
            player,
            source,
            old,
            new,
        } => {
            let mut output = EventObservation::new(EventKind::HeroPowerReplaced);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Source, source, None)?;
            push_entity(&mut output, refs, EventEntityRole::Old, old, None)?;
            push_entity(&mut output, refs, EventEntityRole::New, new, None)?;
            output
        }
        PublicEvent::HeroReplaced { player, old, new } => {
            let mut output = EventObservation::new(EventKind::HeroReplaced);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Old, old, None)?;
            push_entity(&mut output, refs, EventEntityRole::New, new, None)?;
            output
        }
        PublicEvent::SecretPlayed { player, secret } => {
            let mut output = EventObservation::new(EventKind::SecretPlayed);
            output.player = Some(relative(*player, viewer));
            push_optional(&mut output, refs, EventEntityRole::Secret, secret.as_ref())?;
            output
        }
        PublicEvent::SecretRevealed { player, secret } => {
            let mut output = EventObservation::new(EventKind::SecretRevealed);
            output.player = Some(relative(*player, viewer));
            push_entity(&mut output, refs, EventEntityRole::Secret, secret, None)?;
            output
        }
        PublicEvent::ZoneChanged { entity, from, to } => {
            let mut output = EventObservation::new(EventKind::ZoneChanged);
            output.from_zone = Some(*from);
            output.to_zone = Some(*to);
            push_entity(&mut output, refs, EventEntityRole::Entity, entity, None)?;
            output
        }
        PublicEvent::ControllerChanged {
            source,
            entity,
            from,
            to,
        } => {
            let mut output = EventObservation::new(EventKind::ControllerChanged);
            output.from_player = Some(relative(*from, viewer));
            output.to_player = Some(relative(*to, viewer));
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            push_entity(&mut output, refs, EventEntityRole::Entity, entity, None)?;
            output
        }
        PublicEvent::Transformed {
            source,
            entity,
            from_card,
            to_card,
        } => {
            let mut output = EventObservation::new(EventKind::Transformed);
            output.from_card_id = Some(from_card.clone());
            output.to_card_id = Some(to_card.clone());
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            push_entity(&mut output, refs, EventEntityRole::Entity, entity, None)?;
            output
        }
        PublicEvent::Attack {
            attacker,
            defender,
            collateral,
        } => {
            let mut output = EventObservation::new(EventKind::Attack);
            push_entity(&mut output, refs, EventEntityRole::Attacker, attacker, None)?;
            push_entity(&mut output, refs, EventEntityRole::Defender, defender, None)?;
            for (entity, amount) in collateral {
                push_entity(
                    &mut output,
                    refs,
                    EventEntityRole::Collateral,
                    entity,
                    Some(i64::from(*amount)),
                )?;
            }
            output
        }
        PublicEvent::Damaged {
            source,
            target,
            amount,
        } => {
            let mut output = EventObservation::new(EventKind::Damaged);
            output.amount = Some(i64::from(*amount));
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            push_entity(&mut output, refs, EventEntityRole::Target, target, None)?;
            output
        }
        PublicEvent::DamagePrevented {
            source,
            target,
            reason,
        } => {
            let mut output = EventObservation::new(EventKind::DamagePrevented);
            output.reason = Some(reason.clone());
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            push_entity(&mut output, refs, EventEntityRole::Target, target, None)?;
            output
        }
        PublicEvent::Healed {
            source,
            target,
            amount,
        } => {
            let mut output = EventObservation::new(EventKind::Healed);
            output.amount = Some(i64::from(*amount));
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            push_entity(&mut output, refs, EventEntityRole::Target, target, None)?;
            output
        }
        PublicEvent::ArmorGained {
            source,
            target,
            amount,
        } => {
            let mut output = EventObservation::new(EventKind::ArmorGained);
            output.amount = Some(i64::from(*amount));
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            push_entity(&mut output, refs, EventEntityRole::Target, target, None)?;
            output
        }
        PublicEvent::OverloadQueued {
            player,
            source,
            amount,
        } => {
            let mut output = EventObservation::new(EventKind::OverloadQueued);
            output.player = Some(relative(*player, viewer));
            output.amount = Some(i64::from(*amount));
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::ManaLocked { player, amount } => {
            let mut output = EventObservation::new(EventKind::ManaLocked);
            output.player = Some(relative(*player, viewer));
            output.amount = Some(i64::from(*amount));
            output
        }
        PublicEvent::ManaUnlocked {
            player,
            source,
            amount,
        } => {
            let mut output = EventObservation::new(EventKind::ManaUnlocked);
            output.player = Some(relative(*player, viewer));
            output.amount = Some(i64::from(*amount));
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::OverloadCleared {
            player,
            source,
            pending,
            locked,
        } => {
            let mut output = EventObservation::new(EventKind::OverloadCleared);
            output.player = Some(relative(*player, viewer));
            output.pending = Some(*pending);
            output.locked = Some(*locked);
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::TemporaryManaGained {
            player,
            source,
            amount,
        } => {
            let mut output = EventObservation::new(EventKind::TemporaryManaGained);
            output.player = Some(relative(*player, viewer));
            output.amount = Some(i64::from(*amount));
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::TemporaryManaExpired { player, amount } => {
            let mut output = EventObservation::new(EventKind::TemporaryManaExpired);
            output.player = Some(relative(*player, viewer));
            output.amount = Some(i64::from(*amount));
            output
        }
        PublicEvent::ManaCrystalsGained {
            player,
            source,
            amount,
            filled,
        } => {
            let mut output = EventObservation::new(EventKind::ManaCrystalsGained);
            output.player = Some(relative(*player, viewer));
            output.amount = Some(i64::from(*amount));
            output.filled = Some(*filled);
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::ManaCrystalsDestroyed {
            player,
            source,
            amount,
        } => {
            let mut output = EventObservation::new(EventKind::ManaCrystalsDestroyed);
            output.player = Some(relative(*player, viewer));
            output.amount = Some(i64::from(*amount));
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::ManaSpent {
            player,
            source,
            amount,
            temporary,
        } => {
            let mut output = EventObservation::new(EventKind::ManaSpent);
            output.player = Some(relative(*player, viewer));
            output.amount = Some(i64::from(*amount));
            output.temporary = Some(*temporary);
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::PlayerResourceGained {
            player,
            source,
            resource,
            amount,
        } => {
            let mut output = EventObservation::new(EventKind::PlayerResourceGained);
            output.player = Some(relative(*player, viewer));
            output.resource = Some(resource.clone());
            output.amount = Some(i64::from(*amount));
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::PlayerResourceSpent {
            player,
            source,
            resource,
            amount,
        } => {
            let mut output = EventObservation::new(EventKind::PlayerResourceSpent);
            output.player = Some(relative(*player, viewer));
            output.resource = Some(resource.clone());
            output.amount = Some(i64::from(*amount));
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::KeywordDisabled {
            source,
            target,
            keyword,
        } => {
            let mut output = EventObservation::new(EventKind::KeywordDisabled);
            output.keyword = Some(keyword.clone());
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            push_entity(&mut output, refs, EventEntityRole::Target, target, None)?;
            output
        }
        PublicEvent::Frozen { source, target } => {
            let mut output = EventObservation::new(EventKind::Frozen);
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            push_entity(&mut output, refs, EventEntityRole::Target, target, None)?;
            output
        }
        PublicEvent::EntityDied {
            entity,
            player,
            position,
            source,
        } => {
            let mut output = EventObservation::new(EventKind::EntityDied);
            output.player = Some(relative(*player, viewer));
            output.position =
                Some(u32::try_from(*position).map_err(|_| EnvError::PublicEventValueTooLarge)?);
            push_entity(&mut output, refs, EventEntityRole::Entity, entity, None)?;
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::TurnEnded { player, turn } => {
            let mut output = EventObservation::new(EventKind::TurnEnded);
            output.player = Some(relative(*player, viewer));
            output.event_turn = Some(*turn);
            output
        }
        PublicEvent::Conceded { player } => {
            let mut output = EventObservation::new(EventKind::Conceded);
            output.player = Some(relative(*player, viewer));
            output
        }
        PublicEvent::GameEnded { outcome } => {
            let mut output = EventObservation::new(EventKind::GameEnded);
            output.outcome = Some(match outcome {
                GameOutcome::Winner(player) => {
                    OutcomeObservation::Winner(relative(*player, viewer))
                }
                GameOutcome::Draw => OutcomeObservation::Draw,
            });
            output
        }
        PublicEvent::ChoiceRequested {
            player,
            source,
            options,
        } => {
            let mut output = EventObservation::new(EventKind::ChoiceRequested);
            output.player = Some(relative(*player, viewer));
            output.option_count =
                Some(u32::try_from(*options).map_err(|_| EnvError::PublicEventValueTooLarge)?);
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
        PublicEvent::ChoiceMade {
            player,
            source,
            index,
        } => {
            let mut output = EventObservation::new(EventKind::ChoiceMade);
            output.player = Some(relative(*player, viewer));
            output.choice_index = index
                .map(|index| u32::try_from(index).map_err(|_| EnvError::PublicEventValueTooLarge))
                .transpose()?;
            push_optional(&mut output, refs, EventEntityRole::Source, source.as_ref())?;
            output
        }
    };

    Ok(EventRecordObservation {
        cursor,
        turn: record.turn,
        event,
    })
}
