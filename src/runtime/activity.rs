//! Per-user request activity used by the control center's idle shutdown policy.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug)]
struct ActivityState {
    in_flight: usize,
    last_activity: Instant,
}

/// Shared request counter and last-activity timestamp for one control center.
#[derive(Debug)]
pub(crate) struct RuntimeActivity {
    state: Mutex<ActivityState>,
}

impl RuntimeActivity {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(ActivityState {
                in_flight: 0,
                last_activity: Instant::now(),
            }),
        })
    }

    pub(crate) fn request(self: &Arc<Self>) -> RequestActivityGuard {
        let mut state = self.state.lock().unwrap();
        state.in_flight = state.in_flight.saturating_add(1);
        state.last_activity = Instant::now();
        RequestActivityGuard {
            activity: Arc::clone(self),
        }
    }

    pub(crate) fn touch(&self) {
        self.state.lock().unwrap().last_activity = Instant::now();
    }

    pub(crate) fn is_idle_for(&self, duration: Duration) -> bool {
        self.is_idle_at(Instant::now(), duration)
    }

    fn is_idle_at(&self, now: Instant, duration: Duration) -> bool {
        let state = self.state.lock().unwrap();
        state.in_flight == 0 && now.saturating_duration_since(state.last_activity) >= duration
    }
}

/// One in-flight request. Dropping it records completion even on cancellation or panic unwind.
pub(crate) struct RequestActivityGuard {
    activity: Arc<RuntimeActivity>,
}

impl Drop for RequestActivityGuard {
    fn drop(&mut self) {
        let mut state = self.activity.state.lock().unwrap();
        state.in_flight = state.in_flight.saturating_sub(1);
        state.last_activity = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeActivity;
    use std::time::{Duration, Instant};

    #[test]
    fn in_flight_work_prevents_idle_and_completion_restarts_the_clock() {
        let activity = RuntimeActivity::new();
        let guard = activity.request();
        assert!(!activity.is_idle_at(
            Instant::now() + Duration::from_secs(60),
            Duration::from_secs(1)
        ));
        drop(guard);
        assert!(!activity.is_idle_at(Instant::now(), Duration::from_secs(1)));
        assert!(activity.is_idle_at(
            Instant::now() + Duration::from_secs(2),
            Duration::from_secs(1)
        ));
    }
}
