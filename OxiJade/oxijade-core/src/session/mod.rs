pub mod local;
pub mod manager;

#[derive(Debug)]
pub enum SessionEvent {
    Output(Vec<u8>),
    Exited,
}

pub type SessionId = String;
