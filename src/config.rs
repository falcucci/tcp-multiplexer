use std::net::IpAddr;

use serde_derive::Deserialize;
use serde_derive::Serialize;

/// # Address Enum
///
/// Represents a network address. Currently supports TCP addresses composed of
/// an IP address and a port.
///
/// ## Variants
///
/// - `Tcp(IpAddr, u16)`: Represents a TCP address with an IP address and a port
///   number.
///
/// ## Example
///
/// ```rust
/// use std::net::IpAddr;
/// use std::net::Ipv4Addr;
/// let address = Address::Tcp(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 27_632);
/// ```
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Address {
    Tcp(IpAddr, u16),
}

mod default {
    use std::net::Ipv4Addr;

    use super::*;

    /// # Default Listen Address
    ///
    /// Provides a default TCP address for listening, which is set to
    /// `127.0.0.1:27632`.
    ///
    /// ## Returns
    ///
    /// An `Address` enum variant with the default IP and port.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let default_address = default::listen(); 
    /// ```
    pub fn listen() -> Address { Address::Tcp(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 27_632) }
}

// # Config Struct
///
/// Represents the configuration settings for the application, specifically for
/// network listening.
///
/// ## Fields
///
/// - `listen`: An `Address` specifying where the application should listen for
///   incoming connections. Defaults to `127.0.0.1:27632` if not specified.
///
/// ## Example
///
/// ```rust
/// let config = Config::default();
/// println!("{:?}", config.listen); // Outputs: Tcp(127.0.0.1:27632)
/// ```
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default::listen")]
    pub listen: Address,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            listen: default::listen(),
        }
    }
}
