use types::*;

pub struct ElectionTimeoutRange {
    pub minimum_milliseconds: u16,
    pub maximum_milliseconds: u16,
}


impl ElectionTimeoutRange {
    /// Used in unit testing
    pub fn testing(server_id: ServerIdentity) -> ElectionTimeoutRange {
        ElectionTimeoutRange {
            server_id: server_id,
            election_timeout_range_milliseconds: ElectionTimeoutRange { minimum_milliseconds: 1, maximum_milliseconds: 1 }
        }
    }

    /// Standard LAN timing
    pub fn lan(server_id: ServerIdentity) -> ElectionTimeoutRange {
        ElectionTimeoutRange {
            server_id: server_id,
            election_timeout_range_milliseconds: ElectionTimeoutRange { minimum_milliseconds: 150, maximum_milliseconds: 300 }
        }
    }

    /// Geographical close WAN timing
    pub fn close_wan(server_id: ServerIdentity) -> ElectionTimeoutRange {
        ElectionTimeoutRange {
            server_id: server_id,
            election_timeout_range_milliseconds: ElectionTimeoutRange { minimum_milliseconds: 250, maximum_milliseconds: 500 }
        }
    }

    /// Globalally disperse WAN timing
    pub fn global_wan(server_id: ServerIdentity) -> ElectionTimeoutRange {
        ElectionTimeoutRange {
            server_id: server_id,
            election_timeout_range_milliseconds: ElectionTimeoutRange { minimum_milliseconds: 500, maximum_milliseconds: 2500 }
        }
    }
}
