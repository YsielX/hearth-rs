use std::{path::PathBuf, process::ExitCode};

use hearth_fuzz::{FuzzOptions, run_campaign};

#[derive(Debug)]
struct BinaryOptions {
    data: PathBuf,
    fuzz: FuzzOptions,
}

fn usage() -> &'static str {
    "Usage: hearth-state-fuzz [--data DIR] [--start-seed N] [--seeds N] [--steps N]\n\
     \n\
     Options:\n\
       --data DIR      Lua card data directory (default: repository data/)\n\
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

fn parse_options() -> Result<Option<BinaryOptions>, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut options = BinaryOptions {
        data: root.join("data"),
        fuzz: FuzzOptions::default(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--data" => options.data = parse_value(&mut args, "--data")?,
            "--start-seed" => options.fuzz.start_seed = parse_value(&mut args, "--start-seed")?,
            "--seeds" => options.fuzz.seeds = parse_value(&mut args, "--seeds")?,
            "--steps" => options.fuzz.steps = parse_value(&mut args, "--steps")?,
            "-h" | "--help" => return Ok(None),
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }
    options.fuzz.validate().map_err(|error| error.to_string())?;
    Ok(Some(options))
}

fn run() -> Result<(), String> {
    let Some(options) = parse_options()? else {
        println!("{}", usage());
        return Ok(());
    };
    println!(
        "state-machine fuzz: data={}, start_seed={}, seeds={}, max_steps={}",
        options.data.display(),
        options.fuzz.start_seed,
        options.fuzz.seeds,
        options.fuzz.steps
    );
    run_campaign(&options.data, &options.fuzz).map_err(|error| error.to_string())?;
    println!(
        "state-machine fuzz passed: {} deterministic seeds",
        options.fuzz.seeds
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
