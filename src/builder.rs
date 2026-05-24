use anyhow::{anyhow, Result};
use std::collections::HashMap;
use crate::config::{ClientConfig, ClientServiceConfig, MaskedString, ServiceType, TransportConfig, TransportType, TcpConfig, TlsConfig, NoiseConfig, WebsocketConfig};

pub struct Service {
    pub(crate) config: ClientServiceConfig,
}

impl Service {
    pub fn builder() -> ServiceBuilder {
        ServiceBuilder::new()
    }
}

pub struct ServiceBuilder {
    name: Option<String>,
    local_addr: Option<String>,
    token: Option<String>,
    service_type: ServiceType,
    prefer_ipv6: bool,
    nodelay: Option<bool>,
    retry_interval: Option<u64>,
}

impl ServiceBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            local_addr: None,
            token: None,
            service_type: ServiceType::Tcp,
            prefer_ipv6: false,
            nodelay: None,
            retry_interval: None,
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn local_addr(mut self, addr: &str) -> Self {
        self.local_addr = Some(addr.to_string());
        self
    }

    pub fn token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }

    pub fn service_type(mut self, service_type: ServiceType) -> Self {
        self.service_type = service_type;
        self
    }

    pub fn prefer_ipv6(mut self, prefer: bool) -> Self {
        self.prefer_ipv6 = prefer;
        self
    }

    pub fn nodelay(mut self, nodelay: bool) -> Self {
        self.nodelay = Some(nodelay);
        self
    }

    pub fn retry_interval(mut self, retry_interval: u64) -> Self {
        self.retry_interval = Some(retry_interval);
        self
    }

    pub fn build(self) -> Result<Service> {
        let name = self.name.ok_or_else(|| anyhow!("Service name is required"))?;
        let local_addr = self.local_addr.ok_or_else(|| anyhow!("Service local address is required"))?;

        Ok(Service {
            config: ClientServiceConfig {
                service_type: self.service_type,
                name,
                local_addr,
                prefer_ipv6: self.prefer_ipv6,
                token: self.token.map(MaskedString),
                nodelay: self.nodelay,
                retry_interval: self.retry_interval,
            },
        })
    }
}

pub struct Client {
    pub(crate) config: ClientConfig,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub async fn spawn_background(self) -> crate::handle::TunnelHandle {
        crate::client::spawn_client(self.config).await
    }
}

pub struct ClientBuilder {
    remote_addr: Option<String>,
    default_token: Option<MaskedString>,
    prefer_ipv6: Option<bool>,
    services: Vec<Service>,
    transport: TransportConfig,
    heartbeat_timeout: Option<u64>,
    retry_interval: Option<u64>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            remote_addr: None,
            default_token: None,
            prefer_ipv6: None,
            services: Vec::new(),
            transport: TransportConfig::default(),
            heartbeat_timeout: None,
            retry_interval: None,
        }
    }

    pub fn remote_addr(mut self, addr: &str) -> Self {
        self.remote_addr = Some(addr.to_string());
        self
    }

    pub fn default_token(mut self, token: &str) -> Self {
        self.default_token = Some(MaskedString(token.to_string()));
        self
    }

    pub fn prefer_ipv6(mut self, prefer: bool) -> Self {
        self.prefer_ipv6 = Some(prefer);
        self
    }

    pub fn add_service(mut self, service: Service) -> Self {
        self.services.push(service);
        self
    }

    pub fn heartbeat_timeout(mut self, timeout: u64) -> Self {
        self.heartbeat_timeout = Some(timeout);
        self
    }

    pub fn retry_interval(mut self, interval: u64) -> Self {
        self.retry_interval = Some(interval);
        self
    }

    pub fn tcp(mut self, nodelay: bool, keepalive_secs: u64, keepalive_interval: u64) -> Self {
        self.transport.transport_type = TransportType::Tcp;
        self.transport.tcp = TcpConfig {
            nodelay,
            keepalive_secs,
            keepalive_interval,
            proxy: None,
        };
        self
    }

    pub fn tls(mut self, hostname: Option<String>, trusted_root: Option<String>, pkcs12: Option<String>, pkcs12_password: Option<String>) -> Self {
        self.transport.transport_type = TransportType::Tls;
        self.transport.tls = Some(TlsConfig {
            hostname,
            trusted_root,
            pkcs12,
            pkcs12_password: pkcs12_password.map(MaskedString),
        });
        self
    }

    pub fn noise(mut self, pattern: &str, local_private_key: Option<String>, remote_public_key: Option<String>) -> Self {
        self.transport.transport_type = TransportType::Noise;
        self.transport.noise = Some(NoiseConfig {
            pattern: pattern.to_string(),
            local_private_key: local_private_key.map(MaskedString),
            remote_public_key,
        });
        self
    }

    pub fn websocket(mut self, tls: bool) -> Self {
        self.transport.transport_type = TransportType::Websocket;
        self.transport.websocket = Some(WebsocketConfig { tls });
        self
    }

    pub fn build(self) -> Result<Client> {
        let remote_addr = self.remote_addr.ok_or_else(|| anyhow!("Client remote address is required"))?;
        
        let mut services_map = HashMap::new();
        for s in self.services {
            services_map.insert(s.config.name.clone(), s.config);
        }

        let heartbeat_timeout = self.heartbeat_timeout.unwrap_or(40);
        let retry_interval = self.retry_interval.unwrap_or(1);

        let mut config = ClientConfig {
            remote_addr,
            default_token: self.default_token,
            prefer_ipv6: self.prefer_ipv6,
            services: services_map,
            transport: self.transport,
            heartbeat_timeout,
            retry_interval,
        };

        crate::config::Config::validate_client_config(&mut config)?;

        Ok(Client { config })
    }
}
