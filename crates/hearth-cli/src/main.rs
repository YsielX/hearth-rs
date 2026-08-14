use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::path::PathBuf;

use hearth_core::{
    CardKind, CardRuntime, DEFAULT_HERO_POWER, EntityId, Game, GameEvent, GameOutcome,
    GameSnapshot, Locale, PlayerCommand, PlayerId, Replay,
};
use hearth_script::LuaCardRuntime;
use serde::Deserialize;

macro_rules! lt {
    ($locale:expr, $en:literal, $zh_cn:literal, $zh_tw:literal) => {
        match $locale {
            Locale::EnUs => $en,
            Locale::ZhCn => $zh_cn,
            Locale::ZhTw => $zh_tw,
        }
    };
}

macro_rules! lf {
    ($locale:expr, $en:literal, $zh_cn:literal, $zh_tw:literal $(, $arg:expr)* $(,)?) => {
        match $locale {
            Locale::EnUs => format!($en $(, $arg)*),
            Locale::ZhCn => format!($zh_cn $(, $arg)*),
            Locale::ZhTw => format!($zh_tw $(, $arg)*),
        }
    };
}

#[derive(Debug)]
struct CliOptions {
    data: PathBuf,
    deck_one: PathBuf,
    deck_two: PathBuf,
    seed: u64,
    replay: Option<PathBuf>,
    snapshot: Option<PathBuf>,
    locale: Locale,
}

#[derive(Debug, Deserialize)]
struct DeckFile {
    name: String,
    #[serde(default = "default_deck_class")]
    class: String,
    cards: Vec<String>,
    #[serde(default)]
    hero_power: Option<String>,
    /// Explicit escape hatch for showcase/sandbox decks that intentionally mix classes.
    #[serde(default)]
    unrestricted: bool,
}

fn default_deck_class() -> String {
    "mage".to_owned()
}

fn main() -> Result<(), Box<dyn Error>> {
    let Some(options) = parse_options()? else {
        return Ok(());
    };
    let locale = options.locale;
    let runtime = LuaCardRuntime::load_dir_with_locale(&options.data, locale)?;
    println!(
        "{}",
        lf!(
            locale,
            "Loaded {0} Lua card and Hero Power definitions from {1}.",
            "已从 {1} 加载 {0} 个 Lua 卡牌与英雄技能定义。",
            "已從 {1} 載入 {0} 個 Lua 卡牌與英雄能力定義。",
            runtime.card_ids().len(),
            options.data.display()
        )
    );

    let mut game = if let Some(path) = &options.snapshot {
        let snapshot: GameSnapshot = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        println!(
            "{}",
            lf!(
                locale,
                "Restored snapshot {} at turn {} after {} commands.",
                "恢复快照 {}：回合 {}，已执行 {} 条命令。",
                "恢復快照 {}：回合 {}，已執行 {} 條命令。",
                path.display(),
                snapshot.state.turn,
                snapshot.replay.commands.len()
            )
        );
        Game::from_snapshot(runtime, &snapshot)?
    } else if let Some(path) = &options.replay {
        let replay: Replay = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        println!(
            "{}",
            lf!(
                locale,
                "Replaying {}: {} commands, card pack {}.",
                "重放 {}：{} 条命令，卡牌包 {}。",
                "重播 {}：{} 條命令，卡牌包 {}。",
                path.display(),
                replay.commands.len(),
                replay.card_pack_hash
            )
        );
        Game::from_replay(runtime, &replay)?
    } else {
        let deck_one = load_deck(&options.deck_one)?;
        let deck_two = load_deck(&options.deck_two)?;
        validate_deck(&runtime, &deck_one, locale)?;
        validate_deck(&runtime, &deck_two, locale)?;
        println!(
            "{}",
            lf!(
                locale,
                "P1 deck: {} [{}] ({} cards)",
                "P1 牌组：{} [{}]（{}张）",
                "P1 牌組：{} [{}]（{}張）",
                deck_one.name,
                deck_one.class,
                deck_one.cards.len()
            )
        );
        println!(
            "{}",
            lf!(
                locale,
                "P2 deck: {} [{}] ({} cards)",
                "P2 牌组：{} [{}]（{}张）",
                "P2 牌組：{} [{}]（{}張）",
                deck_two.name,
                deck_two.class,
                deck_two.cards.len()
            )
        );
        let hero_powers = [
            deck_one
                .hero_power
                .unwrap_or_else(|| DEFAULT_HERO_POWER.to_owned()),
            deck_two
                .hero_power
                .unwrap_or_else(|| DEFAULT_HERO_POWER.to_owned()),
        ];
        let classes = [deck_one.class, deck_two.class];
        Game::new_with_hero_powers_and_classes(
            runtime,
            deck_one.cards,
            deck_two.cards,
            options.seed,
            hero_powers,
            classes,
        )?
    };
    print_help(locale);
    print_state(&game, locale);

    let stdin = io::stdin();
    loop {
        print!("{}> ", game.state().active_player);
        io::stdout().flush()?;
        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }
        let words: Vec<_> = line.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }
        match words[0] {
            "help" | "?" => print_help(locale),
            "state" | "s" => print_state(&game, locale),
            "hand" | "h" => print_hand(&game, locale),
            "cards" => {
                for id in game.runtime().card_ids() {
                    let card = game.runtime().definition(&id).unwrap();
                    let localized = card.localized(locale);
                    println!(
                        "{}",
                        lf!(
                            locale,
                            "{}: {} [{} | {} mana {:?} {}/{}] {}",
                            "{}: {} [{} | {}费 {:?} {}/{}] {}",
                            "{}: {} [{} | {}費 {:?} {}/{}] {}",
                            card.id,
                            localized.name,
                            card.set,
                            card.cost,
                            card.kind,
                            card.attack,
                            card.health,
                            localized.text
                        )
                    );
                }
            }
            "legal" | "l" => match game.legal_actions() {
                Ok(actions) => {
                    for action in actions {
                        println!("  {}", display_command(&action));
                    }
                }
                Err(error) => eprintln!(
                    "{}",
                    lf!(locale, "Error: {error}", "错误：{error}", "錯誤：{error}")
                ),
            },
            "save" if words.len() == 2 => {
                let replay = serde_json::to_string_pretty(&game.replay())?;
                std::fs::write(words[1], replay)?;
                println!(
                    "{}",
                    lf!(
                        locale,
                        "Saved replay: {}",
                        "已保存 replay：{}",
                        "已儲存 replay：{}",
                        words[1]
                    )
                );
            }
            "snapshot" if words.len() == 2 => {
                let snapshot = serde_json::to_string_pretty(&game.snapshot())?;
                std::fs::write(words[1], snapshot)?;
                println!(
                    "{}",
                    lf!(
                        locale,
                        "Saved snapshot: {}",
                        "已保存状态快照：{}",
                        "已儲存狀態快照：{}",
                        words[1]
                    )
                );
            }
            "targets" if words.len() == 2 => match parse_entity(words[1], locale) {
                Ok(card) => match game.valid_targets(card) {
                    Ok(targets) => println!(
                        "{}",
                        lf!(
                            locale,
                            "Valid targets: {}",
                            "合法目标：{}",
                            "合法目標：{}",
                            display_ids(&targets)
                        )
                    ),
                    Err(error) => eprintln!(
                        "{}",
                        lf!(locale, "Error: {error}", "错误：{error}", "錯誤：{error}")
                    ),
                },
                Err(error) => eprintln!(
                    "{}",
                    lf!(locale, "Error: {error}", "错误：{error}", "錯誤：{error}")
                ),
            },
            "play" | "p" if words.len() == 2 || words.len() == 3 => {
                let command = parse_entity(words[1], locale).and_then(|card| {
                    let target = words
                        .get(2)
                        .map(|value| parse_entity(value, locale))
                        .transpose()?;
                    Ok(PlayerCommand::PlayCard { card, target })
                });
                run_command(&mut game, command, locale);
            }
            "playat" | "pa" if words.len() == 3 || words.len() == 4 => {
                let command = parse_entity(words[1], locale).and_then(|card| {
                    let position = words[2].parse::<usize>().map_err(|_| {
                        lf!(
                            locale,
                            "{} is not a valid board position",
                            "{} 不是有效的战场位置",
                            "{} 不是有效的戰場位置",
                            words[2]
                        )
                    })?;
                    let target = words
                        .get(3)
                        .map(|value| parse_entity(value, locale))
                        .transpose()?;
                    Ok(PlayerCommand::PlayCardAt {
                        card,
                        target,
                        position,
                    })
                });
                run_command(&mut game, command, locale);
            }
            "trade" | "tr" if words.len() == 2 => {
                let command =
                    parse_entity(words[1], locale).map(|card| PlayerCommand::TradeCard { card });
                run_command(&mut game, command, locale);
            }
            "action" | "act" if words.len() == 3 || words.len() == 4 => {
                let command = parse_entity(words[1], locale).and_then(|card| {
                    let target = words
                        .get(3)
                        .map(|value| parse_entity(value, locale))
                        .transpose()?;
                    Ok(PlayerCommand::UseCardAction {
                        card,
                        action: words[2].to_owned(),
                        target,
                    })
                });
                run_command(&mut game, command, locale);
            }
            "attack" | "a" if words.len() == 3 => {
                let command = parse_entity(words[1], locale).and_then(|attacker| {
                    Ok(PlayerCommand::Attack {
                        attacker,
                        defender: parse_entity(words[2], locale)?,
                    })
                });
                run_command(&mut game, command, locale);
            }
            "choose" | "c" if words.len() == 2 => {
                let command = words[1]
                    .parse::<usize>()
                    .map(|index| PlayerCommand::Choose { index })
                    .map_err(|_| {
                        lf!(
                            locale,
                            "{} is not a valid choice index",
                            "{} 不是有效的选项编号",
                            "{} 不是有效的選項編號",
                            words[1]
                        )
                    });
                run_command(&mut game, command, locale);
            }
            "mulligan" | "m" => {
                let replace = words[1..]
                    .iter()
                    .map(|value| parse_entity(value, locale))
                    .collect::<Result<Vec<_>, _>>()
                    .map(|replace| PlayerCommand::Mulligan { replace });
                run_command(&mut game, replace, locale);
            }
            "keep" | "k" if words.len() == 1 => run_command(
                &mut game,
                Ok(PlayerCommand::Mulligan {
                    replace: Vec::new(),
                }),
                locale,
            ),
            "power" if words.len() == 1 || words.len() == 2 => {
                let command = words
                    .get(1)
                    .map(|value| parse_entity(value, locale))
                    .transpose()
                    .map(|target| PlayerCommand::UseHeroPower { target });
                run_command(&mut game, command, locale);
            }
            "location" | "loc" if words.len() == 2 || words.len() == 3 => {
                let command = parse_entity(words[1], locale).and_then(|location| {
                    let target = words
                        .get(2)
                        .map(|value| parse_entity(value, locale))
                        .transpose()?;
                    Ok(PlayerCommand::UseLocation { location, target })
                });
                run_command(&mut game, command, locale);
            }
            "end" | "e" => run_command(&mut game, Ok(PlayerCommand::EndTurn), locale),
            "concede" => run_command(&mut game, Ok(PlayerCommand::Concede), locale),
            "quit" | "q" => break,
            _ => eprintln!(
                "{}",
                lt!(
                    locale,
                    "Unknown command. Enter help for usage.",
                    "无法识别的命令，输入 help 查看用法。",
                    "無法識別的命令，輸入 help 查看用法。"
                )
            ),
        }
        if let Some(outcome) = game.state().outcome {
            match outcome {
                GameOutcome::Winner(winner) => println!(
                    "{}",
                    lf!(
                        locale,
                        "Game over. {} wins.",
                        "游戏结束，{} 获胜。",
                        "遊戲結束，{} 獲勝。",
                        winner
                    )
                ),
                GameOutcome::Draw => println!(
                    "{}",
                    lt!(
                        locale,
                        "Game over. Draw.",
                        "游戏结束，平局。",
                        "遊戲結束，平手。"
                    )
                ),
            }
            break;
        }
    }
    Ok(())
}

fn parse_options() -> Result<Option<CliOptions>, Box<dyn Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let default_deck = root.join("decks/demo.json");
    let mut options = CliOptions {
        data: root.join("data"),
        deck_one: default_deck.clone(),
        deck_two: default_deck,
        seed: 20260813,
        replay: None,
        snapshot: None,
        locale: Locale::EnUs,
    };
    let mut show_help = false;
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--data" => options.data = required_value(&mut args, "--data")?.into(),
            // Compatibility alias for launch scripts created before the data/ migration.
            "--cards" => options.data = required_value(&mut args, "--cards")?.into(),
            "--deck-one" => options.deck_one = required_value(&mut args, "--deck-one")?.into(),
            "--deck-two" => options.deck_two = required_value(&mut args, "--deck-two")?.into(),
            "--seed" => {
                options.seed = required_value(&mut args, "--seed")?.parse().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "--seed must be a non-negative integer",
                    )
                })?
            }
            "--replay" => options.replay = Some(required_value(&mut args, "--replay")?.into()),
            "--snapshot" => {
                options.snapshot = Some(required_value(&mut args, "--snapshot")?.into())
            }
            "--locale" => {
                options.locale =
                    required_value(&mut args, "--locale")?
                        .parse()
                        .map_err(|message: String| {
                            io::Error::new(io::ErrorKind::InvalidInput, message)
                        })?
            }
            "--help" | "-h" => {
                show_help = true;
            }
            value => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown option {value}; use --help for usage"),
                )
                .into());
            }
        }
    }
    if options.replay.is_some() && options.snapshot.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--replay and --snapshot cannot be used together",
        )
        .into());
    }
    if show_help {
        print_usage(options.locale);
        return Ok(None);
    }
    Ok(Some(options))
}

fn required_value(
    args: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, io::Error> {
    args.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("option {option} requires a value"),
        )
    })
}

fn load_deck(path: &PathBuf) -> Result<DeckFile, Box<dyn Error>> {
    let source = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&source)?)
}

fn validate_deck(
    runtime: &LuaCardRuntime,
    deck: &DeckFile,
    locale: Locale,
) -> Result<(), Box<dyn Error>> {
    if deck.class.trim().is_empty() || deck.class.len() > 64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            lf!(
                locale,
                "deck {:?} class must contain 1 to 64 bytes",
                "牌组“{}”的 class 必须包含 1 到 64 个字节",
                "牌組「{}」的 class 必須包含 1 到 64 個位元組",
                deck.name
            ),
        )
        .into());
    }
    let mut allowances = Vec::new();
    for card in &deck.cards {
        let valid = runtime.definition(card).is_some_and(|definition| {
            definition.collectible
                && matches!(
                    definition.kind,
                    CardKind::Hero
                        | CardKind::Minion
                        | CardKind::Spell
                        | CardKind::Weapon
                        | CardKind::Location
                )
        });
        if !valid {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                lf!(
                    locale,
                    "deck {:?} references non-deckable card {card}",
                    "牌组“{}”引用了不可加入牌组的卡牌 {card}",
                    "牌組「{}」引用了不可加入牌組的卡牌 {card}",
                    deck.name
                ),
            )
            .into());
        }
        allowances.extend(
            runtime
                .deck_allowances(card)
                .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?,
        );
    }
    if !deck.unrestricted {
        for card in &deck.cards {
            let definition = runtime.definition(card).unwrap();
            let own_class = if definition.classes.is_empty() {
                definition.class.eq_ignore_ascii_case("neutral")
                    || definition.class.eq_ignore_ascii_case(&deck.class)
            } else {
                definition
                    .classes
                    .iter()
                    .any(|class| class.eq_ignore_ascii_case(&deck.class))
            };
            let allowed_by_tourist = allowances.iter().any(|allowance| {
                definition.class.eq_ignore_ascii_case(&allowance.class)
                    && definition.set.eq_ignore_ascii_case(&allowance.set)
                    && allowance.excluded_keywords.iter().all(|excluded| {
                        !definition
                            .keywords
                            .iter()
                            .any(|keyword| keyword == excluded)
                    })
            });
            if !own_class && !allowed_by_tourist {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    lf!(
                        locale,
                        "deck {:?} with class {} cannot include {} ({})",
                        "牌组“{}”的职业 {} 不能加入 {}（{}）",
                        "牌組「{}」的職業 {} 不能加入 {}（{}）",
                        deck.name,
                        deck.class,
                        definition.name,
                        definition.class
                    ),
                )
                .into());
            }
        }
    }
    if let Some(hero_power) = deck.hero_power.as_deref() {
        let valid = runtime
            .definition(hero_power)
            .is_some_and(|definition| definition.kind == CardKind::HeroPower);
        if !valid {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                lf!(
                    locale,
                    "deck {:?} references invalid hero power {hero_power}",
                    "牌组“{}”引用了无效英雄技能 {hero_power}",
                    "牌組「{}」引用了無效英雄能力 {hero_power}",
                    deck.name
                ),
            )
            .into());
        }
    }
    Ok(())
}

fn run_command(
    game: &mut Game<LuaCardRuntime>,
    command: Result<PlayerCommand, String>,
    locale: Locale,
) {
    let before = game.state().log.len();
    match command {
        Ok(command) => match game.dispatch(command) {
            Ok(()) => {
                for event in &game.state().log[before..] {
                    println!("  {}", display_event(game, event, locale));
                }
                print_state(game, locale);
            }
            Err(error) => eprintln!(
                "{}",
                lf!(locale, "Error: {error}", "错误：{error}", "錯誤：{error}")
            ),
        },
        Err(error) => eprintln!(
            "{}",
            lf!(locale, "Error: {error}", "错误：{error}", "錯誤：{error}")
        ),
    }
}

fn parse_entity(value: &str, locale: Locale) -> Result<EntityId, String> {
    value.parse::<u64>().map(EntityId).map_err(|_| {
        lf!(
            locale,
            "{value} is not a valid entity ID",
            "{value} 不是有效的实体 ID",
            "{value} 不是有效的實體 ID"
        )
    })
}

fn print_state(game: &Game<LuaCardRuntime>, locale: Locale) {
    let state = game.state();
    if state.mulligan.is_some() {
        println!(
            "{}",
            lf!(
                locale,
                "\n=== Mulligan / Current: {} ===",
                "\n=== 起手调度 / 当前 {} ===",
                "\n=== 起手調度 / 目前 {} ===",
                state.active_player
            )
        );
    } else {
        println!(
            "{}",
            lf!(
                locale,
                "\n=== Turn {} / Current: {} ===",
                "\n=== 回合 {} / 当前 {} ===",
                "\n=== 回合 {} / 目前 {} ===",
                state.turn,
                state.active_player
            )
        );
    }
    for player_id in [PlayerId::ONE, PlayerId::TWO] {
        let player = state.player(player_id);
        let hero = state.hero(player_id);
        println!(
            "{}",
            lf!(
                locale,
                "{} [{}] Hero {} Health/{} Armor | Mana {}/{} (temporary {}, locked {}, pending Overload {}) | Cards played {} | Hand {} | Deck {}",
                "{} [{}] 英雄 {}生命/{}护甲 | 法力 {}/{}（临时{}，锁定{}，待过载{}） | 本回合出牌 {} | 手牌 {} | 牌库 {}",
                "{} [{}] 英雄 {}生命/{}護甲 | 法力 {}/{}（暫時{}，鎖定{}，待超載{}） | 本回合出牌 {} | 手牌 {} | 牌庫 {}",
                player_id,
                player.class,
                hero.health(),
                hero.armor,
                player.mana,
                player.max_mana,
                player.temporary_mana,
                player.overloaded_mana,
                player.overload_pending,
                player.cards_played_this_turn,
                player.hand.len(),
                player.deck.len()
            )
        );
        print!("{}", lt!(locale, "   Board:", "   战场:", "   戰場:"));
        if player.board.is_empty() {
            print!("{}", lt!(locale, " (empty)", " （空）", " （空）"));
        }
        for id in &player.board {
            let entity = state.entity(*id).unwrap();
            if entity.kind == CardKind::Location {
                let status = if entity.location_cooldown == 0 {
                    lt!(locale, "ready", "可用", "可用").to_owned()
                } else {
                    lf!(
                        locale,
                        "cooldown {}",
                        "冷却{}",
                        "冷卻{}",
                        entity.location_cooldown
                    )
                };
                print!(
                    "{}",
                    lf!(
                        locale,
                        " [{}]{} Location Durability {} {}",
                        " [{}]{} 地标 耐久{} {}",
                        " [{}]{} 地標 耐久度{} {}",
                        id,
                        localized_entity_name(game, entity.id, locale),
                        entity.health(),
                        status
                    )
                );
                continue;
            }
            let status = if entity.exhausted || entity.attacks_this_turn > 0 {
                lt!(locale, "exhausted", "休眠", "休眠")
            } else {
                lt!(locale, "ready", "可攻击", "可攻擊")
            };
            print!(
                " [{}]{} {}/{} {}",
                id,
                localized_entity_name(game, entity.id, locale),
                entity.attack,
                entity.health(),
                status
            );
        }
        println!();
        if let Some(weapon) = player.weapon {
            let entity = state.entity(weapon).unwrap();
            println!(
                "{}",
                lf!(
                    locale,
                    "   Weapon: [{}]{} {}/{}",
                    "   武器: [{}]{} {}/{}",
                    "   武器：[{}]{} {}/{}",
                    weapon,
                    localized_entity_name(game, entity.id, locale),
                    entity.attack,
                    entity.health()
                )
            );
        }
        let hero_power = state.entity(player.hero_power).unwrap();
        println!(
            "{}",
            lf!(
                locale,
                "   Hero Power: [{}]{} ({} mana, {})",
                "   英雄技能: [{}]{}（{}费，{}）",
                "   英雄能力：[{}]{}（{}費，{}）",
                hero_power.id,
                localized_entity_name(game, hero_power.id, locale),
                hero_power.cost,
                if player.hero_power_used {
                    lt!(locale, "used this turn", "本回合已使用", "本回合已使用")
                } else {
                    lt!(locale, "ready", "可用", "可用")
                }
            )
        );
        println!(
            "{}",
            lf!(
                locale,
                "   Secrets: {}",
                "   奥秘: {}个",
                "   秘密：{}個",
                player.secrets.len()
            )
        );
    }
    print_hand(game, locale);
    if let Some(pending) = &state.pending_input {
        println!(
            "{}",
            lf!(
                locale,
                "Waiting for {}: {}",
                "等待 {}：{}",
                "等待 {}：{}",
                pending.player,
                pending.prompt
            )
        );
        for (index, option) in pending.options.iter().enumerate() {
            println!("   ({index}) {}", option.label);
        }
    }
    println!();
}

fn print_hand(game: &Game<LuaCardRuntime>, locale: Locale) {
    let state = game.state();
    let player = state.player(state.active_player);
    print!(
        "{}",
        lf!(
            locale,
            "{} hand:",
            "{} 手牌:",
            "{} 手牌：",
            state.active_player
        )
    );
    for id in &player.hand {
        let entity = state.entity(*id).unwrap();
        let stats = if matches!(entity.kind, CardKind::Minion | CardKind::Weapon) {
            format!(" {}/{}", entity.attack, entity.health())
        } else if entity.kind == CardKind::Location {
            lf!(
                locale,
                " Durability {}",
                " 耐久{}",
                " 耐久度{}",
                entity.health()
            )
        } else {
            String::new()
        };
        print!(
            "{}",
            lf!(
                locale,
                " [{}]{}({} mana{})",
                " [{}]{}({}费{})",
                " [{}]{}({}費{})",
                id,
                localized_entity_name(game, *id, locale),
                entity.cost,
                stats
            )
        );
    }
    println!();
}

fn localized_entity_name(game: &Game<LuaCardRuntime>, id: EntityId, locale: Locale) -> String {
    let Some(entity) = game.state().entity(id) else {
        return format!("Entity {id}");
    };
    game.runtime()
        .definition(&entity.card_id)
        .map(|definition| definition.localized(locale).name)
        .unwrap_or_else(|| entity.name.clone())
}

fn display_event(game: &Game<LuaCardRuntime>, event: &GameEvent, locale: Locale) -> String {
    let name = |id: &EntityId| {
        game.state()
            .entity(*id)
            .map(|_| format!("{}[{}]", localized_entity_name(game, *id, locale), id))
            .unwrap_or_else(|| lf!(locale, "Entity[{id}]", "实体[{id}]", "實體[{id}]"))
    };
    match event {
        GameEvent::GameStarted => lt!(locale, "Game started", "对战开始", "對戰開始").to_owned(),
        GameEvent::TurnStarted { player, turn } => lf!(
            locale,
            "Turn {turn} started: {player}",
            "回合 {turn} 开始：{player}",
            "回合 {turn} 開始：{player}"
        ),
        GameEvent::TurnEnded { player, .. } => lf!(
            locale,
            "{player} ended the turn",
            "{player} 结束回合",
            "{player} 結束回合"
        ),
        GameEvent::CardDrawn { player, card, .. } => lf!(
            locale,
            "{player} drew {}",
            "{player} 抽到 {}",
            "{player} 抽到 {}",
            name(card)
        ),
        GameEvent::CardBurned { player, card, .. } => lf!(
            locale,
            "{player} burned {}",
            "{player} 爆掉 {}",
            "{player} 爆掉 {}",
            name(card)
        ),
        GameEvent::CardCreated { player, card, .. } => {
            lf!(
                locale,
                "{player} received {}",
                "{player} 获得 {}",
                "{player} 獲得 {}",
                name(card)
            )
        }
        GameEvent::Fatigue { player, amount } => lf!(
            locale,
            "{player} took {amount} Fatigue damage",
            "{player} 受到 {amount} 点疲劳伤害",
            "{player} 受到 {amount} 點疲勞傷害"
        ),
        GameEvent::PlayerScriptDataChanged {
            player,
            key,
            old,
            new,
            ..
        } => lf!(
            locale,
            "{player} changed {key}: {old} -> {new}",
            "{player} 更改 {key}：{old} -> {new}",
            "{player} 更改 {key}：{old} -> {new}"
        ),
        GameEvent::CardPlayed { player, card, .. } => lf!(
            locale,
            "{player} played {}",
            "{player} 打出 {}",
            "{player} 打出 {}",
            name(card)
        ),
        GameEvent::SpellCast {
            player,
            spell,
            generated_by,
            target: _,
            ..
        } => match generated_by {
            Some(source) => lf!(
                locale,
                "{player} cast {0} from {1}",
                "{player} 由 {1} 施放 {0}",
                "{player} 由 {1} 施放 {0}",
                name(spell),
                name(source)
            ),
            None => lf!(
                locale,
                "{player} cast {}",
                "{player} 施放 {}",
                "{player} 施放 {}",
                name(spell)
            ),
        },
        GameEvent::SpellTargeted {
            player,
            spell,
            target,
            ..
        } => lf!(
            locale,
            "{player} targeted {1} with {0}",
            "{player} 用 {0} 选中了 {1}",
            "{player} 用 {0} 選中了 {1}",
            name(spell),
            name(target)
        ),
        GameEvent::MinionPlayed { player, minion } => {
            lf!(
                locale,
                "{player} played minion {}",
                "{player} 打出随从 {}",
                "{player} 打出手下 {}",
                name(minion)
            )
        }
        GameEvent::WeaponPlayed { player, weapon } => {
            lf!(
                locale,
                "{player} played weapon {}",
                "{player} 打出武器 {}",
                "{player} 打出武器 {}",
                name(weapon)
            )
        }
        GameEvent::CardCountered { player, card } => {
            lf!(
                locale,
                "{player}'s {} was Countered",
                "{player} 的 {} 被反制",
                "{player} 的 {} 被反制",
                name(card)
            )
        }
        GameEvent::CardDiscarded {
            source,
            player,
            card,
        } => lf!(
            locale,
            "{player} discarded {0} because of {1}",
            "{player} 因 {1} 弃掉 {0}",
            "{player} 因 {1} 棄掉 {0}",
            name(card),
            name(source)
        ),
        GameEvent::CardTraded { player, card } => {
            lf!(
                locale,
                "{player} Traded {}",
                "{player} 交易了 {}",
                "{player} 交易了 {}",
                name(card)
            )
        }
        GameEvent::TradeDraw {
            player,
            replacement,
            ..
        } => match replacement {
            Some(card) => lf!(
                locale,
                "{player} replaced the Trade draw with {}",
                "{player} 将交易抽牌替换为 {}",
                "{player} 將交易抽牌替換為 {}",
                name(card)
            ),
            None => lf!(
                locale,
                "{player} completed the Trade draw",
                "{player} 完成交易抽牌",
                "{player} 完成交易抽牌"
            ),
        },
        GameEvent::MinionSummoned { player, entity } => {
            lf!(
                locale,
                "{player} summoned {}",
                "{player} 召唤 {}",
                "{player} 召喚 {}",
                name(entity)
            )
        }
        GameEvent::Magnetized {
            player,
            attachment,
            target,
        } => lf!(
            locale,
            "{player} Magnetized {} onto {}",
            "{player} 将 {} 磁化到 {}",
            "{player} 將 {} 磁化到 {}",
            name(attachment),
            name(target)
        ),
        GameEvent::WeaponEquipped { player, weapon } => {
            lf!(
                locale,
                "{player} equipped {}",
                "{player} 装备 {}",
                "{player} 裝備 {}",
                name(weapon)
            )
        }
        GameEvent::WeaponDestroyed { player, weapon } => {
            lf!(
                locale,
                "{player}'s {} was destroyed",
                "{player} 的 {} 被摧毁",
                "{player} 的 {} 被摧毀",
                name(weapon)
            )
        }
        GameEvent::LocationPlayed { player, location } => {
            lf!(
                locale,
                "{player} played Location {}",
                "{player} 打出地标 {}",
                "{player} 打出地標 {}",
                name(location)
            )
        }
        GameEvent::LocationUsed {
            player, location, ..
        } => lf!(
            locale,
            "{player} used Location {}",
            "{player} 激活地标 {}",
            "{player} 啟用地標 {}",
            name(location)
        ),
        GameEvent::LocationDestroyed { player, location } => {
            lf!(
                locale,
                "{player}'s Location {} was depleted",
                "{player} 的地标 {} 耗尽",
                "{player} 的地標 {} 耗盡",
                name(location)
            )
        }
        GameEvent::HeroPowerUsed {
            player, hero_power, ..
        } => lf!(
            locale,
            "{player} used {}",
            "{player} 使用 {}",
            "{player} 使用 {}",
            name(hero_power)
        ),
        GameEvent::HeroPowerReplaced {
            player, old, new, ..
        } => lf!(
            locale,
            "{player} replaced Hero Power {} with {}",
            "{player} 将英雄技能 {} 替换为 {}",
            "{player} 將英雄能力 {} 替換為 {}",
            name(old),
            name(new)
        ),
        GameEvent::HeroReplaced { player, old, new } => lf!(
            locale,
            "{player} replaced Hero {} with {}",
            "{player} 将英雄 {} 替换为 {}",
            "{player} 將英雄 {} 替換為 {}",
            name(old),
            name(new)
        ),
        GameEvent::SecretPlayed { player, secret } => {
            lf!(
                locale,
                "{player} played Secret {}",
                "{player} 挂上奥秘 {}",
                "{player} 掛上秘密 {}",
                name(secret)
            )
        }
        GameEvent::SecretRevealed { player, secret } => {
            lf!(
                locale,
                "{player}'s Secret {} triggered",
                "{player} 的奥秘 {} 被触发",
                "{player} 的秘密 {} 被觸發",
                name(secret)
            )
        }
        GameEvent::ZoneChanged { entity, from, to } => {
            lf!(
                locale,
                "{} moved from {from:?} to {to:?}",
                "{} 从 {from:?} 移动到 {to:?}",
                "{} 從 {from:?} 移動到 {to:?}",
                name(entity)
            )
        }
        GameEvent::ControllerChanged {
            entity, from, to, ..
        } => lf!(
            locale,
            "control of {} moved from {from} to {to}",
            "{} 的控制权从 {from} 转移给 {to}",
            "{} 的控制權從 {from} 轉移給 {to}",
            name(entity)
        ),
        GameEvent::Transformed {
            entity,
            from_card,
            to_card,
            ..
        } => lf!(
            locale,
            "{} transformed from {from_card} into {to_card}",
            "{} 从 {from_card} 变形为 {to_card}",
            "{} 從 {from_card} 變形為 {to_card}",
            name(entity)
        ),
        GameEvent::Attack {
            attacker, defender, ..
        } => {
            lf!(
                locale,
                "{} attacked {}",
                "{} 攻击 {}",
                "{} 攻擊 {}",
                name(attacker),
                name(defender)
            )
        }
        GameEvent::Damaged {
            source,
            target,
            amount,
        } => lf!(
            locale,
            "{} dealt {amount} damage to {}",
            "{} 对 {} 造成 {amount} 点伤害",
            "{} 對 {} 造成 {amount} 點傷害",
            name(source),
            name(target)
        ),
        GameEvent::DamagePrevented {
            source,
            target,
            reason,
        } => lf!(
            locale,
            "damage from {} to {} was prevented ({reason})",
            "{} 对 {} 的伤害被阻止（{reason}）",
            "{} 對 {} 的傷害被阻止（{reason}）",
            name(source),
            name(target)
        ),
        GameEvent::Healed {
            source,
            target,
            amount,
        } => lf!(
            locale,
            "{} restored {amount} Health to {}",
            "{} 为 {} 恢复 {amount} 点生命",
            "{} 為 {} 恢復 {amount} 點生命",
            name(source),
            name(target)
        ),
        GameEvent::ArmorGained {
            source,
            target,
            amount,
        } => lf!(
            locale,
            "{} gave {} {amount} Armor",
            "{} 使 {} 获得 {amount} 点护甲",
            "{} 使 {} 獲得 {amount} 點護甲",
            name(source),
            name(target)
        ),
        GameEvent::OverloadQueued {
            source,
            player,
            amount,
        } => lf!(
            locale,
            "{} gave {player} Overload: ({amount})",
            "{} 使 {player} 下回合过载 {amount}",
            "{} 使 {player} 下回合超載 {amount}",
            name(source)
        ),
        GameEvent::ManaLocked { player, amount } => {
            lf!(
                locale,
                "{player} has {amount} Mana Crystals locked this turn",
                "{player} 本回合锁定 {amount} 个法力水晶",
                "{player} 本回合鎖定 {amount} 個法力水晶"
            )
        }
        GameEvent::ManaUnlocked {
            source,
            player,
            amount,
        } => lf!(
            locale,
            "{} unlocked {amount} Mana Crystals for {player}",
            "{} 为 {player} 解锁 {amount} 个法力水晶",
            "{} 為 {player} 解鎖 {amount} 個法力水晶",
            name(source)
        ),
        GameEvent::OverloadCleared {
            source,
            player,
            pending,
            locked,
        } => lf!(
            locale,
            "{} cleared {player}'s Overload (locked {locked}, pending {pending})",
            "{} 为 {player} 清除过载（当前 {locked}，待生效 {pending}）",
            "{} 為 {player} 清除超載（目前 {locked}，待生效 {pending}）",
            name(source)
        ),
        GameEvent::TemporaryManaGained {
            source,
            player,
            amount,
        } => lf!(
            locale,
            "{} gave {player} {amount} temporary Mana",
            "{} 使 {player} 获得 {amount} 点临时法力",
            "{} 使 {player} 獲得 {amount} 點暫時法力",
            name(source)
        ),
        GameEvent::TemporaryManaExpired { player, amount } => {
            lf!(
                locale,
                "{player}'s {amount} temporary Mana expired",
                "{player} 的 {amount} 点临时法力过期",
                "{player} 的 {amount} 點暫時法力失效"
            )
        }
        GameEvent::ManaCrystalsGained {
            source,
            player,
            amount,
            filled,
        } => lf!(
            locale,
            "{} gave {player} {amount} {} Mana Crystals",
            "{} 使 {player} 获得 {amount} 个{}法力水晶",
            "{} 使 {player} 獲得 {amount} 個{}法力水晶",
            name(source),
            if *filled {
                lt!(locale, "full", "已充能", "已充能")
            } else {
                lt!(locale, "empty", "空", "空")
            }
        ),
        GameEvent::ManaCrystalsDestroyed {
            source,
            player,
            amount,
        } => lf!(
            locale,
            "{} destroyed {amount} of {player}'s Mana Crystals",
            "{} 摧毁 {player} 的 {amount} 个法力水晶",
            "{} 摧毀 {player} 的 {amount} 個法力水晶",
            name(source)
        ),
        GameEvent::ManaSpent {
            player,
            source,
            amount,
            temporary,
        } => lf!(
            locale,
            "{player} spent {amount} Mana on {} ({temporary} temporary)",
            "{player} 为 {} 花费 {amount} 点法力（临时 {temporary}）",
            "{player} 為 {} 花費 {amount} 點法力（暫時 {temporary}）",
            name(source)
        ),
        GameEvent::KeywordDisabled {
            source,
            target,
            keyword,
        } => {
            lf!(
                locale,
                "{0} removed keyword {1} from {2}",
                "{0} 使 {2} 失去关键词 {1}",
                "{0} 使 {2} 失去關鍵字 {1}",
                name(source),
                keyword,
                name(target)
            )
        }
        GameEvent::Frozen { source, target } => {
            lf!(
                locale,
                "{} Froze {}",
                "{} 冻结了 {}",
                "{} 凍結了 {}",
                name(source),
                name(target)
            )
        }
        GameEvent::EntityDied { entity, .. } => {
            lf!(locale, "{} died", "{} 死亡", "{} 死亡", name(entity))
        }
        GameEvent::Conceded { player } => lf!(
            locale,
            "{player} conceded",
            "{player} 认输",
            "{player} 投降"
        ),
        GameEvent::GameEnded { outcome } => match outcome {
            GameOutcome::Winner(winner) => {
                lf!(locale, "{winner} won", "{winner} 获胜", "{winner} 獲勝")
            }
            GameOutcome::Draw => lt!(locale, "Draw", "平局", "平手").to_owned(),
        },
        GameEvent::ChoiceRequested {
            player, options, ..
        } => lf!(
            locale,
            "{player} must choose from {options} options",
            "{player} 需要从 {options} 个选项中选择",
            "{player} 需要從 {options} 個選項中選擇"
        ),
        GameEvent::ChoiceMade { player, index, .. } => {
            lf!(
                locale,
                "{player} chose option {index}",
                "{player} 选择了选项 {index}",
                "{player} 選擇了選項 {index}"
            )
        }
        GameEvent::RandomChoiceMade {
            source,
            index,
            options,
        } => lf!(
            locale,
            "{} randomly selected option {index}/{options}",
            "{} 的随机选择命中选项 {index}/{options}",
            "{} 的隨機選擇命中選項 {index}/{options}",
            name(source)
        ),
        GameEvent::RandomCardsSampled {
            source,
            cards,
            population,
        } => lf!(
            locale,
            "{} sampled {} from {population} candidate cards",
            "{} 从 {population} 张候选牌中抽样得到 {}",
            "{} 從 {population} 張候選牌中抽樣得到 {}",
            name(source),
            cards.join(", ")
        ),
        GameEvent::RandomEntitiesSampled {
            source,
            entities,
            population,
        } => lf!(
            locale,
            "{} sampled {} from {population} candidate entities",
            "{} 从 {population} 个候选实体中抽样得到 {}",
            "{} 從 {population} 個候選實體中抽樣得到 {}",
            name(source),
            entities
                .iter()
                .map(|entity| name(entity))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn display_ids(ids: &[EntityId]) -> String {
    ids.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn display_command(command: &PlayerCommand) -> String {
    match command {
        PlayerCommand::Mulligan { replace } if replace.is_empty() => "keep".to_owned(),
        PlayerCommand::Mulligan { replace } => format!(
            "mulligan {}",
            replace
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        ),
        PlayerCommand::PlayCard { card, target } => match target {
            Some(target) => format!("play {card} {target}"),
            None => format!("play {card}"),
        },
        PlayerCommand::PlayCardAt {
            card,
            target,
            position,
        } => match target {
            Some(target) => format!("playat {card} {position} {target}"),
            None => format!("playat {card} {position}"),
        },
        PlayerCommand::TradeCard { card } => format!("trade {card}"),
        PlayerCommand::UseCardAction {
            card,
            action,
            target,
        } => match target {
            Some(target) => format!("action {card} {action} {target}"),
            None => format!("action {card} {action}"),
        },
        PlayerCommand::Attack { attacker, defender } => {
            format!("attack {attacker} {defender}")
        }
        PlayerCommand::EndTurn => "end".to_owned(),
        PlayerCommand::Concede => "concede".to_owned(),
        PlayerCommand::Choose { index } => format!("choose {index}"),
        PlayerCommand::UseHeroPower { target } => match target {
            Some(target) => format!("power {target}"),
            None => "power".to_owned(),
        },
        PlayerCommand::UseLocation { location, target } => match target {
            Some(target) => format!("location {location} {target}"),
            None => format!("location {location}"),
        },
    }
}

fn print_help(locale: Locale) {
    println!(
        "{}",
        lt!(
            locale,
            "Commands: state | hand | cards | legal | save <replay-file> | snapshot <snapshot-file> | mulligan [card-ids...] | keep | targets <card> | play <card> [target] | playat <card> <position> [target] | trade <card> | action <card> <action> [target] | attack <attacker> <target> | power [target] | location <location> [target] | choose <index> | end | concede | quit",
            "命令：state | hand | cards | legal | save <replay文件> | snapshot <快照文件> | mulligan [换牌ID...] | keep | targets <卡牌> | play <卡牌> [目标] | playat <卡牌> <位置> [目标] | trade <卡牌> | action <卡牌> <动作> [目标] | attack <攻击者> <目标> | power [目标] | location <地标> [目标] | choose <编号> | end | concede | quit",
            "命令：state | hand | cards | legal | save <replay檔案> | snapshot <快照檔案> | mulligan [換牌ID...] | keep | targets <卡牌> | play <卡牌> [目標] | playat <卡牌> <位置> [目標] | trade <卡牌> | action <卡牌> <動作> [目標] | attack <攻擊者> <目標> | power [目標] | location <地標> [目標] | choose <編號> | end | concede | quit"
        )
    );
}

fn print_usage(locale: Locale) {
    println!(
        "{}",
        lt!(
            locale,
            "Usage: hearth-cli [--data DIR] [--deck-one FILE] [--deck-two FILE] [--seed N] [--locale enUS|zhCN|zhTW] [--replay FILE | --snapshot FILE]",
            "用法：hearth-cli [--data DIR] [--deck-one FILE] [--deck-two FILE] [--seed N] [--locale enUS|zhCN|zhTW] [--replay FILE | --snapshot FILE]",
            "用法：hearth-cli [--data DIR] [--deck-one FILE] [--deck-two FILE] [--seed N] [--locale enUS|zhCN|zhTW] [--replay FILE | --snapshot FILE]"
        )
    );
}
