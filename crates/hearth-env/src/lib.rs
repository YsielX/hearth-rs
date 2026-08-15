//! A learning-framework-neutral environment adapter for `hearth-rs`.
//!
//! The rules engine remains authoritative and knows nothing about tensor
//! layouts, rewards, episode truncation, or action indices. This crate owns
//! those concerns and never exposes authoritative entity IDs to a policy.

use std::path::Path;

use hearth_core::{
    CardDefinition, CardRuntime, EntityId, Game, GameError, GameOutcome, PlayerCommand, PlayerId,
    Replay,
};
use hearth_script::{LuaCardRuntime, ScriptLoadError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod action;
mod config;
mod entity_refs;
mod history;
mod observation;

pub use action::{ActionKind, ActionObservation};
pub use config::{EnvConfig, MatchConfig};
pub use history::{
    EventEntityObservation, EventEntityRole, EventKind, EventObservation, EventRecordObservation,
    EventWindow, OutcomeObservation, PublicHistory,
};
pub use observation::{
    ChoiceObservation, ChoiceOptionObservation, ChoiceOptionValueObservation, DecisionPhase,
    EntityArea, EntityObservation, EntityRef, Observation, PlayerObservation, RelativePlayer,
};

use action::encode_action;
use history::ViewerMemory;
use observation::build_observation;

pub const OBSERVATION_SCHEMA_VERSION: u32 = 3;

/// Operator/training metadata. This is deliberately not part of `Decision`,
/// so a policy cannot mistake implementation source for in-game information.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardCatalogEntry {
    pub definition: CardDefinition,
    pub lua_path: String,
    pub lua_source: String,
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
    #[error("an episode contains too many observed entities")]
    TooManyObservedEntities,
    #[error("a public event value is too large for the observation schema")]
    PublicEventValueTooLarge,
    #[error("public history rewound from {processed} processed events to {available}")]
    PublicHistoryRewound { processed: usize, available: usize },
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
    env_config: EnvConfig,
    viewers: [ViewerMemory; 2],
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
        Self::load_configured(
            data_path,
            match_config,
            seed,
            EnvConfig::with_max_steps(max_steps),
        )
    }

    pub fn load_configured(
        data_path: impl AsRef<Path>,
        match_config: MatchConfig,
        seed: u64,
        env_config: EnvConfig,
    ) -> Result<Self, EnvError> {
        let runtime = LuaCardRuntime::load_dir(data_path)?;
        Self::with_runtime_configured(runtime, match_config, seed, env_config)
    }

    pub fn with_runtime(
        runtime: LuaCardRuntime,
        match_config: MatchConfig,
        seed: u64,
        max_steps: usize,
    ) -> Result<Self, EnvError> {
        Self::with_runtime_configured(
            runtime,
            match_config,
            seed,
            EnvConfig::with_max_steps(max_steps),
        )
    }

    pub fn with_runtime_configured(
        runtime: LuaCardRuntime,
        match_config: MatchConfig,
        seed: u64,
        env_config: EnvConfig,
    ) -> Result<Self, EnvError> {
        let game = build_game(runtime, &match_config, seed)?;
        let mut environment = Self {
            game: Some(game),
            match_config,
            env_config,
            viewers: std::array::from_fn(|_| ViewerMemory::default()),
            steps: 0,
            next_decision_id: 1,
            current: None,
            truncated: false,
        };
        environment.refresh_decision()?;
        Ok(environment)
    }

    pub fn reset(&mut self, seed: u64) -> Result<&Decision, EnvError> {
        self.reset_match(self.match_config.clone(), seed)
    }

    /// Start a fresh match with different decks while retaining the loaded Lua
    /// runtime. This is an environment operation, not a game-rule feature.
    pub fn reset_match(
        &mut self,
        match_config: MatchConfig,
        seed: u64,
    ) -> Result<&Decision, EnvError> {
        let old_game = self.game.take().ok_or(EnvError::MissingGame)?;
        let runtime = old_game.into_runtime();
        self.game = Some(build_game(runtime, &match_config, seed)?);
        self.match_config = match_config;
        self.steps = 0;
        self.truncated = false;
        self.current = None;
        self.viewers = std::array::from_fn(|_| ViewerMemory::default());
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
        self.truncated =
            !terminated && self.env_config.max_steps > 0 && self.steps >= self.env_config.max_steps;
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

    pub fn card_catalog(&self) -> Result<Vec<CardCatalogEntry>, EnvError> {
        Ok(self
            .game
            .as_ref()
            .ok_or(EnvError::MissingGame)?
            .runtime()
            .scripted_definitions()
            .map(|(definition, lua_path, lua_source)| CardCatalogEntry {
                definition: definition.clone(),
                lua_path: lua_path.to_owned(),
                lua_source: lua_source.to_owned(),
            })
            .collect())
    }

    /// Operator-only reproducibility output. Policies receive `Decision`, not
    /// the environment itself, so authoritative replay data stays out of their
    /// observation boundary.
    pub fn replay(&self) -> Result<Replay, EnvError> {
        Ok(self.game.as_ref().ok_or(EnvError::MissingGame)?.replay())
    }

    fn refresh_decision(&mut self) -> Result<(), EnvError> {
        let (actor, view, legal) = {
            let game = self.game.as_ref().ok_or(EnvError::MissingGame)?;
            if game.state().outcome.is_some() || self.truncated {
                self.current = None;
                return Ok(());
            }
            let actor = game.state().input_player();
            (
                actor,
                game.state().player_view(actor),
                game.legal_action_options()?,
            )
        };
        let memory = &mut self.viewers[actor.index()];
        memory.sync(&view)?;
        let observation = build_observation(&view, memory, self.env_config.history_limit)?;
        let actions = legal
            .iter()
            .enumerate()
            .map(|(index, action)| encode_action(index, action, &memory.refs, &view))
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use hearth_core::PublicEvent;

    use super::*;
    use crate::config::{default_classes, default_hero_powers};

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

    fn action_index(decision: &Decision, kind: ActionKind) -> usize {
        decision
            .actions
            .iter()
            .position(|action| {
                action.kind == kind && (kind != ActionKind::Mulligan || action.sources.is_empty())
            })
            .unwrap()
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
        assert!(json.to_string().find("sequence").is_none());
        assert_eq!(decision.observation.schema_version, 3);
    }

    #[test]
    fn entity_references_are_stable_for_the_whole_episode() {
        let mut environment =
            HearthEnv::load(data_path(), config("CS2_120", "CS2_120"), 11, 100).unwrap();
        let first = environment.decision().unwrap().clone();
        let hero = first.observation.self_player.hero;
        let opening_hand = first.observation.self_player.hand.clone();
        assert!(!opening_hand.is_empty());
        assert!(first.observation.history.events.iter().any(|record| {
            record.event.kind == EventKind::CardDrawn
                && record.event.player == Some(RelativePlayer::SelfPlayer)
                && record.event.entities.iter().any(|entity| {
                    entity.role == EventEntityRole::Card && opening_hand.contains(&entity.entity)
                })
        }));

        let keep = action_index(&first, ActionKind::Mulligan);
        let second = environment.step(first.id, keep).unwrap().next.unwrap();
        let keep = action_index(&second, ActionKind::Mulligan);
        let third = environment.step(second.id, keep).unwrap().next.unwrap();

        assert_eq!(third.observation.self_player.hero, hero);
        assert!(
            opening_hand
                .iter()
                .all(|entity| third.observation.self_player.hand.contains(entity))
        );
        let cursors = third
            .observation
            .history
            .events
            .iter()
            .map(|record| record.cursor)
            .collect::<Vec<_>>();
        assert!(cursors.windows(2).all(|pair| pair[0] + 1 == pair[1]));
    }

    #[test]
    fn history_window_has_explicit_cursors() {
        let environment = HearthEnv::load_configured(
            data_path(),
            config("CS2_120", "CS2_120"),
            5,
            EnvConfig {
                max_steps: 100,
                history_limit: Some(2),
            },
        )
        .unwrap();
        let history = &environment.decision().unwrap().observation.history;
        assert_eq!(history.events.len(), 2);
        assert!(history.has_earlier_events);
        assert_eq!(
            history.start_cursor + history.events.len() as u64,
            history.next_cursor
        );
        assert_eq!(history.events.first().unwrap().cursor, history.start_cursor);
        assert!(history.next_cursor > 2);
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
        let match_config = config("CS2_120", "CS2_120");
        let mut environment = HearthEnv::load(data_path(), match_config.clone(), 1, 100).unwrap();
        let pack_hash = environment.pack_hash().unwrap().to_owned();
        let first_id = environment.decision().unwrap().id;
        environment.step(first_id, 0).unwrap();
        let reset = environment.reset(99).unwrap().clone();
        assert_eq!(environment.steps(), 0);
        assert_eq!(environment.pack_hash().unwrap(), pack_hash);
        assert_eq!(reset.actor_seat, PlayerId::ONE.0);
        assert_eq!(reset.observation.phase, DecisionPhase::Mulligan);
        let fresh = HearthEnv::load(data_path(), match_config, 99, 100).unwrap();
        assert_eq!(reset.observation, fresh.decision().unwrap().observation);
        assert_eq!(reset.actions, fresh.decision().unwrap().actions);
    }

    #[test]
    fn reset_match_reuses_runtime_with_different_decks() {
        let mut environment =
            HearthEnv::load(data_path(), config("CS2_120", "CS2_120"), 1, 100).unwrap();
        let pack_hash = environment.pack_hash().unwrap().to_owned();
        let replacement = config("CFM_800", "CFM_800");
        let reset = environment
            .reset_match(replacement.clone(), 2)
            .unwrap()
            .clone();
        assert_eq!(environment.pack_hash().unwrap(), pack_hash);
        let fresh = HearthEnv::load(data_path(), replacement, 2, 100).unwrap();
        assert_eq!(reset.observation, fresh.decision().unwrap().observation);
        assert_eq!(reset.actions, fresh.decision().unwrap().actions);
    }

    #[test]
    fn training_catalog_keeps_definition_and_portable_lua_source_together() {
        let environment =
            HearthEnv::load(data_path(), config("CS2_120", "CS2_120"), 1, 100).unwrap();
        let catalog = environment.card_catalog().unwrap();
        let river_crocolisk = catalog
            .iter()
            .find(|entry| entry.definition.id == "CS2_120")
            .unwrap();
        assert_eq!(river_crocolisk.definition.name, "River Crocolisk");
        assert!(river_crocolisk.lua_path.ends_with("river_crocolisk.lua"));
        assert!(river_crocolisk.lua_source.contains("CS2_120"));
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
        let mut environment = HearthEnv::load_configured(
            data_path(),
            config("CS2_120", "CS2_120"),
            3,
            EnvConfig {
                max_steps: 100,
                history_limit: Some(1),
            },
        )
        .unwrap();
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
        assert_eq!(next.observation.history.events.len(), 1);
        assert!(next.observation.history.has_earlier_events);
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
