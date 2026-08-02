//! Asynchronous event triggers for background connection orchestration.
//!
//! This module defines the [`Trigger`] trait, permitting autonomous tasks or background workers
//! (e.g. status polling, cron intervals, external notification listeners) to interact with the
//! connection session asynchronously. Background orchestration of these tasks is handled via [`TriggerRunner`].

use crate::connection::Connection;
use futures_util::future::BoxFuture;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// What a trigger is allowed to do to the session.
///
/// Triggers used to receive the whole `AnyConnection`, which let a background
/// task disconnect the agent, answer a tool confirmation, or halt a turn the
/// user had just started. A trigger's job is to nudge the agent, so nudging is
/// all this exposes — mirroring upstream's one-method `TriggerContext`.
#[derive(Clone, Debug)]
pub struct TriggerContext {
    connection: crate::connection::AnyConnection,
}

impl TriggerContext {
    pub(crate) const fn new(connection: crate::connection::AnyConnection) -> Self {
        Self { connection }
    }

    /// Pushes a notification into the agent's queue.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection cannot accept the message.
    pub async fn send(&self, message: &str) -> Result<(), anyhow::Error> {
        self.connection.send_trigger_notification(message).await
    }
}

/// A trait for defining asynchronous background tasks that execute during a connection lifecycle.
pub trait Trigger: Send + Sync {
    /// Launches the trigger task.
    ///
    /// # Errors
    ///
    /// Returns an error if the background execution encounters a fatal issue.
    fn run(
        &self,
        context: TriggerContext,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;
}

/// Object-safe version of the [`Trigger`] trait, automatically implemented via a blanket impl.
///
/// This trait is used internally by the SDK to allow dynamic dispatch and storage of triggers.
pub trait DynTrigger: Send + Sync {
    /// Launches the trigger task.
    fn run(&self, context: TriggerContext) -> BoxFuture<'_, Result<(), anyhow::Error>>;
}

impl<T: Trigger + ?Sized> DynTrigger for T {
    fn run(&self, context: TriggerContext) -> BoxFuture<'_, Result<(), anyhow::Error>> {
        Box::pin(async move { self.run(context).await })
    }
}

/// Orchestrator for launching background [`Trigger`] loops.
pub struct TriggerRunner {
    /// Registered trigger instances.
    pub triggers: Vec<Arc<dyn DynTrigger>>,
    stop_tx: tokio::sync::watch::Sender<bool>,
    running: Arc<AtomicBool>,
}

impl std::fmt::Debug for TriggerRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TriggerRunner")
            .field("triggers_count", &self.triggers.len())
            .field("running", &self.is_running())
            .finish_non_exhaustive()
    }
}

impl TriggerRunner {
    /// Creates a new `TriggerRunner` initialized with the given list of triggers.
    pub fn new(triggers: Vec<Arc<dyn DynTrigger>>) -> Self {
        let (stop_tx, _) = tokio::sync::watch::channel(false);
        Self {
            triggers,
            stop_tx,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Whether the trigger tasks are live.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Spawns each registered trigger as a background task.
    ///
    /// # Errors
    ///
    /// Returns an error if the runner is already running. Starting twice used
    /// to double every trigger silently, so a heartbeat fired at twice its
    /// configured rate.
    pub fn start(
        &self,
        connection: &crate::connection::AnyConnection,
    ) -> Result<(), anyhow::Error> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(anyhow::anyhow!("the trigger runner is already running"));
        }
        for trigger in &self.triggers {
            let context = TriggerContext::new(connection.clone());
            let tr = trigger.clone();
            let mut stop_rx = self.stop_tx.subscribe();
            crate::spawn_task(async move {
                // Racing against the stop signal is what makes `stop()` work
                // for a trigger parked in a long sleep — triggers used to
                // outlive the agent entirely, still holding a connection.
                let run = std::pin::pin!(async move { tr.run(context).await });
                let stop = std::pin::pin!(async move {
                    let _ = stop_rx.wait_for(|stopped| *stopped).await;
                });
                if let futures_util::future::Either::Left((Err(e), _)) =
                    futures_util::future::select(run, stop).await
                {
                    tracing::error!("Trigger execution failed: {e:?}");
                }
            });
        }
        Ok(())
    }

    /// Signals every running trigger to stop.
    ///
    /// Idempotent, and safe to call on a runner that was never started.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        let _ = self.stop_tx.send(true);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::{Trigger, TriggerContext, TriggerRunner};
    use crate::connection::{AnyConnection, MockConnection};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct Counter(Arc<AtomicU32>);

    impl Trigger for Counter {
        async fn run(&self, _context: TriggerContext) -> Result<(), anyhow::Error> {
            loop {
                self.0.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    }

    fn mock() -> AnyConnection {
        AnyConnection::Mock(Arc::new(MockConnection::new("conv")))
    }

    #[tokio::test]
    async fn starting_twice_is_refused() {
        let runner = TriggerRunner::new(vec![]);
        let conn = mock();
        runner.start(&conn).unwrap();
        assert!(runner.is_running());
        runner
            .start(&conn)
            .expect_err("a second start would double every trigger");
        runner.stop();
        assert!(!runner.is_running());
    }

    #[tokio::test]
    async fn stop_halts_a_running_trigger() {
        let ticks = Arc::new(AtomicU32::new(0));
        let runner = TriggerRunner::new(vec![Arc::new(Counter(ticks.clone()))]);
        runner.start(&mock()).unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(60)).await;
        runner.stop();
        let after_stop = ticks.load(Ordering::SeqCst);
        assert!(after_stop > 0, "the trigger never ran");

        // Nothing more may be counted once stopped.
        tokio::time::sleep(std::time::Duration::from_millis(60)).await;
        assert_eq!(
            ticks.load(Ordering::SeqCst),
            after_stop,
            "the trigger outlived stop()"
        );
    }
}
