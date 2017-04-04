#![allow(dead_code)]
use role::*;

pub type NodeIdentity = u16;
pub type Term = u64;
pub type LogIndex = u64;

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
    term: Term,
    /// so follower can redirect clients
    leader_id: NodeIdentity,
    /// index of log entry immediately preceding new ones
    prev_log_index: LogIndex,
    /// term of prevLogIndex entry
    prev_log_term: Term,
    /// log entries to store (empty for heartbeat; may send more than one for efficiency)
    entries: Vec<Vec<u8>>,
    /// leader's commitIndex
    leaders_commit: LogIndex,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct AppendEntriesResultPayload {
    /// currentTerm, for leader to update itself
    term: Term,
    /// true if follower contained entry matching prevLogIndex and prevLogTerm
    success: bool,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct RequestVotePayload {
    /// candidate's term
    pub term: Term,
    /// candidate requesting vote
    pub candidate_id: NodeIdentity,
    /// index of candidate's last log entry
    pub last_log_index: LogIndex,
    /// term of candidate's last log entry
    pub last_log_term: Term,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct RequestVoteResultPayload {
    /// currentTerm, for candidate to update itself
    term: Term,
    /// true means candidate received vote
    vote_granted: bool,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct RequestToFollowPayload {
    /// Supplicate node requesting membership
    supplicant_id: NodeIdentity,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct RequestToFollowResultPayload {
    /// Supplicate node requesting membership
    supplicant_id: NodeIdentity,
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
    pub term: Term,
    /// server role
    pub role: Role,
    /// commit index
    pub commit_index: LogIndex,
}
