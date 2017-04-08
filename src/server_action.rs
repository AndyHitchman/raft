use messages::Message;

#[derive(PartialEq, Debug)]
pub enum ServerAction {
    Continue,
    Broadcast(Message),
    Reply(Message),
    NewTerm,
    Stop,
}
