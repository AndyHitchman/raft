#![allow(dead_code)]
use types::*;
use std::sync::mpsc::{Sender,Receiver};

pub struct Dispatch {
    pub tx: Sender<Envelope>,
    pub rx: Receiver<Envelope>,
}

pub struct Endpoint {
    pub tx: Sender<Envelope>,
    pub rx: Receiver<Envelope>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Addressee {
    SingleServer(ServerIdentity),
    Broadcast,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Envelope {
    pub from: ServerIdentity,
    pub to: Addressee,
    pub message: Message,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Message {
    AppendEntries(AppendEntriesPayload),
    AppendEntriesResult(AppendEntriesResultPayload),
    RequestVote(RequestVotePayload),
    RequestVoteResult(RequestVoteResultPayload),
    Stop
}

impl Payload for Message {
    fn term(&self) -> Term {
        1
    }
}

pub trait Payload {
    fn term(&self) -> Term;
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct AppendEntriesPayload {
    /// leader's term
    pub term: Term,
    /// so follower can redirect clients
    pub leader_id: ServerIdentity,
    /// index of log entry immediately preceding new ones
    pub prev_log_index: LogIndex,
    /// term of prevLogIndex entry
    pub prev_log_term: Term,
    /// log entries to store (empty for heartbeat; may send more than one for efficiency)
    pub entries: Vec<Vec<u8>>,
    /// leader's commitIndex
    pub leaders_commit: LogIndex,
}

impl Payload for AppendEntriesPayload {
    fn term(&self) -> Term {
        self.term
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct AppendEntriesResultPayload {
    /// currentTerm, for leader to update itself
    term: Term,
    /// true if follower contained entry matching prevLogIndex and prevLogTerm
    success: bool,
}

impl Payload for AppendEntriesResultPayload {
    fn term(&self) -> Term {
        self.term
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct RequestVotePayload {
    /// candidate's term
    pub term: Term,
    /// candidate requesting vote
    pub candidate_id: ServerIdentity,
    /// index of candidate's last log entry
    pub last_log_index: LogIndex,
    /// term of candidate's last log entry
    pub last_log_term: Term,
}

impl Payload for RequestVotePayload {
    fn term(&self) -> Term {
        self.term
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct RequestVoteResultPayload {
    /// currentTerm, for candidate to update itself
    pub term: Term,
    /// true means candidate received vote
    pub vote_granted: bool,
}

impl Payload for RequestVoteResultPayload {
    fn term(&self) -> Term {
        self.term
    }
}
//
// #[derive(Eq, PartialEq, Clone, Debug)]
// pub struct RequestToFollowPayload {
//     /// Supplicate server requesting membership
//     supplicant_id: ServerIdentity,
// }
//
// #[derive(Eq, PartialEq, Clone, Debug)]
// pub struct RequestToFollowResultPayload {
//     /// Supplicate server requesting membership
//     supplicant_id: ServerIdentity,
//     /// Outcome of request to follow.
//     outcome: SupplicationResult,
// }
//
// #[derive(Eq, PartialEq, Clone, Debug)]
// pub enum SupplicationResult {
//     Granted,
//     Rejected,
//     NotTheLeader
// }
//
// #[derive(Eq, PartialEq, Clone, Debug)]
// pub struct Status {
//     /// leader's term
//     pub term: Term,
//     /// server role
//     pub role: Role,
//     /// commit index
//     pub commit_index: LogIndex,
// }
