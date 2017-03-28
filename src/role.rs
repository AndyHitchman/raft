/// Role of the node
#[derive(PartialEq, Eq, Debug)]
pub enum Role {
    Follower,
    Candidate,
    Leader
}
