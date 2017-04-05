//! The main entry point for a host process.

use std::time::Duration;
use std::thread;
use std::sync::mpsc::{channel,Receiver,RecvTimeoutError};

use rand;
use rand::Rng;

use types::{ServerIdentity,Role};
use election_timeout::ElectionTimeoutRange;
use messages::*;
use server::{Server,ServerAction};

pub struct Raft {}


impl Raft {

    /// Starts a server and returns the endoint used
    /// for communication.
    ///
    /// The peers should contain the known membership of the cluster.
    /// The election timeout should be a random duration between 150 and 300ms.
    pub fn start_server(identity: ServerIdentity, election_timeout_range: ElectionTimeoutRange) -> Endpoint {
        let (endpoint, dispatch, rx) = Raft::get_channels();

        thread::spawn(move || {
            let server = Server::new(identity);
            Raft::run(&server, &dispatch, &rx, election_timeout_range)
        });

        endpoint
    }

    /// Get channels for communication
    ///
    /// Endpoint is given to the host
    /// Dispatch is given to the server
    /// The inward message receiver is retained by Raft
    pub fn get_channels() -> (Endpoint, Dispatch, Receiver<InwardMessage>) {
        let (tx, client_rx) = channel::<OutwardMessage>();
        let (client_tx, rx) = channel::<InwardMessage>();
        let (status_tx, status_rx) = channel::<Status>();

        (
            Endpoint { tx: client_tx.clone(), rx: client_rx, status: status_rx },
            Dispatch { tx: tx, status: status_tx, loopback: client_tx },
            rx
        )
    }

    fn new_election_timeout(election_timeout_range: &ElectionTimeoutRange) -> Duration {
        Duration::from_millis(
            rand::thread_rng().gen_range(
                election_timeout_range.minimum_milliseconds as u64,
                election_timeout_range.maximum_milliseconds as u64 + 1
            )
        )
    }

    pub fn run(server: &Server, dispatch: &Dispatch, rx: &Receiver<InwardMessage>, election_timeout_range: ElectionTimeoutRange) {
        let mut election_timeout = Raft::new_election_timeout(&election_timeout_range);

        loop {
            match Raft::handle_event(server, dispatch, rx, election_timeout) {
                ServerAction::Stop => return,
                ServerAction::NewRole(role) => {
                    election_timeout = Raft::new_election_timeout(&election_timeout_range);
                    server.change_role(role);
                },
                ServerAction::Continue => (),
            }
        }
    }

    fn handle_event(server: &Server, dispatch: &Dispatch, rx: &Receiver<InwardMessage>, election_timeout: Duration) -> ServerAction {
        match server.current_role() {
            Role::Follower => Raft::handle_event_as_follower(server, dispatch, rx, election_timeout),
            Role::Candidate => Raft::handle_event_as_candidate(server, dispatch, rx, election_timeout),
            Role::Leader => ServerAction::Continue,
        }
    }

    fn handle_event_as_follower(server: &Server, dispatch: &Dispatch, rx: &Receiver<InwardMessage>, election_timeout: Duration) -> ServerAction {
        return match rx.recv_timeout(election_timeout) {
            Ok(InwardMessage::AppendEntries(ref ae)) => server.append_entries(ae),
            Ok(InwardMessage::RequestVote(ref rv)) => server.consider_vote(rv),
            Ok(InwardMessage::Stop) => ServerAction::Stop,
            Err(RecvTimeoutError::Timeout) => server.become_candidate_leader(dispatch),
            Err(RecvTimeoutError::Disconnected) => ServerAction::Stop,
            // Ignore events that have no meaning for this role.
            _ => ServerAction::Continue
        }
    }

    fn handle_event_as_candidate(server: &Server, dispatch: &Dispatch, rx: &Receiver<InwardMessage>, election_timeout: Duration) -> ServerAction {
        return match rx.recv_timeout(election_timeout) {
            Ok(InwardMessage::AppendEntries(ref ae)) => server.consider_conceding(ae),
            Ok(InwardMessage::RequestVote(rv)) => ServerAction::Continue,
            Ok(InwardMessage::RequestVoteResult(rvr)) => ServerAction::Continue,
            Ok(InwardMessage::Stop) => ServerAction::Stop,
            Err(RecvTimeoutError::Timeout) => server.start_new_election(dispatch),
            Err(RecvTimeoutError::Disconnected) => ServerAction::Stop,
            _ => ServerAction::Continue
        }
    }

    // fn report_status(server: &Server, dispatch: &Dispatch) {
    //     dispatch.status.send(server.report_status());
    // }

}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::sync::mpsc::channel;
    use election_timeout::*;
    use super::*;

    #[test]
    fn new_election_timeout_is_between_150_and_300ms_for_lan_config() {
        election_timeout_sample(&ElectionTimeoutRange::lan());
    }

    #[test]
    fn new_election_timeout_is_between_250_and_500ms_for_close_wan_config() {
        election_timeout_sample(&ElectionTimeoutRange::close_wan());
    }

    #[test]
    fn new_election_timeout_is_between_500_and_2500ms_for_global_wan_config() {
        election_timeout_sample(&ElectionTimeoutRange::global_wan());
    }

    fn election_timeout_sample(election_timeout_range: &ElectionTimeoutRange) {
        let mut hit_lower = false;
        let mut hit_upper = false;

        for tries in 1..10000000 {
            let timeout = Raft::new_election_timeout(election_timeout_range);
            assert!(timeout >= Duration::from_millis(election_timeout_range.minimum_milliseconds));
            assert!(timeout <= Duration::from_millis(election_timeout_range.maximum_milliseconds));

            if timeout == Duration::from_millis(election_timeout_range.minimum_milliseconds) { hit_lower = true; }
            if timeout == Duration::from_millis(election_timeout_range.maximum_milliseconds) { hit_upper = true; }
            if tries > 10000 && hit_upper && hit_lower { break; }
        }

        assert!(hit_lower);
        assert!(hit_upper);
    }

    #[test]
    fn thread_returns_when_stopped() {
        let (endpoint, dispatch, rx) = Raft::get_channels();
        let server = Server::new(1);
        endpoint.tx.send(InwardMessage::Stop);
        Raft::run(&server, &dispatch, &rx, ElectionTimeoutRange::testing());
    }
}
