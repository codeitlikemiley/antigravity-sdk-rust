# Triggers

Background tasks and external events.

## Overview

Triggers are long-running async tasks that run concurrently with the agent
loop. They are spawned when the agent starts and can push notifications to the
agent via the connection. Common use cases include periodic health checks,
heartbeat monitors, file watchers, and external event listeners.

### Python SDK Comparison

The trigger concept is consistent across both SDKs. In Python, triggers are
registered via `Agent(triggers=[...])`. In Rust, triggers are registered
through the builder's `.trigger()` or `.triggers()` methods and implement the
`Trigger` trait.

---

## Trigger Trait

The `Trigger` trait defines a single `run` method that receives the active
connection and executes for the lifetime of the agent:

```rust,no_run
use antigravity_sdk_rust::connection::AnyConnection;

/// A trait for defining asynchronous background tasks that execute
/// during a connection lifecycle.
pub trait Trigger: Send + Sync {
    /// Launches the trigger task with the active connection.
    ///
    /// This method runs for the lifetime of the agent. Use the connection
    /// to send notifications back to the agent.
    async fn run(&self, connection: AnyConnection) -> Result<(), anyhow::Error>;
}
```

The connection parameter gives triggers access to
`send_trigger_notification()`, which pushes a message string into the agent's
event stream.

---

## DynTrigger (Object-Safe Wrapper)

Like the `Hook`/`DynHook` pattern, triggers have an object-safe counterpart
that wraps the async `run` method in a `BoxFuture`:

```rust,no_run
use futures_util::future::BoxFuture;
use antigravity_sdk_rust::connection::AnyConnection;

/// Object-safe version of `Trigger`, automatically implemented
/// via a blanket impl for any `T: Trigger`.
pub trait DynTrigger: Send + Sync {
    fn run(&self, connection: AnyConnection) -> BoxFuture<'_, Result<(), anyhow::Error>>;
}

// Blanket impl: any type implementing Trigger automatically implements DynTrigger.
// impl<T: Trigger + ?Sized> DynTrigger for T { ... }
```

You always implement `Trigger` — the SDK handles the `DynTrigger` conversion
automatically.

---

## TriggerRunner

The `TriggerRunner` orchestrates the spawning and lifecycle of all registered
triggers:

```rust,no_run
use antigravity_sdk_rust::triggers::TriggerRunner;
use std::sync::Arc;

// TriggerRunner stores triggers as Vec<Arc<dyn DynTrigger>>
```

### API

| Method | Description |
|---|---|
| `TriggerRunner::new(triggers)` | Creates a runner wrapping a `Vec<Arc<dyn DynTrigger>>` |
| `runner.start(connection)` | Spawns each trigger as an independent tokio task |

When `start()` is called, each trigger is cloned (via `Arc`) and spawned into
its own `tokio::spawn` block. If a trigger's `run` method returns an error,
it is logged via `tracing::error!` but does not crash the agent.

> [!NOTE]
> Triggers run independently — one trigger failing does not affect others.
> Errors are logged but silently swallowed to maintain agent stability.

---

## Trigger Helpers

### `every()` — Periodic Timer

The `every()` factory function creates a trigger that fires at regular
intervals, sending a message to the agent each time:

```rust,no_run
use antigravity_sdk_rust::trigger_helpers::every;
use std::time::Duration;

// Send "check_status" to the agent every 30 seconds
let heartbeat = every(Duration::from_secs(30), "check_status");

// With a custom message
let monitor = every(Duration::from_millis(500), "fast_poll");
```

The returned `PeriodicTrigger` loops indefinitely:

```text
loop {
    sleep(interval)
    connection.send_trigger_notification(message)
}
```

### `PeriodicTrigger` Internals

```rust,no_run
use std::time::Duration;

/// A trigger that fires at regular intervals.
/// Created via `every()`.
pub struct PeriodicTrigger {
    /// How long to wait between notifications.
    interval: Duration,
    /// The message sent to the agent on each tick.
    message: String,
}
```

---

## Types

### TriggerDelivery

Controls when trigger notifications are delivered to the agent:

```rust,no_run
/// Controls when trigger notifications are delivered to the agent.
#[derive(Debug, Clone, Copy)]
pub enum TriggerDelivery {
    /// Deliver the notification immediately, even if the agent is busy.
    SendImmediately,
    /// Wait until the agent is idle before delivering.
    WaitIdle,
}
```

### FileChange

Represents a single filesystem change event, useful for file-watching triggers:

```rust,no_run
/// The kind of filesystem change detected by a file-watching trigger.
#[derive(Debug, Clone, Copy)]
pub enum FileChangeKind {
    /// A new file was created.
    Added,
    /// An existing file was modified.
    Modified,
    /// A file was deleted.
    Deleted,
}

/// A single filesystem change event.
#[derive(Debug, Clone)]
pub struct FileChange {
    /// The type of change.
    pub kind: FileChangeKind,
    /// The path of the affected file.
    pub path: String,
}
```

---

## Examples

### 1. Heartbeat Trigger

The simplest trigger — periodic status checks using the built-in `every()` helper:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::trigger_helpers::every;
use antigravity_sdk_rust::triggers::DynTrigger;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let heartbeat = every(Duration::from_secs(60), "heartbeat_check");

    let agent = Agent::builder()
        .trigger(Arc::new(heartbeat) as Arc<dyn DynTrigger>)
        .allow_all()
        .build();

    let agent = agent.start().await?;
    // The heartbeat trigger is now running in the background,
    // sending "heartbeat_check" every 60 seconds.

    let response = agent.chat("Monitor the system").await?;
    println!("{}", response.text);

    agent.stop().await?;
    Ok(())
}
```

### 2. Custom Monitoring Trigger

A trigger that monitors a resource and sends notifications when conditions change:

```rust,no_run
use antigravity_sdk_rust::connection::AnyConnection;
use antigravity_sdk_rust::connection::Connection;
use antigravity_sdk_rust::triggers::Trigger;
use std::time::Duration;

struct DiskSpaceMonitor {
    path: String,
    threshold_mb: u64,
    check_interval: Duration,
}

impl DiskSpaceMonitor {
    fn new(path: impl Into<String>, threshold_mb: u64) -> Self {
        Self {
            path: path.into(),
            threshold_mb,
            check_interval: Duration::from_secs(300), // every 5 minutes
        }
    }
}

impl Trigger for DiskSpaceMonitor {
    async fn run(&self, connection: AnyConnection) -> Result<(), anyhow::Error> {
        loop {
            tokio::time::sleep(self.check_interval).await;

            // Simulate checking disk space
            // In production, use a filesystem API
            let available_mb = 500u64; // placeholder

            if available_mb < self.threshold_mb {
                let msg = format!(
                    "⚠️ Low disk space on {}: {}MB remaining (threshold: {}MB)",
                    self.path, available_mb, self.threshold_mb
                );
                connection.send_trigger_notification(&msg).await?;
            }
        }
    }
}
```

### 3. Builder Integration

Registering multiple triggers with the agent builder:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::trigger_helpers::every;
use antigravity_sdk_rust::triggers::DynTrigger;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let agent = Agent::builder()
        // Add individual triggers
        .trigger(Arc::new(every(Duration::from_secs(30), "health_check")) as Arc<dyn DynTrigger>)
        .trigger(Arc::new(every(Duration::from_secs(300), "metrics_report")) as Arc<dyn DynTrigger>)
        .allow_all()
        .build();

    let agent = agent.start().await?;
    // Both triggers are now running concurrently in the background.

    agent.stop().await?;
    Ok(())
}
```

Or register after construction but before starting:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::trigger_helpers::every;
use antigravity_sdk_rust::triggers::DynTrigger;
use std::sync::Arc;
use std::time::Duration;

let mut agent = Agent::builder().allow_all().build();
agent.register_trigger(
    Arc::new(every(Duration::from_secs(60), "late_trigger")) as Arc<dyn DynTrigger>
).expect("trigger registration failed");
// agent.start().await?;
```

### 4. Batch Triggers via `.triggers()`

You can set all triggers at once using the plural form:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::trigger_helpers::every;
use antigravity_sdk_rust::triggers::DynTrigger;
use std::sync::Arc;
use std::time::Duration;

let all_triggers: Vec<Arc<dyn DynTrigger>> = vec![
    Arc::new(every(Duration::from_secs(30), "check_a")),
    Arc::new(every(Duration::from_secs(60), "check_b")),
    Arc::new(every(Duration::from_secs(120), "check_c")),
];

let agent = Agent::builder()
    .triggers(all_triggers)
    .allow_all()
    .build();
```
