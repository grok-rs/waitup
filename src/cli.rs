use std::process::Command;
use std::time::Duration;

use clap::Parser;

use crate::connection::wait_for_targets;
use crate::types::{Error, Headers, Result, Target, WaitConfig};

#[derive(Parser)]
#[command(name = "waitup")]
#[command(about = "Block until host:port is reachable; exit non-zero on timeout")]
#[command(version)]
struct Args {
    #[arg(value_name = "TARGET")]
    targets: Vec<String>,

    #[arg(short, long, env = "WAITUP_TIMEOUT", default_value = "30s")]
    timeout: String,

    #[arg(short, long, env = "WAITUP_INTERVAL", default_value = "1s")]
    interval: String,

    #[arg(long, conflicts_with = "all")]
    any: bool,

    #[arg(long, conflicts_with = "any")]
    all: bool,

    #[arg(long, action = clap::ArgAction::Append)]
    header: Vec<String>,

    #[arg(long, default_value = "10s")]
    connection_timeout: String,

    #[arg(last = true)]
    command: Vec<String>,
}

struct Config {
    targets: Vec<Target>,
    wait: WaitConfig,
    command: Vec<String>,
}

fn parse_duration(s: &str, label: &str) -> Result<Duration> {
    s.parse::<humantime::Duration>()
        .map(Into::into)
        .map_err(|e| Error::Config(format!("Invalid {label} '{s}': {e}")))
}

fn parse_headers(raw: &[String]) -> Result<Headers> {
    raw.iter()
        .map(|h| {
            let (key, value) = h.split_once(':').ok_or_else(|| {
                Error::Config(format!("Invalid header format '{h}': expected 'key:value'"))
            })?;
            Ok((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

fn build_config(args: Args) -> Result<Config> {
    if args.targets.is_empty() {
        return Err(Error::Config(
            "At least one target must be specified".to_string(),
        ));
    }

    let headers = parse_headers(&args.header)?;
    let targets: Vec<Target> = args
        .targets
        .iter()
        .map(|s| Target::parse(s, &headers))
        .collect::<Result<_>>()?;
    let wait_for_any = args.any || (!args.all && targets.len() == 1);

    Ok(Config {
        targets,
        wait: WaitConfig {
            timeout: parse_duration(&args.timeout, "timeout")?,
            initial_interval: parse_duration(&args.interval, "interval")?,
            wait_for_any,
            connection_timeout: parse_duration(&args.connection_timeout, "connection-timeout")?,
        },
        command: args.command,
    })
}

fn execute_command(command: &[String]) -> Result<()> {
    if command.is_empty() {
        return Ok(());
    }

    let status = Command::new(&command[0])
        .args(&command[1..])
        .status()
        .map_err(|e| Error::Command(e.to_string()))?;

    if !status.success() {
        return Err(Error::Command(format!(
            "Exited with code: {:?}",
            status.code()
        )));
    }

    Ok(())
}

pub async fn run() -> i32 {
    let args = Args::parse();

    let config = match build_config(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            return 2;
        }
    };

    if let Err(e) = wait_for_targets(&config.targets, &config.wait).await {
        eprintln!("Error: {e}");
        return 1;
    }

    if let Err(e) = execute_command(&config.command) {
        eprintln!("Command error: {e}");
        return 3;
    }

    0
}
