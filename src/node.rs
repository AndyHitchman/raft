#![allow(dead_code)]
//! The distributed node

use std::sync::mpsc::{channel,Sender,Receiver,RecvTimeoutError};
use std::sync::{Arc,Mutex};
use std::thread;
use std::time::Duration;
use messages::*;
use role::*;

struct Raft {}

struct Config {
    election_timeout: Duration,
}

struct Dispatch {
    tx: Sender<OutwardMessage>,
    rx: Receiver<InwardMessage>,
    status: Sender<StatusPayload>,
}

struct Endpoint {
    tx: Sender<InwardMessage>,
    rx: Receiver<OutwardMessage>,
    status: Receiver<StatusPayload>,
}

pub struct Follower {
    id: u16,
    next_index: u64,
    match_index: u64,
}


struct Node {
    role: Role,
    config: Config,
    followers: Vec<Follower>,
    dispatch: Dispatch,
}


impl Raft {

    fn start_node(config: Config) -> Endpoint {
        let (tx, client_rx) = channel();
        let (client_tx, rx) = channel();
        let (status_tx, status_rx) = channel();

        let node = Node {
            role: Role::Follower,
            config: config,
            followers: vec![],
            dispatch: Dispatch { tx: tx, rx: rx, status: status_tx }
        };

        thread::spawn(move || node.run());

        Endpoint { tx: client_tx, rx: client_rx, status: status_rx }
    }
}

impl Node {

    fn run(mut self) {
        self.change_role(Role::Follower);

        loop {
            match self.dispatch.rx.recv_timeout(self.config.election_timeout) {
                Ok(InwardMessage::AppendEntries(ae)) => {
                    // svr.append_entries(ae);
                },
                Ok(InwardMessage::AppendEntriesResult(aer)) => {
                },
                Ok(InwardMessage::RequestVote(rv)) => {
                },
                Ok(InwardMessage::RequestVoteResult(rvr)) => {
                },
                Ok(InwardMessage::Stop) => {
                    println!("Stopping");
                    self.dispatch.tx.send(OutwardMessage::Stopped);
                    return;
                },
                Ok(InwardMessage::ReportStatus) => {
                    println!("Reporting status");
                }
                Err(RecvTimeoutError::Timeout) => {
                    self.become_candidate_leader();
                },
                Err(RecvTimeoutError::Disconnected) => {}
            }
        }
    }

    fn become_candidate_leader(&mut self) {
        self.change_role(Role::Candidate);

        self.dispatch.tx.send(OutwardMessage::RequestVote(
            RequestVotePayload {
                term: 0,
                candidate_id: 0,
                last_log_index: 0,
                last_log_term: 0
            }
        ));
    }

    fn change_role(&mut self, new_role: Role) {
        self.role = new_role;
        self.report_status();
    }

    fn report_status(&self) {
        self.dispatch.status.send(StatusPayload {
            term: 0, //svr.state.current_term,
            role: self.role.clone(),
            commit_index: 0 //svr.commit_index
        });
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

    #[test]
    fn node_starts_as_follower() {
        let endpoint = Raft::start_node(fast_config());
        let actual_status = endpoint.status.recv().unwrap();
        assert_eq!(Role::Follower, actual_status.role)
    }

    mod election {
        mod only_node {
            use super::super::super::*;
            use std::thread;
            use std::time::Duration;

            #[test]
            fn node_becomes_a_candidate_if_it_doesnt_hear_from_a_leader() {
                let endpoint = Raft::start_node(tests::fast_config());
                thread::sleep(Duration::new(0, 200));
                let actual_status = endpoint.status.try_iter().last().unwrap();
                assert_eq!(Role::Candidate, actual_status.role)
            }

            #[test]
            fn candidate_node_will_request_votes_from_other_nodes_if_no_leader_appends_entries() {
                let endpoint = Raft::start_node(tests::fast_config());
                thread::sleep(Duration::new(0, 200));

                match endpoint.rx.recv().unwrap() {
                    OutwardMessage::RequestVote(_) => (),
                    _ => panic!()
                }
            }

            #[test]
            fn candidate_becomes_leader_on_receiving_majority_of_votes() {
                let endpoint = Raft::start_node(tests::fast_config());
                //..stuff
                // assert_eq!(Role::Leader, node.role)
            }
        }
    }
}
