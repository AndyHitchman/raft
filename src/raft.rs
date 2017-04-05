//! The main entry point for a host process.

use std::thread;
use std::sync::mpsc::channel;
use messages::*;
use config::Config;
use server::Server;

pub struct Raft {}

impl Raft {

    /// Starts a server and returns the endoint used
    /// for communication.
    ///
    /// The peers should contain the known membership of the cluster.
    /// The election timeout should be a random duration between 150 and 300ms.
    pub fn start_server(config: Config) -> Endpoint {
        let (endpoint, dispatch) = Raft::get_channels();

        let server = Server::new(config);

        thread::spawn(move || server.run(&dispatch));

        endpoint
    }

    pub fn get_channels() -> (Endpoint, Dispatch) {
        let (tx, client_rx) = channel::<OutwardMessage>();
        let (client_tx, rx) = channel::<InwardMessage>();
        let (status_tx, status_rx) = channel::<Status>();

        (
            Endpoint { tx: client_tx.clone(), rx: client_rx, status: status_rx },
            Dispatch { tx: tx, rx: rx, status: status_tx, loopback: client_tx.clone() }
        )
    }
}
