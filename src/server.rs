#![allow(dead_code)]
//! The distributed server

use std::cell::RefCell;
use std::time::Duration;
use std::sync::{Arc,Mutex};
use std::sync::mpsc::RecvTimeoutError;
use rand;
use rand::Rng;

use messages::*;
use types::*;
use config::*;


#[derive(Clone, Debug)]
struct Follower {
    id: ServerIdentity,
    next_index: LogIndex,
    match_index: LogIndex,
}

#[derive(Clone, Debug)]
enum Vote {
    For,
    Against
}

#[derive(Clone, Debug)]
struct ElectionResults {
    id: ServerIdentity,
    voted: Vote,
}

#[derive(Clone, Debug)]
struct PersistentState {
    current_term: Term,
    voted_for: Option<ServerIdentity>,
}

#[derive(Clone, Debug)]
struct VolatileState {
    role: Role,
    followers: Option<Vec<Follower>>,
    election_results: Option<Vec<ElectionResults>>,
}

pub struct Server {
    pub config: Config,
    persistent_state: RefCell<PersistentState>,
    volatile_state: RefCell<VolatileState>,
}

trait FollowerServer {}
trait CandidateServer {}
trait LeaderServer {}

enum ServerAction {
    Continue,
    NewRole(Role),
    Stop,
}

/// The server event loop
impl Server {

    pub fn new(config: Config) -> Server {

        Server {
            config: config,
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

    pub fn run(&self, dispatch: &Dispatch) {
        self.change_role(Role::Follower);
        self.report_status(dispatch);
        let mut election_timeout = Server::new_election_timeout(&self.config.election_timeout_range_milliseconds);

        loop {
            match self.handle_event(dispatch, election_timeout) {
                ServerAction::Stop => return,
                ServerAction::NewRole(role) => {
                    election_timeout = Server::new_election_timeout(&self.config.election_timeout_range_milliseconds);
                    self.change_role(role);
                },
                ServerAction::Continue => (),
            }
            self.report_status(dispatch);
        }

    }

    fn handle_event(&self, dispatch: &Dispatch, election_timeout: Duration) -> ServerAction {
        match self.volatile_state.borrow().role {
            Role::Follower => self.handle_event_as_follower(dispatch, election_timeout),
            Role::Candidate => self.handle_event_as_candidate(dispatch, election_timeout),
            Role::Leader => ServerAction::Continue,
            Role::Disqualified => ServerAction::Stop,
        }
    }

    fn new_election_timeout(election_timeout_range: &ElectionTimeoutRange) -> Duration {
        Duration::from_millis(
            rand::thread_rng().gen_range(
                election_timeout_range.minimum_milliseconds as u64,
                election_timeout_range.maximum_milliseconds as u64 + 1
            )
        )
    }

    fn next_term(&self) {
        self.persistent_state.borrow_mut().current_term += 1;
    }

    fn change_role(&self, new_role: Role) {
        self.volatile_state.borrow_mut().role = new_role;
    }

    fn report_status(&self, dispatch: &Dispatch) {
        dispatch.status.send(
            Status {
                term: self.persistent_state.borrow().current_term,
                role: self.volatile_state.borrow().role.clone(),
                commit_index: 0 //svr.commit_index
            }
        );
    }
}

impl Server {
    fn handle_event_as_follower(&self, dispatch: &Dispatch, election_timeout: Duration) -> ServerAction {
        return match dispatch.rx.recv_timeout(election_timeout) {
            Ok(InwardMessage::AppendEntries(ref ae)) => self.append_entries(ae),
            Ok(InwardMessage::RequestVote(ref rv)) => self.consider_vote(rv),
            Ok(InwardMessage::Stop) => ServerAction::NewRole(Role::Disqualified),
            Err(RecvTimeoutError::Timeout) => self.become_candidate_leader(dispatch),
            Err(RecvTimeoutError::Disconnected) => ServerAction::Stop,
            // Ignore events that have no meaning for this role.
            _ => ServerAction::Continue
        }
    }

    fn append_entries(&self, entries: &AppendEntriesPayload) -> ServerAction {
        ServerAction::Continue
    }

    fn consider_vote(&self, request_vote: &RequestVotePayload) -> ServerAction {
        //QUESTION:: should we only vote for a server in our current or next config? DoS attack?
        let mut persistent_state = self.persistent_state.borrow_mut();
        if persistent_state.voted_for == None {
            persistent_state.voted_for = Some(request_vote.candidate_id);
            //QUESTION:: do we set current_term to candidates term, or wait for append entries?
        }
        ServerAction::Continue
    }

    fn become_candidate_leader(&self, dispatch: &Dispatch) -> ServerAction {
        self.next_term();

        let request_vote =
            RequestVotePayload {
                term: self.persistent_state.borrow().current_term,
                candidate_id: self.config.server_id,
                last_log_index: 0,
                last_log_term: 0
            };

        dispatch.loopback.send(InwardMessage::RequestVote(request_vote.clone()));
        dispatch.tx.send(OutwardMessage::RequestVote(request_vote));

        ServerAction::NewRole(Role::Candidate)
    }
}

impl Server {
    fn handle_event_as_candidate(&self, dispatch: &Dispatch, election_timeout: Duration) -> ServerAction {
        return match dispatch.rx.recv_timeout(election_timeout) {
            Ok(InwardMessage::AppendEntries(ref ae)) => self.consider_conceding(ae),
            Ok(InwardMessage::RequestVote(rv)) => ServerAction::Continue,
            Ok(InwardMessage::RequestVoteResult(rvr)) => ServerAction::Continue,
            Ok(InwardMessage::Stop) => ServerAction::NewRole(Role::Disqualified),
            Err(RecvTimeoutError::Timeout) => self.start_new_election(dispatch),
            Err(RecvTimeoutError::Disconnected) => ServerAction::Stop,
            _ => ServerAction::Continue
        }
    }

    fn consider_conceding(&self, append_entries: &AppendEntriesPayload) -> ServerAction {
        if Server::should_concede(self.persistent_state.borrow().current_term, append_entries.term) {
            self.append_entries(append_entries);
            ServerAction::NewRole(Role::Follower)
        } else {
            ServerAction::Continue
        }
    }

    fn should_concede(my_term: Term, other_candidates_term: Term) -> bool {
        other_candidates_term >= my_term
    }

    fn start_new_election(&self, dispatch: &Dispatch) -> ServerAction {
        self.become_candidate_leader(dispatch)
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
        election_timeout_sample(Config::lan(0));
    }

    #[test]
    fn new_election_timeout_is_between_250_and_500ms_for_close_wan_config() {
        election_timeout_sample(Config::close_wan(0));
    }

    #[test]
    fn new_election_timeout_is_between_500_and_2500ms_for_global_wan_config() {
        election_timeout_sample(Config::global_wan(0));
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
        let (endpoint, dispatch) = Raft::get_channels();

        let server = Server::new(Config::testing(0));
        assert_eq!(Role::Disqualified, server.volatile_state.borrow().role);
    }

    #[test]
    fn server_starts_as_follower() {
        let endpoint = Raft::start_server(Config::testing(0));
        let actual_status = endpoint.status.recv().unwrap();
        assert_eq!(Role::Follower, actual_status.role);
    }

    #[test]
    fn server_disqualifies_itself_when_stopped() {
        let endpoint = Raft::start_server(Config::testing(0));
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
            use std::sync::{Arc,Mutex};
            use raft::Raft;

            #[test]
            fn server_becomes_a_candidate_if_it_doesnt_hear_from_a_leader() {
                let endpoint = Raft::start_server(Config::testing(0));
                let initial_status = endpoint.status.recv().unwrap();
                let running_status = endpoint.status.recv().unwrap();
                assert_eq!(Role::Follower, initial_status.role);
                assert_eq!(Role::Candidate, running_status.role);
            }

            #[test]
            fn candidate_votes_for_itself() {
                let (endpoint, dispatch) = Raft::get_channels();
                let server = Server::new(Config::testing(1));

                thread::spawn(move || {
                    server.run(&dispatch)
                });

                // assert_eq!(1, server.persistent_state.borrow().voted_for.unwrap());
            }

            #[test]
            fn candidate_server_will_request_votes_from_other_servers_if_no_leader_appends_entries() {
                let endpoint = Raft::start_server(Config::testing(0));
                let initial_status = endpoint.status.recv().unwrap();
                let running_status = endpoint.status.recv().unwrap();

                match endpoint.rx.recv().unwrap() {
                    OutwardMessage::RequestVote(_) => (),
                    _ => panic!()
                }
            }

            #[test]
            fn candidate_becomes_leader_on_receiving_majority_of_votes() {
                let endpoint = Raft::start_server(Config::testing(0));
                let initial_status = endpoint.status.recv().unwrap();
                let running_status = endpoint.status.recv().unwrap();

                // let actual_status = endpoint.status.recv().unwrap();
                // assert_eq!(Role::Leader, actual_status.role)
            }
        }
    }
}
