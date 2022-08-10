use rand::{thread_rng, Rng};
use std::ops::Range;
use std::time::{SystemTime, UNIX_EPOCH};

// time values as millis, u64
pub struct ElectionTimer {
    election_timeout: Range<u64>,
    heartbeat_time: u64,
    next_election_timeout: u64,
}

impl ElectionTimer {
    pub fn new(election_timeout: Range<u64>) -> ElectionTimer {
        ElectionTimer {
            heartbeat_time: 0,
            election_timeout: election_timeout,
            next_election_timeout: 0,
        }
    }

    pub fn reset(&mut self) {
        let now = ElectionTimer::now();
        self.heartbeat_time = now;
        self.next_election_timeout = thread_rng().gen_range(self.election_timeout.clone());
    }

    pub fn is_timeout(&self) -> bool {
        let now = ElectionTimer::now();
        now > self.heartbeat_time + self.next_election_timeout
    }

    // half timeout to prevent leader switching due to delayed response
    pub fn _until_next_election(&self) -> u64 {
        let now = ElectionTimer::now();
        (self.heartbeat_time + self.next_election_timeout / 2)
            .checked_sub(now)
            .unwrap_or(0)
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}
