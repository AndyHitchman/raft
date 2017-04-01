#![allow(dead_code)]
//! The distributed node

use std::time::Duration;
use std::sync::mpsc::{Sender,Receiver,RecvTimeoutError};
use rand;
use rand::Rng;
use rand::distributions::{IndependentSample,Range};

use messages::*;
use role::*;
use config::*;

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

impl Node {

    pub fn new(config: Config, tx: Sender<OutwardMessage>, rx: Receiver<InwardMessage>, status: Sender<Status>) -> Node {
        Node {
            role: Role::Disqualified,
            config: config,
            followers: vec![],
            dispatch: Dispatch { tx: tx, rx: rx, status: status }
        }
    }

    pub fn run(mut self) {
        self.change_role(Role::Follower);
        let mut election_timeout = Node::new_election_timeout(&self.config.election_timeout_range_milliseconds);

        loop {
            match self.dispatch.rx.recv_timeout(election_timeout) {
                Ok(InwardMessage::AppendEntries(ae)) => {
                    if self.role == Role::Follower {
                        //
                    }
                },
                Ok(InwardMessage::AppendEntriesResult(aer)) => {
                },
                Ok(InwardMessage::RequestVote(rv)) => {
                },
                Ok(InwardMessage::RequestVoteResult(rvr)) => {
                },
                Ok(InwardMessage::RequestToFollow(rtf)) => {
                },
                Ok(InwardMessage::RequestToFollowResult(rtfr)) => {
                },
                Ok(InwardMessage::Stop) => {
                    assert!(self.dispatch.tx.send(OutwardMessage::Stopped).is_ok());
                    break;
                },
                Err(RecvTimeoutError::Timeout) => {
                    self.become_candidate_leader();
                    election_timeout = Node::new_election_timeout(&self.config.election_timeout_range_milliseconds);
                },
                Err(RecvTimeoutError::Disconnected) => {}
            }
        }

        self.change_role(Role::Disqualified);
    }

    fn new_election_timeout(election_timeout_range: &ElectionTimeoutRange) -> Duration {
        Duration::from_millis(
            rand::thread_rng().gen_range(
                election_timeout_range.minimum_milliseconds as u64,
                election_timeout_range.maximum_milliseconds as u64 + 1
            )
        )
    }

    fn become_candidate_leader(&mut self) {
        self.change_role(Role::Candidate);

        assert!(
            self.dispatch.tx.send(
                OutwardMessage::RequestVote(
                    RequestVotePayload {
                        term: 0,
                        candidate_id: 0,
                        last_log_index: 0,
                        last_log_term: 0
                    }
                )
            ).is_ok()
        );
    }

    fn change_role(&mut self, new_role: Role) {
        self.role = new_role;
        self.report_status();
    }

    fn report_status(&self) {
        assert!(
            self.dispatch.status.send(
                Status {
                    term: 0, //svr.state.current_term,
                    role: self.role.clone(),
                    commit_index: 0 //svr.commit_index
                }
            ).is_ok()
        );
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::sync::mpsc::channel;
    use config::*;
    use raft::Raft;
    use super::*;

    #[test]
    fn new_election_timeout_is_between_150_and_300ms_for_lan_config() {
        election_timeout_sample(Config::lan());
    }

    #[test]
    fn new_election_timeout_is_between_250_and_500ms_for_close_wan_config() {
        election_timeout_sample(Config::close_wan());
    }

    #[test]
    fn new_election_timeout_is_between_500_and_2500ms_for_global_wan_config() {
        election_timeout_sample(Config::global_wan());
    }

    fn election_timeout_sample(config: Config) {
        let lower_limit = config.election_timeout_range_milliseconds.minimum_milliseconds;
        let upper_limit = config.election_timeout_range_milliseconds.maximum_milliseconds;
        let mut hit_lower = false;
        let mut hit_upper = false;

        for tries in 1..10000000 {
            let timeout = Node::new_election_timeout(&ElectionTimeoutRange { minimum_milliseconds: lower_limit, maximum_milliseconds: upper_limit } );
            assert!(timeout >= Duration::from_millis(lower_limit as u64));
            assert!(timeout <= Duration::from_millis(upper_limit as u64));

            if timeout == Duration::from_millis(lower_limit as u64) { hit_lower = true; }
            if timeout == Duration::from_millis(upper_limit as u64) { hit_upper = true; }
            if tries > 10000 && hit_upper && hit_lower { break; }
        }

        assert!(hit_lower);
        assert!(hit_upper);
    }


    #[test]
    fn new_node_is_disqualified_before_run() {
        let (tx, _) = channel::<OutwardMessage>();
        let (_, rx) = channel::<InwardMessage>();
        let (status_tx, status_rx) = channel::<Status>();

        let node = Node::new(Config::testing(), tx, rx, status_tx);
        assert_eq!(Role::Disqualified, node.role);
    }

    #[test]
    fn node_starts_as_follower() {
        let endpoint = Raft::start_node(Config::testing());
        let actual_status = endpoint.status.recv().unwrap();
        assert_eq!(Role::Follower, actual_status.role);
    }

    #[test]
    fn node_sends_stopped_when_told_to_stop() {
        let endpoint = Raft::start_node(Config::testing());
        assert!(endpoint.tx.send(InwardMessage::Stop).is_ok());
        let received = endpoint.rx.recv().unwrap();
        assert_eq!(OutwardMessage::Stopped, received);
    }

    #[test]
    fn node_disqualifies_itself_when_stopped() {
        let endpoint = Raft::start_node(Config::testing());
        let initial_status = endpoint.status.recv().unwrap();
        assert!(endpoint.tx.send(InwardMessage::Stop).is_ok());
        let actual_status = endpoint.status.recv().unwrap();
        assert_eq!(Role::Disqualified, actual_status.role);
    }

    mod election {
        mod only_node {
            use super::super::super::*;
            use std::time::Duration;
            use std::thread;
            use raft::Raft;

            #[test]
            fn node_becomes_a_candidate_if_it_doesnt_hear_from_a_leader() {
                let endpoint = Raft::start_node(Config::testing());
                let initial_status = endpoint.status.recv().unwrap();
                let running_status = endpoint.status.recv().unwrap();
                assert_eq!(Role::Follower, initial_status.role);
                assert_eq!(Role::Candidate, running_status.role);
            }

            #[test]
            fn candidate_node_will_request_votes_from_other_nodes_if_no_leader_appends_entries() {
                let endpoint = Raft::start_node(Config::testing());
                thread::sleep(Duration::new(0, 200));

                match endpoint.rx.recv().unwrap() {
                    OutwardMessage::RequestVote(_) => (),
                    _ => panic!()
                }
            }

            #[test]
            fn candidate_becomes_leader_on_receiving_majority_of_votes() {
                let endpoint = Raft::start_node(Config::testing());
                thread::sleep(Duration::new(0, 200));
                let actual_status = endpoint.status.try_iter().last().unwrap();
                assert_eq!(Role::Leader, actual_status.role)
            }
        }
    }
}
