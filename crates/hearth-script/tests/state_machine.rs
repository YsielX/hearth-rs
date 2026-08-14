use std::path::PathBuf;

use hearth_core::{CardKind, Game, PlayerCommand};
use hearth_script::LuaCardRuntime;

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
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

#[test]
fn randomized_full_catalog_legal_action_walks_are_executable_and_replayable() {
    let seed_count = std::env::var("HEARTH_STATE_MACHINE_SEEDS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8_u64);
    let start_seed = std::env::var("HEARTH_STATE_MACHINE_START_SEED")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0_u64);
    for seed in start_seed..start_seed.saturating_add(seed_count) {
        let runtime = LuaCardRuntime::load_dir(data_path()).unwrap();
        let mut pool = runtime
            .definitions()
            .filter(|card| {
                card.collectible
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
        assert!(!pool.is_empty());

        let mut random = seed ^ 0xa076_1d64_78bd_642f;
        let one = catalog_deck(&pool, &mut random);
        let two = catalog_deck(&pool, &mut random);
        let mut game = Game::new(runtime, one, two, seed).unwrap();

        for step in 0..180 {
            if game.state().outcome.is_some() {
                break;
            }
            let mut actions = game.legal_actions().unwrap_or_else(|error| {
                panic!("seed {seed}, step {step}: legal action enumeration failed: {error}")
            });
            assert!(!actions.is_empty(), "seed {seed}, step {step}");
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
            game.dispatch(command.clone()).unwrap_or_else(|error| {
                panic!(
                    "seed {seed}, step {step}: legal command {command:?} from {source:?} failed: {error}"
                )
            });
            game.state()
                .validate()
                .unwrap_or_else(|error| panic!("seed {seed}, step {step}: {error}"));
        }

        let replayed = Game::from_replay(
            LuaCardRuntime::load_dir(data_path()).unwrap(),
            &game.replay(),
        )
        .unwrap_or_else(|error| panic!("seed {seed}: replay failed: {error}"));
        assert_eq!(replayed.state(), game.state(), "seed {seed}");
    }
}
