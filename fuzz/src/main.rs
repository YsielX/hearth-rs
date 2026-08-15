use std::{path::PathBuf, process::ExitCode};

use hearth_core::{CardKind, DEFAULT_HERO_POWER, Game, PlayerCommand};
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

#[derive(Debug)]
struct Options {
    start_seed: u64,
    seeds: u64,
    steps: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            start_seed: 0,
            seeds: 8,
            steps: 180,
        }
    }
}

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data")
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

fn usage() -> &'static str {
    "Usage: hearth-state-fuzz [--start-seed N] [--seeds N] [--steps N]\n\
     \n\
     Options:\n\
       --start-seed N  first deterministic seed (default: 0)\n\
       --seeds N       number of games to explore (default: 8)\n\
       --steps N       maximum actions per game (default: 180)\n\
       -h, --help      show this help"
}

fn parse_value<T: std::str::FromStr>(
    args: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<T, String> {
    let value = args
        .next()
        .ok_or_else(|| format!("{option} requires a value"))?;
    value
        .parse()
        .map_err(|_| format!("invalid value for {option}: {value}"))
}

fn parse_options() -> Result<Option<Options>, String> {
    let mut options = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--start-seed" => options.start_seed = parse_value(&mut args, "--start-seed")?,
            "--seeds" => options.seeds = parse_value(&mut args, "--seeds")?,
            "--steps" => options.steps = parse_value(&mut args, "--steps")?,
            "-h" | "--help" => return Ok(None),
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }
    if options.seeds == 0 {
        return Err("--seeds must be greater than zero".to_owned());
    }
    if options.steps == 0 {
        return Err("--steps must be greater than zero".to_owned());
    }
    options
        .start_seed
        .checked_add(options.seeds - 1)
        .ok_or_else(|| "seed range exceeds u64::MAX".to_owned())?;
    Ok(Some(options))
}

fn fuzz_seed(seed: u64, steps: usize) -> Result<(), String> {
    let runtime = LuaCardRuntime::load_dir(data_path())
        .map_err(|error| format!("seed {seed}: failed to load Lua data: {error}"))?;
    let mut random = seed ^ 0xa076_1d64_78bd_642f;
    let classes = [
        CLASSES[next_random(&mut random) as usize % CLASSES.len()].to_owned(),
        CLASSES[next_random(&mut random) as usize % CLASSES.len()].to_owned(),
    ];
    let pool_one = class_pool(&runtime, &classes[0]);
    let pool_two = class_pool(&runtime, &classes[1]);
    if pool_one.is_empty() || pool_two.is_empty() {
        return Err(format!(
            "seed {seed}: an eligible card pool is empty for classes {classes:?}"
        ));
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
    .map_err(|error| format!("seed {seed}: game construction failed: {error}"))?;

    for step in 0..steps {
        if game.state().outcome.is_some() {
            break;
        }
        let mut actions = game.legal_actions().map_err(|error| {
            format!("seed {seed}, step {step}: legal action enumeration failed: {error}")
        })?;
        if actions.is_empty() {
            return Err(format!("seed {seed}, step {step}: no legal actions"));
        }
        if actions.len() > 1 {
            actions.retain(|action| !matches!(action, PlayerCommand::Concede));
        }
        let command = actions[next_random(&mut random) as usize % actions.len()].clone();
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
            format!(
                "seed {seed}, step {step}: legal command {command:?} from {source:?} failed: {error}"
            )
        })?;
        game.state()
            .validate()
            .map_err(|error| format!("seed {seed}, step {step}: invalid state: {error}"))?;
    }

    let replayed = Game::from_replay(
        LuaCardRuntime::load_dir(data_path())
            .map_err(|error| format!("seed {seed}: failed to reload Lua data: {error}"))?,
        &game.replay(),
    )
    .map_err(|error| format!("seed {seed}: replay failed: {error}"))?;
    if replayed.state() != game.state() {
        return Err(format!("seed {seed}: replay produced a different state"));
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let Some(options) = parse_options()? else {
        println!("{}", usage());
        return Ok(());
    };
    println!(
        "state-machine fuzz: start_seed={}, seeds={}, max_steps={}",
        options.start_seed, options.seeds, options.steps
    );
    for offset in 0..options.seeds {
        fuzz_seed(options.start_seed + offset, options.steps)?;
    }
    println!(
        "state-machine fuzz passed: {} deterministic seeds",
        options.seeds
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}\n\n{}", usage());
            ExitCode::FAILURE
        }
    }
}
