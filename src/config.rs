use std::time::Duration;

/// The configuration given to a node.
pub struct Config {
    pub election_timeout: Duration,
}
