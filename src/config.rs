use types::*;

pub struct ElectionTimeoutRange {
    pub minimum_milliseconds: u16,
    pub maximum_milliseconds: u16,
}

/// The configuration given to a node.
pub struct Config {
    pub server_id: ServerIdentity,
    pub election_timeout_range_milliseconds: ElectionTimeoutRange,
}

impl Config {
    /// Used in unit testing
    pub fn testing(server_id: ServerIdentity) -> Config {
        Config {
            server_id: server_id,
            election_timeout_range_milliseconds: ElectionTimeoutRange { minimum_milliseconds: 1, maximum_milliseconds: 1 }
        }
    }

    /// Standard LAN timing
    pub fn lan(server_id: ServerIdentity) -> Config {
        Config {
            server_id: server_id,
            election_timeout_range_milliseconds: ElectionTimeoutRange { minimum_milliseconds: 150, maximum_milliseconds: 300 }
        }
    }

    /// Geographical close WAN timing
    pub fn close_wan(server_id: ServerIdentity) -> Config {
        Config {
            server_id: server_id,
            election_timeout_range_milliseconds: ElectionTimeoutRange { minimum_milliseconds: 250, maximum_milliseconds: 500 }
        }
    }

    /// Globalally disperse WAN timing
    pub fn global_wan(server_id: ServerIdentity) -> Config {
        Config {
            server_id: server_id,
            election_timeout_range_milliseconds: ElectionTimeoutRange { minimum_milliseconds: 500, maximum_milliseconds: 2500 }
        }
    }
}
