use types::*;
use messages::*;
use server_action::*;

pub trait RaftServer {
    fn current_role(&self) -> Role;

    fn append_entries(&self, &AppendEntriesPayload) -> ServerAction;
    fn consider_vote(&self, &RequestVotePayload) -> ServerAction;
    fn collect_vote(&self, &RequestVoteResultPayload) -> ServerAction;

    fn ensure_term_is_latest(&self, &Payload) -> ServerAction;
    fn start_new_election(&self) -> ServerAction;
}
