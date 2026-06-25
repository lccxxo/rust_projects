use std::time::{Duration, Instant};

const WORK_SECS: u16 = 25 * 60;
const SHORT_BREAK_SECS: u16 = 5 * 60;
const LONG_BREAK_SECS: u16 = 15 * 60;
const POMODOROS_PER_CYCLE: u8 = 4;

#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RunState {
    Running,
    Paused,
}

pub struct PomodoroFSM {
    pub phase: Phase,
    pub run_state: RunState,
    pub completed: u8,
    pub remaining: u16,
    last_tick: Instant,
}

impl PomodoroFSM {
    pub fn new() -> Self {
        Self {
            phase: Phase::Work,
            run_state: RunState::Paused,
            completed: 0,
            remaining: WORK_SECS,
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.run_state != RunState::Running {
            return false;
        }
        let elapsed = self.last_tick.elapsed();
        if elapsed < Duration::from_secs(1) {
            return false;
        }
        let secs = elapsed.as_secs().min(u16::MAX as u64) as u16;
        // Drift-free: accumulate exact 1s steps; re-sync if suspended >5s
        if elapsed > Duration::from_secs(5) {
            self.last_tick = Instant::now();
        } else {
            self.last_tick += Duration::from_secs(secs as u64);
        }
        if secs >= self.remaining {
            self.remaining = 0;
            self.advance_phase();
            return true;
        }
        self.remaining -= secs;
        false
    }

    fn advance_phase(&mut self) {
        match self.phase {
            Phase::Work => {
                self.completed += 1;
                if self.completed >= POMODOROS_PER_CYCLE {
                    self.phase = Phase::LongBreak;
                    self.remaining = LONG_BREAK_SECS;
                    self.completed = 0;
                } else {
                    self.phase = Phase::ShortBreak;
                    self.remaining = SHORT_BREAK_SECS;
                }
            }
            Phase::ShortBreak | Phase::LongBreak => {
                self.phase = Phase::Work;
                self.remaining = WORK_SECS;
            }
        }
    }

    pub fn toggle_pause(&mut self) {
        self.run_state = match self.run_state {
            RunState::Running => RunState::Paused,
            RunState::Paused => RunState::Running,
        };
        self.last_tick = Instant::now();
    }

    pub fn reset_phase(&mut self) {
        self.remaining = match self.phase {
            Phase::Work => WORK_SECS,
            Phase::ShortBreak => SHORT_BREAK_SECS,
            Phase::LongBreak => LONG_BREAK_SECS,
        };
        self.last_tick = Instant::now();
    }

    pub fn phase_label(&self) -> &'static str {
        match self.phase {
            Phase::Work => "工 作",
            Phase::ShortBreak => "短 休",
            Phase::LongBreak => "长 休",
        }
    }
}
