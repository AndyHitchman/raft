
mod new_server {
    use super::super::*;

    #[test]
    fn is_a_follower() {
        let server = Server::new(ServerIdentity::new(), Vec::new());

        assert_eq!(Role::Follower, server.current_role());
    }

    #[test]
    fn current_term_is_1() {
        let server = Server::new(ServerIdentity::new(), Vec::new());

        assert_eq!(1, server.current_term());
    }
}

mod update_term {
    use super::super::*;

    #[test]
    fn is_recorded() {
        let server = Server::new(ServerIdentity::new(), Vec::new());

        server.update_term(10);

        assert_eq!(10, server.current_term());
    }

    #[test]
    fn doesnt_panic_for_same_term() {
        //Ergo we lost this election, same term.
        let server = Server::new(ServerIdentity::new(), Vec::new());

        server.update_term(2);
        server.update_term(2);

        assert_eq!(2, server.current_term());
    }

    #[test]
    #[should_panic]
    fn panics_for_a_lower_term() {
        let server = Server::new(ServerIdentity::new(), Vec::new());

        server.update_term(0);
    }

    #[test]
    fn changes_candidate_to_follower() {
        let server = Server::new(ServerIdentity::new(), Vec::new());
        server.change_role(Role::Candidate);

        server.update_term(2);

        assert_eq!(Role::Follower, server.current_role());
    }

    #[test]
    fn changes_leader_to_follower() {
        let server = Server::new(ServerIdentity::new(), Vec::new());
        server.change_role(Role::Leader);

        server.update_term(2);

        assert_eq!(Role::Follower, server.current_role());
    }

    #[test]
    fn clears_servers_vote() {
        let server = Server::new(ServerIdentity::new(), Vec::new());
        server.persistent_state.borrow_mut().voted_for = Some(ServerIdentity::new());

        server.update_term(2);

        assert_eq!(None, server.persistent_state.borrow().voted_for);
    }

    #[test]
    fn clears_followers() {
        let server = Server::new(ServerIdentity::new(), Vec::new());
        server.volatile_state.borrow_mut().followers = Some(Vec::new());

        server.update_term(2);

        assert_eq!(None, server.volatile_state.borrow().followers);
    }

    #[test]
    fn clears_received_votes() {
        let server = Server::new(ServerIdentity::new(), Vec::new());
        server.volatile_state.borrow_mut().votes = 2;

        server.update_term(2);

        assert_eq!(0, server.volatile_state.borrow().votes);
    }
}

mod next_term {
    use super::super::*;

    #[test]
    fn increments_the_current_term_from_one_to_two() {
        let server = Server::new(ServerIdentity::new(), Vec::new());

        server.next_term();

        assert_eq!(2, server.current_term());
    }

    #[test]
    fn increments_the_current_term_from_33_to_34() {
        let server = Server::new(ServerIdentity::new(), Vec::new());
        server.update_term(33);

        server.next_term();

        assert_eq!(34, server.current_term());
    }

    #[test]
    fn changes_candidate_to_follower() {
        let server = Server::new(ServerIdentity::new(), Vec::new());
        server.change_role(Role::Candidate);

        server.next_term();

        assert_eq!(Role::Follower, server.current_role());
    }
}

mod i_am_out_of_date {
    use super::super::*;

    #[test]
    fn says_false_when_my_term_equals_other_term() {
        assert_eq!(false, Server::i_am_out_of_date(12, 12));
    }

    #[test]
    fn says_false_when_my_term_is_greater_then_other_term() {
        assert_eq!(false, Server::i_am_out_of_date(13, 12));
    }

    #[test]
    fn says_true_when_my_term_is_less_then_other_term() {
        assert_eq!(true, Server::i_am_out_of_date(11, 12));
    }
}

mod assert_term_is_current {
    use super::super::*;

    #[test]
    fn will_not_panic_if_this_term_is_current() {
        let server = Server::new(ServerIdentity::new(), Vec::new());

        server.assert_term_is_current(
            &RequestVoteResultPayload {
                term: 1,
                vote_granted: false,
            }
        )
    }

    #[test]
    fn will_not_panic_if_this_term_is_later() {
        let server = Server::new(ServerIdentity::new(), Vec::new());

        server.assert_term_is_current(
            &RequestVoteResultPayload {
                term: 0,
                vote_granted: false,
            }
        )
    }

    #[test]
    #[should_panic]
    fn panics_if_this_term_is_out_of_date() {
        let server = Server::new(ServerIdentity::new(), Vec::new());

        server.assert_term_is_current(
            &RequestVoteResultPayload {
                term: 2,
                vote_granted: false,
            }
        )
    }
}

mod change_role {
    use super::super::*;

    #[test]
    fn to_candidate() {
        let server = Server::new(ServerIdentity::new(), Vec::new());

        server.change_role(Role::Candidate);

        assert_eq!(Role::Candidate, server.current_role());
    }

    #[test]
    fn to_leader() {
        let server = Server::new(ServerIdentity::new(), Vec::new());

        server.change_role(Role::Leader);

        assert_eq!(Role::Leader, server.current_role());
    }
}

mod start_new_election {
    use super::super::*;

    #[test]
    fn single_server_will_self_appoint() {
        let this_server_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), Vec::new());

        server.start_new_election();

        assert_eq!(Role::Leader, server.current_role());
    }

    #[test]
    fn self_appointed_leader_will_broadcast_append_entires() {
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
    fn candidate_votes_for_itself() {
        let this_server_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), Vec::new());

        server.start_new_election();

        assert_eq!(Some(this_server_id), server.persistent_state.borrow().voted_for);
    }


    #[test]
    fn follower_will_broadcast_request_for_votes() {
        let this_server_id = ServerIdentity::new();
        let peer_1_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), vec![peer_1_id]);

        let result = server.start_new_election();

        if let ServerAction::Broadcast(Message::RequestVote(rv)) = result {
            assert_eq!(this_server_id, rv.candidate_id);
        } else {
            panic!();
        };
    }

    #[test]
    fn follower_with_peers_becomes_candidate() {
        let this_server_id = ServerIdentity::new();
        let peer_1_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), vec![peer_1_id]);

        server.start_new_election();

        assert_eq!(Role::Candidate, server.current_role());
    }

    #[test]
    fn a_peered_election_starts_a_new_term() {
        let this_server_id = ServerIdentity::new();
        let peer_1_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), vec![peer_1_id]);

        let result = server.start_new_election();

        // No peers means self is elected leader immediately.
        if let ServerAction::Broadcast(Message::RequestVote(rv)) = result {
            assert_eq!(2, rv.term());
        } else {
            panic!();
        };
    }

    #[test]
    fn a_self_appointing_election_starts_a_new_term() {
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
}

mod consider_vote {
    use super::super::*;

    #[test]
    fn follower_will_vote_for_the_first_candidate_that_requests_a_vote() {
        let server = Server::new(ServerIdentity::new(), Vec::new());
        let other_server_id = ServerIdentity::new();

        server.consider_vote_request(&RequestVotePayload {
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

        server.consider_vote_request(&RequestVotePayload {
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

        server.consider_vote_request(&RequestVotePayload {
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

        let result = server.consider_vote_request(&RequestVotePayload {
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

        server.consider_vote_request(rv);

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

        server.consider_vote_request(rv);

        assert_eq!(4, server.persistent_state.borrow().current_term);
        assert_eq!(Some(this_server_id), server.persistent_state.borrow().voted_for);
        assert_eq!(Role::Candidate, server.current_role());
    }
}

mod collect_vote {
    use super::super::*;

    #[test]
    fn candidate_becomes_leader_on_receiving_majority_of_votes() {
        let this_server_id = ServerIdentity::new();
        let peer_1_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), vec![peer_1_id]);
        server.update_term(2);

        server.start_new_election();
        server.collect_vote(
            &RequestVoteResultPayload {
                term: 3,
                vote_granted: true,
            }
        );

        assert_eq!(Role::Leader, server.current_role());
    }

    #[test]
    fn candidate_is_not_promoted_when_other_peer_denies_vote() {
        let this_server_id = ServerIdentity::new();
        let peer_1_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), vec![peer_1_id]);
        server.update_term(2);

        server.start_new_election();
        let result = server.collect_vote(
            &RequestVoteResultPayload {
                term: 3,
                vote_granted: false,
            }
        );

        assert_eq!(Role::Candidate, server.current_role());
        assert_eq!(ServerAction::Continue, result);
    }

    #[test]
    fn candidate_is_not_promoted_when_vote_does_not_give_balance_of_support() {
        let this_server_id = ServerIdentity::new();
        let peer_1_id = ServerIdentity::new();
        let peer_2_id = ServerIdentity::new();
        let peer_3_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), vec![peer_1_id, peer_2_id, peer_3_id]);
        server.update_term(2);

        server.start_new_election();
        let result = server.collect_vote(
            &RequestVoteResultPayload {
                term: 3,
                vote_granted: true,
            }
        );

        assert_eq!(Role::Candidate, server.current_role());
        assert_eq!(ServerAction::Continue, result);
    }

    #[test]
    fn candidate_is_promoted_when_vote_gives_balance_of_support() {
        let this_server_id = ServerIdentity::new();
        let peer_1_id = ServerIdentity::new();
        let peer_2_id = ServerIdentity::new();
        let peer_3_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), vec![peer_1_id, peer_2_id, peer_3_id]);
        server.update_term(2);

        server.start_new_election();
        server.collect_vote(
            &RequestVoteResultPayload {
                term: 3,
                vote_granted: true,
            }
        );

        server.collect_vote(
            &RequestVoteResultPayload {
                term: 3,
                vote_granted: true,
            }
        );

        assert_eq!(Role::Leader, server.current_role());
    }
}

mod append_entries {
    use super::super::*;

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
}

mod broadcast_heartbeat {
    use super::super::*;

    #[test]
    fn test_name() {
        unimplemented!()
    }
}
