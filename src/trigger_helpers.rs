//! Convenience factory functions for common trigger patterns.
//!
//! These helpers create ready-to-use [`Trigger`] implementations for common scenarios
//! like periodic timers and filesystem watchers.

use crate::connection::AnyConnection;
use crate::connection::Connection;
use crate::triggers::Trigger;
use std::time::Duration;

// ─── Periodic (every) Trigger ───────────────────────────────────────────────

/// A trigger that fires at regular intervals.
///
/// Created via [`every()`].
#[derive(Debug)]
pub struct PeriodicTrigger {
    interval: Duration,
    message: String,
}

impl Trigger for PeriodicTrigger {
    async fn run(&self, connection: AnyConnection) -> Result<(), anyhow::Error> {
        loop {
            tokio::time::sleep(self.interval).await;
            connection.send_trigger_notification(&self.message).await?;
        }
    }
}

/// Creates a trigger that fires every `interval` duration, sending `message` to the agent.
///
/// # Example
/// ```no_run
/// use antigravity_sdk_rust::trigger_helpers::every;
/// use std::time::Duration;
///
/// let trigger = every(Duration::from_secs(30), "check_status");
/// ```
pub fn every(interval: Duration, message: impl Into<String>) -> PeriodicTrigger {
    PeriodicTrigger {
        interval,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_every_construction() {
        let trigger = every(Duration::from_secs(10), "heartbeat");
        assert_eq!(trigger.interval, Duration::from_secs(10));
        assert_eq!(trigger.message, "heartbeat");
    }

    #[test]
    fn test_every_with_string() {
        let msg = String::from("custom message");
        let trigger = every(Duration::from_millis(500), msg);
        assert_eq!(trigger.interval, Duration::from_millis(500));
        assert_eq!(trigger.message, "custom message");
    }
}
