#![allow(dead_code)]
//! The distributed server

use std::cell::RefCell;

use messages::*;
use types::*;
use server_traits::*;


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
    peers: Vec<ServerIdentity>,
    followers: Option<Vec<Follower>>,
    election_results: Option<Vec<ElectionResults>>,
}

pub struct Server {
    identity: ServerIdentity,
    persistent_state: RefCell<PersistentState>,
    volatile_state: RefCell<VolatileState>,
}

impl Server {

    fn next_term(&self) {
        self.persistent_state.borrow_mut().current_term += 1;
    }

    pub fn report_status(&self) -> Status {
        Status {
            term: self.persistent_state.borrow().current_term,
            role: self.volatile_state.borrow().role.clone(),
            commit_index: 0 //svr.commit_index
        }
    }
}

impl RaftServer for Server {

    fn new(identity: ServerIdentity) -> Server {

        Server {
            identity: identity,
            persistent_state: RefCell::new(
                PersistentState {
                    current_term: 1,
                    voted_for: None,
                }
            ),
            volatile_state: RefCell::new(
                VolatileState {
                    role: Role::Follower,
                    peers: Vec::new(),
                    followers: None,
                    election_results: None
                }
            )
        }
    }

    fn current_role(&self) -> Role {
        self.volatile_state.borrow().role.clone()
    }

    fn change_role(&self, new_role: Role) {
        self.volatile_state.borrow_mut().role = new_role;
    }

}

impl FollowingServer for Server {

    fn append_entries(&self, entries: &AppendEntriesPayload) -> ServerAction {
        ServerAction::Continue
    }

    fn consider_vote(&self, request_vote: &RequestVotePayload) -> ServerAction {
        //QUESTION:: should we only vote for a server in our current or next config? DoS attack?
        let mut persistent_state = self.persistent_state.borrow_mut();
        if persistent_state.voted_for == None {
            persistent_state.voted_for = Some(request_vote.candidate_id.clone());
            //QUESTION:: do we set current_term to candidates term, or wait for append entries?
        }
        ServerAction::Continue
    }

    fn become_candidate_leader(&self, dispatch: &Dispatch) -> ServerAction {
        self.next_term();

        let request_vote =
            RequestVotePayload {
                term: self.persistent_state.borrow().current_term,
                candidate_id: self.identity.clone(),
                last_log_index: 0,
                last_log_term: 0
            };

        dispatch.loopback.send(InwardMessage::RequestVote(request_vote.clone())).unwrap();
        dispatch.tx.send(OutwardMessage::RequestVote(request_vote)).unwrap();

        ServerAction::NewRole(Role::Candidate)
    }
}

impl CandidateServer for Server {

    fn consider_conceding(&self, append_entries: &AppendEntriesPayload) -> ServerAction {
        if Server::should_concede(self.persistent_state.borrow().current_term, append_entries.term) {
            self.append_entries(append_entries);
            ServerAction::NewRole(Role::Follower)
        } else {
            ServerAction::Continue
        }
    }

    fn start_new_election(&self, dispatch: &Dispatch) -> ServerAction {
        self.become_candidate_leader(dispatch)
    }
}

impl Server {

    fn should_concede(my_term: Term, other_candidates_term: Term) -> bool {
        other_candidates_term >= my_term
    }
}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::sync::mpsc::channel;
    use super::*;
    use raft::Raft;


    #[test]
    fn new_server_is_a_follower() {
        let (endpoint, dispatch, _) = Raft::get_channels();

        let server = Server::new(ServerIdentity::new());
        assert_eq!(Role::Follower, server.current_role());
    }

    #[test]
    fn change_role_to_candidate() {
        let (endpoint, dispatch, _) = Raft::get_channels();
        let server = Server::new(ServerIdentity::new());
        server.change_role(Role::Candidate);

        assert_eq!(Role::Candidate, server.current_role());

    }

    mod election {
        mod only_server {
            use super::super::super::*;
            use std::time::Duration;
            use std::thread;
            use std::sync::{Arc,Mutex};
            use raft::Raft;

            #[test]
            fn follower_becomes_a_candidate_if_it_doesnt_hear_from_a_leader() {
                let (endpoint, dispatch, loopback) = Raft::get_channels();
                let server = Server::new(ServerIdentity::new());

                let result = server.become_candidate_leader(&dispatch);
                assert_eq!(ServerAction::NewRole(Role::Candidate), result);
            }

            #[test]
            fn candidate_server_will_request_votes_from_other_servers() {
                let (endpoint, dispatch, loopback) = Raft::get_channels();
                let this_server_id = ServerIdentity::new();
                let server = Server::new(this_server_id.clone());

                server.become_candidate_leader(&dispatch);

                match endpoint.rx.recv().unwrap() {
                    OutwardMessage::RequestVote(rv) => assert_eq!(this_server_id, rv.candidate_id),
                    _ => panic!()
                };
            }

            #[test]
            fn an_election_starts_a_new_term() {
                let (endpoint, dispatch, loopback) = Raft::get_channels();
                let server = Server::new(ServerIdentity::new());

                server.become_candidate_leader(&dispatch);

                assert_eq!(2, server.persistent_state.borrow().current_term);
            }

            #[test]
            fn follower_will_vote_for_the_first_candidate_that_requests_a_vote() {
                let (endpoint, dispatch, loopback) = Raft::get_channels();
                let server = Server::new(ServerIdentity::new());
                let other_server_id = ServerIdentity::new();

                server.consider_vote(&RequestVotePayload {
                    term: 2,
                    candidate_id: other_server_id.clone(),
                    last_log_term: 1,
                    last_log_index: 0,
                });

                assert_eq!(Some(other_server_id), server.persistent_state.borrow().voted_for);
            }

            #[test]
            fn follower_will_only_vote_once() {
                let (endpoint, dispatch, loopback) = Raft::get_channels();
                let this_server_id = ServerIdentity::new();
                let server = Server::new(this_server_id.clone());
                server.persistent_state.borrow_mut().voted_for = Some(this_server_id.clone());

                server.consider_vote(&RequestVotePayload {
                    term: 2,
                    candidate_id: ServerIdentity::new(),
                    last_log_term: 1,
                    last_log_index: 0,
                });

                assert_eq!(Some(this_server_id), server.persistent_state.borrow().voted_for);
            }

            #[test]
            fn candidate_votes_for_itself() {
                let (endpoint, dispatch, loopback) = Raft::get_channels();
                let this_server_id = ServerIdentity::new();
                let server = Server::new(this_server_id.clone());

                server.become_candidate_leader(&dispatch);

                match loopback.recv().unwrap() {
                    InwardMessage::RequestVote(rv) => assert_eq!(this_server_id, rv.candidate_id),
                    _ => panic!()
                };
            }

            #[test]
            fn candidate_will_start_a_new_term_if_the_election_fails() {
                let (endpoint, dispatch, loopback) = Raft::get_channels();
                let this_server_id = ServerIdentity::new();
                let server = Server::new(this_server_id.clone());

                server.start_new_election(&dispatch);

                match loopback.recv().unwrap() {
                    InwardMessage::RequestVote(rv) => assert_eq!(this_server_id, rv.candidate_id),
                    _ => panic!()
                };
            }

            #[test]
            fn candidate_will_ignore_append_entries_from_an_out_of_date_candidate() {
                let (endpoint, dispatch, loopback) = Raft::get_channels();
                let server = Server::new(ServerIdentity::new());
                server.persistent_state.borrow_mut().current_term = 3;
                server.volatile_state.borrow_mut().role = Role::Candidate;

                let result = server.consider_conceding(&AppendEntriesPayload {
                    term: 2,
                    leader_id: ServerIdentity::new(),
                    prev_log_index: 0,
                    prev_log_term: 1,
                    entries: vec![],
                    leaders_commit: 1,
                });

                assert_eq!(ServerAction::Continue, result);
            }

            #[test]
            fn candidate_will_concede_election_if_it_gets_entries_to_append_for_the_same_term() {
                let (endpoint, dispatch, loopback) = Raft::get_channels();
                let server = Server::new(ServerIdentity::new());
                server.persistent_state.borrow_mut().current_term = 3;
                server.volatile_state.borrow_mut().role = Role::Candidate;

                let result = server.consider_conceding(&AppendEntriesPayload {
                    term: 3,
                    leader_id: ServerIdentity::new(),
                    prev_log_index: 0,
                    prev_log_term: 1,
                    entries: vec![],
                    leaders_commit: 1,
                });

                assert_eq!(ServerAction::NewRole(Role::Follower), result);
            }


            #[test]
            fn candidate_becomes_leader_on_receiving_majority_of_votes() {
                let (endpoint, dispatch, loopback) = Raft::get_channels();
                let this_server_id = ServerIdentity::new();
                let server = Server::new(this_server_id.clone());


                // let actual_status = endpoint.status.recv().unwrap();
                // assert_eq!(Role::Leader, actual_status.role)
            }
        }
    }
}
