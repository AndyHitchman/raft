//! The distributed node

use std::sync::mpsc::{channel,Sender,Receiver,RecvTimeoutError};
use std::thread;
use std::time::Duration;
use messages::*;
use role::*;


struct Config {
    electionTimeout: Duration,
}

struct Dispatch {
    tx: Sender<OutwardMessage>,
    rx: Receiver<InwardMessage>,
}

struct Endpoint {
    tx: Sender<InwardMessage>,
    rx: Receiver<OutwardMessage>,
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
}

impl Node {
    fn new(config: Config) -> Node {
        Node {
            role: Role::Follower,
            config: config,
            followers: vec![],
        }
    }

    fn run(&self) -> Endpoint {
        let (tx, stubRx) = channel();
        let (stubTx, rx) = channel();

        self.handle_received(tx, rx, self.config.electionTimeout);

        Endpoint { tx: stubTx, rx: stubRx }
    }

    fn handle_received(&self, tx: Sender<OutwardMessage>, rx: Receiver<InwardMessage>, waitForMessage: Duration) {
        thread::spawn(move || {
            loop {
                match rx.recv_timeout(waitForMessage) {
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
                        tx.send(OutwardMessage::Stopped);
                        return;
                    },
                    Ok(InwardMessage::ReportStatus) => {
                        println!("Reporting status");
                        // tx.send(OutwardMessage::Status(StatusPayload {
                        //     term: svr.state.current_term,
                        //     mode: svr.mode.clone(),
                        //     commit_index: svr.commit_index}));
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        tx.send(OutwardMessage::RequestVote(
                            RequestVotePayload {
                                term: 0,
                                candidate_id: 0,
                                last_log_index: 0,
                                last_log_term: 0 
                            }
                        ));
                    },
                    Err(RecvTimeoutError::Disconnected) => {}
                }
            }
        });

        ()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::sync::mpsc::channel;
    use super::*;

    fn fast_config() -> Config {
        Config {
            electionTimeout: Duration::new(0, 100),
        }
    }

    #[test]
    fn node_starts_as_follower() {
        assert_eq!(Role::Follower, Node::new(fast_config()).role)
    }

    mod election {
        mod only_node {
            use super::super::super::*;

            #[test]
            fn node_becomes_a_candidate_if_it_doesnt_hear_from_a_leader() {
                let node = Node::new(tests::fast_config());
                //..stuff
                assert_eq!(Role::Candidate, node.role)
            }

            #[test]
            fn candidate_node_will_request_votes_from_other_nodes() {
                let node = Node::new(tests::fast_config());
                let endpoint = node.run();
                //..stuff
                match endpoint.rx.recv().unwrap() {
                    OutwardMessage::RequestVote(_) => (),
                    _ => panic!()
                }
            }

            #[test]
            fn candidate_becomes_leader_on_receiving_majority_of_votes() {
                let node = Node::new(tests::fast_config());
                //..stuff
                assert_eq!(Role::Leader, node.role)
            }
        }
    }
}
