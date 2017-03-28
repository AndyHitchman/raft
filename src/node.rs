//! The distributed node

use std::sync::mpsc::{channel,Sender,Receiver};
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

        ;
        self.handle_received(Dispatch { tx: tx, rx: rx });

        Endpoint { tx: stubTx, rx: stubRx }
    }

    fn handle_received(&self, dispatch: Dispatch) {
        // let rx = self.dispatcher.rx;
        // thread::spawn(|| {
        //     let recvd = rx.recv_timeout(self.config.electionTimeout);
        // });

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
