//! # Google Antigravity Rust SDK
//!
//! The Google Antigravity SDK is a Rust library for building AI agents powered by Antigravity and Gemini.
//! It provides a secure, scalable, and stateful infrastructure layer that abstracts the agentic loop,
//! letting you focus on what your agent *does* rather than how it runs.
//!
//! ## Core Architecture Components
//!
//! - **[`Agent`](crate::agent::Agent)**: High-level manager of the agent lifecycle. It handles process discovery,
//!   tool registration, hook dispatch, and starts the communication backend.
//! - **[`Conversation`](crate::conversation::Conversation)**: Stateful wrapper around a connection that
//!   tracks full step history, accumulates token usage metadata, and processes stream chunks.
//! - **[`Connection`](crate::connection::Connection)**: Core trait abstracted to communicate with the
//!   underlying harness (e.g. standard subprocess IPC or `WebSockets`).
//! - **[`Hook`](crate::hooks::Hook)**: Lifecycle hooks allowing you to observe or modify agent transitions
//!   and execute custom policies.
//! - **[`Policy`](crate::policy::Policy)**: Middleware structures enforcing security rules (e.g. restricting files inside workspace
//!   via `workspace_only` or prompting command runs via `confirm_run_command`).
//! - **[`Tool`](crate::tools::Tool)**: Custom function capabilities written in Rust and exposed to the model.
//! - **[`Trigger`](crate::triggers::Trigger)**: Asynchronous workers triggered at agent start.
//!
//! ## Quickstart Example
//!
//! ```no_run
//! use antigravity_sdk_rust::agent::Agent;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!     let agent = Agent::builder()
//!         .allow_all()
//!         .build();
//!     let agent = agent.start().await?;
//!
//!     let response = agent.chat("Say hello").await?;
//!     println!("Agent: {}", response.text);
//!
//!     agent.stop().await?;
//!     Ok(())
//! }
//! ```

pub mod proto {
    #[allow(
        warnings,
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::unwrap_used,
        clippy::expect_used
    )]
    pub mod localharness {
        // Include the prost-generated code
        include!(concat!(env!("OUT_DIR"), "/antigravity.localharness.rs"));
        // Include the pbjson-generated serde implementations
        include!(concat!(
            env!("OUT_DIR"),
            "/antigravity.localharness.serde.rs"
        ));
    }
}

pub mod agent;
pub mod connection;
pub mod context;
pub mod conversation;
pub mod error;
pub mod hooks;
#[cfg(not(target_arch = "wasm32"))]
pub mod local;
#[cfg(any(target_arch = "wasm32", test))]
pub mod wasm;

pub mod policy;
pub mod tool_context;
pub mod tools;
pub mod trigger_helpers;
pub mod triggers;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod interactive;

/// Helper to spawn asynchronous tasks in a target-agnostic manner.
pub fn spawn_task<F>(future: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::spawn(future);
    }
    #[cfg(target_arch = "wasm32")]
    {
        any_spawner::Executor::spawn_local(future);
    }
}
