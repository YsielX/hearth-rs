//! A learning-framework-neutral environment adapter for `hearth-rs`.
//!
//! The rules engine remains authoritative and knows nothing about tensor
//! layouts, rewards, episode truncation, or action indices. This crate owns
//! those concerns and never exposes authoritative entity IDs to a policy.

use std::collections::BTreeMap;
use std::path::Path;

use hearth_core::{
    CardKind, CardRuntime, DEFAULT_HERO_POWER, EntityId, Game, GameError, GameOutcome, LegalAction,
    PlayerCommand, PlayerId, PlayerView, PublicEvent, Replay,
};
use hearth_script::{LuaCardRuntime, ScriptLoadError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const OBSERVATION_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchConfig {
    pub decks: [Vec<String>; 2],
    #[serde(default = "default_hero_powers")]
    pub hero_powers: [String; 2],
    #[serde(default = "default_classes")]
    pub classes: [String; 2],
    #[serde(default)]
    pub unrestricted: bool,
}

fn default_hero_powers() -> [String; 2] {
    [DEFAULT_HERO_POWER.to_owned(), DEFAULT_HERO_POWER.to_owned()]
}

fn default_classes() -> [String; 2] {
    ["mage".to_owned(), "mage".to_owned()]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelativePlayer {
    SelfPlayer,
    Opponent,
}

impl RelativePlayer {
    fn from_player(player: PlayerId, viewer: PlayerId) -> Self {
        if player == viewer {
            Self::SelfPlayer
        } else {
            Self::Opponent
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionPhase {
    Mulligan,
    Choice,
    Main,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityRef(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityArea {
    Hero,
    HeroPower,
    Weapon,
    Board,
    Hand,
    Secret,
    PublicObjective,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityObservation {
    pub entity: EntityRef,
    pub card_id: String,
    pub kind: CardKind,
    pub owner: RelativePlayer,
    pub controller: RelativePlayer,
    pub area: EntityArea,
    pub position: u8,
    pub attack: i32,
    pub max_health: i32,
    pub damage: i32,
    pub armor: i32,
    pub cost: u8,
    pub spell_damage: i32,
    pub exhausted: bool,
    pub frozen: bool,
    pub attacks_this_turn: u8,
    pub location_cooldown: u8,
    pub keywords: Vec<String>,
    pub silenced: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicHistory {
    pub cards_played: Vec<String>,
    pub spells_cast: Vec<String>,
    pub minions_played: Vec<String>,
    pub weapons_played: Vec<String>,
    pub locations_played: Vec<String>,
    pub discarded_cards: Vec<String>,
    pub minions_died: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerObservation {
    pub class: String,
    pub hero: EntityRef,
    pub hero_power: EntityRef,
    pub weapon: Option<EntityRef>,
    pub board: Vec<EntityRef>,
    /// Empty for the opponent.
    pub hand: Vec<EntityRef>,
    /// Ordinary Secret identities are present only for the observing player.
    pub secrets: Vec<EntityRef>,
    pub public_objectives: Vec<EntityRef>,
    pub deck_size: u8,
    pub hand_size: u8,
    pub secrets_count: u8,
    pub mana: u8,
    pub max_mana: u8,
    pub temporary_mana: u8,
    pub overload_pending: u8,
    pub overloaded_mana: u8,
    pub fatigue: u32,
    pub hero_power_used: bool,
    pub hero_power_uses_this_turn: u8,
    pub cards_played_this_turn: u32,
    pub history: PublicHistory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceObservation {
    pub prompt: String,
    pub options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub schema_version: u32,
    /// Physical seat is metadata, never an input for ownership decisions.
    pub seat: u8,
    pub turn: u32,
    pub active_player: RelativePlayer,
    pub phase: DecisionPhase,
    pub self_player: PlayerObservation,
    pub opponent: PlayerObservation,
    pub entities: Vec<EntityObservation>,
    pub mulligan_eligible: Vec<EntityRef>,
    pub pending_choice: Option<ChoiceObservation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Mulligan,
    PlayCard,
    PlayCardAt,
    TradeCard,
    UseCardAction,
    Attack,
    UseHeroPower,
    UseLocation,
    EndTurn,
    Concede,
    Choose,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionObservation {
    /// Index accepted by `HearthEnv::step` for this decision only.
    pub index: u32,
    pub kind: ActionKind,
    pub sources: Vec<EntityRef>,
    pub target: Option<EntityRef>,
    pub board_position: Option<u8>,
    pub mana_cost: u8,
    pub card_action: Option<String>,
    pub choice_index: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    /// Prevents an action index from an older observation being reused.
    pub id: u64,
    pub actor_seat: u8,
    pub observation: Observation,
    pub actions: Vec<ActionObservation>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    /// Rewards use physical seat order `[P1, P2]`.
    pub rewards: [f32; 2],
    pub terminated: bool,
    pub truncated: bool,
    pub outcome: Option<GameOutcome>,
    pub next: Option<Decision>,
}

#[derive(Debug, Error)]
pub enum EnvError {
    #[error(transparent)]
    ScriptLoad(#[from] ScriptLoadError),
    #[error(transparent)]
    Game(#[from] GameError),
    #[error("the episode has already ended")]
    EpisodeEnded,
    #[error("stale decision {received}; current decision is {expected}")]
    StaleDecision { expected: u64, received: u64 },
    #[error("action index {index} is outside 0..{actions}")]
    InvalidActionIndex { index: usize, actions: usize },
    #[error("legal action refers to entity {0} that is absent from the player's view")]
    HiddenActionEntity(EntityId),
    #[error("player view contains too many visible entities")]
    TooManyVisibleEntities,
    #[error("environment has no active game")]
    MissingGame,
}

struct CachedDecision {
    public: Decision,
    commands: Vec<PlayerCommand>,
}

pub struct HearthEnv {
    game: Option<Game<LuaCardRuntime>>,
    match_config: MatchConfig,
    max_steps: usize,
    steps: usize,
    next_decision_id: u64,
    current: Option<CachedDecision>,
    truncated: bool,
}

impl HearthEnv {
    pub fn load(
        data_path: impl AsRef<Path>,
        match_config: MatchConfig,
        seed: u64,
        max_steps: usize,
    ) -> Result<Self, EnvError> {
        let runtime = LuaCardRuntime::load_dir(data_path)?;
        Self::with_runtime(runtime, match_config, seed, max_steps)
    }

    pub fn with_runtime(
        runtime: LuaCardRuntime,
        match_config: MatchConfig,
        seed: u64,
        max_steps: usize,
    ) -> Result<Self, EnvError> {
        let game = build_game(runtime, &match_config, seed)?;
        let mut environment = Self {
            game: Some(game),
            match_config,
            max_steps,
            steps: 0,
            next_decision_id: 1,
            current: None,
            truncated: false,
        };
        environment.refresh_decision()?;
        Ok(environment)
    }

    pub fn reset(&mut self, seed: u64) -> Result<&Decision, EnvError> {
        let old_game = self.game.take().ok_or(EnvError::MissingGame)?;
        let runtime = old_game.into_runtime();
        self.game = Some(build_game(runtime, &self.match_config, seed)?);
        self.steps = 0;
        self.truncated = false;
        self.current = None;
        self.refresh_decision()?;
        self.decision().ok_or(EnvError::MissingGame)
    }

    pub fn decision(&self) -> Option<&Decision> {
        self.current.as_ref().map(|decision| &decision.public)
    }

    pub fn step(&mut self, decision_id: u64, action_index: usize) -> Result<Transition, EnvError> {
        if self.truncated
            || self
                .game
                .as_ref()
                .ok_or(EnvError::MissingGame)?
                .state()
                .outcome
                .is_some()
        {
            return Err(EnvError::EpisodeEnded);
        }
        let cached = self.current.take().ok_or(EnvError::EpisodeEnded)?;
        if cached.public.id != decision_id {
            let expected = cached.public.id;
            self.current = Some(cached);
            return Err(EnvError::StaleDecision {
                expected,
                received: decision_id,
            });
        }
        let Some(command) = cached.commands.get(action_index).cloned() else {
            let actions = cached.commands.len();
            self.current = Some(cached);
            return Err(EnvError::InvalidActionIndex {
                index: action_index,
                actions,
            });
        };
        self.game
            .as_mut()
            .ok_or(EnvError::MissingGame)?
            .dispatch(command)?;
        self.steps = self.steps.saturating_add(1);

        let outcome = self
            .game
            .as_ref()
            .ok_or(EnvError::MissingGame)?
            .state()
            .outcome;
        let terminated = outcome.is_some();
        self.truncated = !terminated && self.max_steps > 0 && self.steps >= self.max_steps;
        let rewards = rewards(outcome);
        if !terminated && !self.truncated {
            self.refresh_decision()?;
        }
        Ok(Transition {
            rewards,
            terminated,
            truncated: self.truncated,
            outcome,
            next: self.decision().cloned(),
        })
    }

    pub fn steps(&self) -> usize {
        self.steps
    }

    pub fn pack_hash(&self) -> Result<&str, EnvError> {
        Ok(self
            .game
            .as_ref()
            .ok_or(EnvError::MissingGame)?
            .runtime()
            .pack_hash())
    }

    pub fn card_ids(&self) -> Result<Vec<String>, EnvError> {
        Ok(self
            .game
            .as_ref()
            .ok_or(EnvError::MissingGame)?
            .runtime()
            .card_ids())
    }

    /// Operator-only reproducibility output. Policies receive `Decision`, not
    /// the environment itself, so authoritative replay data stays out of their
    /// observation boundary.
    pub fn replay(&self) -> Result<Replay, EnvError> {
        Ok(self.game.as_ref().ok_or(EnvError::MissingGame)?.replay())
    }

    fn refresh_decision(&mut self) -> Result<(), EnvError> {
        let game = self.game.as_ref().ok_or(EnvError::MissingGame)?;
        if game.state().outcome.is_some() || self.truncated {
            self.current = None;
            return Ok(());
        }
        let actor = game.state().input_player();
        let view = game.state().player_view(actor);
        let legal = game.legal_action_options()?;
        let (observation, refs) = build_observation(&view)?;
        let actions = legal
            .iter()
            .enumerate()
            .map(|(index, action)| encode_action(index, action, &refs, &view))
            .collect::<Result<Vec<_>, _>>()?;
        let commands = legal.into_iter().map(|action| action.command).collect();
        let id = self.next_decision_id;
        self.next_decision_id = self.next_decision_id.wrapping_add(1).max(1);
        self.current = Some(CachedDecision {
            public: Decision {
                id,
                actor_seat: actor.0,
                observation,
                actions,
            },
            commands,
        });
        Ok(())
    }
}

fn build_game(
    runtime: LuaCardRuntime,
    config: &MatchConfig,
    seed: u64,
) -> Result<Game<LuaCardRuntime>, GameError> {
    let decks = config.decks.clone();
    if config.unrestricted {
        Game::new_unrestricted_with_hero_powers_and_classes(
            runtime,
            decks[0].clone(),
            decks[1].clone(),
            seed,
            config.hero_powers.clone(),
            config.classes.clone(),
        )
    } else {
        Game::new_with_hero_powers_and_classes(
            runtime,
            decks[0].clone(),
            decks[1].clone(),
            seed,
            config.hero_powers.clone(),
            config.classes.clone(),
        )
    }
}

fn rewards(outcome: Option<GameOutcome>) -> [f32; 2] {
    match outcome {
        Some(GameOutcome::Winner(player)) if player == PlayerId::ONE => [1.0, -1.0],
        Some(GameOutcome::Winner(_)) => [-1.0, 1.0],
        Some(GameOutcome::Draw) | None => [0.0, 0.0],
    }
}

struct RefTable {
    by_authoritative: BTreeMap<EntityId, EntityRef>,
}

fn build_observation(view: &PlayerView) -> Result<(Observation, RefTable), EnvError> {
    let self_id = view.viewer;
    let opponent_id = self_id.opponent();
    let mut entities = Vec::new();
    let mut refs = BTreeMap::new();

    let mut add =
        |id: EntityId, area: EntityArea, position: usize| -> Result<EntityRef, EnvError> {
            if let Some(existing) = refs.get(&id) {
                return Ok(*existing);
            }
            let reference = EntityRef(
                u16::try_from(entities.len()).map_err(|_| EnvError::TooManyVisibleEntities)?,
            );
            let entity = view.entity(id).ok_or(EnvError::HiddenActionEntity(id))?;
            entities.push(EntityObservation {
                entity: reference,
                card_id: entity.card_id.clone(),
                kind: entity.kind,
                owner: RelativePlayer::from_player(entity.owner, self_id),
                controller: RelativePlayer::from_player(entity.controller, self_id),
                area,
                position: u8::try_from(position).map_err(|_| EnvError::TooManyVisibleEntities)?,
                attack: entity.attack,
                max_health: entity.max_health,
                damage: entity.damage,
                armor: entity.armor,
                cost: entity.cost,
                spell_damage: entity.spell_damage,
                exhausted: entity.exhausted,
                frozen: entity.frozen,
                attacks_this_turn: entity.attacks_this_turn,
                location_cooldown: entity.location_cooldown,
                keywords: entity.keywords.clone(),
                silenced: entity.silenced,
            });
            refs.insert(id, reference);
            Ok(reference)
        };

    for player_id in [self_id, opponent_id] {
        let player = view.player(player_id);
        add(player.hero, EntityArea::Hero, 0)?;
        add(player.hero_power, EntityArea::HeroPower, 0)?;
        if let Some(weapon) = player.weapon {
            add(weapon, EntityArea::Weapon, 0)?;
        }
        for (position, entity) in player.board.iter().copied().enumerate() {
            add(entity, EntityArea::Board, position)?;
        }
        for (position, entity) in player.public_objectives.iter().copied().enumerate() {
            add(entity, EntityArea::PublicObjective, position)?;
        }
    }
    for (position, entity) in view.player(self_id).hand.iter().copied().enumerate() {
        add(entity, EntityArea::Hand, position)?;
    }
    for (position, entity) in view.player(self_id).secrets.iter().copied().enumerate() {
        add(entity, EntityArea::Secret, position)?;
    }
    drop(add);

    let histories = derive_public_histories(view);

    let player_observation = |player_id: PlayerId| -> Result<PlayerObservation, EnvError> {
        let player = view.player(player_id);
        let map = |id: EntityId| {
            refs.get(&id)
                .copied()
                .ok_or(EnvError::HiddenActionEntity(id))
        };
        Ok(PlayerObservation {
            class: player.class.clone(),
            hero: map(player.hero)?,
            hero_power: map(player.hero_power)?,
            weapon: player.weapon.map(map).transpose()?,
            board: player
                .board
                .iter()
                .copied()
                .map(map)
                .collect::<Result<_, _>>()?,
            hand: player
                .hand
                .iter()
                .copied()
                .map(map)
                .collect::<Result<_, _>>()?,
            secrets: player
                .secrets
                .iter()
                .copied()
                .filter(|entity| !player.public_objectives.contains(entity))
                .map(map)
                .collect::<Result<_, _>>()?,
            public_objectives: player
                .public_objectives
                .iter()
                .copied()
                .map(map)
                .collect::<Result<_, _>>()?,
            deck_size: player.deck_size.min(usize::from(u8::MAX)) as u8,
            hand_size: player.hand_size.min(usize::from(u8::MAX)) as u8,
            secrets_count: player.secrets_count.min(usize::from(u8::MAX)) as u8,
            mana: player.mana,
            max_mana: player.max_mana,
            temporary_mana: player.temporary_mana,
            overload_pending: player.overload_pending,
            overloaded_mana: player.overloaded_mana,
            fatigue: player.fatigue,
            hero_power_used: player.hero_power_used,
            hero_power_uses_this_turn: player.hero_power_uses_this_turn,
            cards_played_this_turn: player.cards_played_this_turn,
            history: histories[player_id.index()].clone(),
        })
    };

    let phase = if !view.mulligan_eligible.is_empty() {
        DecisionPhase::Mulligan
    } else if view.pending_input.is_some() {
        DecisionPhase::Choice
    } else {
        DecisionPhase::Main
    };
    let mulligan_eligible = view
        .mulligan_eligible
        .iter()
        .map(|entity| {
            refs.get(entity)
                .copied()
                .ok_or(EnvError::HiddenActionEntity(*entity))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let observation = Observation {
        schema_version: OBSERVATION_SCHEMA_VERSION,
        seat: self_id.0,
        turn: view.turn,
        active_player: RelativePlayer::from_player(view.active_player, self_id),
        phase,
        self_player: player_observation(self_id)?,
        opponent: player_observation(opponent_id)?,
        entities,
        mulligan_eligible,
        pending_choice: view.pending_input.as_ref().map(|input| ChoiceObservation {
            prompt: input.prompt.clone(),
            options: input.options.clone(),
        }),
    };
    Ok((
        observation,
        RefTable {
            by_authoritative: refs,
        },
    ))
}

fn derive_public_histories(view: &PlayerView) -> [PublicHistory; 2] {
    let mut histories: [PublicHistory; 2] = std::array::from_fn(|_| PublicHistory {
        cards_played: Vec::new(),
        spells_cast: Vec::new(),
        minions_played: Vec::new(),
        weapons_played: Vec::new(),
        locations_played: Vec::new(),
        discarded_cards: Vec::new(),
        minions_died: Vec::new(),
    });
    for record in view.history.iter() {
        match &record.event {
            PublicEvent::CardPlayed { player, card, .. }
            | PublicEvent::CardCountered { player, card } => {
                histories[player.index()]
                    .cards_played
                    .push(card.card_id.clone());
            }
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
    histories
}

fn encode_action(
    index: usize,
    action: &LegalAction,
    refs: &RefTable,
    view: &PlayerView,
) -> Result<ActionObservation, EnvError> {
    let map = |entity: EntityId| {
        refs.by_authoritative
            .get(&entity)
            .copied()
            .ok_or(EnvError::HiddenActionEntity(entity))
    };
    let mut encoded = ActionObservation {
        index: u32::try_from(index).map_err(|_| EnvError::TooManyVisibleEntities)?,
        kind: ActionKind::EndTurn,
        sources: Vec::new(),
        target: None,
        board_position: None,
        mana_cost: action.mana_cost,
        card_action: None,
        choice_index: None,
    };
    match &action.command {
        PlayerCommand::Mulligan { replace } => {
            encoded.kind = ActionKind::Mulligan;
            encoded.sources = replace.iter().copied().map(map).collect::<Result<_, _>>()?;
        }
        PlayerCommand::PlayCard { card, target } => {
            encoded.kind = ActionKind::PlayCard;
            encoded.sources.push(map(*card)?);
            encoded.target = target.map(map).transpose()?;
        }
        PlayerCommand::PlayCardAt {
            card,
            target,
            position,
        } => {
            encoded.kind = ActionKind::PlayCardAt;
            encoded.sources.push(map(*card)?);
            encoded.target = target.map(map).transpose()?;
            encoded.board_position =
                Some(u8::try_from(*position).map_err(|_| EnvError::TooManyVisibleEntities)?);
        }
        PlayerCommand::TradeCard { card } => {
            encoded.kind = ActionKind::TradeCard;
            encoded.sources.push(map(*card)?);
        }
        PlayerCommand::UseCardAction {
            card,
            action,
            target,
        } => {
            encoded.kind = ActionKind::UseCardAction;
            encoded.sources.push(map(*card)?);
            encoded.target = target.map(map).transpose()?;
            encoded.card_action = Some(action.clone());
        }
        PlayerCommand::Attack { attacker, defender } => {
            encoded.kind = ActionKind::Attack;
            encoded.sources.push(map(*attacker)?);
            encoded.target = Some(map(*defender)?);
        }
        PlayerCommand::UseHeroPower { target } => {
            encoded.kind = ActionKind::UseHeroPower;
            encoded
                .sources
                .push(map(view.player(view.viewer).hero_power)?);
            encoded.target = target.map(map).transpose()?;
        }
        PlayerCommand::UseLocation { location, target } => {
            encoded.kind = ActionKind::UseLocation;
            encoded.sources.push(map(*location)?);
            encoded.target = target.map(map).transpose()?;
        }
        PlayerCommand::EndTurn => encoded.kind = ActionKind::EndTurn,
        PlayerCommand::Concede => encoded.kind = ActionKind::Concede,
        PlayerCommand::Choose { index } => {
            encoded.kind = ActionKind::Choose;
            encoded.choice_index =
                Some(u16::try_from(*index).map_err(|_| EnvError::TooManyVisibleEntities)?);
        }
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn data_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
    }

    fn repeated(card: &str) -> Vec<String> {
        std::iter::repeat_n(card.to_owned(), 20).collect()
    }

    fn config(one: &str, two: &str) -> MatchConfig {
        MatchConfig {
            decks: [repeated(one), repeated(two)],
            hero_powers: default_hero_powers(),
            classes: default_classes(),
            unrestricted: true,
        }
    }

    fn mixed_config(opponent: Vec<String>) -> MatchConfig {
        MatchConfig {
            decks: [repeated("CS2_120"), opponent],
            hero_powers: default_hero_powers(),
            classes: default_classes(),
            unrestricted: true,
        }
    }

    #[test]
    fn observation_and_actions_contain_no_authoritative_entity_ids() {
        let environment =
            HearthEnv::load(data_path(), config("CS2_120", "CFM_800"), 7, 100).unwrap();
        let decision = environment.decision().unwrap();
        let json = serde_json::to_value(decision).unwrap();
        assert!(json.get("observation").is_some());
        assert!(json.pointer("/observation/entities").is_some());
        assert!(json.to_string().find("rng_seed").is_none());
        assert!(json.to_string().find("random_counter").is_none());
        assert!(json.to_string().find("command").is_none());
    }

    #[test]
    fn views_are_identical_when_only_opponent_hidden_cards_change() {
        let hidden_a = ["CFM_800", "CS2_029"]
            .into_iter()
            .cycle()
            .take(20)
            .map(str::to_owned)
            .collect();
        let hidden_b = ["CS2_029", "CFM_800"]
            .into_iter()
            .cycle()
            .take(20)
            .map(str::to_owned)
            .collect();
        let first = HearthEnv::load(data_path(), mixed_config(hidden_a), 17, 100).unwrap();
        let second = HearthEnv::load(data_path(), mixed_config(hidden_b), 17, 100).unwrap();
        assert_eq!(first.decision(), second.decision());
    }

    #[test]
    fn action_indices_are_scoped_to_one_decision() {
        let mut environment =
            HearthEnv::load(data_path(), config("CS2_120", "CS2_120"), 1, 100).unwrap();
        let first = environment.decision().unwrap().clone();
        environment.step(first.id, 0).unwrap();
        let second = environment.decision().unwrap().clone();
        assert_ne!(first.id, second.id);
        assert!(matches!(
            environment.step(first.id, 0),
            Err(EnvError::StaleDecision { .. })
        ));
        assert_eq!(environment.decision().unwrap().id, second.id);
    }

    #[test]
    fn reset_reuses_runtime_and_starts_a_fresh_episode() {
        let mut environment =
            HearthEnv::load(data_path(), config("CS2_120", "CS2_120"), 1, 100).unwrap();
        let pack_hash = environment.pack_hash().unwrap().to_owned();
        let first_id = environment.decision().unwrap().id;
        environment.step(first_id, 0).unwrap();
        let reset = environment.reset(99).unwrap().clone();
        assert_eq!(environment.steps(), 0);
        assert_eq!(environment.pack_hash().unwrap(), pack_hash);
        assert_eq!(reset.actor_seat, PlayerId::ONE.0);
        assert_eq!(reset.observation.phase, DecisionPhase::Mulligan);
    }

    #[test]
    fn concede_returns_zero_sum_terminal_rewards() {
        let mut environment =
            HearthEnv::load(data_path(), config("CS2_120", "CS2_120"), 1, 100).unwrap();
        let decision = environment.decision().unwrap().clone();
        let concede = decision
            .actions
            .iter()
            .position(|action| action.kind == ActionKind::Concede)
            .unwrap();
        let transition = environment.step(decision.id, concede).unwrap();
        assert!(transition.terminated);
        assert!(!transition.truncated);
        assert_eq!(transition.rewards, [-1.0, 1.0]);
        assert!(transition.next.is_none());
    }

    #[test]
    fn step_limit_is_adapter_truncation_not_a_game_outcome() {
        let mut environment =
            HearthEnv::load(data_path(), config("CS2_120", "CS2_120"), 1, 1).unwrap();
        let decision = environment.decision().unwrap().clone();
        let keep = decision
            .actions
            .iter()
            .position(|action| action.kind == ActionKind::Mulligan && action.sources.is_empty())
            .unwrap();
        let transition = environment.step(decision.id, keep).unwrap();
        assert!(!transition.terminated);
        assert!(transition.truncated);
        assert_eq!(transition.outcome, None);
        assert_eq!(transition.rewards, [0.0, 0.0]);
        assert!(transition.next.is_none());
    }

    #[test]
    fn legal_action_encoding_survives_long_random_walks() {
        for seed in 0_u64..4 {
            let mut environment =
                HearthEnv::load(data_path(), config("CS2_120", "CS2_120"), seed, 200).unwrap();
            for step in 0_usize..200 {
                let Some(decision) = environment.decision().cloned() else {
                    break;
                };
                assert!(!decision.actions.is_empty());
                assert!(
                    decision
                        .actions
                        .iter()
                        .enumerate()
                        .all(|(index, action)| action.index as usize == index)
                );
                let candidates = decision
                    .actions
                    .iter()
                    .enumerate()
                    .filter(|(_, action)| action.kind != ActionKind::Concede)
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>();
                let action = candidates[(seed as usize * 37 + step * 17) % candidates.len()];
                let transition = environment.step(decision.id, action).unwrap();
                if transition.terminated || transition.truncated {
                    break;
                }
            }
        }
    }

    #[test]
    fn aggregate_histories_are_derived_from_the_public_event_stream() {
        let mut environment =
            HearthEnv::load(data_path(), config("CS2_120", "CS2_120"), 3, 100).unwrap();
        for expected in [
            ActionKind::Mulligan,
            ActionKind::Mulligan,
            ActionKind::EndTurn,
            ActionKind::EndTurn,
        ] {
            let decision = environment.decision().unwrap().clone();
            let action = decision
                .actions
                .iter()
                .position(|action| {
                    action.kind == expected
                        && (expected != ActionKind::Mulligan || action.sources.is_empty())
                })
                .unwrap();
            environment.step(decision.id, action).unwrap();
        }
        let decision = environment.decision().unwrap().clone();
        let play = decision
            .actions
            .iter()
            .position(|action| matches!(action.kind, ActionKind::PlayCard | ActionKind::PlayCardAt))
            .unwrap();
        let transition = environment.step(decision.id, play).unwrap();
        let next = transition.next.unwrap();
        assert_eq!(
            next.observation.self_player.history.cards_played,
            ["CS2_120"]
        );
        assert_eq!(
            next.observation.self_player.history.minions_played,
            ["CS2_120"]
        );
        let core_view = environment
            .game
            .as_ref()
            .unwrap()
            .state()
            .player_view(PlayerId::ONE);
        assert!(core_view.history.iter().any(|record| matches!(
            &record.event,
            PublicEvent::MinionPlayed { player, minion }
                if *player == PlayerId::ONE && minion.card_id == "CS2_120"
        )));
    }
}
