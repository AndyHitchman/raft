#![allow(dead_code)]
use role::*;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum InwardMessage {
    AppendEntries(AppendEntriesPayload),
    AppendEntriesResult(AppendEntriesResultPayload),
    RequestVote(RequestVotePayload),
    RequestVoteResult(RequestVoteResultPayload),
    RequestToFollow(RequestToFollowPayload),
    RequestToFollowResult(RequestToFollowResultPayload),
    Stop,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum OutwardMessage {
    AppendEntries(AppendEntriesPayload),
    AppendEntriesResult(AppendEntriesResultPayload),
    RequestVote(RequestVotePayload),
    RequestVoteResult(RequestVoteResultPayload),
    RequestToFollow(RequestToFollowPayload),
    RequestToFollowResult(RequestToFollowResultPayload),
    Stopped,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct AppendEntriesPayload {
    /// leader's term
    term: u64,
    /// so follower can redirect clients
    leader_id: u16,
    /// index of log entry immediately preceding new ones
    prev_log_index: u64,
    /// term of prevLogIndex entry
    prev_log_term: u64,
    /// log entries to store (empty for heartbeat; may send more than one for efficiency)
    entries: Vec<Vec<u8>>,
    /// leader's commitIndex
    leaders_commit: u64,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct AppendEntriesResultPayload {
    /// currentTerm, for leader to update itself
    term: u64,
    /// true if follower contained entry matching prevLogIndex and prevLogTerm
    success: bool,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct RequestVotePayload {
    /// candidate's term
    pub term: u64,
    /// candidate requesting vote
    pub candidate_id: u16,
    /// index of candidate's last log entry
    pub last_log_index: u64,
    /// term of candidate's last log entry
    pub last_log_term: u64,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct RequestVoteResultPayload {
    /// currentTerm, for candidate to update itself
    term: u64,
    /// true means candidate received vote
    vote_granted: bool,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct RequestToFollowPayload {
    /// Supplicate node requesting membership
    supplicant_id: u16,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct RequestToFollowResultPayload {
    /// Supplicate node requesting membership
    supplicant_id: u16,
    /// Outcome of request to follow.
    outcome: SupplicationResult,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum SupplicationResult {
    Granted,
    Rejected,
    NotTheLeader
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Status {
    /// leader's term
    pub term: u64,
    /// server role
    pub role: Role,
    /// commit index
    pub commit_index: u64,
}
