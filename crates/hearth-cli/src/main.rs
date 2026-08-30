use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use hearth_app::{MatchSession, MatchSetup};
use hearth_bot::SimpleBot;
use hearth_core::{
    CardKind, CardRuntime, EntityId, GameOutcome, GameSnapshot, LegalAction, Locale, PlayerCommand,
    PlayerController, PlayerId, PlayerView, PublicEvent, Replay,
};
use hearth_fuzz::{FuzzController, FuzzOptions, run_campaign};

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
    controllers: [ControllerKind; 2],
    debug_state: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ControllerKind {
    Interactive,
    Bot,
    Fuzzer,
}

impl std::str::FromStr for ControllerKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "interactive" | "human" | "cli" => Ok(Self::Interactive),
            "bot" => Ok(Self::Bot),
            "fuzzer" | "fuzz" => Ok(Self::Fuzzer),
            _ => Err(format!(
                "unknown controller {value}; expected interactive, bot, or fuzzer"
            )),
        }
    }
}

#[derive(Debug)]
enum Controller {
    Interactive,
    Bot(SimpleBot),
    Fuzzer(FuzzController),
}

impl Controller {
    fn from_kind(kind: ControllerKind, seed: u64) -> Self {
        match kind {
            ControllerKind::Interactive => Self::Interactive,
            ControllerKind::Bot => Self::Bot(SimpleBot),
            ControllerKind::Fuzzer => Self::Fuzzer(FuzzController::new(seed)),
        }
    }

    fn is_interactive(&self) -> bool {
        matches!(self, Self::Interactive)
    }

    fn choose_action(
        &mut self,
        view: &PlayerView,
        legal_actions: &[LegalAction],
    ) -> Result<PlayerCommand, String> {
        match self {
            Self::Interactive => Err("interactive controller requires terminal input".to_owned()),
            Self::Bot(bot) => bot.choose_action(view, legal_actions),
            Self::Fuzzer(fuzzer) => fuzzer.choose_action(view, legal_actions),
        }
    }
}

#[derive(Debug)]
enum CliInvocation {
    Play(CliOptions),
    Fuzz { data: PathBuf, options: FuzzOptions },
}

fn main() -> Result<(), Box<dyn Error>> {
    let Some(invocation) = parse_options()? else {
        return Ok(());
    };
    let options = match invocation {
        CliInvocation::Play(options) => options,
        CliInvocation::Fuzz { data, options } => {
            println!(
                "state-machine fuzz: data={}, start_seed={}, seeds={}, max_steps={}",
                data.display(),
                options.start_seed,
                options.seeds,
                options.steps
            );
            run_campaign(&data, &options)?;
            println!(
                "state-machine fuzz passed: {} deterministic seeds",
                options.seeds
            );
            return Ok(());
        }
    };
    let locale = options.locale;
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
        MatchSession::from_snapshot(&options.data, locale, &snapshot)?
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
        MatchSession::from_replay(&options.data, locale, &replay)?
    } else {
        let session = MatchSession::load(&MatchSetup {
            data_dir: options.data.clone(),
            deck_one: options.deck_one.clone(),
            deck_two: options.deck_two.clone(),
            seed: options.seed,
            locale,
        })?;
        println!(
            "{}",
            lf!(
                locale,
                "P1 deck: {} [{}] ({} cards)",
                "P1 牌组：{} [{}]（{}张）",
                "P1 牌組：{} [{}]（{}張）",
                session.deck_name(PlayerId::ONE),
                session.state().player(PlayerId::ONE).class,
                session.state().player(PlayerId::ONE).starting_deck.len()
            )
        );
        println!(
            "{}",
            lf!(
                locale,
                "P2 deck: {} [{}] ({} cards)",
                "P2 牌组：{} [{}]（{}张）",
                "P2 牌組：{} [{}]（{}張）",
                session.deck_name(PlayerId::TWO),
                session.state().player(PlayerId::TWO).class,
                session.state().player(PlayerId::TWO).starting_deck.len()
            )
        );
        session
    };
    println!(
        "{}",
        lf!(
            locale,
            "Loaded {0} Lua card and Hero Power definitions from {1}.",
            "已从 {1} 加载 {0} 个 Lua 卡牌与英雄技能定义。",
            "已從 {1} 載入 {0} 個 Lua 卡牌與英雄能力定義。",
            game.runtime().card_ids().len(),
            options.data.display()
        )
    );
    let match_seed = game.state().rng_seed;
    let mut controllers = [
        Controller::from_kind(options.controllers[0], match_seed ^ 0x243f_6a88_85a3_08d3),
        Controller::from_kind(options.controllers[1], match_seed ^ 0x1319_8a2e_0370_7344),
    ];
    print_help(locale);
    let initial_viewer = observer_for(&controllers, game.state().input_player());
    print_state(&game, initial_viewer, locale);

    let (input_sender, input_receiver) = mpsc::channel::<io::Result<String>>();
    thread::spawn(move || {
        let stdin = io::stdin();
        loop {
            let mut line = String::new();
            match stdin.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if input_sender.send(Ok(line)).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let _ = input_sender.send(Err(error));
                    break;
                }
            }
        }
    });
    let hotseat = controllers.iter().all(Controller::is_interactive);
    let mut last_interactive_player = hotseat.then_some(game.state().input_player());
    let mut automated_actions = 0usize;
    let mut turn_deadline: Option<(u32, PlayerId, u64, Instant)> = None;
    loop {
        let input_player = game.state().input_player();
        let viewer = observer_for(&controllers, input_player);
        if hotseat && last_interactive_player != Some(input_player) {
            println!(
                "{}",
                lf!(
                    locale,
                    "Pass control to {0}, then press Enter.",
                    "请将控制权交给 {0}，然后按回车。",
                    "請將控制權交給 {0}，然後按 Enter。",
                    input_player
                )
            );
            io::stdout().flush()?;
            let confirmation = match input_receiver.recv() {
                Ok(line) => line?,
                Err(_) => break,
            };
            if confirmation.is_empty() {
                break;
            }
            print!("\x1b[2J\x1b[3J\x1b[H");
            print_state(&game, input_player, locale);
            last_interactive_player = Some(input_player);
        }
        let time_limit = game.turn_time_limit_seconds()?;
        let timer_key = (game.state().turn, input_player, time_limit.unwrap_or(0));
        if time_limit.is_none() {
            turn_deadline = None;
        } else if !turn_deadline
            .is_some_and(|(turn, player, seconds, _)| (turn, player, seconds) == timer_key)
        {
            turn_deadline = time_limit.map(|seconds| {
                (
                    game.state().turn,
                    input_player,
                    seconds,
                    Instant::now() + Duration::from_secs(seconds),
                )
            });
        }
        if !controllers[input_player.index()].is_interactive() {
            automated_actions = automated_actions.saturating_add(1);
            if automated_actions > 10_000 {
                return Err(io::Error::other("automated game exceeded 10,000 commands").into());
            }
            let legal_actions = game.legal_action_options()?;
            let view = game.state().player_view(input_player);
            let command = controllers[input_player.index()]
                .choose_action(&view, &legal_actions)
                .map_err(io::Error::other)?;
            println!(
                "{}: {}",
                input_player,
                display_automated_command(&command, input_player, viewer, locale)
            );
            let before = game.state().public_history(viewer).len();
            game.dispatch(command)?;
            for record in &game.state().public_history(viewer)[before..] {
                if let Some(summary) = display_public_event(&game, viewer, &record.event) {
                    println!("  {summary}");
                }
            }
            print_state(&game, viewer, locale);
            if let Some(outcome) = game.state().outcome {
                print_outcome(outcome, locale);
                break;
            }
            continue;
        }
        print!("{}> ", input_player);
        io::stdout().flush()?;
        let line = if let Some((_, _, seconds, deadline)) = turn_deadline {
            match input_receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                Ok(line) => line?,
                Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {
                    println!(
                        "{}",
                        lf!(
                            locale,
                            "Turn timer expired after {0} seconds.",
                            "回合计时在 {0} 秒后结束。",
                            "回合計時在 {0} 秒後結束。",
                            seconds
                        )
                    );
                    let command = hearth_app::timeout_command(&game.legal_action_options()?)
                        .ok_or_else(|| io::Error::other("timed turn has no legal action"))?;
                    game.dispatch(command)?;
                    print_state(
                        &game,
                        observer_for(&controllers, game.state().input_player()),
                        locale,
                    );
                    continue;
                }
            }
        } else {
            match input_receiver.recv() {
                Ok(line) => line?,
                Err(_) => break,
            }
        };
        if line.is_empty() {
            break;
        }
        let words: Vec<_> = line.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }
        match words[0] {
            "help" | "?" => print_help(locale),
            "state" | "s" => print_state(&game, viewer, locale),
            "hand" | "h" => print_hand(&game, viewer, locale),
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
            "save" | "snapshot" if !options.debug_state => eprintln!(
                "{}",
                lt!(
                    locale,
                    "Authoritative replay/snapshot export is disabled during player-visible games; restart with --debug-state to enable it.",
                    "玩家可见对局中已禁用权威 replay/snapshot 导出；如需调试，请使用 --debug-state 重新启动。",
                    "玩家可見對局中已停用權威 replay/snapshot 匯出；如需除錯，請使用 --debug-state 重新啟動。"
                )
            ),
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
                run_command(&mut game, command, viewer, locale);
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
                run_command(&mut game, command, viewer, locale);
            }
            "trade" | "tr" if words.len() == 2 => {
                let command =
                    parse_entity(words[1], locale).map(|card| PlayerCommand::TradeCard { card });
                run_command(&mut game, command, viewer, locale);
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
                run_command(&mut game, command, viewer, locale);
            }
            "attack" | "a" if words.len() == 3 => {
                let command = parse_entity(words[1], locale).and_then(|attacker| {
                    Ok(PlayerCommand::Attack {
                        attacker,
                        defender: parse_entity(words[2], locale)?,
                    })
                });
                run_command(&mut game, command, viewer, locale);
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
                run_command(&mut game, command, viewer, locale);
            }
            "mulligan" | "m" => {
                let replace = words[1..]
                    .iter()
                    .map(|value| parse_entity(value, locale))
                    .collect::<Result<Vec<_>, _>>()
                    .map(|replace| PlayerCommand::Mulligan { replace });
                run_command(&mut game, replace, viewer, locale);
            }
            "keep" | "k" if words.len() == 1 => run_command(
                &mut game,
                Ok(PlayerCommand::Mulligan {
                    replace: Vec::new(),
                }),
                viewer,
                locale,
            ),
            "power" if words.len() == 1 || words.len() == 2 => {
                let command = words
                    .get(1)
                    .map(|value| parse_entity(value, locale))
                    .transpose()
                    .map(|target| PlayerCommand::UseHeroPower { target });
                run_command(&mut game, command, viewer, locale);
            }
            "location" | "loc" if words.len() == 2 || words.len() == 3 => {
                let command = parse_entity(words[1], locale).and_then(|location| {
                    let target = words
                        .get(2)
                        .map(|value| parse_entity(value, locale))
                        .transpose()?;
                    Ok(PlayerCommand::UseLocation { location, target })
                });
                run_command(&mut game, command, viewer, locale);
            }
            "end" | "e" => run_command(&mut game, Ok(PlayerCommand::EndTurn), viewer, locale),
            "concede" => run_command(&mut game, Ok(PlayerCommand::Concede), viewer, locale),
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
            print_outcome(outcome, locale);
            break;
        }
    }
    Ok(())
}

fn parse_options() -> Result<Option<CliInvocation>, Box<dyn Error>> {
    let root = hearth_app::runtime_root();
    let mut args = env::args().skip(1).peekable();
    let Some(command) = args.next() else {
        print_usage();
        return Ok(None);
    };
    match command.as_str() {
        "play" => parse_play_options(&root, args),
        "fuzz" => parse_fuzz_options(&root, args),
        "--help" | "-h" => {
            print_usage();
            Ok(None)
        }
        value => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown subcommand {value}; use --help for usage"),
        )
        .into()),
    }
}

fn parse_play_options(
    root: &std::path::Path,
    mut args: impl Iterator<Item = String>,
) -> Result<Option<CliInvocation>, Box<dyn Error>> {
    let default_deck = root.join("decks/demo.json");
    let mut options = CliOptions {
        data: root.join("data"),
        deck_one: default_deck.clone(),
        deck_two: default_deck,
        seed: 20260813,
        replay: None,
        snapshot: None,
        locale: Locale::EnUs,
        controllers: [ControllerKind::Interactive, ControllerKind::Interactive],
        debug_state: false,
    };
    let mut show_help = false;
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
            "--player-one" | "--p1" => {
                options.controllers[0] =
                    required_value(&mut args, "--player-one")?.parse().map_err(
                        |message: String| io::Error::new(io::ErrorKind::InvalidInput, message),
                    )?
            }
            "--player-two" | "--p2" => {
                options.controllers[1] =
                    required_value(&mut args, "--player-two")?.parse().map_err(
                        |message: String| io::Error::new(io::ErrorKind::InvalidInput, message),
                    )?
            }
            "--locale" => {
                options.locale =
                    required_value(&mut args, "--locale")?
                        .parse()
                        .map_err(|message: String| {
                            io::Error::new(io::ErrorKind::InvalidInput, message)
                        })?
            }
            "--debug-state" => options.debug_state = true,
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
        print_play_usage(options.locale);
        return Ok(None);
    }
    Ok(Some(CliInvocation::Play(options)))
}

fn observer_for(controllers: &[Controller; 2], input_player: PlayerId) -> PlayerId {
    let interactive = [PlayerId::ONE, PlayerId::TWO]
        .into_iter()
        .filter(|player| controllers[player.index()].is_interactive())
        .collect::<Vec<_>>();
    if interactive.len() == 1 {
        interactive[0]
    } else {
        input_player
    }
}

fn print_outcome(outcome: GameOutcome, locale: Locale) {
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
}

fn parse_fuzz_options(
    root: &std::path::Path,
    mut args: impl Iterator<Item = String>,
) -> Result<Option<CliInvocation>, Box<dyn Error>> {
    let mut data = root.join("data");
    let mut options = FuzzOptions::default();
    let mut show_help = false;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--data" => data = required_value(&mut args, "--data")?.into(),
            // Compatibility alias for launch scripts created before the data/ migration.
            "--cards" => data = required_value(&mut args, "--cards")?.into(),
            "--start-seed" => options.start_seed = parse_fuzz_value(&mut args, "--start-seed")?,
            "--seeds" => options.seeds = parse_fuzz_value(&mut args, "--seeds")?,
            "--steps" => options.steps = parse_fuzz_value(&mut args, "--steps")?,
            "--help" | "-h" => show_help = true,
            value => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown fuzz option {value}; use fuzz --help for usage"),
                )
                .into());
            }
        }
    }
    if show_help {
        print_fuzz_usage();
        return Ok(None);
    }
    options.validate()?;
    Ok(Some(CliInvocation::Fuzz { data, options }))
}

fn parse_fuzz_value<T: std::str::FromStr>(
    args: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<T, io::Error> {
    let value = required_value(args, option)?;
    value.parse().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{option} must be a non-negative integer"),
        )
    })
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

fn run_command(
    game: &mut MatchSession,
    command: Result<PlayerCommand, String>,
    viewer: PlayerId,
    locale: Locale,
) {
    let before = game.state().public_history(viewer).len();
    match command {
        Ok(command) => match game.dispatch(command) {
            Ok(()) => {
                for record in &game.state().public_history(viewer)[before..] {
                    if let Some(summary) = display_public_event(game, viewer, &record.event) {
                        println!("  {summary}");
                    }
                }
                print_state(game, viewer, locale);
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

fn print_state(game: &MatchSession, viewer: PlayerId, locale: Locale) {
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
                "{} [{}] Hero {} Health/{} Armor | Mana {}/{} (temporary {}, locked {}, pending Overload {}) | Corpses {} (spent {}) | Cards played {} | Hand {} | Deck {}",
                "{} [{}] 英雄 {}生命/{}护甲 | 法力 {}/{}（临时{}，锁定{}，待过载{}） | 残骸 {}（已消耗{}） | 本回合出牌 {} | 手牌 {} | 牌库 {}",
                "{} [{}] 英雄 {}生命/{}護甲 | 法力 {}/{}（暫時{}，鎖定{}，待超載{}） | 屍體 {}（已消耗{}） | 本回合出牌 {} | 手牌 {} | 牌庫 {}",
                player_id,
                player.class,
                hero.health(),
                hero.armor,
                player.mana,
                player.max_mana,
                player.temporary_mana,
                player.overloaded_mana,
                player.overload_pending,
                player.resource("corpses"),
                player.resource_spent("corpses"),
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
        let public_objectives = player
            .secrets
            .iter()
            .copied()
            .filter(|entity| {
                state
                    .entity(*entity)
                    .is_some_and(hearth_core::Entity::is_public_objective)
            })
            .collect::<Vec<_>>();
        println!(
            "{}",
            lf!(
                locale,
                "   Secrets: {}",
                "   奥秘: {}个",
                "   秘密：{}個",
                player.secrets.len().saturating_sub(public_objectives.len())
            )
        );
        if !public_objectives.is_empty() {
            println!(
                "{}",
                lf!(
                    locale,
                    "   Public objectives: {}",
                    "   公开任务：{}",
                    "   公開任務：{}",
                    public_objectives
                        .iter()
                        .map(|entity| localized_entity_name(game, *entity, locale))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            );
        }
    }
    print_hand(game, viewer, locale);
    if let Some(pending) = &state.pending_input {
        if pending.player != viewer {
            println!(
                "{}",
                lf!(
                    locale,
                    "Waiting for {0} to make a hidden choice.",
                    "等待 {0} 完成隐藏选择。",
                    "等待 {0} 完成隱藏選擇。",
                    pending.player
                )
            );
            println!();
            return;
        }
        let public_choice = state
            .player_view(viewer)
            .pending_input
            .expect("the choosing player must receive its public choice view");
        println!(
            "{}",
            lf!(
                locale,
                "Waiting for {}: {}",
                "等待 {}：{}",
                "等待 {}：{}",
                pending.player,
                public_choice.prompt
            )
        );
        for (index, option) in public_choice.options.iter().enumerate() {
            println!("   ({index}) {option}");
        }
    }
    println!();
}

fn print_hand(game: &MatchSession, viewer: PlayerId, locale: Locale) {
    let state = game.state();
    let player = state.player(viewer);
    print!(
        "{}",
        lf!(locale, "{} hand:", "{} 手牌:", "{} 手牌：", viewer)
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

fn localized_entity_name(game: &MatchSession, id: EntityId, locale: Locale) -> String {
    let Some(entity) = game.state().entity(id) else {
        return format!("Entity {id}");
    };
    game.runtime()
        .definition(&entity.card_id)
        .map(|definition| definition.localized(locale).name)
        .unwrap_or_else(|| entity.name.clone())
}

fn display_public_event(
    game: &MatchSession,
    viewer: PlayerId,
    event: &PublicEvent,
) -> Option<String> {
    hearth_app::presentation::event_text::event_summary_with_options(
        game,
        viewer,
        event,
        hearth_app::presentation::event_text::EventTextOptions {
            players: hearth_app::presentation::event_text::PlayerTextStyle::Absolute,
            entities: hearth_app::presentation::event_text::EntityTextStyle::NameAndId,
            verbosity: hearth_app::presentation::event_text::EventVerbosity::Detailed,
        },
    )
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
        PlayerCommand::ConcedePlayer { player } => format!("concede {player}"),
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

fn display_automated_command(
    command: &PlayerCommand,
    actor: PlayerId,
    viewer: PlayerId,
    locale: Locale,
) -> String {
    if actor == viewer {
        return display_command(command);
    }
    match command {
        PlayerCommand::Mulligan { replace } => lf!(
            locale,
            "replaced {} opening cards",
            "更换了 {} 张起手牌",
            "更換了 {} 張起手牌",
            replace.len()
        ),
        PlayerCommand::Choose { .. } => lt!(
            locale,
            "made a hidden choice",
            "完成了一次隐藏选择",
            "完成了一次隱藏選擇"
        )
        .to_owned(),
        PlayerCommand::TradeCard { .. } => {
            lt!(locale, "Traded a card", "交易了一张牌", "交易了一張牌").to_owned()
        }
        PlayerCommand::EndTurn => "end".to_owned(),
        PlayerCommand::Concede | PlayerCommand::ConcedePlayer { .. } => "concede".to_owned(),
        _ => lt!(locale, "acted", "执行了一个动作", "執行了一個動作").to_owned(),
    }
}

fn print_help(locale: Locale) {
    println!(
        "{}",
        lt!(
            locale,
            "Commands: state | hand | cards | legal | save <replay-file> [debug] | snapshot <snapshot-file> [debug] | mulligan [card-ids...] | keep | targets <card> | play <card> [target] | playat <card> <position> [target] | trade <card> | action <card> <action> [target] | attack <attacker> <target> | power [target] | location <location> [target] | choose <index> | end | concede | quit",
            "命令：state | hand | cards | legal | save <replay文件> [调试] | snapshot <快照文件> [调试] | mulligan [换牌ID...] | keep | targets <卡牌> | play <卡牌> [目标] | playat <卡牌> <位置> [目标] | trade <卡牌> | action <卡牌> <动作> [目标] | attack <攻击者> <目标> | power [目标] | location <地标> [目标] | choose <编号> | end | concede | quit",
            "命令：state | hand | cards | legal | save <replay檔案> [除錯] | snapshot <快照檔案> [除錯] | mulligan [換牌ID...] | keep | targets <卡牌> | play <卡牌> [目標] | playat <卡牌> <位置> [目標] | trade <卡牌> | action <卡牌> <動作> [目標] | attack <攻擊者> <目標> | power [目標] | location <地標> [目標] | choose <編號> | end | concede | quit"
        )
    );
}

fn print_usage() {
    println!(
        "Usage: hearth-cli <COMMAND> [OPTIONS]\n\
         \n\
         Commands:\n\
           play  run an interactive, bot, or fuzzer-controlled game\n\
           fuzz  run deterministic state-machine fuzzing\n\
         \n\
         Run `hearth-cli <COMMAND> --help` for command-specific options."
    );
}

fn print_play_usage(locale: Locale) {
    println!(
        "{}",
        lt!(
            locale,
            "Usage: hearth-cli play [--data DIR] [--deck-one FILE] [--deck-two FILE] [--player-one interactive|bot|fuzzer] [--player-two interactive|bot|fuzzer] [--seed N] [--locale enUS|zhCN|zhTW] [--replay FILE | --snapshot FILE] [--debug-state]",
            "用法：hearth-cli play [--data DIR] [--deck-one FILE] [--deck-two FILE] [--player-one interactive|bot|fuzzer] [--player-two interactive|bot|fuzzer] [--seed N] [--locale enUS|zhCN|zhTW] [--replay FILE | --snapshot FILE] [--debug-state]",
            "用法：hearth-cli play [--data DIR] [--deck-one FILE] [--deck-two FILE] [--player-one interactive|bot|fuzzer] [--player-two interactive|bot|fuzzer] [--seed N] [--locale enUS|zhCN|zhTW] [--replay FILE | --snapshot FILE] [--debug-state]"
        )
    );
}

fn print_fuzz_usage() {
    println!(
        "Usage: hearth-cli fuzz [--data DIR] [--start-seed N] [--seeds N] [--steps N]\n\
         \n\
         Options:\n\
           --data DIR      Lua card data directory (default: repository data/)\n\
           --start-seed N  first deterministic seed (default: 0)\n\
           --seeds N       number of games to explore (default: 8)\n\
           --steps N       maximum actions per game (default: 180)\n\
           -h, --help      show this help"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automated_hidden_actions_do_not_leak_entity_ids() {
        let mulligan = PlayerCommand::Mulligan {
            replace: vec![EntityId(40), EntityId(41)],
        };
        let automated =
            display_automated_command(&mulligan, PlayerId::TWO, PlayerId::ONE, Locale::EnUs);
        assert_eq!(automated, "replaced 2 opening cards");
        assert!(!automated.contains("40"));
        assert!(!automated.contains("41"));
    }
}
