#![allow(dead_code)]
//! The distributed node

use std::time::Duration;
use std::sync::mpsc::{channel,Sender,Receiver,RecvTimeoutError};
use rand;
use rand::ThreadRng;
use rand::distributions::{IndependentSample,Range};

use messages::*;
use role::*;
use config::*;
use raft::*;

pub struct Dispatch {
    tx: Sender<OutwardMessage>,
    rx: Receiver<InwardMessage>,
    status: Sender<Status>,
}

pub struct Follower {
    id: u16,
    next_index: u64,
    match_index: u64,
}

pub struct Node {
    pub role: Role,
    pub config: Config,
    followers: Vec<Follower>,
    dispatch: Dispatch,
}

const lower_election_timeout_delay: u64 = 150;
const upper_election_timeout_delay: u64 = 300;

impl Node {

    pub fn new(config: Config, tx: Sender<OutwardMessage>, rx: Receiver<InwardMessage>, status_tx: Sender<Status>) -> Node {
        Node {
            role: Role::Disqualified,
            config: config,
            followers: vec![],
            dispatch: Dispatch { tx: tx, rx: rx, status: status_tx }
        }
    }

    pub fn run(mut self) {
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

    fn new_election_timeout() -> Duration {
        let between = Range::new(lower_election_timeout_delay, upper_election_timeout_delay + 1);
        let mut rng = rand::thread_rng();
        Duration::from_millis(between.ind_sample(&mut rng))
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
        self.dispatch.status.send(Status {
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

    #[test]
    fn new_election_timeout_is_between_150_and_300ms() {
        let mut hit_lower = false;
        let mut hit_upper = false;

        for tries in 1..10000000 {
            let timeout = Node::new_election_timeout();
            assert!(timeout >= Duration::from_millis(lower_election_timeout_delay));
            assert!(timeout <= Duration::from_millis(upper_election_timeout_delay));

            if timeout == Duration::from_millis(lower_election_timeout_delay) { hit_lower = true; }
            if timeout == Duration::from_millis(upper_election_timeout_delay) { hit_upper = true; }
            if tries > 10000 && hit_upper && hit_lower { break; }
        }

        assert!(hit_lower);
        assert!(hit_upper);
    }

    fn fast_config() -> Config {
        Config {
            election_timeout: Duration::new(0, 100),
        }
    }

    #[test]
    fn new_node_is_disqualified_before_run() {
        let (tx, _) = channel::<OutwardMessage>();
        let (_, rx) = channel::<InwardMessage>();
        let (status, _) = channel::<Status>();

        let node = Node::new(fast_config(), tx, rx, status);
        assert_eq!(Role::Disqualified, node.role)
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
                thread::sleep(Duration::new(0, 200));
                let actual_status = endpoint.status.try_iter().last().unwrap();
                assert_eq!(Role::Leader, actual_status.role)
            }
        }
    }
}
