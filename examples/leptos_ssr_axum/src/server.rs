use any_spawner::Executor as LeptosExecutor;
use leptos::config::get_configuration;
use leptos_wasi::{
    handler::HandlerError,
    prelude::{IncomingRequest, ResponseOutparam, WasiExecutor},
};
use wasi::exports::http::incoming_handler::Guest;
use wasi::http::proxy::export;

use crate::app::{
    shell, App, ClearMessages, GetMessages, SendMessage, SaveChatTurn,
    ListSessions, CreateSession, GetSessionBlocks, SaveTurnBlocks, DeleteSession,
    RenameSession, GetSession, ResolveSessionConfirmKv,
};

struct LeptosServer;

impl Guest for LeptosServer {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        let executor = WasiExecutor::new(leptos_wasi::executor::Mode::Stalled);
        if let Err(e) = LeptosExecutor::init_local_custom_executor(executor.clone()) {
            eprintln!("Got error while initializing leptos_wasi executor: {e:?}");
            return;
        }
        executor.run_until(async {
            if let Err(e) = handle_request(request, response_out).await {
                eprintln!("Got error while handling request: {e:?}");
            }
        })
    }
}

async fn handle_request(
    request: IncomingRequest,
    response_out: ResponseOutparam,
) -> Result<(), HandlerError> {
    use leptos_wasi::prelude::Handler;

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;

    Handler::build(request, response_out)?
        .with_server_fn::<SendMessage, _>()
        .with_server_fn::<GetMessages, _>()
        .with_server_fn::<ClearMessages, _>()
        .with_server_fn::<SaveChatTurn, _>()
        .with_server_fn::<ListSessions, _>()
        .with_server_fn::<CreateSession, _>()
        .with_server_fn::<GetSessionBlocks, _>()
        .with_server_fn::<GetSession, _>()
        .with_server_fn::<SaveTurnBlocks, _>()
        .with_server_fn::<DeleteSession, _>()
        .with_server_fn::<RenameSession, _>()
        .with_server_fn::<ResolveSessionConfirmKv, _>()
        .generate_routes(App)
        .handle_with_context(move || shell(leptos_options.clone()), || {})
        .await?;
    Ok(())
}

export!(LeptosServer with_types_in wasi);
