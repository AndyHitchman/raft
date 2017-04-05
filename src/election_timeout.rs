use types::*;

pub struct ElectionTimeoutRange {
    pub minimum_milliseconds: u64,
    pub maximum_milliseconds: u64,
}


impl ElectionTimeoutRange {
    /// Used in unit testing
    pub fn testing() -> ElectionTimeoutRange {
        ElectionTimeoutRange { minimum_milliseconds: 1, maximum_milliseconds: 1 }
    }

    /// Standard LAN timing
    pub fn lan() -> ElectionTimeoutRange {
        ElectionTimeoutRange { minimum_milliseconds: 150, maximum_milliseconds: 300 }
    }

    /// Geographical close WAN timing
    pub fn close_wan() -> ElectionTimeoutRange {
        ElectionTimeoutRange { minimum_milliseconds: 250, maximum_milliseconds: 500 }
    }

    /// Globalally disperse WAN timing
    pub fn global_wan() -> ElectionTimeoutRange {
        ElectionTimeoutRange { minimum_milliseconds: 500, maximum_milliseconds: 2500 }
    }
}
