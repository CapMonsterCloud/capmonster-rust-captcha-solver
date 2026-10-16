//! Proxy descriptions: the proxy a *worker* uses to open the target site (`ProxyInfo`),
//! and the proxy the *SDK itself* uses to reach the CapMonster API (`ClientProxy`).

use serde::Serialize;

/// Protocol of a proxy server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Http,
    Https,
    Socks4,
    Socks5,
}

impl ProxyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::Socks4 => "socks4",
            ProxyType::Socks5 => "socks5",
        }
    }
}

/// Proxy used by CapMonster workers to reach the captcha page.
///
/// It is flattened into the task JSON as `proxyType`, `proxyAddress`, `proxyPort`,
/// `proxyLogin`, `proxyPassword`.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyInfo {
    pub proxy_type: ProxyType,
    /// IPv4/IPv6 address or hostname.
    pub proxy_address: String,
    pub proxy_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_password: Option<String>,
}

impl ProxyInfo {
    pub fn new(proxy_type: ProxyType, address: impl Into<String>, port: u16) -> Self {
        Self {
            proxy_type,
            proxy_address: address.into(),
            proxy_port: port,
            proxy_login: None,
            proxy_password: None,
        }
    }

    /// Adds login/password authentication.
    pub fn with_auth(mut self, login: impl Into<String>, password: impl Into<String>) -> Self {
        self.proxy_login = Some(login.into());
        self.proxy_password = Some(password.into());
        self
    }
}

/// Proxy through which the SDK sends its own HTTP requests to the CapMonster Cloud API.
/// Only `Http` and `Https` are supported.
#[derive(Clone, PartialEq, Eq)]
pub struct ClientProxy {
    pub proxy_type: ProxyType,
    pub address: String,
    pub port: u16,
    pub login: Option<String>,
    pub password: Option<String>,
}

impl ClientProxy {
    pub fn new(proxy_type: ProxyType, address: impl Into<String>, port: u16) -> Self {
        Self {
            proxy_type,
            address: address.into(),
            port,
            login: None,
            password: None,
        }
    }

    pub fn with_auth(mut self, login: impl Into<String>, password: impl Into<String>) -> Self {
        self.login = Some(login.into());
        self.password = Some(password.into());
        self
    }
}

fn redact(secret: &Option<String>) -> &'static str {
    if secret.is_some() {
        "Some(\"***\")"
    } else {
        "None"
    }
}

// Debug is written by hand so that passwords never end up in logs.
impl std::fmt::Debug for ProxyInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyInfo")
            .field("proxy_type", &self.proxy_type)
            .field("proxy_address", &self.proxy_address)
            .field("proxy_port", &self.proxy_port)
            .field("proxy_login", &self.proxy_login)
            .field(
                "proxy_password",
                &format_args!("{}", redact(&self.proxy_password)),
            )
            .finish()
    }
}

impl std::fmt::Debug for ClientProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientProxy")
            .field("proxy_type", &self.proxy_type)
            .field("address", &self.address)
            .field("port", &self.port)
            .field("login", &self.login)
            .field("password", &format_args!("{}", redact(&self.password)))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_does_not_leak_passwords() {
        let p = ProxyInfo::new(ProxyType::Http, "h", 1).with_auth("u", "SECRET");
        let c = ClientProxy::new(ProxyType::Http, "h", 1).with_auth("u", "SECRET");
        assert!(!format!("{p:?}").contains("SECRET"));
        assert!(!format!("{c:?}").contains("SECRET"));
    }
}
