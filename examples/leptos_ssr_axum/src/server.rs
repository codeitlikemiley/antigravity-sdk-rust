use leptos::config::get_configuration;
use leptos_wasi::executor::init_wasip3_spawner;
use leptos_wasi::prelude::Handler;
use wasip3::http::types::{Request, Response, ErrorCode};

use crate::app::{
    shell, App, ClearMessages, GetMessages, SendMessage, SaveChatTurn,
    ListSessions, CreateSession, GetSessionBlocks, SaveTurnBlocks, DeleteSession,
    RenameSession,
};
use crate::types::AgentServerUrl;

struct LeptosServer;

impl wasip3::exports::http::handler::Guest for LeptosServer {
    async fn handle(request: Request) -> Result<Response, ErrorCode> {
        // 1. Initialize host async task scheduling
        let _ = init_wasip3_spawner();

        let conf = get_configuration(None).unwrap();
        let leptos_options = conf.leptos_options;

        // Convert the WASI request to http::Request
        let req = wasip3::http_compat::http_from_wasi_request(request)?;

        // Query the variable agent_server_url asynchronously using .await
        let agent_url_val = spin_sdk::variables::get("agent_server_url")
            .await
            .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
        let agent_url = AgentServerUrl(agent_url_val);

        // 2. Build and handle request natively
        let wasi_res = Handler::build(req).await
            .map_err(|e| {
                eprintln!("Error building handler: {:?}", e);
                ErrorCode::InternalError(None)
            })?
            .static_files_handler("/pkg", serve_static_files)
            .with_server_fn::<SendMessage>()
            .with_server_fn::<GetMessages>()
            .with_server_fn::<ClearMessages>()
            .with_server_fn::<SaveChatTurn>()
            .with_server_fn::<ListSessions>()
            .with_server_fn::<CreateSession>()
            .with_server_fn::<GetSessionBlocks>()
            .with_server_fn::<SaveTurnBlocks>()
            .with_server_fn::<DeleteSession>()
            .with_server_fn::<RenameSession>()
            .generate_routes(App)
            .handle_with_context(
                move || shell(leptos_options.clone()),
                move || {
                    leptos::prelude::provide_context(agent_url.clone());
                }
            )
            .await
            .map_err(|e| {
                eprintln!("Error handling request: {:?}", e);
                ErrorCode::InternalError(None)
            })?;

        Ok(wasi_res)
    }
}

fn serve_static_files(path: String) -> Option<leptos_wasi::response::Body> {
    use std::fs;
    let path = path.strip_prefix("/").unwrap_or(&path);
    // Wasmtime mounts site directory at root, so look at /path directly
    let file_path = format!("/{}", path);
    println!("serving static file: {}", file_path);

    if let Ok(bytes) = fs::read(&file_path) {
        Some(leptos_wasi::response::Body::Sync(bytes.into()))
    } else {
        println!("Could not read file at {}", file_path);
        None
    }
}

// Export the server for standard WASIp3 http trigger
wasip3::http::service::export!(LeptosServer);
