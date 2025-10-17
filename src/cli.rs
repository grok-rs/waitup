use clap::{CommandFactory, Parser};
use indicatif::{ProgressBar, ProgressStyle};
use std::borrow::Cow;
use std::process::Command;
use std::time::Duration;
use waitup::{Target, WaitConfig, WaitForError, WaitResult, wait_for_connection};

/// Extended error type for CLI-specific errors
#[derive(thiserror::Error, Debug)]
enum CliError {
    #[error("Wait error: {0}")]
    WaitError(#[from] WaitForError),
    #[error("Invalid timeout format '{0}': {1}")]
    InvalidTimeout(String, String),
    #[error("Invalid interval format '{0}': {1}")]
    InvalidInterval(String, String),
    #[error("Command execution failed: {0}")]
    CommandExecution(String),
    #[error("JSON serialization failed: {0}")]
    JsonSerialization(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, CliError>;

#[derive(Parser)]
#[command(name = "waitup")]
#[command(about = "Block until host:port is reachable; exit non-zero on timeout")]
#[command(version)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "CLI arg structures naturally have many boolean flags"
)]
struct Args {
    /// Targets to wait for (host:port or http(s)://host/path)
    #[arg(value_name = "TARGET")]
    targets: Vec<String>,

    /// Connection timeout (e.g., "30s", "2m", "1h")
    #[arg(short, long, env = "WAITUP_TIMEOUT", default_value = "30s")]
    timeout: String,

    /// Initial retry interval (e.g., "1s", "500ms", "2s")
    #[arg(short, long, env = "WAITUP_INTERVAL", default_value = "1s")]
    interval: String,

    /// Maximum retry interval for exponential backoff
    #[arg(long, default_value = "30s")]
    max_interval: String,

    /// Expected HTTP status code for HTTP targets
    #[arg(long, default_value = "200")]
    expect_status: u16,

    /// Wait for any target to be ready (default: wait for all)
    #[arg(long, conflicts_with = "all")]
    any: bool,

    /// Wait for all targets to be ready (explicit flag)
    #[arg(long, conflicts_with = "any")]
    all: bool,

    /// Suppress output messages
    #[arg(short, long, conflicts_with = "json")]
    quiet: bool,

    /// Verbose output with progress information
    #[arg(short, long, conflicts_with = "quiet")]
    verbose: bool,

    /// Output result in JSON format
    #[arg(long, conflicts_with = "quiet")]
    json: bool,

    /// Maximum number of retry attempts
    #[arg(long)]
    retry_limit: Option<u32>,

    /// Custom HTTP headers (format: "key:value")
    #[arg(long, action = clap::ArgAction::Append)]
    header: Vec<String>,

    /// Connection timeout for individual attempts
    #[arg(long, default_value = "10s")]
    connection_timeout: String,

    /// Generate shell completion script
    #[arg(long, value_enum)]
    generate_completion: Option<clap_complete::Shell>,

    /// Command to execute after successful connection
    #[arg(last = true)]
    command: Vec<String>,
}

#[derive(Debug, Clone)]
struct CliConfig {
    targets: Vec<Target>,
    wait_config: WaitConfig,
    quiet: bool,
    verbose: bool,
    json: bool,
    command: Vec<String>,
}

impl CliConfig {
    fn from_args(args: Args) -> Result<Self> {
        // When generating completions, we don't need to validate targets
        if args.generate_completion.is_some() {
            return Ok(Self {
                targets: Vec::new(),
                wait_config: WaitConfig::default(),
                quiet: true,
                verbose: false,
                json: false,
                command: Vec::new(),
            });
        }

        // Validate that targets are provided when not generating completions
        if args.targets.is_empty() {
            return Err(CliError::WaitError(WaitForError::InvalidTarget(
                Cow::Borrowed("At least one target must be specified"),
            )));
        }

        let mut targets = Vec::new();

        let mut headers = Vec::new();
        for header_str in &args.header {
            let parts: Vec<&str> = header_str.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(CliError::WaitError(WaitForError::InvalidTarget(
                    Cow::Owned(format!(
                        "Invalid header format '{header_str}': expected 'key:value'"
                    )),
                )));
            }
            headers.push((parts[0].trim().to_string(), parts[1].trim().to_string()));
        }

        for target_str in &args.targets {
            if target_str.starts_with("http://") || target_str.starts_with("https://") {
                let url = url::Url::parse(target_str).map_err(|_| {
                    CliError::WaitError(WaitForError::InvalidTarget(Cow::Owned(target_str.clone())))
                })?;

                if headers.is_empty() {
                    targets.push(Target::http(url, args.expect_status)?);
                } else {
                    targets.push(Target::http_with_headers(
                        url,
                        args.expect_status,
                        headers.clone(),
                    )?);
                }
            } else {
                targets.push(Target::parse(target_str, args.expect_status)?);
            }
        }

        let timeout = args
            .timeout
            .parse::<humantime::Duration>()
            .map_err(|e| CliError::InvalidTimeout(args.timeout, e.to_string()))?
            .into();

        let initial_interval = args
            .interval
            .parse::<humantime::Duration>()
            .map_err(|e| CliError::InvalidInterval(args.interval, e.to_string()))?
            .into();

        let max_interval = args
            .max_interval
            .parse::<humantime::Duration>()
            .map_err(|e| CliError::InvalidInterval(args.max_interval, e.to_string()))?
            .into();

        let connection_timeout = args
            .connection_timeout
            .parse::<humantime::Duration>()
            .map_err(|e| CliError::InvalidInterval(args.connection_timeout, e.to_string()))?
            .into();

        let wait_for_any = args.any || (!args.all && targets.len() == 1);

        let wait_config = WaitConfig::builder()
            .timeout(timeout)
            .interval(initial_interval)
            .max_interval(max_interval)
            .wait_for_any(wait_for_any)
            .max_retries(args.retry_limit)
            .connection_timeout(connection_timeout)
            .build();

        Ok(Self {
            targets,
            wait_config,
            quiet: args.quiet,
            verbose: args.verbose,
            json: args.json,
            command: args.command,
        })
    }
}

/// Output formatter for wait results
mod output {
    use super::{CliConfig, Result, WaitResult};
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct JsonOutput {
        pub success: bool,
        pub elapsed_ms: u64,
        pub total_attempts: u32,
        pub targets: Vec<JsonTargetResult>,
    }

    #[derive(Serialize)]
    pub struct JsonTargetResult {
        pub target: String,
        pub success: bool,
        pub elapsed_ms: u64,
        pub attempts: u32,
        pub error: Option<String>,
    }

    #[allow(
        clippy::print_stdout,
        clippy::print_stderr,
        reason = "CLI output to stdout/stderr is required"
    )]
    pub fn format_result(result: &WaitResult, config: &CliConfig) -> Result<()> {
        if config.json {
            let json_output = JsonOutput {
                success: result.success,
                elapsed_ms: u64::try_from(result.elapsed.as_millis().min(u128::from(u64::MAX)))
                    .unwrap_or(u64::MAX),
                total_attempts: result.attempts,
                targets: result
                    .target_results
                    .iter()
                    .map(|tr| JsonTargetResult {
                        target: tr.target.display(),
                        success: tr.success,
                        elapsed_ms: u64::try_from(tr.elapsed.as_millis().min(u128::from(u64::MAX)))
                            .unwrap_or(u64::MAX),
                        attempts: tr.attempts,
                        error: tr.error.clone(),
                    })
                    .collect(),
            };
            println!(
                "{json_output}",
                json_output = serde_json::to_string_pretty(&json_output)?
            );
        } else if !config.quiet && !result.success {
            eprintln!("Failed to connect to targets");
        }
        Ok(())
    }
}

async fn wait_with_progress(config: &CliConfig) -> Result<WaitResult> {
    if config.verbose && !config.quiet && !config.json {
        use futures::StreamExt;
        use futures::stream::FuturesUnordered;
        use waitup::wait_for_single_target;

        let multi_progress = indicatif::MultiProgress::new();
        let progress_bars: Result<Vec<_>> = config
            .targets
            .iter()
            .map(|target| -> Result<ProgressBar> {
                let pb = multi_progress.add(ProgressBar::new_spinner());
                pb.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner:.green} {msg}")
                        .map_err(|_| {
                            CliError::WaitError(WaitForError::InvalidTimeout(
                                std::borrow::Cow::Borrowed("progress"),
                                std::borrow::Cow::Borrowed("Invalid progress template"),
                            ))
                        })?,
                );
                pb.set_message(format!("Waiting for {target}", target = target.display()));
                pb.enable_steady_tick(Duration::from_millis(100));
                Ok(pb)
            })
            .collect();

        let progress_bars = progress_bars?;

        // Spawn per-target futures and update progress bars as each completes.
        let mut futures: FuturesUnordered<_> = FuturesUnordered::new();
        for (target_index, target) in config.targets.iter().enumerate() {
            futures.push(async move {
                (
                    target_index,
                    wait_for_single_target(target, &config.wait_config).await,
                )
            });
        }

        let mut target_results = vec![None; config.targets.len()];

        while let Some((target_index, res)) = futures.next().await {
            match res {
                Ok(target_result) => {
                    if let Some(pb) = progress_bars.get(target_index) {
                        if target_result.success {
                            pb.finish_with_message(format!(
                                "✓ {target}",
                                target = target_result.target.display()
                            ));
                        } else {
                            pb.finish_with_message(format!(
                                "✗ {target} ({error})",
                                target = target_result.target.display(),
                                error = target_result.error.as_deref().unwrap_or("failed")
                            ));
                        }
                    }
                    target_results[target_index] = Some(target_result);
                }
                Err(wferror) => {
                    // If a per-target check errors out (e.g., cancelled), record as failed
                    if let Some(pb) = progress_bars.get(target_index) {
                        pb.finish_with_message(format!(
                            "✗ {target} ({error})",
                            target = config.targets[target_index].display(),
                            error = wferror
                        ));
                    }
                    target_results[target_index] = Some(waitup::TargetResult {
                        target: config.targets[target_index].clone(),
                        success: false,
                        elapsed: std::time::Duration::from_secs(0),
                        attempts: 0,
                        error: Some(wferror.to_string()),
                    });
                }
            }
        }

        // Collect final results
        let mut all_successful = true;
        let mut total_attempts: u32 = 0;
        let final_results = target_results
            .into_iter()
            .flatten()
            .inspect(|tr| {
                if !tr.success {
                    all_successful = false;
                }
                total_attempts += tr.attempts;
            })
            .collect::<Vec<_>>();

        let total_elapsed = final_results
            .iter()
            .map(|tr| tr.elapsed)
            .max()
            .unwrap_or_else(|| Duration::from_millis(0));
        if !all_successful {
            let failed_targets: Vec<_> = final_results
                .iter()
                .filter(|r| !r.success)
                .map(|r| r.target.display())
                .collect();
            return Err(CliError::WaitError(WaitForError::Timeout {
                targets: std::borrow::Cow::Owned(failed_targets.join(", ")),
            }));
        }

        Ok(WaitResult {
            success: all_successful,
            elapsed: total_elapsed,
            attempts: total_attempts,
            target_results: final_results,
        })
    } else {
        wait_for_connection(&config.targets, &config.wait_config)
            .await
            .map_err(CliError::WaitError)
    }
}

fn execute_command(command: &[String]) -> Result<()> {
    if command.is_empty() {
        return Ok(());
    }

    let mut cmd = Command::new(&command[0]);
    if command.len() > 1 {
        cmd.args(&command[1..]);
    }

    let status = cmd
        .status()
        .map_err(|e| CliError::CommandExecution(e.to_string()))?;

    if !status.success() {
        return Err(CliError::CommandExecution(format!(
            "Command exited with code: {:?}",
            status.code()
        )));
    }

    Ok(())
}

/// Main CLI entry point
#[allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::if_not_else,
    reason = "CLI functions require stdout/stderr output and complex conditional logic"
)]
pub async fn run() -> i32 {
    let args = Args::parse();

    if let Some(shell) = args.generate_completion {
        let mut cmd = Args::command();
        let name = cmd.get_name().to_string();
        clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
        return 0;
    }

    let config = match CliConfig::from_args(args) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {e}");
            return 2;
        }
    };

    let result = match wait_with_progress(&config).await {
        Ok(result) => result,
        Err(e) => {
            if !config.json {
                eprintln!("Error: {e}");
            } else {
                let json_error = serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                });
                println!("{json_error}");
            }
            return 1;
        }
    };

    if let Err(e) = output::format_result(&result, &config) {
        eprintln!("Output error: {e}");
        return 1;
    }

    if !result.success {
        return 1;
    }

    if let Err(e) = execute_command(&config.command) {
        if !config.json {
            eprintln!("Command execution error: {e}");
        } else {
            let json_error = serde_json::json!({
                "success": false,
                "error": format!("Command execution failed: {e}")
            });
            println!("{json_error}");
        }
        return 3;
    }

    0
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    reason = "test code where panics are acceptable"
)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn verbose_streaming_internal_returns_timeout_with_failed_target() {
        // Start one server that will accept a single connection
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (_stream, _addr) = listener.accept().await.unwrap();
        });

        let targets = vec![
            Target::loopback(addr.port()).unwrap(),
            Target::loopback(65534).unwrap(), // likely unreachable
        ];

        let wait_cfg = WaitConfig::builder()
            .timeout(Duration::from_secs(1))
            .build();

        let cli_cfg = CliConfig {
            targets,
            wait_config: wait_cfg,
            quiet: false,
            verbose: true,
            json: false,
            command: Vec::new(),
        };

        let res = wait_with_progress(&cli_cfg).await;

        match res {
            Ok(_) => panic!("expected timeout error when one target is unreachable"),
            Err(CliError::WaitError(wf)) => match wf {
                WaitForError::Timeout { targets } => {
                    assert!(
                        targets.contains("127.0.0.1:65534"),
                        "timeout targets did not contain unreachable target: {targets}"
                    );
                }
                other => panic!("unexpected WaitForError: {other:?}"),
            },
            Err(e) => panic!("unexpected CLI error: {e}"),
        }
    }

    #[tokio::test]
    async fn verbose_streaming_internal_all_success_returns_expected_waitresult() {
        let timeout = Duration::from_secs(1);
        // Start two servers that will accept a single connection each
        let listener1 = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr1 = listener1.local_addr().unwrap();

        let listener2 = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr2 = listener2.local_addr().unwrap();

        // Spawn accept tasks so the servers will accept the client's connections
        tokio::spawn(async move {
            let (_stream, _addr) = listener1.accept().await.unwrap();
        });
        tokio::spawn(async move {
            let (_stream, _addr) = listener2.accept().await.unwrap();
        });

        let targets = vec![
            Target::loopback(addr1.port()).unwrap(),
            Target::loopback(addr2.port()).unwrap(),
        ];

        let wait_cfg = WaitConfig::builder().timeout(timeout).build();

        let cli_cfg = CliConfig {
            targets: targets.clone(),
            wait_config: wait_cfg,
            quiet: false,
            verbose: true,
            json: false,
            command: Vec::new(),
        };

        let result = wait_with_progress(&cli_cfg)
            .await
            .expect("expected success WaitResult, got error");

        // All targets must be successful
        assert!(result.success);
        assert_eq!(result.target_results.len(), 2);
        for tr in &result.target_results {
            assert!(tr.success, "target {:?} should be successful", tr.target);
        }

        // requests should succeed once per target
        assert_eq!(result.attempts, 2);

        // elapsed must be under timeout
        assert!(result.elapsed < timeout);
    }
}
