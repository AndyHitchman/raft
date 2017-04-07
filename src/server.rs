#![allow(dead_code)]
//! The distributed server

use std::cell::RefCell;

use messages::*;
use types::*;
use server_traits::*;
use server_action::*;


#[derive(PartialEq, Clone, Debug)]
struct Follower {
    id: ServerIdentity,
    next_index: LogIndex,
    match_index: LogIndex,
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
    votes: u32,
}

pub struct Server {
    identity: ServerIdentity,
    persistent_state: RefCell<PersistentState>,
    volatile_state: RefCell<VolatileState>,
}

impl Server {

    pub fn new(identity: ServerIdentity, peers: Vec<ServerIdentity>) -> Server {

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
                    peers: peers,
                    followers: None,
                    votes: 0
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
        {
            let mut persistent_state = self.persistent_state.borrow_mut();
            assert!(new_term >= persistent_state.current_term);

            persistent_state.current_term = new_term;
            persistent_state.voted_for = None;
        }
        {
            let mut volatile_state = self.volatile_state.borrow_mut();
            volatile_state.followers = None;
            volatile_state.votes = 0;
        }
        self.change_role(Role::Follower);
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
            ServerAction::NewTerm
        } else {
            ServerAction::Continue
        }
    }

    fn append_entries(&self, entries: &AppendEntriesPayload) -> ServerAction {
        self.ensure_term_is_latest(entries);

        // If we are a candidate and get append entries for our current or a later term,
        // we have lost the election, so accept leaders term, revert to follower and append. Else ignore.
        if self.current_role() == Role::Candidate && self.current_term() <= entries.term() {
            self.update_term(entries.term());
        }

        if self.current_role() == Role::Follower {
            //TODO: Append entries to log
        }

        ServerAction::Continue
    }

    fn start_new_election(&self) -> ServerAction {
        self.next_term();
        self.change_role(Role::Candidate);

        let request_vote = RequestVotePayload {
            term: self.persistent_state.borrow().current_term,
            candidate_id: self.identity.clone(),
            last_log_index: 0,
            last_log_term: 0
        };

        match self.consider_vote(&request_vote) {
            ServerAction::Reply(Message::RequestVoteResult(ref rvr)) => {
                let collected_vote = self.collect_vote(rvr);
                if let ServerAction::Broadcast(Message::AppendEntries(_)) = collected_vote {
                    // The degenerate case where there is only one server (this one) and no peers.
                    // But this is allowable.
                    collected_vote
                } else {
                    // Where we have peers, ask for votes.
                    ServerAction::Broadcast(Message::RequestVote(request_vote))
                }
            },
            _ => unreachable!()
        }
    }

    fn consider_vote(&self, request_vote: &RequestVotePayload) -> ServerAction {
        self.ensure_term_is_latest(request_vote);

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

    fn collect_vote(&self, request_vote_result: &RequestVoteResultPayload) -> ServerAction {
        self.ensure_term_is_latest(request_vote_result);

        if self.current_role() == Role::Candidate {
            let has_more_than_half_the_votes: bool;

            {
                let mut volatile_state = self.volatile_state.borrow_mut();

                if request_vote_result.vote_granted {
                    volatile_state.votes += 1;
                }

                // Count self and peers.
                has_more_than_half_the_votes =
                    volatile_state.votes * 2 > volatile_state.peers.len() as u32 + 1;
            }

            if has_more_than_half_the_votes {
                self.change_role(Role::Leader);

                return ServerAction::Broadcast(Message::AppendEntries(
                    AppendEntriesPayload {
                        term: self.current_term(),
                        leader_id: self.identity.clone(),
                        prev_log_index: 0,
                        prev_log_term: 0,
                        entries: Vec::new(),
                        leaders_commit: 0,
                    }
                ))
            }
        }

        ServerAction::Continue
    }

    fn election_timeout_occurred(&self) -> ServerAction {

        ServerAction::Continue
    }
}


#[cfg(test)]
#[path = "./server_tests.rs"]
mod tests;
