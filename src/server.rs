#![allow(dead_code)]
//! The distributed server

use std::cell::RefCell;
use std::time::Duration;
use std::sync::mpsc::{Sender,Receiver,RecvTimeoutError};
use rand;
use rand::Rng;

use messages::*;
use types::*;
use config::*;


pub struct Dispatch {
    tx: Sender<OutwardMessage>,
    rx: Receiver<InwardMessage>,
    loopback: Sender<InwardMessage>,
    status: Sender<Status>,
}

struct Follower {
    id: ServerIdentity,
    next_index: LogIndex,
    match_index: LogIndex,
}

enum Vote {
    For,
    Against
}

struct ElectionResults {
    id: ServerIdentity,
    voted: Vote,
}

struct PersistentState {
    current_term: Term,
    voted_for: Option<ServerIdentity>,
}

struct VolatileState {
    role: Role,
    followers: Option<Vec<Follower>>,
    election_results: Option<Vec<ElectionResults>>,
}

pub struct Server {
    pub config: Config,
    dispatch: Dispatch,
    persistent_state: RefCell<PersistentState>,
    volatile_state: RefCell<VolatileState>,
}

impl Server {

    pub fn new(config: Config, tx: Sender<OutwardMessage>,
               rx: Receiver<InwardMessage>,
               loopback: Sender<InwardMessage>,
               status: Sender<Status>) -> Server {

        Server {
            config: config,
            dispatch: Dispatch { tx: tx, rx: rx, loopback: loopback, status: status },
            persistent_state: RefCell::new(
                PersistentState {
                    current_term: 0,
                    voted_for: None,
                }
            ),
            volatile_state: RefCell::new(
                VolatileState {
                    role: Role::Disqualified,
                    followers: None,
                    election_results: None
                }
            )
        }
    }

    pub fn run(self) {
        self.change_role(Role::Follower);
        let mut election_timeout = Server::new_election_timeout(&self.config.election_timeout_range_milliseconds);

        loop {
            match self.dispatch.rx.recv_timeout(election_timeout) {
                Ok(InwardMessage::AppendEntries(ae)) => {
                    if self.volatile_state.borrow().role == Role::Follower {
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
                    election_timeout = Server::new_election_timeout(&self.config.election_timeout_range_milliseconds);
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

    fn become_candidate_leader(&self) {
        self.change_role(Role::Candidate);

        self.dispatch.tx.send(
            OutwardMessage::RequestVote(
                RequestVotePayload {
                    term: 0,
                    candidate_id: 0,
                    last_log_index: 0,
                    last_log_term: 0
                }
            )
        );

        self.vote_for_self_as_leader();
    }

    fn vote_for_self_as_leader(&self) {

    }

    fn change_role(&self, new_role: Role) {
        self.volatile_state.borrow_mut().role = new_role;
        self.report_status();
    }

    fn report_status(&self) {
        self.dispatch.status.send(
            Status {
                term: 0, //svr.state.current_term,
                role: self.volatile_state.borrow().role.clone(),
                commit_index: 0 //svr.commit_index
            }
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
            let timeout = Server::new_election_timeout(&ElectionTimeoutRange { minimum_milliseconds: lower_limit, maximum_milliseconds: upper_limit } );
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
    fn new_server_is_disqualified_before_run() {
        let (tx, _) = channel::<OutwardMessage>();
        let (loopback, rx) = channel::<InwardMessage>();
        let (status_tx, status_rx) = channel::<Status>();

        let server = Server::new(Config::testing(), tx, rx, loopback, status_tx);
        assert_eq!(Role::Disqualified, server.volatile_state.borrow().role);
    }

    #[test]
    fn server_starts_as_follower() {
        let endpoint = Raft::start_server(Config::testing());
        let actual_status = endpoint.status.recv().unwrap();
        assert_eq!(Role::Follower, actual_status.role);
    }

    #[test]
    fn server_sends_stopped_when_told_to_stop() {
        let endpoint = Raft::start_server(Config::testing());
        assert!(endpoint.tx.send(InwardMessage::Stop).is_ok());
        let received = endpoint.rx.recv().unwrap();
        assert_eq!(OutwardMessage::Stopped, received);
    }

    #[test]
    fn server_disqualifies_itself_when_stopped() {
        let endpoint = Raft::start_server(Config::testing());
        let initial_status = endpoint.status.recv().unwrap();
        assert!(endpoint.tx.send(InwardMessage::Stop).is_ok());
        let actual_status = endpoint.status.recv().unwrap();
        assert_eq!(Role::Disqualified, actual_status.role);
    }

    mod election {
        mod only_server {
            use super::super::super::*;
            use std::time::Duration;
            use std::thread;
            use raft::Raft;

            #[test]
            fn server_becomes_a_candidate_if_it_doesnt_hear_from_a_leader() {
                let endpoint = Raft::start_server(Config::testing());
                let initial_status = endpoint.status.recv().unwrap();
                let running_status = endpoint.status.recv().unwrap();
                assert_eq!(Role::Follower, initial_status.role);
                assert_eq!(Role::Candidate, running_status.role);
            }

            #[test]
            fn candidate_server_will_request_votes_from_other_servers_if_no_leader_appends_entries() {
                let endpoint = Raft::start_server(Config::testing());
                let initial_status = endpoint.status.recv().unwrap();
                let running_status = endpoint.status.recv().unwrap();

                match endpoint.rx.recv().unwrap() {
                    OutwardMessage::RequestVote(_) => (),
                    _ => panic!()
                }
            }

            #[test]
            fn candidate_becomes_leader_on_receiving_majority_of_votes() {
                let endpoint = Raft::start_server(Config::testing());
                let initial_status = endpoint.status.recv().unwrap();
                let running_status = endpoint.status.recv().unwrap();

                // let actual_status = endpoint.status.recv().unwrap();
                // assert_eq!(Role::Leader, actual_status.role)
            }
        }
    }
}
