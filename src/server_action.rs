use messages::Message;
use types::Role;

#[derive(PartialEq, Debug)]
pub enum ServerAction {
    Continue,
    Broadcast(Message),
    Reply(Message),
    NewTerm,
    NewRole(Role),
    Stop,
}
