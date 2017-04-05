use types::{ServerIdentity, Role};
use messages::*;

pub trait RaftServer {
    fn current_role(&self) -> Role;
    fn change_role(&self, Role);

    fn append_entries(&self, &AppendEntriesPayload) -> ServerAction;
    fn consider_vote(&self, &RequestVotePayload) -> ServerAction;
    fn become_candidate_leader(&self, &Dispatch) -> ServerAction;

    fn consider_conceding(&self, &AppendEntriesPayload) -> ServerAction;
    fn start_new_election(&self, &Dispatch) -> ServerAction;
}

#[derive(PartialEq, Debug)]
pub enum ServerAction {
    Continue,
    NewRole(Role),
    Stop,
}
