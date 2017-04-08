//! A set of election timeouts designed for different network topologies.
use std::time::Duration;
use rand;
use rand::Rng;


pub struct ElectionTimeoutRange {
    minimum_milliseconds: u64,
    maximum_milliseconds: u64,
    heartbeat_interval_milliseconds: u64,
}


impl ElectionTimeoutRange {

    pub fn new_timeout(&self) -> Duration {
        Duration::from_millis(
            rand::thread_rng().gen_range(
                self.minimum_milliseconds as u64,
                self.maximum_milliseconds as u64 + 1
            )
        )
    }

    pub fn leader_heartbeat(&self) -> Duration {
        Duration::from_millis(self.heartbeat_interval_milliseconds)
    }

    /// Used in unit testing
    pub fn testing() -> ElectionTimeoutRange {
        ElectionTimeoutRange {
            minimum_milliseconds: 1,
            maximum_milliseconds: 1,
            heartbeat_interval_milliseconds: 1,
        }
    }

    /// Standard LAN timing
    pub fn lan() -> ElectionTimeoutRange {
        ElectionTimeoutRange {
            minimum_milliseconds: 150,
            maximum_milliseconds: 300,
            heartbeat_interval_milliseconds: 50,
        }
    }

    /// Geographical close WAN timing
    pub fn close_wan() -> ElectionTimeoutRange {
        ElectionTimeoutRange {
            minimum_milliseconds: 250,
            maximum_milliseconds: 500,
            heartbeat_interval_milliseconds: 50,
        }
    }

    /// Globalally disperse WAN timing
    pub fn global_wan() -> ElectionTimeoutRange {
        ElectionTimeoutRange {
            minimum_milliseconds: 500,
            maximum_milliseconds: 2500,
            heartbeat_interval_milliseconds: 50,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_election_timeout_is_between_150_and_300ms_for_lan_config() {
        election_timeout_sample(&ElectionTimeoutRange::lan());
    }

    #[test]
    fn new_election_timeout_is_between_250_and_500ms_for_close_wan_config() {
        election_timeout_sample(&ElectionTimeoutRange::close_wan());
    }

    #[test]
    fn new_election_timeout_is_between_500_and_2500ms_for_global_wan_config() {
        election_timeout_sample(&ElectionTimeoutRange::global_wan());
    }

    fn election_timeout_sample(election_timeout_range: &ElectionTimeoutRange) {
        let mut hit_lower = false;
        let mut hit_upper = false;

        for tries in 1..10000000 {
            let timeout = election_timeout_range.new_timeout();
            assert!(timeout >= Duration::from_millis(election_timeout_range.minimum_milliseconds));
            assert!(timeout <= Duration::from_millis(election_timeout_range.maximum_milliseconds));

            if timeout == Duration::from_millis(election_timeout_range.minimum_milliseconds) { hit_lower = true; }
            if timeout == Duration::from_millis(election_timeout_range.maximum_milliseconds) { hit_upper = true; }
            if tries > 10000 && hit_upper && hit_lower { break; }
        }

        assert!(hit_lower);
        assert!(hit_upper);
    }

}
