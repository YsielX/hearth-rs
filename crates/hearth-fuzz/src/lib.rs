use std::{error::Error, fmt, path::Path};

use hearth_core::{
    CardKind, DEFAULT_HERO_POWER, Game, LegalAction, PlayerCommand, PlayerController, PlayerView,
};
use hearth_script::LuaCardRuntime;

const CLASSES: [&str; 11] = [
    "death_knight",
    "demon_hunter",
    "druid",
    "hunter",
    "mage",
    "paladin",
    "priest",
    "rogue",
    "shaman",
    "warlock",
    "warrior",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FuzzOptions {
    pub start_seed: u64,
    pub seeds: u64,
    pub steps: usize,
}

impl Default for FuzzOptions {
    fn default() -> Self {
        Self {
            start_seed: 0,
            seeds: 8,
            steps: 180,
        }
    }
}

impl FuzzOptions {
    pub fn validate(&self) -> Result<(), FuzzError> {
        if self.seeds == 0 {
            return Err(FuzzError::new("--seeds must be greater than zero"));
        }
        if self.steps == 0 {
            return Err(FuzzError::new("--steps must be greater than zero"));
        }
        self.start_seed
            .checked_add(self.seeds - 1)
            .ok_or_else(|| FuzzError::new("seed range exceeds u64::MAX"))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FuzzError {
    message: String,
}

impl FuzzError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for FuzzError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for FuzzError {}

#[derive(Clone, Debug)]
pub struct FuzzController {
    random: u64,
}

impl FuzzController {
    pub fn new(seed: u64) -> Self {
        Self {
            random: seed ^ 0xa076_1d64_78bd_642f,
        }
    }
}

impl PlayerController for FuzzController {
    fn choose_action(
        &mut self,
        view: &PlayerView,
        legal_actions: &[LegalAction],
    ) -> Result<PlayerCommand, String> {
        if view.viewer != view.input_player {
            return Err(format!(
                "{} cannot choose an action for {}",
                view.viewer, view.input_player
            ));
        }
        let mut candidates = legal_actions
            .iter()
            .filter(|action| !matches!(action.command, PlayerCommand::Concede))
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            candidates.extend(legal_actions);
        }
        if candidates.is_empty() {
            return Err("no legal actions are available".to_owned());
        }
        let index = next_random(&mut self.random) as usize % candidates.len();
        Ok(candidates[index].command.clone())
    }
}

fn next_random(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

fn catalog_deck(pool: &[String], state: &mut u64) -> Vec<String> {
    (0..30)
        .map(|_| pool[next_random(state) as usize % pool.len()].clone())
        .collect()
}

fn class_pool(runtime: &LuaCardRuntime, class: &str) -> Vec<String> {
    let mut pool = runtime
        .definitions()
        .filter(|card| {
            let class_eligible = if card.classes.is_empty() {
                card.class == "neutral" || card.class == class
            } else {
                card.classes.iter().any(|candidate| candidate == class)
            };
            card.collectible
                && class_eligible
                && matches!(
                    card.kind,
                    CardKind::Hero
                        | CardKind::Minion
                        | CardKind::Spell
                        | CardKind::Weapon
                        | CardKind::Location
                )
        })
        .map(|card| card.id.clone())
        .collect::<Vec<_>>();
    pool.sort();
    pool
}

fn fuzz_seed(data_path: &Path, seed: u64, steps: usize) -> Result<(), FuzzError> {
    let runtime = LuaCardRuntime::load_dir(data_path).map_err(|error| {
        FuzzError::new(format!("seed {seed}: failed to load Lua data: {error}"))
    })?;
    let mut random = seed ^ 0xa076_1d64_78bd_642f;
    let classes = [
        CLASSES[next_random(&mut random) as usize % CLASSES.len()].to_owned(),
        CLASSES[next_random(&mut random) as usize % CLASSES.len()].to_owned(),
    ];
    let pool_one = class_pool(&runtime, &classes[0]);
    let pool_two = class_pool(&runtime, &classes[1]);
    if pool_one.is_empty() || pool_two.is_empty() {
        return Err(FuzzError::new(format!(
            "seed {seed}: an eligible card pool is empty for classes {classes:?}"
        )));
    }
    let one = catalog_deck(&pool_one, &mut random);
    let two = catalog_deck(&pool_two, &mut random);
    let mut game = Game::new_with_hero_powers_and_classes(
        runtime,
        one,
        two,
        seed,
        [DEFAULT_HERO_POWER.to_owned(), DEFAULT_HERO_POWER.to_owned()],
        classes,
    )
    .map_err(|error| FuzzError::new(format!("seed {seed}: game construction failed: {error}")))?;
    let controller_seed_one = next_random(&mut random);
    let controller_seed_two = next_random(&mut random);
    let mut controllers = [
        FuzzController::new(controller_seed_one),
        FuzzController::new(controller_seed_two),
    ];

    for step in 0..steps {
        if game.state().outcome.is_some() {
            break;
        }
        let actions = game.legal_action_options().map_err(|error| {
            FuzzError::new(format!(
                "seed {seed}, step {step}: legal action enumeration failed: {error}"
            ))
        })?;
        if actions.is_empty() {
            return Err(FuzzError::new(format!(
                "seed {seed}, step {step}: no legal actions"
            )));
        }
        let player = game.state().input_player();
        let view = game.state().player_view(player);
        let command = controllers[player.index()]
            .choose_action(&view, &actions)
            .map_err(|error| {
                FuzzError::new(format!(
                    "seed {seed}, step {step}: Fuzzer controller failed: {error}"
                ))
            })?;
        let source = match &command {
            PlayerCommand::PlayCard { card, .. }
            | PlayerCommand::PlayCardAt { card, .. }
            | PlayerCommand::TradeCard { card }
            | PlayerCommand::UseCardAction { card, .. } => game
                .state()
                .entity(*card)
                .map(|entity| entity.card_id.clone()),
            PlayerCommand::Attack { attacker, defender } => Some(format!(
                "{} {:?} -> {} {:?}; defender board {:?}",
                game.state().entities[attacker].card_id,
                game.state().entities[attacker].keywords,
                game.state().entities[defender].card_id,
                game.state().entities[defender].keywords,
                game.state()
                    .player(game.state().entities[defender].controller)
                    .board
                    .iter()
                    .map(|entity| {
                        let entity = &game.state().entities[entity];
                        (entity.id, entity.card_id.clone(), entity.keywords.clone())
                    })
                    .collect::<Vec<_>>()
            )),
            _ => None,
        };
        game.dispatch(command.clone()).map_err(|error| {
            FuzzError::new(format!(
                "seed {seed}, step {step}: legal command {command:?} from {source:?} failed: {error}"
            ))
        })?;
        game.state().validate().map_err(|error| {
            FuzzError::new(format!("seed {seed}, step {step}: invalid state: {error}"))
        })?;
    }

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path).map_err(|error| {
            FuzzError::new(format!("seed {seed}: failed to reload Lua data: {error}"))
        })?,
        &game.replay(),
    )
    .map_err(|error| FuzzError::new(format!("seed {seed}: replay failed: {error}")))?;
    if replayed.state() != game.state() {
        return Err(FuzzError::new(format!(
            "seed {seed}: replay produced a different state"
        )));
    }
    Ok(())
}

pub fn run_campaign(data_path: impl AsRef<Path>, options: &FuzzOptions) -> Result<(), FuzzError> {
    options.validate()?;
    for offset in 0..options.seeds {
        fuzz_seed(
            data_path.as_ref(),
            options.start_seed + offset,
            options.steps,
        )?;
    }
    Ok(())
}
