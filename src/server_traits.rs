use types::{ServerIdentity, Role};
use messages::*;

pub trait RaftServer {
    fn new(ServerIdentity) -> Self;
    fn current_role(&self) -> Role;
    fn change_role(&self, Role);
}

pub trait FollowingServer : RaftServer {
    fn append_entries(&self, &AppendEntriesPayload) -> ServerAction;
    fn consider_vote(&self, &RequestVotePayload) -> ServerAction;
    fn become_candidate_leader(&self, &Dispatch) -> ServerAction;
}

pub trait LeadingServer : RaftServer {

}

pub trait CandidateServer : RaftServer {
    fn consider_conceding(&self, &AppendEntriesPayload) -> ServerAction;
    fn start_new_election(&self, &Dispatch) -> ServerAction;
}

#[derive(PartialEq, Debug)]
pub enum ServerAction {
    Continue,
    NewRole(Role),
    Stop,
}
