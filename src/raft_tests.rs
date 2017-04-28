mod get_channels {
    use super::super::*;

    #[test]
    fn endpoint_rx_is_listening_to_dispatch_tx() {
        let (endpoint, dispatch) = Raft::get_channels();

        dispatch.tx.send(Envelope {
            from: ServerIdentity::new(),
            to: Addressee::Broadcast,
            message: Message::Stop,
        }).unwrap();

        endpoint.rx.recv().unwrap();
    }

    #[test]
    fn dispatch_rx_is_listening_to_endpoint_tx() {
        let (endpoint, dispatch) = Raft::get_channels();

        endpoint.tx.send(Envelope {
            from: ServerIdentity::new(),
            to: Addressee::Broadcast,
            message: Message::Stop,
        }).unwrap();

        dispatch.rx.recv().unwrap();
    }
}

mod stop_message {
    use election_timeout::*;
    use super::super::*;

    #[test]
    fn server_returns_when_stopped() {
        let (endpoint, dispatch) = Raft::get_channels();
        let this_server_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), Vec::new());

        endpoint.tx.send(Envelope {
            from: this_server_id.clone(),
            to: Addressee::SingleServer(this_server_id.clone()),
            message: Message::Stop
        }).unwrap();

        Raft::run(&server, &dispatch, ElectionTimeoutRange::testing());
    }
}

mod event_loop {
    use election_timeout::*;
    use super::super::*;
    use std::thread;

    #[test]
    fn a_follower_that_does_not_receive_a_heartbeat_will_start_a_new_election() {
        let (endpoint, dispatch) = Raft::get_channels();
        let timeout = ElectionTimeoutRange::lan();
        let this_server_id = ServerIdentity::new();
        let peer_1_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), vec![peer_1_id]);
        let event_loop = thread::spawn(move || {
            Raft::run(&server, &dispatch, timeout);
        });

        let result = endpoint.rx.recv().unwrap();
        assert_eq!(this_server_id.clone(), result.from);
        assert_eq!(Addressee::Broadcast, result.to);
        match result.message {
            Message::RequestVote(ref rv) => {
                assert_eq!(2, rv.term);
                assert!(true);
            },
            _ => assert!(false),
        }

        endpoint.tx.send(Envelope {
            from: this_server_id.clone(),
            to: Addressee::SingleServer(this_server_id.clone()),
            message: Message::Stop
        }).unwrap();

        event_loop.join().unwrap();
    }

    #[test]
    fn a_candidate_that_does_not_receive_enough_votes_or_append_entries_from_a_new_leader_before_the_election_timesout_will_start_a_new_election() {
        let (endpoint, dispatch) = Raft::get_channels();
        let timeout = ElectionTimeoutRange::lan();
        let this_server_id = ServerIdentity::new();
        let peer_1_id = ServerIdentity::new();
        let server = Server::new(this_server_id.clone(), vec![peer_1_id]);

        let event_loop = thread::spawn(move || {
            Raft::run(&server, &dispatch, timeout);
        });

        let _ = endpoint.rx.recv().unwrap();

        let result = endpoint.rx.recv().unwrap();
        assert_eq!(this_server_id.clone(), result.from);
        assert_eq!(Addressee::Broadcast, result.to);
        match result.message {
            Message::RequestVote(ref rv) => {
                assert_eq!(3, rv.term);
                assert!(true);
            },
            _ => assert!(false),
        }


        endpoint.tx.send(Envelope {
            from: this_server_id.clone(),
            to: Addressee::SingleServer(this_server_id.clone()),
            message: Message::Stop
        }).unwrap();

        event_loop.join().unwrap();
    }

}
