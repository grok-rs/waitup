use std::time::Duration;

use tokio::net::TcpStream;
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep, timeout};

use crate::types::{Error, Header, Result, Target, WaitConfig};

async fn try_tcp_connect(host: &str, port: u16, conn_timeout: Duration) -> Result<()> {
    timeout(conn_timeout, TcpStream::connect((host, port)))
        .await
        .map_err(|_| {
            Error::Connection(format!(
                "Connection timeout after {}ms",
                conn_timeout.as_millis()
            ))
        })?
        .map_err(|e| Error::Connection(e.to_string()))?;
    Ok(())
}

async fn try_http_connect(
    url: &reqwest::Url,
    headers: &[Header],
    conn_timeout: Duration,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(conn_timeout)
        .build()
        .map_err(|e| Error::Connection(format!("HTTP client error for {url}: {e}")))?;

    let mut request = client.get(url.clone());
    for (key, value) in headers {
        request = request.header(key, value);
    }

    let response = request
        .send()
        .await
        .map_err(|e| Error::Connection(format!("HTTP request failed for {url}: {e}")))?;

    let status = response.status();
    if status.is_success() {
        Ok(())
    } else {
        Err(Error::Connection(format!(
            "Expected 2xx status, got {status}"
        )))
    }
}

async fn try_connect(target: &Target, conn_timeout: Duration) -> Result<()> {
    match target {
        Target::Tcp { host, port } => try_tcp_connect(host, *port, conn_timeout).await,
        Target::Http { url, headers } => try_http_connect(url, headers, conn_timeout).await,
    }
}

async fn wait_for_single_target(target: &Target, config: &WaitConfig) -> Result<()> {
    let deadline = Instant::now() + config.timeout;

    loop {
        let now = Instant::now();
        if now >= deadline {
            return Err(Error::Timeout(target.to_string()));
        }

        let remaining = deadline.duration_since(now);
        let conn_timeout = config.connection_timeout.min(remaining);

        if try_connect(target, conn_timeout).await.is_ok() {
            return Ok(());
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        sleep(config.initial_interval.min(remaining)).await;
    }
}

pub async fn wait_for_targets(targets: &[Target], config: &WaitConfig) -> Result<()> {
    if targets.is_empty() {
        return Ok(());
    }

    let mut set = JoinSet::new();
    for target in targets {
        let target = target.clone();
        let config = config.clone();
        set.spawn(async move { wait_for_single_target(&target, &config).await });
    }

    if config.wait_for_any {
        while let Some(result) = set.join_next().await {
            if result.unwrap().is_ok() {
                return Ok(());
            }
        }
        return Err(Error::Timeout("all targets timed out".into()));
    }

    let mut failed = Vec::new();
    while let Some(result) = set.join_next().await {
        if let Err(e) = result.unwrap() {
            failed.push(e.to_string());
        }
    }

    if !failed.is_empty() {
        return Err(Error::Timeout(failed.join(", ")));
    }

    Ok(())
}
