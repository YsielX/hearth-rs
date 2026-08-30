use std::collections::{BTreeMap, VecDeque};

use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use thiserror::Error;

use crate::{
    CardDefinition, CardKind, CardRuntime, ChoiceOption, ChoicePolicy, ChoiceValue, EffectDuration,
    EffectSpec, Enchantment, EnchantmentExpiry, EnchantmentId, Entity, EntityId, EventId,
    EventTiming, GameEvent, GameOutcome, GameSnapshot, GameState, LegalAction, MinionDeathRecord,
    ModifierOperation, PendingEvent, PlayerCommand, PlayerId, PlayerState, Replay,
    ReservedSummonOrigin, ResolutionItem, RuneCost, ScriptEvent, SpellCastRecord, Stat,
    StatModifier, Zone, ZonePlacement,
};

mod casting;
mod commands;
mod creation;
mod effect_queue;
mod effects;
mod entities;
mod events;
mod modification;
mod resolution;
mod setup;
mod transitions;
mod zones;

const MAX_HAND_SIZE: usize = 10;
const MAX_BOARD_SIZE: usize = 7;
const MAX_SECRET_SIZE: usize = 5;
const MAX_RESOLUTION_STEPS: usize = 10_000;
const MAX_CHOICE_OPTIONS: usize = 256;
const MAX_RANDOM_CHOICE_OPTIONS: usize = 16 * 1024;
const MAX_CHOICE_PROMPT_BYTES: usize = 4 * 1024;
const MAX_CHOICE_LABEL_BYTES: usize = 1024;
pub const DEFAULT_HERO_POWER: &str = "HERO_08bp";
pub const DEFAULT_COIN: &str = "GAME_005";
/// The official constructed-game limit: player one completes turn 89 as their
/// 45th turn, then the game ends in a draw without starting turn 90.
pub const MAX_GAME_TURNS: u32 = 89;

/// Returns the canonical base portrait for a constructed class. Custom and
/// neutral mechanics sandboxes intentionally keep the built-in generic Hero.
pub fn default_hero_for_class(class: &str) -> Option<&'static str> {
    match class {
        "warrior" => Some("HERO_01"),
        "shaman" => Some("HERO_02"),
        "rogue" => Some("HERO_03"),
        "paladin" => Some("HERO_04"),
        "hunter" => Some("HERO_05"),
        "druid" => Some("HERO_06"),
        "warlock" => Some("HERO_07"),
        "mage" => Some("HERO_08"),
        "priest" => Some("HERO_09"),
        "demon_hunter" => Some("HERO_10"),
        "death_knight" => Some("HERO_11"),
        _ => None,
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GameError {
    #[error("game is already over")]
    GameOver,
    #[error("unknown card definition: {0}")]
    UnknownCard(String),
    #[error("{0} deck is empty")]
    EmptyDeck(PlayerId),
    #[error("{player} deck has {cards} cards; the maximum is {maximum}")]
    DeckTooLarge {
        player: PlayerId,
        cards: usize,
        maximum: usize,
    },
    #[error("{player} deck contains non-collectible or non-deck card {card}")]
    InvalidDeckCard { player: PlayerId, card: String },
    #[error("{player} {class} deck cannot include {card} ({card_class})")]
    InvalidDeckClassCard {
        player: PlayerId,
        class: String,
        card: String,
        card_class: String,
    },
    #[error(
        "{player} Death Knight deck needs {total} rune slots (Blood {blood}, Frost {frost}, Unholy {unholy}); only 3 are available"
    )]
    InvalidDeckRunes {
        player: PlayerId,
        total: u8,
        blood: u8,
        frost: u8,
        unholy: u8,
    },
    #[error("card {0} is not a hero power")]
    InvalidHeroPower(String),
    #[error("card {0} is not a hero")]
    InvalidHero(String),
    #[error("{player} has invalid class {class:?}")]
    InvalidPlayerClass { player: PlayerId, class: String },
    #[error("invalid starting player {0}")]
    InvalidStartingPlayer(PlayerId),
    #[error("unknown entity: {0}")]
    UnknownEntity(EntityId),
    #[error("entity {0} is not in the active player's hand")]
    CardNotInHand(EntityId),
    #[error("card {0} cannot be played right now")]
    CardCannotBePlayed(EntityId),
    #[error("card {0} cannot be traded")]
    CardNotTradeable(EntityId),
    #[error("action {action:?} is not available on card {card}")]
    CardActionUnavailable { card: EntityId, action: String },
    #[error("traded card reservation {0} is invalid")]
    InvalidTradedCard(EntityId),
    #[error("not enough mana: need {needed}, have {available}")]
    NotEnoughMana { needed: u8, available: u8 },
    #[error("not enough Health to pay the card cost: need {needed}, can spend {available}")]
    NotEnoughHealth { needed: u8, available: u8 },
    #[error("a target is required")]
    TargetRequired,
    #[error("target {0} is not valid")]
    InvalidTarget(EntityId),
    #[error("the board is full")]
    BoardFull,
    #[error("board position {position} is invalid; expected 0..={max}")]
    InvalidBoardPosition { position: usize, max: usize },
    #[error("the secret zone is full")]
    SecretZoneFull,
    #[error("entity {0} cannot attack")]
    CannotAttack(EntityId),
    #[error("script error: {0}")]
    Script(String),
    #[error("effect resolution exceeded {MAX_RESOLUTION_STEPS} steps")]
    ResolutionLimit,
    #[error("a player choice is pending")]
    ChoicePending,
    #[error("there is no pending player choice")]
    NoChoicePending,
    #[error("the opening mulligan is still pending")]
    MulliganPending,
    #[error("there is no opening mulligan pending")]
    NoMulliganPending,
    #[error("card {0} is not eligible for the current mulligan")]
    InvalidMulliganCard(EntityId),
    #[error("choice {index} is invalid; expected 0..{options}")]
    InvalidChoice { index: usize, options: usize },
    #[error("card {card_id} ({entity}) requested a choice without options")]
    EmptyChoice { entity: EntityId, card_id: String },
    #[error("a card requested {options} choice options; the maximum is {MAX_CHOICE_OPTIONS}")]
    TooManyChoiceOptions { options: usize },
    #[error("invalid serialized choice value: {0}")]
    InvalidChoiceValue(String),
    #[error("choice prompt must contain between 1 and {MAX_CHOICE_PROMPT_BYTES} bytes")]
    InvalidChoicePrompt,
    #[error("choice option label must contain between 1 and {MAX_CHOICE_LABEL_BYTES} bytes")]
    InvalidChoiceLabel,
    #[error("a card requested a random choice without options")]
    EmptyRandomChoice,
    #[error(
        "a card requested {options} random choice options; the maximum is {MAX_RANDOM_CHOICE_OPTIONS}"
    )]
    TooManyRandomChoiceOptions { options: usize },
    #[error("a card requested discovery from an empty card pool")]
    EmptyDiscoverPool,
    #[error("discover count must be at least one")]
    InvalidDiscoverCount,
    #[error("replay card pack hash is {replay}, but the loaded pack is {loaded}")]
    ReplayPackMismatch { replay: String, loaded: String },
    #[error("replay command {index} failed: {message}")]
    ReplayCommandFailed { index: usize, message: String },
    #[error("unsupported snapshot format {0}")]
    UnsupportedSnapshot(u32),
    #[error("snapshot state does not match its replay proof")]
    SnapshotStateMismatch,
    #[error("script data key must contain between 1 and 64 bytes")]
    InvalidScriptDataKey,
    #[error("hero power has already been used this turn")]
    HeroPowerAlreadyUsed,
    #[error("passive hero powers cannot be activated")]
    PassiveHeroPower,
    #[error("location {0} cannot be used now")]
    CannotUseLocation(EntityId),
    #[error("event {0} is no longer pending and cannot be replaced")]
    EventNotPending(EventId),
    #[error("event {0} does not have a replaceable amount")]
    EventAmountNotReplaceable(EventId),
    #[error("event {0} is not an attack and cannot replace its defender")]
    EventAttackNotReplaceable(EventId),
    #[error("event {0} is not damage and cannot replace its target")]
    EventDamageNotReplaceable(EventId),
    #[error("event {0} is not a targeted spell and cannot replace its target")]
    EventSpellTargetNotReplaceable(EventId),
    #[error("event {0} is not a trade draw and cannot select a replacement")]
    EventTradeDrawNotReplaceable(EventId),
    #[error("event {0} cannot be committed because its reserved entity is invalid")]
    InvalidReservedEntity(EventId),
    #[error("card {0} is not a minion and cannot be summoned")]
    CardCannotBeSummoned(String),
    #[error("entity {0} is not a minion in the requested player's deck and cannot be recruited")]
    EntityCannotBeRecruited(EntityId),
    #[error("card {0} is not a spell and cannot be cast")]
    CardCannotBeCast(String),
    #[error("card {0} is not a weapon and cannot be equipped")]
    CardCannotBeEquipped(String),
    #[error("card {0} is not a minion and cannot be used as a transformation")]
    CardCannotTransformInto(String),
    #[error("entity {entity} cannot move from {zone:?}")]
    EntityCannotMove { entity: EntityId, zone: Zone },
    #[error("game state invariant failed: {0}")]
    Invariant(String),
    #[error("continuation hook must contain between 1 and 64 bytes")]
    InvalidContinuationHook,
    #[error("sideboard {owner} for player {player} is invalid: {message}")]
    InvalidSideboard {
        player: PlayerId,
        owner: String,
        message: String,
    },
    #[error("card {card_id} is not available in sideboard {owner} for player {player}")]
    SideboardCardMissing {
        player: PlayerId,
        owner: String,
        card_id: String,
    },
}

pub struct Game<R> {
    runtime: R,
    state: GameState,
    rng: ChaCha8Rng,
    initial_decks: [Vec<String>; 2],
    initial_sideboards: [BTreeMap<String, Vec<String>>; 2],
    initial_hero_powers: [String; 2],
    initial_classes: [String; 2],
    enforce_deck_classes: [bool; 2],
    command_history: Vec<PlayerCommand>,
}

impl<R: CardRuntime> Game<R> {
    fn apply_damage(
        &mut self,
        source: EntityId,
        target: EntityId,
        amount: i32,
    ) -> Result<GameEvent, GameError> {
        let entity = self
            .state
            .entities
            .get_mut(&target)
            .ok_or(GameError::UnknownEntity(target))?;
        let actual = amount.max(0);
        if entity.kind == CardKind::Location {
            return Ok(GameEvent::DamagePrevented {
                source,
                target,
                reason: "location".to_owned(),
            });
        }
        let absorbed = actual.min(entity.armor);
        entity.armor -= absorbed;
        entity.damage += actual - absorbed;
        if entity.kind == CardKind::Minion && entity.health() <= 0 {
            entity.death_source = Some(source);
        }
        Ok(GameEvent::Damaged {
            source,
            target,
            amount: actual,
        })
    }

    fn apply_spell_damage_bonus(&self, source: EntityId, amount: i32) -> i32 {
        let amount = amount.max(0);
        if amount == 0 {
            return 0;
        }
        let Some(source) = self.state.entity(source) else {
            return amount;
        };
        if source.kind != CardKind::Spell {
            return amount;
        }
        let player = self.state.player(source.controller);
        player
            .board
            .iter()
            .copied()
            .chain(std::iter::once(player.hero))
            .filter_map(|entity| self.state.entity(entity))
            .filter(|entity| matches!(entity.kind, CardKind::Minion | CardKind::Hero))
            .fold(amount, |total, entity| {
                total.saturating_add(entity.spell_damage.max(0))
            })
    }

    fn publish(&mut self, event: GameEvent) -> Result<Vec<EffectSpec>, GameError> {
        let pending = self.begin_event(event)?;
        self.publish_after(pending.id, pending.event)
    }

    fn publish_after(
        &mut self,
        id: EventId,
        event: GameEvent,
    ) -> Result<Vec<EffectSpec>, GameError> {
        let hand_history_changed = self.event_adds_card_to_hand(&event);
        self.record_typed_play_history(&event);
        if hand_history_changed {
            self.refresh_auras()?;
        }
        self.state.record_event(event.clone());
        self.collect_triggers(&ScriptEvent {
            id,
            timing: EventTiming::After,
            event,
        })
    }

    fn publish_after_group(
        &mut self,
        events: Vec<(EventId, GameEvent)>,
    ) -> Result<Vec<EffectSpec>, GameError> {
        let hand_history_changed = events
            .iter()
            .any(|(_, event)| self.event_adds_card_to_hand(event));
        for (_, event) in &events {
            self.record_typed_play_history(event);
            self.state.record_event(event.clone());
        }
        if hand_history_changed {
            self.refresh_auras()?;
        }
        let mut effects = Vec::new();
        for (id, event) in events {
            effects.extend(self.collect_triggers(&ScriptEvent {
                id,
                timing: EventTiming::After,
                event,
            })?);
        }
        Ok(effects)
    }

    fn event_adds_card_to_hand(&self, event: &GameEvent) -> bool {
        match event {
            GameEvent::CardDrawn { card, .. } | GameEvent::CardCreated { card, .. } => self
                .state
                .entity(*card)
                .is_some_and(|entity| entity.zone == Zone::Hand),
            GameEvent::ZoneChanged { to, .. } => *to == Zone::Hand,
            _ => false,
        }
    }

    fn record_typed_play_history(&mut self, event: &GameEvent) {
        if let GameEvent::Healed { target, amount, .. } = event
            && *amount > 0
            && let Some(player) = self
                .state
                .players
                .iter()
                .find(|player| player.hero == *target)
                .map(|player| player.id)
        {
            self.state.player_mut(player).hero_last_healed_turn = Some(self.state.turn);
        }
        if self.state.turn > 0 {
            let added = match event {
                GameEvent::CardDrawn { player, card, .. }
                | GameEvent::CardCreated { player, card, .. }
                    if self
                        .state
                        .entity(*card)
                        .is_some_and(|entity| entity.zone == Zone::Hand) =>
                {
                    Some((*player, *card))
                }
                GameEvent::ZoneChanged {
                    entity,
                    to: Zone::Hand,
                    ..
                } => self
                    .state
                    .entity(*entity)
                    .map(|card| (card.controller, *entity)),
                _ => None,
            };
            if let Some((player, entity)) = added
                && let Some(card_id) = self.state.entity(entity).map(|card| card.card_id.clone())
            {
                self.state
                    .player_mut(player)
                    .cards_added_to_hand_history
                    .push(card_id);
            }
        }
        if let GameEvent::MinionSummoned { player, entity } = event {
            if let Some(card_id) = self
                .state
                .entity(*entity)
                .map(|entity| entity.card_id.clone())
            {
                self.state
                    .player_mut(*player)
                    .minions_summoned_history
                    .push(card_id);
            }
            return;
        }
        let (player, entity, kind) = match event {
            GameEvent::SpellCast {
                player,
                spell,
                generated_by: None,
                cost,
                target_was_friendly_minion,
                ..
            } => {
                if let Some(card_id) = self
                    .state
                    .entity(*spell)
                    .map(|entity| entity.card_id.clone())
                {
                    self.state
                        .player_mut(*player)
                        .spell_cast_records
                        .push(SpellCastRecord {
                            card_id,
                            cost: *cost,
                            target_was_friendly_minion: *target_was_friendly_minion,
                        });
                }
                (*player, *spell, CardKind::Spell)
            }
            GameEvent::SpellCast {
                generated_by: Some(_),
                ..
            } => return,
            GameEvent::MinionPlayed { player, minion } => (*player, *minion, CardKind::Minion),
            GameEvent::WeaponPlayed { player, weapon } => (*player, *weapon, CardKind::Weapon),
            GameEvent::LocationPlayed { player, location } => {
                (*player, *location, CardKind::Location)
            }
            _ => return,
        };
        let Some(card_id) = self
            .state
            .entity(entity)
            .map(|entity| entity.card_id.clone())
        else {
            return;
        };
        let player = self.state.player_mut(player);
        match kind {
            CardKind::Spell => player.spells_cast_history.push(card_id),
            CardKind::Minion => player.minions_played_history.push(card_id),
            CardKind::Weapon => player.weapons_played_history.push(card_id),
            CardKind::Location => player.locations_played_history.push(card_id),
            CardKind::Hero | CardKind::HeroPower => unreachable!(),
        }
    }

    fn collect_triggers(&mut self, event: &ScriptEvent) -> Result<Vec<EffectSpec>, GameError> {
        let active_player = self.state.active_player;
        let mut listeners: Vec<_> = self
            .state
            .entities
            .values()
            .map(|entity| {
                (
                    u8::from(entity.controller != active_player),
                    entity.timestamp,
                    entity.id,
                )
            })
            .collect();
        listeners.sort_unstable();

        let mut effects = Vec::new();
        for (_, _, listener) in listeners {
            let listener_effects = self
                .runtime
                .on_event(&self.state, listener, &event)
                .map_err(GameError::Script)?;
            let repetitions = match &event.event {
                GameEvent::TurnEnded { player, .. }
                    if event.timing == EventTiming::After
                        && self.state.entity(listener).is_some_and(|entity| {
                            entity.zone == Zone::Board && entity.controller == *player
                        }) =>
                {
                    let hero = self.state.player(*player).hero;
                    self.keyword_i32(hero, "end_of_turn_repetitions", 1, None)?
                        .clamp(1, i32::from(u8::MAX)) as usize
                }
                _ => 1,
            };
            for _ in 0..repetitions {
                effects.extend(listener_effects.iter().cloned());
            }
        }
        Ok(effects)
    }

    fn collect_deaths(&self) -> Vec<EntityId> {
        let mut deaths: Vec<_> = self
            .state
            .entities
            .values()
            .filter(|entity| entity.is_mortally_wounded())
            .map(|entity| (entity.timestamp, entity.id))
            .collect();
        deaths.sort_unstable();
        deaths.into_iter().map(|(_, id)| id).collect()
    }

    fn run_death_check(&mut self, queue: &mut VecDeque<ResolutionItem>) -> Result<(), GameError> {
        let deaths = self.collect_deaths();
        if deaths.is_empty() {
            self.check_winner();
            return Ok(());
        }
        let mut death_info = Vec::with_capacity(deaths.len());
        // Hearthstone remembers each death position when that minion is removed, in play
        // order. Earlier deaths therefore no longer occupy a slot when later positions are
        // measured. Lua deathrattles receive this stable position on entity_died.
        for entity in deaths.iter().copied() {
            let controller = self.state.entities[&entity].controller;
            let repetitions = self
                .keyword_i32(entity, "deathrattle_repetitions", 1, None)?
                .clamp(1, i32::from(u8::MAX)) as u8;
            let position = self
                .state
                .player(controller)
                .board
                .iter()
                .position(|candidate| *candidate == entity)
                .expect("mortal minion must be present on its controller's board");
            let source = self.state.entities[&entity].death_source;
            let leaves_corpse = !self.state.entities[&entity].has_keyword("no_corpse");
            death_info.push((
                entity,
                controller,
                position,
                repetitions,
                source,
                leaves_corpse,
            ));
            self.kill(entity);
        }
        self.refresh_auras()?;

        let mut corpse_counts = [0_u32; 2];
        for (_, player, _, _, _, leaves_corpse) in &death_info {
            if self
                .state
                .player(*player)
                .class
                .eq_ignore_ascii_case("death_knight")
                && *leaves_corpse
            {
                corpse_counts[player.index()] = corpse_counts[player.index()].saturating_add(1);
            }
        }
        let mut events = Vec::with_capacity(deaths.len() + 2);
        for player in [PlayerId::ONE, PlayerId::TWO] {
            let amount = corpse_counts[player.index()];
            if amount > 0 {
                let state = self.state.player_mut(player);
                let old = state.corpses;
                state.corpses = old.saturating_add(amount);
                let gained = state.corpses - old;
                if gained > 0 {
                    let event = self.begin_event(GameEvent::CorpsesGained {
                        source: None,
                        player,
                        amount: gained,
                    })?;
                    events.push((event.id, event.event));
                }
            }
        }
        for (entity, player, position, repetitions, source, _) in death_info {
            let event = self.begin_event(GameEvent::EntityDied {
                entity,
                player,
                position,
                source,
                repetitions,
            })?;
            events.push((event.id, event.event));
        }
        let mut items = Vec::with_capacity(2);
        items.push(ResolutionItem::PublishAfterGroup { events });
        items.push(ResolutionItem::DeathCheck);
        for item in items.into_iter().rev() {
            queue.push_front(item);
        }
        Ok(())
    }

    fn any_hero_dead(&self) -> bool {
        self.state.hero(PlayerId::ONE).health() <= 0 || self.state.hero(PlayerId::TWO).health() <= 0
    }

    fn kill(&mut self, entity: EntityId) {
        let controller = self.state.entities[&entity].controller;
        let card_id = self.state.entities[&entity].card_id.clone();
        let attack_at_death = self.state.entities[&entity].attack;
        let had_deathrattle = self.state.entities[&entity]
            .base_keywords
            .iter()
            .any(|keyword| keyword == "deathrattle");
        let keywords = self.state.entities[&entity].keywords.clone();
        let turn = self.state.turn;
        self.state
            .player_mut(controller)
            .minions_died_history
            .push(MinionDeathRecord {
                card_id,
                turn,
                had_deathrattle,
                keywords,
            });
        self.remove_from_zone(entity, Zone::Board, controller);
        self.state
            .entities
            .get_mut(&entity)
            .unwrap()
            .attack_at_death = Some(attack_at_death);
        self.move_to_graveyard(entity, controller);
    }

    fn check_winner(&mut self) {
        if self.state.outcome.is_some() {
            return;
        }
        let dead_one = self.state.hero(PlayerId::ONE).health() <= 0;
        let dead_two = self.state.hero(PlayerId::TWO).health() <= 0;
        let outcome = match (dead_one, dead_two) {
            (true, true) => Some(GameOutcome::Draw),
            (true, false) => Some(GameOutcome::Winner(PlayerId::TWO)),
            (false, true) => Some(GameOutcome::Winner(PlayerId::ONE)),
            (false, false) => None,
        };
        if let Some(outcome) = outcome {
            self.finish_game(outcome);
        }
    }

    fn finish_game(&mut self, outcome: GameOutcome) {
        self.state.outcome = Some(outcome);
        self.state.mulligan = None;
        self.state.pending_input = None;
        self.state.record_event(GameEvent::GameEnded { outcome });
    }
}
