//! Convenience factory functions for common trigger patterns.
//!
//! These helpers create ready-to-use [`Trigger`] implementations for common scenarios
//! like periodic timers.

use crate::triggers::{Trigger, TriggerContext};
use std::future::Future;
use std::time::Duration;

// ─── Periodic (every) Trigger ───────────────────────────────────────────────

/// A trigger that runs a callback at regular intervals.
///
/// Created via [`every()`].
pub struct PeriodicTrigger<F> {
    interval: Duration,
    callback: F,
}

impl<F> std::fmt::Debug for PeriodicTrigger<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PeriodicTrigger")
            .field("interval", &self.interval)
            .finish_non_exhaustive()
    }
}

impl<F, Fut> Trigger for PeriodicTrigger<F>
where
    F: Fn(TriggerContext) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), anyhow::Error>> + Send,
{
    async fn run(&self, context: TriggerContext) -> Result<(), anyhow::Error> {
        loop {
            tokio::time::sleep(self.interval).await;
            (self.callback)(context.clone()).await?;
        }
    }
}

/// Runs `callback` every `interval`.
///
/// The callback decides what to do — send a notification, check something
/// first, do nothing this tick. Previously this took a fixed string and always
/// sent it, so a trigger could not decide whether it had anything to say.
///
/// # Errors
///
/// Returns an error for a zero interval, which would spin the task at full
/// speed rather than firing "as often as possible".
///
/// # Example
/// ```no_run
/// use antigravity_sdk_rust::trigger_helpers::every;
/// use std::time::Duration;
///
/// let trigger = every(Duration::from_secs(30), |ctx| async move {
///     ctx.send("check_status").await
/// })?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn every<F, Fut>(interval: Duration, callback: F) -> Result<PeriodicTrigger<F>, anyhow::Error>
where
    F: Fn(TriggerContext) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), anyhow::Error>> + Send,
{
    if interval.is_zero() {
        return Err(anyhow::anyhow!(
            "a trigger interval must be greater than zero"
        ));
    }
    Ok(PeriodicTrigger { interval, callback })
}

/// Sends a fixed `message` every `interval`.
///
/// The common case of [`every`], kept as its own function so the simple use
/// does not need a closure.
///
/// # Errors
///
/// Returns an error for a zero interval.
pub fn every_notification(
    interval: Duration,
    message: impl Into<String>,
) -> Result<
    PeriodicTrigger<
        impl Fn(
            TriggerContext,
        ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>>,
    >,
    anyhow::Error,
> {
    let message: Arc<str> = Arc::from(message.into());
    every(interval, move |ctx: TriggerContext| {
        let message = message.clone();
        Box::pin(async move { ctx.send(&message).await })
            as std::pin::Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>>
    })
}

use std::sync::Arc;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn a_zero_interval_is_rejected() {
        let err = every(Duration::ZERO, |ctx: TriggerContext| async move {
            ctx.send("tick").await
        })
        .expect_err("a zero interval would spin the task");
        assert!(err.to_string().contains("greater than zero"), "{err}");
        assert!(every_notification(Duration::ZERO, "tick").is_err());
    }

    #[test]
    fn a_positive_interval_is_accepted() {
        let trigger = every_notification(Duration::from_secs(10), "heartbeat").unwrap();
        assert_eq!(trigger.interval, Duration::from_secs(10));
    }
}
