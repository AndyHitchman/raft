use uuid::Uuid;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ServerIdentity {
    id: Uuid
}

impl ServerIdentity {
    pub fn new() -> ServerIdentity {
        ServerIdentity {
            id: Uuid::new_v4()
        }
    }
}

pub type Term = u64;
pub type LogIndex = u64;

/// Role of the node
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Role {
    /// Following a leader
    Follower,
    /// Candidate for leadership, during an election
    Candidate,
    /// Leader for this term
    Leader
}
