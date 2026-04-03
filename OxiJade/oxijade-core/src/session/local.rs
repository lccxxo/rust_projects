use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;

use crate::session::SessionEvent;

pub struct LocalSession {
    pub id: String,
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child: Box<dyn portable_pty::Child + Send>,
    // Keep master alive: dropping it calls ClosePseudoConsole on Windows ConPTY,
    // which would kill the attached child process.
    _master: Box<dyn MasterPty + Send>,
}

impl LocalSession {
    pub fn new(id: String, args: Vec<String>, tx: Sender<SessionEvent>) -> anyhow::Result<Self> {
        anyhow::ensure!(!args.is_empty(), "args must not be empty");
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(&args[0]);
        for arg in &args[1..] {
            cmd.arg(arg);
        }
        cmd.cwd(std::env::current_dir().unwrap_or_default());

        let child = pair.slave.spawn_command(cmd)?;
        let writer = Arc::new(Mutex::new(pair.master.take_writer()?));
        let mut reader = pair.master.try_clone_reader()?;

        tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => {
                        let _ = tx.blocking_send(SessionEvent::Exited);
                        break;
                    }
                    Ok(n) => {
                        let _ = tx.blocking_send(SessionEvent::Output(buf[..n].to_vec()));
                    }
                }
            }
        });

        Ok(Self {
            id,
            writer,
            child,
            _master: pair.master,
        })
    }

    pub fn resize(&self, cols: u16, rows: u16) {
        let _ = self._master.resize(portable_pty::PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
    }

    pub fn kill(&mut self) {
        let _ = self.child.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, timeout, Duration};

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_local_session_receives_output() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let mut session = LocalSession::new("test-id".to_string(), vec!["cmd.exe".to_string()], tx)
            .expect("failed to create session");

        // Give cmd.exe a moment to initialise before sending input
        sleep(Duration::from_millis(500)).await;

        // Write echo command
        {
            let mut w = session.writer.lock().unwrap();
            w.write_all(b"echo hello_oxijade\r\n").unwrap();
            w.flush().unwrap();
        }

        let found = timeout(Duration::from_secs(10), async {
            let mut buf = String::new();
            while let Some(event) = rx.recv().await {
                match event {
                    SessionEvent::Output(bytes) => {
                        buf.push_str(&String::from_utf8_lossy(&bytes));
                        if buf.contains("hello_oxijade") {
                            return true;
                        }
                    }
                    SessionEvent::Exited => break,
                }
            }
            false
        })
        .await;

        assert_eq!(found.unwrap(), true);
        session.kill();
    }
}
