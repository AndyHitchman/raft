use super::*;


#[test]
fn new_server_is_a_follower() {
    let server = Server::new(ServerIdentity::new(), Vec::new());

    assert_eq!(Role::Follower, server.current_role());
}

#[test]
fn change_role_to_candidate() {
    let server = Server::new(ServerIdentity::new(), Vec::new());

    server.change_role(Role::Candidate);

    assert_eq!(Role::Candidate, server.current_role());
}

mod election {
    use super::super::*;

    #[test]
    fn only_server_will_start_election_if_no_leader_appends_entries_and_elect_itself() {
        let this_server_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), Vec::new());

        let result = server.start_new_election();

        if let ServerAction::Broadcast(Message::AppendEntries(ae)) = result {
            assert_eq!(this_server_id, ae.leader_id);
        } else {
            panic!();
        };
    }

    #[test]
    fn follower_will_start_election_if_no_leader_appends_entries() {
        let this_server_id = ServerIdentity::new();
        //TODO: Add peers
        let server = Server::new(this_server_id.clone(), Vec::new());

        let result = server.start_new_election();

        if let ServerAction::Broadcast(Message::RequestVote(rv)) = result {
            assert_eq!(this_server_id, rv.candidate_id);
        } else {
            panic!();
        };
    }

    #[test]
    fn an_election_starts_a_new_term() {
        let server = Server::new(ServerIdentity::new(), Vec::new());
        server.update_term(1);

        let result = server.start_new_election();

        // No peers means self is elected leader immediately.
        if let ServerAction::Broadcast(Message::AppendEntries(ae)) = result {
            assert_eq!(2, ae.term());
        } else {
            panic!();
        };
    }

    #[test]
    fn follower_will_vote_for_the_first_candidate_that_requests_a_vote() {
        let server = Server::new(ServerIdentity::new(), Vec::new());
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
        let server = Server::new(ServerIdentity::new(), Vec::new());
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
        let this_server_id = ServerIdentity::new();
        let candidate_id = ServerIdentity::new();
        let another_candidate_id = ServerIdentity::new();
        let server = Server::new(this_server_id, Vec::new());
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
        let this_server_id = ServerIdentity::new();
        let candidate_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), Vec::new());

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
        let this_server_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), Vec::new());

        server.start_new_election();

        assert_eq!(Some(this_server_id), server.persistent_state.borrow().voted_for);
    }

    #[test]
    fn candidate_will_ignore_append_entries_from_an_out_of_date_candidate() {
        let server = Server::new(ServerIdentity::new(), Vec::new());
        server.update_term(3);
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
        let this_server_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), Vec::new());
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

        server.consider_vote(rv);

        assert_eq!(4, server.persistent_state.borrow().current_term);
        assert_eq!(Some(new_candidate_id), server.persistent_state.borrow().voted_for);
        assert_eq!(Role::Follower, server.current_role());
    }

    #[test]
    fn candidate_will_continue_as_a_candiate_when_a_competing_candidate_in_the_same_term_requests_a_vote() {
        let this_server_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), Vec::new());
        let new_candidate_id = ServerIdentity::new();
        {
            server.volatile_state.borrow_mut().role = Role::Candidate;
            let mut persistent_state = server.persistent_state.borrow_mut();
            persistent_state.current_term = 4;
            persistent_state.voted_for = Some(this_server_id.clone());
        }

        let rv = &RequestVotePayload {
            term: 4,
            candidate_id: new_candidate_id.clone(),
            last_log_term: 1,
            last_log_index: 0,
        };

        server.consider_vote(rv);

        assert_eq!(4, server.persistent_state.borrow().current_term);
        assert_eq!(Some(this_server_id), server.persistent_state.borrow().voted_for);
        assert_eq!(Role::Candidate, server.current_role());
    }

    #[test]
    fn candidate_will_concede_election_if_it_gets_entries_to_append_for_the_same_term() {
        let server = Server::new(ServerIdentity::new(), Vec::new());
        server.update_term(3);
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
        let server = Server::new(ServerIdentity::new(), Vec::new());
        server.update_term(3);
        server.volatile_state.borrow_mut().role = Role::Candidate;

        let ae = &AppendEntriesPayload {
            term: 4,
            leader_id: ServerIdentity::new(),
            prev_log_index: 0,
            prev_log_term: 1,
            entries: vec![],
            leaders_commit: 1,
        };

        let result = server.append_entries(ae);

        assert_eq!(ServerAction::Continue, result);
        assert_eq!(4, server.current_term());
        assert_eq!(None, server.persistent_state.borrow().voted_for);
        assert_eq!(Role::Follower, server.current_role());
    }


    #[test]
    fn candidate_becomes_leader_on_receiving_majority_of_votes() {
        let this_server_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), Vec::new());
        server.update_term(5);

        let result = server.start_new_election();

        assert_eq!(6, server.current_term());
        match result {
            ServerAction::Broadcast(Message::AppendEntries(ref ae)) => {
                assert_eq!(this_server_id.clone(), ae.leader_id);
                assert_eq!(6, ae.term);
            }
            _ => panic!()
        }
        assert_eq!(Role::Leader, server.current_role());
    }
}
