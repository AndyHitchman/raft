/// Role of the node
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Role {
    Follower,
    Candidate,
    Leader
}
