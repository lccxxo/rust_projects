pub mod local;

#[derive(Debug)]
pub enum SessionEvent {
    Output(Vec<u8>),
    Exited,
}

pub type SessionId = String;
