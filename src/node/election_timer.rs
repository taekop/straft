use rand::{thread_rng, Rng};
use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// time values as millis, u128
pub struct ElectionTimer {
    election_timeout: Range<u128>,
    heartbeat_time: Arc<Mutex<u128>>,
    next_election_timeout: Arc<Mutex<u128>>,
}

impl ElectionTimer {
    pub fn new(election_timeout: Range<u128>) -> ElectionTimer {
        ElectionTimer {
            heartbeat_time: Arc::new(Mutex::new(0)),
            election_timeout: election_timeout,
            next_election_timeout: Arc::new(Mutex::new(0)),
        }
    }

    pub fn reset(&self) {
        let mut heartbeat_time = self.heartbeat_time.lock().unwrap();
        let mut next_election_timeout = self.next_election_timeout.lock().unwrap();
        let now = ElectionTimer::now();
        *heartbeat_time = now;
        *next_election_timeout = thread_rng().gen_range(self.election_timeout.clone());
    }

    pub fn is_election_timeout(&self) -> bool {
        let mut heartbeat_time = self.heartbeat_time.lock().unwrap();
        let mut next_election_timeout = self.next_election_timeout.lock().unwrap();
        let now = ElectionTimer::now();
        now > *heartbeat_time + *next_election_timeout
    }

    // half timeout to prevent leader switching due to delayed response
    pub fn until_next_timeout(&self) -> u128 {
        let mut heartbeat_time = self.heartbeat_time.lock().unwrap();
        let mut next_election_timeout = self.next_election_timeout.lock().unwrap();
        let now = ElectionTimer::now();
        (*heartbeat_time + *next_election_timeout / 2)
            .checked_sub(now)
            .unwrap_or(0)
    }

    fn now() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }
}
