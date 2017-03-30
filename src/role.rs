/// Role of the node
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Role {
    /// Not part of the cluster configuration
    Disqualified,
    /// Following a leader
    Follower,
    /// Candidate for leadership, during an election
    Candidate,
    /// Leader for this term
    Leader
}
