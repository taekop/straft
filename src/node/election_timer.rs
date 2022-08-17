use rand::{thread_rng, Rng};
use std::time::{SystemTime, UNIX_EPOCH};

// time values as millis, u64
pub struct ElectionTimer {
    minimum_election_timeout: u64,
    maximum_election_timeout: u64,
    last_timestamp: u64,
    last_current_leader_timestamp: u64,
    next_election_timeout: u64,
}

impl ElectionTimer {
    pub fn new(minimum_election_timeout: u64, maximum_election_timeout: u64) -> ElectionTimer {
        ElectionTimer {
            minimum_election_timeout,
            maximum_election_timeout,
            last_timestamp: 0,
            last_current_leader_timestamp: 0,
            next_election_timeout: 0,
        }
    }

    // Check whether request is from current leader
    pub fn reset(&mut self, current_leader: bool) {
        let now = ElectionTimer::now();
        self.last_timestamp = now;
        if current_leader {
            self.last_current_leader_timestamp = now;
        }
        self.next_election_timeout =
            thread_rng().gen_range(self.minimum_election_timeout..self.maximum_election_timeout);
    }

    // Ready to run for leader
    pub fn is_timeout(&self) -> bool {
        let now = ElectionTimer::now();
        now > self.last_timestamp + self.next_election_timeout
    }

    // Check the possibility of current leader crash
    pub fn is_current_leader_timeout(&self) -> bool {
        let now = ElectionTimer::now();
        now > self.last_current_leader_timestamp + self.minimum_election_timeout
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}
