use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use url::Url;

use crate::transport::{DEFAULT_KEEPALIVE_INTERVAL, DEFAULT_KEEPALIVE_SECS, DEFAULT_NODELAY};

// No constants here

/// String with Debug implementation that emits "MASKED"
/// Used to mask sensitive strings when logging
#[derive(Default, PartialEq, Eq, Clone)]
pub struct MaskedString(pub String);

impl Debug for MaskedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_str("MASKED")
    }
}

impl Deref for MaskedString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for MaskedString {
    fn from(s: &str) -> MaskedString {
        MaskedString(String::from(s))
    }
}

impl From<String> for MaskedString {
    fn from(s: String) -> MaskedString {
        MaskedString(s)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TransportType {
    #[default]
    Tcp,
    Tls,
    Noise,
    Websocket,
}

/// Per service config
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientServiceConfig {
    pub service_type: ServiceType,
    pub name: String,
    pub local_addr: String,
    pub prefer_ipv6: bool,
    pub token: Option<MaskedString>,
    pub nodelay: Option<bool>,
    pub retry_interval: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServiceType {
    #[default]
    Tcp,
    Udp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TlsConfig {
    pub hostname: Option<String>,
    pub trusted_root: Option<String>,
    pub pkcs12: Option<String>,
    pub pkcs12_password: Option<MaskedString>,
}

fn default_noise_pattern() -> String {
    String::from("Noise_NK_25519_ChaChaPoly_BLAKE2s")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoiseConfig {
    pub pattern: String,
    pub local_private_key: Option<MaskedString>,
    pub remote_public_key: Option<String>,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            pattern: default_noise_pattern(),
            local_private_key: None,
            remote_public_key: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebsocketConfig {
    pub tls: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpConfig {
    pub nodelay: bool,
    pub keepalive_secs: u64,
    pub keepalive_interval: u64,
    pub proxy: Option<Url>,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            nodelay: DEFAULT_NODELAY,
            keepalive_secs: DEFAULT_KEEPALIVE_SECS,
            keepalive_interval: DEFAULT_KEEPALIVE_INTERVAL,
            proxy: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TransportConfig {
    pub transport_type: TransportType,
    pub tcp: TcpConfig,
    pub tls: Option<TlsConfig>,
    pub noise: Option<NoiseConfig>,
    pub websocket: Option<WebsocketConfig>,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ClientConfig {
    pub remote_addr: String,
    pub default_token: Option<MaskedString>,
    pub prefer_ipv6: Option<bool>,
    pub services: HashMap<String, ClientServiceConfig>,
    pub transport: TransportConfig,
    pub heartbeat_timeout: u64,
    pub retry_interval: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Config {
    pub client: Option<ClientConfig>,
}

impl Config {
    pub fn validate_client_config(client: &mut ClientConfig) -> Result<()> {
        for (name, s) in &mut client.services {
            s.name = name.clone();
            if s.token.is_none() {
                s.token = client.default_token.clone();
                if s.token.is_none() {
                    bail!("The token of service {} is not set", name);
                }
            }
            if s.retry_interval.is_none() {
                s.retry_interval = Some(client.retry_interval);
            }
        }

        Config::validate_transport_config(&client.transport)?;
        Ok(())
    }

    fn validate_transport_config(config: &TransportConfig) -> Result<()> {
        config
            .tcp
            .proxy
            .as_ref()
            .map_or(Ok(()), |u| match u.scheme() {
                "socks5" => Ok(()),
                "http" => Ok(()),
                _ => Err(anyhow!(format!("Unknown proxy scheme: {}", u.scheme()))),
            })?;
        match config.transport_type {
            TransportType::Tcp => Ok(()),
            TransportType::Tls => {
                let _tls_config = config
                    .tls
                    .as_ref()
                    .ok_or_else(|| anyhow!("Missing TLS configuration"))?;
                Ok(())
            }
            TransportType::Noise => Ok(()),
            TransportType::Websocket => Ok(()),
        }
    }
}
