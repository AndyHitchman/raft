//! The main entry point for a host process.

use std::thread;
use std::sync::mpsc::{channel,Receiver,RecvTimeoutError};

use types::{ServerIdentity,Role};
use election_timeout::ElectionTimeoutRange;
use messages::*;
use server_action::*;
use server::Server;


pub trait RaftServer {
    fn get_identity(&self) -> ServerIdentity;
    fn current_role(&self) -> Role;

    fn broadcast_heartbeat(&self) -> ServerAction;
    fn broadcast_log_changes(&self) -> ServerAction;
    fn append_entries(&self, &AppendEntriesPayload) -> ServerAction;
    fn append_entries_result(&self, &AppendEntriesResultPayload) -> ServerAction;

    fn start_new_election(&self) -> ServerAction;
    fn election_timeout_occurred(&self) -> ServerAction;
    fn consider_vote_request(&self, &RequestVotePayload) -> ServerAction;
    fn collect_vote(&self, &RequestVoteResultPayload) -> ServerAction;
}


pub struct Raft {}

impl Raft {

    /// Starts a server and returns the endoint used
    /// for communication.
    ///
    /// The peers should contain the known membership of the cluster.
    /// The election timeout should be a random duration between 150 and 300ms.
    pub fn start_server(identity: ServerIdentity, election_timeout_range: ElectionTimeoutRange) -> Endpoint {
        let (endpoint, dispatch) = Raft::get_channels();

        thread::spawn(move || {
            let server = Server::new(identity, Vec::new());
            Raft::run(&server, &dispatch, election_timeout_range)
        });

        endpoint
    }

    /// Get channels for communication
    ///
    /// Endpoint is given to the host
    /// Dispatch is given to the server
    /// The inward message receiver is retained by Raft
    pub fn get_channels() -> (Endpoint, Dispatch) {
        let (local_tx, remote_rx) = channel::<Envelope>();
        let (remote_tx, local_rx) = channel::<Envelope>();

        (
            Endpoint { tx: remote_tx, rx: remote_rx },
            Dispatch { tx: local_tx, rx: local_rx },
        )
    }


    pub fn run(server: &RaftServer, dispatch: &Dispatch, election_timeout_range: ElectionTimeoutRange) {
        let mut timeout = election_timeout_range.new_timeout();

        loop {
            let received = dispatch.rx.recv_timeout(timeout);
            //TODO: discard messages not from a server in our current or next config? DoS attack?

            let next_action = match received {
                Ok(ref envelope) => {
                    match envelope.message {
                        Message::AppendEntries(ref ae) => server.append_entries(ae),
                        Message::AppendEntriesResult(ref aer) => server.append_entries_result(aer),
                        Message::RequestVote(ref rv) => server.consider_vote_request(rv),
                        Message::RequestVoteResult(ref rvr) => server.collect_vote(rvr),
                        Message::Stop => {
                            //TODO: Shutdown server and flush.
                            ServerAction::Stop
                        }
                    }
                },
                Err(RecvTimeoutError::Timeout) => {
                    match server.current_role() {
                        Role::Leader => server.broadcast_heartbeat(),
                        Role::Follower | Role::Candidate => server.start_new_election(),
                    }
                }
                Err(RecvTimeoutError::Disconnected) => ServerAction::Stop,
            };

            match next_action {
                ServerAction::NewTerm => {
                    timeout = match server.current_role() {
                        Role::Leader => election_timeout_range.leader_heartbeat(),
                        _ => election_timeout_range.new_timeout(),
                    };
                },
                ServerAction::Broadcast(message) => {
                    dispatch.tx.send(
                        Envelope {
                            from: server.get_identity(),
                            to: Addressee::Broadcast,
                            message: message,
                        }
                    ).unwrap();
                },
                ServerAction::Reply(message) => {
                    dispatch.tx.send(
                        Envelope {
                            from: server.get_identity(),
                            to: Addressee::SingleServer(received.unwrap().from.clone()),
                            message: message,
                        }
                    ).unwrap();
                },
                ServerAction::Stop => {
                    return;
                },
                ServerAction::Continue => {},
            }
        }
    }
}


#[cfg(test)]
#[path = "./raft_tests.rs"]
mod tests;
