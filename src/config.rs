
pub struct ElectionTimeoutRange {
    pub minimum_milliseconds: u16,
    pub maximum_milliseconds: u16,
}

/// The configuration given to a node.
pub struct Config {
    pub election_timeout_range_milliseconds: ElectionTimeoutRange,
}

impl Config {
    /// Used in unit testing
    pub fn testing() -> Config {
        Config { election_timeout_range_milliseconds: ElectionTimeoutRange { minimum_milliseconds: 1, maximum_milliseconds: 1 } }
    }

    /// Standard LAN timing
    pub fn lan() -> Config {
        Config { election_timeout_range_milliseconds: ElectionTimeoutRange { minimum_milliseconds: 150, maximum_milliseconds: 300 } }
    }

    /// Standard LAN timing
    pub fn close_wan() -> Config {
        Config { election_timeout_range_milliseconds: ElectionTimeoutRange { minimum_milliseconds: 250, maximum_milliseconds: 500 } }
    }

    /// Standard LAN timing
    pub fn global_wan() -> Config {
        Config { election_timeout_range_milliseconds: ElectionTimeoutRange { minimum_milliseconds: 500, maximum_milliseconds: 2500 } }
    }
}
