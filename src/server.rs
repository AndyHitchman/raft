#![allow(dead_code)]
//! The distributed server

use std::cell::RefCell;

use messages::*;
use types::*;
use server_traits::*;
use server_action::*;


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

    pub fn new(identity: ServerIdentity) -> Server {

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

    fn next_term(&self) {
        self.update_term(self.current_term() + 1);
    }

    fn current_term(&self) -> Term {
        self.persistent_state.borrow().current_term
    }

    fn update_term(&self, new_term: Term) {
        let mut persistent_state = self.persistent_state.borrow_mut();
        persistent_state.current_term = new_term;
        persistent_state.voted_for = None;
    }

    // pub fn report_status(&self) -> Status {
    //     Status {
    //         term: self.current_term(),
    //         role: self.current_role(),
    //         commit_index: 0 //svr.commit_index
    //     }
    // }

    fn i_am_out_of_date(my_term: Term, other_candidates_term: Term) -> bool {
        other_candidates_term > my_term
    }

    fn assert_term_is_current(&self, payload: &Payload) {
        assert!(!Server::i_am_out_of_date(self.current_term(), payload.term()));
    }

    fn change_role(&self, new_role: Role) {
        self.volatile_state.borrow_mut().role = new_role;
    }
}

impl RaftServer for Server {

    fn current_role(&self) -> Role {
        self.volatile_state.borrow().role.clone()
    }

    fn ensure_term_is_latest(&self, payload: &Payload) -> ServerAction {
        if Server::i_am_out_of_date(self.current_term(), payload.term()) {
            self.update_term(payload.term());
            self.change_role(Role::Follower);
            ServerAction::NewTerm
        } else {
            ServerAction::Continue
        }
    }

    fn append_entries(&self, entries: &AppendEntriesPayload) -> ServerAction {
        self.ensure_term_is_latest(entries);

        // If we are a candidate and get append entries for our current or a later term,
        // we have lost the election, so revert to follower and append. Else ignore.
        if self.current_role() == Role::Candidate && self.current_term() <= entries.term() {
            self.update_term(entries.term());
            self.change_role(Role::Follower);
        }

        if self.current_role() == Role::Follower {
            //TODO: Append entries to log
        }

        ServerAction::Continue
    }

    fn consider_vote(&self, request_vote: &RequestVotePayload) -> ServerAction {
        self.ensure_term_is_latest(request_vote);

        //QUESTION:: should we only vote for a server in our current or next config? DoS attack?
        let mut payload = RequestVoteResultPayload {
            term: self.current_term(),
            vote_granted: false,
        };

        let mut persistent_state = self.persistent_state.borrow_mut();
        if persistent_state.voted_for == None {
            persistent_state.voted_for = Some(request_vote.candidate_id.clone());
            payload.vote_granted = true;
        }

        ServerAction::Reply(Message::RequestVoteResult(payload))
    }

    fn start_new_election(&self) -> ServerAction {
        self.next_term();
        self.change_role(Role::Candidate);

        let request_vote =
            RequestVotePayload {
                term: self.persistent_state.borrow().current_term,
                candidate_id: self.identity.clone(),
                last_log_index: 0,
                last_log_term: 0
            };

        self.consider_vote(&request_vote);
        ServerAction::Broadcast(Message::RequestVote(request_vote))
    }

    fn collect_vote(&self, request_vote_result: &RequestVoteResultPayload) -> ServerAction {

        ServerAction::Continue
    }
}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::sync::mpsc::channel;
    use super::*;
    use raft::Raft;
    use types::*;


    #[test]
    fn new_server_is_a_follower() {
        let (endpoint, dispatch) = Raft::get_channels();

        let server = Server::new(ServerIdentity::new());
        assert_eq!(Role::Follower, server.current_role());
    }

    #[test]
    fn change_role_to_candidate() {
        let (endpoint, dispatch) = Raft::get_channels();
        let server = Server::new(ServerIdentity::new());
        server.change_role(Role::Candidate);

        assert_eq!(Role::Candidate, server.current_role());

    }

    mod election {
        use super::super::*;
        use std::time::Duration;
        use std::thread;
        use std::sync::{Arc,Mutex};
        use raft::Raft;
        use types::*;

        #[test]
        fn follow_will_start_election_if_no_leader_appends_entires() {
            let (endpoint, dispatch) = Raft::get_channels();
            let this_server_id = ServerIdentity::new();
            let server = Server::new(this_server_id.clone());

            let result = server.start_new_election();

            if let ServerAction::Broadcast(Message::RequestVote(rv)) = result {
                assert_eq!(this_server_id, rv.candidate_id);
            } else {
                panic!();
            };
        }

        #[test]
        fn an_election_starts_a_new_term() {
            let (endpoint, dispatch) = Raft::get_channels();
            let server = Server::new(ServerIdentity::new());
            server.update_term(1);

            let result = server.start_new_election();

            if let ServerAction::Broadcast(Message::RequestVote(rv)) = result {
                assert_eq!(2, rv.term());
            } else {
                panic!();
            };
        }

        #[test]
        fn follower_will_vote_for_the_first_candidate_that_requests_a_vote() {
            let (endpoint, dispatch) = Raft::get_channels();
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
        fn follower_will_revote_if_there_is_a_new_election() {
            let (endpoint, dispatch) = Raft::get_channels();
            let server = Server::new(ServerIdentity::new());
            let old_candidate_id = ServerIdentity::new();
            let new_candidate_id = ServerIdentity::new();
            {
                let mut persistent_state = server.persistent_state.borrow_mut();
                persistent_state.current_term = 3;
                persistent_state.voted_for = Some(old_candidate_id);
            }

            server.consider_vote(&RequestVotePayload {
                term: 4,
                candidate_id: new_candidate_id.clone(),
                last_log_term: 1,
                last_log_index: 0,
            });

            assert_eq!(4, server.current_term());
            assert_eq!(Some(new_candidate_id), server.persistent_state.borrow().voted_for);
        }

        #[test]
        fn follower_will_only_vote_once() {
            let (endpoint, dispatch) = Raft::get_channels();
            let this_server_id = ServerIdentity::new();
            let candidate_id = ServerIdentity::new();
            let another_candidate_id = ServerIdentity::new();
            let server = Server::new(this_server_id);
            server.persistent_state.borrow_mut().voted_for = Some(another_candidate_id.clone());

            server.consider_vote(&RequestVotePayload {
                term: 1,
                candidate_id: candidate_id,
                last_log_term: 1,
                last_log_index: 0,
            });

            assert_eq!(Some(another_candidate_id), server.persistent_state.borrow().voted_for);
        }

        #[test]
        fn follower_will_tell_candidate_it_has_got_their_vote() {
            let (endpoint, dispatch) = Raft::get_channels();
            let this_server_id = ServerIdentity::new();
            let candidate_id = ServerIdentity::new();
            let server = Server::new(this_server_id.clone());

            let result = server.consider_vote(&RequestVotePayload {
                term: 2,
                candidate_id: candidate_id.clone(),
                last_log_term: 1,
                last_log_index: 0,
            });

            if let ServerAction::Reply(Message::RequestVoteResult(rvr)) = result {
                assert_eq!(2, rvr.term());
                assert!(rvr.vote_granted);
            } else {
                panic!();
            };
        }

        #[test]
        fn candidate_votes_for_itself() {
            let (endpoint, dispatch) = Raft::get_channels();
            let this_server_id = ServerIdentity::new();
            let server = Server::new(this_server_id.clone());

            server.start_new_election();

            assert_eq!(Some(this_server_id), server.persistent_state.borrow().voted_for);
        }

        #[test]
        fn candidate_will_start_a_new_term_if_the_election_fails() {
            let (endpoint, dispatch) = Raft::get_channels();
            let this_server_id = ServerIdentity::new();
            let server = Server::new(this_server_id.clone());

            server.start_new_election();

            panic!();
            // match loopback.recv().unwrap() {
            //     InwardMessage::RequestVote(rv) => assert_eq!(this_server_id, rv.candidate_id),
            //     _ => panic!()
            // };
        }

        #[test]
        fn candidate_will_ignore_append_entries_from_an_out_of_date_candidate() {
            let (endpoint, dispatch) = Raft::get_channels();
            let server = Server::new(ServerIdentity::new());
            server.persistent_state.borrow_mut().current_term = 3;
            server.volatile_state.borrow_mut().role = Role::Candidate;

            let result = server.append_entries(&AppendEntriesPayload {
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
        fn candidate_will_return_to_a_follower_and_vote_for_a_new_candidate_with_a_later_term() {
            let (endpoint, dispatch) = Raft::get_channels();
            let this_server_id = ServerIdentity::new();
            let server = Server::new(this_server_id.clone());
            let new_candidate_id = ServerIdentity::new();
            {
                server.volatile_state.borrow_mut().role = Role::Candidate;
                let mut persistent_state = server.persistent_state.borrow_mut();
                persistent_state.current_term = 3;
                persistent_state.voted_for = Some(this_server_id);
            }

            let rv = &RequestVotePayload {
                term: 4,
                candidate_id: new_candidate_id.clone(),
                last_log_term: 1,
                last_log_index: 0,
            };

            server.ensure_term_is_latest(rv);
            server.consider_vote(rv);

            assert_eq!(4, server.persistent_state.borrow().current_term);
            assert_eq!(Some(new_candidate_id), server.persistent_state.borrow().voted_for);
            assert_eq!(Role::Follower, server.volatile_state.borrow().role);
        }

        #[test]
        fn candidate_will_concede_election_if_it_gets_entries_to_append_for_the_same_term() {
            let (endpoint, dispatch) = Raft::get_channels();
            let server = Server::new(ServerIdentity::new());
            server.persistent_state.borrow_mut().current_term = 3;
            server.volatile_state.borrow_mut().role = Role::Candidate;

            let result = server.append_entries(&AppendEntriesPayload {
                term: 3,
                leader_id: ServerIdentity::new(),
                prev_log_index: 0,
                prev_log_term: 1,
                entries: vec![],
                leaders_commit: 1,
            });

            assert_eq!(ServerAction::Continue, result);
            assert_eq!(3, server.current_term());
            assert_eq!(None, server.persistent_state.borrow().voted_for);
            assert_eq!(Role::Follower, server.current_role());
        }

        #[test]
        fn candidate_will_concede_election_if_it_gets_entries_to_append_for_a_later_term() {
            let (endpoint, dispatch) = Raft::get_channels();
            let server = Server::new(ServerIdentity::new());
            server.persistent_state.borrow_mut().current_term = 3;
            server.volatile_state.borrow_mut().role = Role::Candidate;

            let rv = &AppendEntriesPayload {
                term: 4,
                leader_id: ServerIdentity::new(),
                prev_log_index: 0,
                prev_log_term: 1,
                entries: vec![],
                leaders_commit: 1,
            };

            let result = server.ensure_term_is_latest(rv);

            assert_eq!(ServerAction::NewTerm, result);
            assert_eq!(4, server.current_term());
            assert_eq!(None, server.persistent_state.borrow().voted_for);
            assert_eq!(Role::Follower, server.current_role());
        }


        #[test]
        fn candidate_becomes_leader_on_receiving_majority_of_votes() {
            let (endpoint, dispatch) = Raft::get_channels();
            let this_server_id = ServerIdentity::new();
            let server = Server::new(this_server_id.clone());

            panic!()
            // let actual_status = endpoint.status.recv().unwrap();
            // assert_eq!(Role::Leader, actual_status.role)
        }
    }
}
