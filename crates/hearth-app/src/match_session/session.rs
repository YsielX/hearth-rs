use std::collections::BTreeMap;
use std::path::Path;

use hearth_core::{
    CardRuntime, DEFAULT_HERO_POWER, EntityId, Game, GameSnapshot, GameState, LegalAction, Locale,
    PlayerCommand, PlayerId, PlayerView, Replay,
};
use hearth_script::LuaCardRuntime;

use crate::AppError;
use crate::deck::{DeckList, load_deck, validate_deck};

use super::config::{MatchSetup, starting_player_for_seed};

/// Authoritative, controller-neutral match state shared by every local
/// frontend. It owns deck loading, runtime construction, sideboards, replay,
/// snapshots, and viewer-safe projections, but does not decide who controls a
/// seat or when an automated controller should act.
pub struct MatchSession {
    pub(crate) game: Game<LuaCardRuntime>,
    pub(crate) deck_names: [String; 2],
    pub(crate) locale: Locale,
}

impl MatchSession {
    pub fn load(setup: &MatchSetup) -> Result<Self, AppError> {
        let runtime = LuaCardRuntime::load_dir_with_locale(&setup.data_dir, setup.locale)?;
        let deck_one = load_deck(&setup.deck_one)?;
        let deck_two = load_deck(&setup.deck_two)?;
        validate_deck(&runtime, &deck_one)?;
        validate_deck(&runtime, &deck_two)?;

        let hero_powers = [
            hero_power_for_deck(&runtime, &deck_one)?,
            hero_power_for_deck(&runtime, &deck_two)?,
        ];
        let classes = [deck_one.class.clone(), deck_two.class.clone()];
        let unrestricted = deck_one.unrestricted || deck_two.unrestricted;
        let starting_player = starting_player_for_seed(setup.seed);
        let sideboards = [deck_sideboards(&deck_one), deck_sideboards(&deck_two)];
        let game = Game::new_with_sideboards_hero_powers_classes_and_starting_player(
            runtime,
            deck_one.cards,
            deck_two.cards,
            sideboards,
            setup.seed,
            hero_powers,
            classes,
            starting_player,
            unrestricted,
        )?;

        Ok(Self {
            game,
            deck_names: [deck_one.name, deck_two.name],
            locale: setup.locale,
        })
    }

    pub fn from_snapshot(
        data_dir: impl AsRef<Path>,
        locale: Locale,
        snapshot: &GameSnapshot,
    ) -> Result<Self, AppError> {
        Self::from_snapshot_with_deck_names(
            data_dir,
            locale,
            snapshot,
            ["Player 1".to_owned(), "Player 2".to_owned()],
        )
    }

    pub(super) fn from_snapshot_with_deck_names(
        data_dir: impl AsRef<Path>,
        locale: Locale,
        snapshot: &GameSnapshot,
        deck_names: [String; 2],
    ) -> Result<Self, AppError> {
        let runtime = LuaCardRuntime::load_dir_with_locale(data_dir, locale)?;
        let game = Game::from_snapshot(runtime, snapshot)?;
        Ok(Self {
            game,
            deck_names,
            locale,
        })
    }

    pub fn from_replay(
        data_dir: impl AsRef<Path>,
        locale: Locale,
        replay: &Replay,
    ) -> Result<Self, AppError> {
        let runtime = LuaCardRuntime::load_dir_with_locale(data_dir, locale)?;
        let game = Game::from_replay(runtime, replay)?;
        Ok(Self {
            game,
            deck_names: ["Player 1".to_owned(), "Player 2".to_owned()],
            locale,
        })
    }

    pub fn state(&self) -> &GameState {
        self.game.state()
    }

    pub fn runtime(&self) -> &LuaCardRuntime {
        self.game.runtime()
    }

    pub fn locale(&self) -> Locale {
        self.locale
    }

    pub fn deck_name(&self, player: PlayerId) -> &str {
        &self.deck_names[player.index()]
    }

    pub fn view_for(&self, viewer: PlayerId) -> PlayerView {
        self.game.state().player_view(viewer)
    }

    pub fn legal_actions(&self) -> Result<Vec<PlayerCommand>, AppError> {
        self.game.legal_actions().map_err(AppError::from)
    }

    pub fn legal_action_options(&self) -> Result<Vec<LegalAction>, AppError> {
        self.game.legal_action_options().map_err(AppError::from)
    }

    pub fn valid_targets(&self, source: EntityId) -> Result<Vec<EntityId>, AppError> {
        self.game.valid_targets(source).map_err(AppError::from)
    }

    pub fn dispatch(&mut self, command: PlayerCommand) -> Result<(), AppError> {
        self.game.dispatch(command).map_err(AppError::from)
    }

    pub fn replay(&self) -> Replay {
        self.game.replay()
    }

    pub fn snapshot(&self) -> GameSnapshot {
        self.game.snapshot()
    }

    pub fn card_name(&self, card_id: &str) -> String {
        self.game
            .runtime()
            .definition(card_id)
            .map(|definition| definition.name.clone())
            .unwrap_or_else(|| card_id.to_owned())
    }

    pub fn card_text(&self, card_id: &str) -> String {
        self.game
            .runtime()
            .definition(card_id)
            .map(|definition| definition.text.clone())
            .unwrap_or_default()
    }

    pub fn turn_time_limit_seconds(&self) -> Result<Option<u64>, AppError> {
        let mut limit = 0;
        for player in [PlayerId::ONE, PlayerId::TWO] {
            for entity in &self.game.state().player(player).board {
                limit = self
                    .game
                    .runtime()
                    .keyword_i32_rule(
                        self.game.state(),
                        *entity,
                        "turn_time_limit_seconds",
                        limit,
                        None,
                    )
                    .map_err(AppError::RuntimeRule)?;
            }
        }
        Ok((limit > 0).then_some(limit as u64))
    }
}

pub(crate) fn hero_power_for_deck(
    runtime: &LuaCardRuntime,
    deck: &DeckList,
) -> Result<String, AppError> {
    if let Some(hero_power) = &deck.hero_power {
        return Ok(hero_power.clone());
    }
    let heroes = runtime
        .definitions()
        .filter(|definition| {
            definition.starting_hero && definition.class.eq_ignore_ascii_case(&deck.class)
        })
        .collect::<Vec<_>>();
    if heroes.len() > 1 {
        return Err(hearth_core::GameError::AmbiguousStartingHero(deck.class.clone()).into());
    }
    Ok(heroes
        .into_iter()
        .next()
        .and_then(|hero| hero.hero_power.clone())
        .unwrap_or_else(|| DEFAULT_HERO_POWER.to_owned()))
}

fn deck_sideboards(deck: &DeckList) -> BTreeMap<String, Vec<String>> {
    deck.sideboards
        .iter()
        .map(|sideboard| (sideboard.owner.clone(), sideboard.cards.clone()))
        .collect()
}
