use crate::session::{SessionEvent, SessionId};
use crate::terminal::TerminalGrid;
use std::collections::HashMap;
use tokio::sync::mpsc::{channel, Receiver};

pub struct ManagedSession {
    pub grid: TerminalGrid,
    pub rx: Receiver<SessionEvent>,
}

pub struct SessionManager {
    sessions: HashMap<SessionId, ManagedSession>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Adds a new terminal entry with a fresh grid and channel (used in tests; real sessions connect a PTY separately).
    pub fn add_terminal(&mut self, id: SessionId) {
        let (_tx, rx) = channel(32);
        self.sessions.insert(
            id,
            ManagedSession {
                grid: TerminalGrid::new(80, 24),
                rx,
            },
        );
    }

    pub fn get_terminal(&self, id: &str) -> Option<&ManagedSession> {
        self.sessions.get(id)
    }

    pub fn get_terminal_mut(&mut self, id: &str) -> Option<&mut ManagedSession> {
        self.sessions.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) {
        self.sessions.remove(id);
    }

    pub fn session_ids(&self) -> Vec<SessionId> {
        self.sessions.keys().cloned().collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_session() {
        let mut manager = SessionManager::new();
        manager.add_terminal("id-1".to_string());
        assert!(manager.get_terminal("id-1").is_some());
        assert!(manager.get_terminal("id-2").is_none());
    }

    #[test]
    fn test_remove_session() {
        let mut manager = SessionManager::new();
        manager.add_terminal("id-1".to_string());
        manager.remove("id-1");
        assert!(manager.get_terminal("id-1").is_none());
    }

    #[test]
    fn test_session_ids() {
        let mut manager = SessionManager::new();
        manager.add_terminal("a".to_string());
        manager.add_terminal("b".to_string());
        let ids = manager.session_ids();
        assert!(ids.contains(&"a".to_string()));
        assert!(ids.contains(&"b".to_string()));
    }
}
