//! Config parsing.

use std::net::IpAddr;

use boringtun::crypto::x25519::{X25519PublicKey, X25519SecretKey};
use ipnetwork::IpNetwork;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

/// A fully-parsed config
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WireGuardConfig {
    /// Local interface configuration
    pub interface: InterfaceConfig,

    /// Remote peer configuration
    pub peer: PeerConfig,
}

impl WireGuardConfig {
    /// Parse the config from the given string or return an error.
    pub fn from_str(s: &str) -> Result<WireGuardConfig, quick_xml::DeError> {
        quick_xml::de::from_str(s)
    }
}

/// Local VPN interface specific configuration
#[serde_as]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InterfaceConfig {
    /// Our local private key
    #[serde_as(as = "DisplayFromStr")]
    pub private_key: X25519SecretKey,

    /// Addresses to assign to local VPN interface
    pub address: Vec<IpNetwork>,

    /// DNS servers
    #[serde(default)]
    #[serde(rename = "DNS")]
    pub dns_servers: Vec<IpAddr>,

    /// DNS Search Domains
    #[serde(default)]
    #[serde(rename = "DNSSearch")]
    pub search_domains: Vec<String>,
}

/// Remote peer specific configuration
#[serde_as]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PeerConfig {
    /// The remote endpoint's public key
    #[serde_as(as = "DisplayFromStr")]
    pub public_key: X25519PublicKey,

    /// The port the remote endpoint is listening
    pub port: u16,

    /// The list of addresses that will get routed to the remote endpoint
    #[serde(rename = "AllowedIPs")]
    pub allowed_ips: Vec<IpNetwork>,

    /// The list of addresses that won't get routed to the remote endpoint
    #[serde(default)]
    #[serde(rename = "ExcludedIPs")]
    pub excluded_ips: Vec<IpNetwork>,

    /// The interval at which to send KeepAlive packets.
    pub persistent_keepalive: Option<u16>,

    /// An optional pre-shared key to enable an additional layer of security
    #[serde(default)]
    #[serde(deserialize_with = "from_base64")]
    pub preshared_key: Option<[u8; 32]>,
}

/// Try to parse the base64 encoded pre-shared key from the config
/// into the raw bytes it represents.
fn from_base64<'de, D>(deserializer: D) -> Result<Option<[u8; 32]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    match Option::<String>::deserialize(deserializer) {
        Ok(s) => match s {
            Some(s) => match base64::decode(&s) {
                Ok(b) => match b.try_into() {
                    Ok(b) => Ok(Some(b)),
                    Err(_) => Err(Error::custom("invalid pre-shared key")),
                },
                Err(e) => Err(Error::custom(e.to_string())),
            },
            None => Ok(None),
        },
        Err(e) => Err(e),
    }
}
