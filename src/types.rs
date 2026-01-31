use core::fmt;
use core::time::Duration;
use reqwest::Url;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Connection(String),
    #[error("Timeout waiting for {0}")]
    Timeout(String),
    #[error("Command failed: {0}")]
    Command(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub type Header = (String, String);
pub type Headers = Vec<Header>;

#[derive(Debug, Clone)]
pub enum Target {
    Tcp { host: String, port: u16 },
    Http { url: Url, headers: Headers },
}

impl Target {
    pub fn parse(target_str: &str, headers: &[Header]) -> Result<Self> {
        if target_str.starts_with("http://") || target_str.starts_with("https://") {
            let url = Url::parse(target_str)
                .map_err(|e| Error::Config(format!("Invalid URL '{target_str}': {e}")))?;
            validate_headers(headers)?;
            return Ok(Self::Http {
                url,
                headers: headers.to_vec(),
            });
        }

        let (host, port_str) = target_str.split_once(':').ok_or_else(|| {
            Error::Config(format!(
                "Invalid target '{target_str}': expected host:port or URL"
            ))
        })?;

        if host.is_empty() {
            return Err(Error::Config(format!("Empty hostname in '{target_str}'")));
        }

        let port: u16 = port_str
            .parse()
            .map_err(|_| Error::Config(format!("Invalid port '{port_str}' in '{target_str}'")))?;

        if port == 0 {
            return Err(Error::Config(format!(
                "Port must be 1-65535, got 0 in '{target_str}'"
            )));
        }

        Ok(Self::Tcp {
            host: host.to_string(),
            port,
        })
    }
}

fn validate_headers(headers: &[Header]) -> Result<()> {
    for (key, value) in headers {
        if key.is_empty() {
            return Err(Error::Config("HTTP header key cannot be empty".to_string()));
        }
        if value.is_empty() {
            return Err(Error::Config(
                "HTTP header value cannot be empty".to_string(),
            ));
        }
        if !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_".contains(c))
        {
            return Err(Error::Config(format!("Invalid HTTP header name: {key}")));
        }
    }
    Ok(())
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tcp { host, port } => write!(f, "{host}:{port}"),
            Self::Http { url, .. } => write!(f, "{url}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WaitConfig {
    pub timeout: Duration,
    pub initial_interval: Duration,
    pub wait_for_any: bool,
    pub connection_timeout: Duration,
}
