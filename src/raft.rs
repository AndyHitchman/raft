//! The main entry point for a host process.

use std::thread;
use std::sync::mpsc::{channel,Sender,Receiver};
use messages::*;
use config::Config;
use node::Node;

pub struct Raft {}

pub struct Endpoint {
    pub tx: Sender<InwardMessage>,
    pub rx: Receiver<OutwardMessage>,
    pub status: Receiver<Status>,
}

impl Raft {

    /// Starts a node and returns the endoint used
    /// for communication.
    ///
    /// The peers should contain the known membership of the cluster.
    /// The election timeout should be a random duration between 150 and 300ms.
    pub fn start_node(config: Config) -> Endpoint {
        let (tx, client_rx) = channel::<OutwardMessage>();
        let (client_tx, rx) = channel::<InwardMessage>();
        let (status_tx, status_rx) = channel::<Status>();

        let node = Node::new(config, tx, rx, status_tx);

        thread::spawn(move || node.run());

        Endpoint { tx: client_tx, rx: client_rx, status: status_rx }
    }
}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::sync::mpsc::channel;
    use super::*;

    fn fast_config() -> Config {
        Config {
            election_timeout: Duration::new(0, 100),
        }
    }
}