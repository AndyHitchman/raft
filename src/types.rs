pub type ServerIdentity = u16;
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
