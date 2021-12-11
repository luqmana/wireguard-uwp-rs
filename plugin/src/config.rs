//! Config parsing.

use boringtun::crypto::x25519::{X25519PublicKey, X25519SecretKey};
use ipnetwork::IpNetwork;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

/// A fully-parsed config
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WireGuard {
    pub interface: Interface,
    pub peer: Peer,
}

impl WireGuard {
    /// Parse the config from the given string or return an error.
    pub fn from_str(s: &str) -> Result<WireGuard, quick_xml::DeError> {
        quick_xml::de::from_str(s)
    }
}

#[serde_as]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Interface {
    #[serde_as(as = "DisplayFromStr")]
    pub private_key: X25519SecretKey,
    pub address: Vec<IpNetwork>,
}

#[serde_as]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Peer {
    #[serde_as(as = "DisplayFromStr")]
    pub public_key: X25519PublicKey,

    pub port: u16,

    #[serde(rename = "AllowedIPs")]
    pub allowed_ips: Vec<IpNetwork>,

    pub persistent_keepalive: Option<u16>,

    #[serde(default)]
    #[serde(deserialize_with = "from_base64")]
    pub preshared_key: Option<[u8; 32]>,
}

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
