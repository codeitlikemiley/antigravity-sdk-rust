
use components::{Route, Router, Routes};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::*;
use leptos_router::*;

use crate::types::{
    ChatMessage, ClientToolCall, MessageBlock, ToolCallStatus,
    AskQuestionEntry, QuestionResponse, SessionMeta,
};

#[cfg(feature = "hydrate")]
use crate::types::{AnswerPayload, ConfirmPayload};

#[cfg(feature = "ssr")]
use crate::types::{ChatSession, SessionIndex};

#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    let agent_url = leptos::prelude::use_context::<crate::types::AgentServerUrl>()
        .map(|url| url.0)
        .unwrap_or_else(|| "http://127.0.0.1:8080".to_string());
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="agent-server-url" content=agent_url />
                <link rel="preconnect" href="https://fonts.googleapis.com" />
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="true" />
                <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet" />
                <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
                <script type="module" inner_html="import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs'; window.mermaid = mermaid; mermaid.initialize({ startOnLoad: false, theme: 'dark', securityLevel: 'loose' });" />
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() root="" />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let fallback = || view! { "Page not found." }.into_view();

    view! {
        <Stylesheet id="leptos" href="/pkg/leptos_ssr_axum.css" />
        <Meta
            name="description"
            content="Antigravity SDK Chat — AI Agent powered by Gemini, running on Spin WASI"
        />

        <Title text="Antigravity Chat" />

        <Router>
            <main>
                <Routes fallback>
                    <Route path=path!("") view=ChatPage />
                    <Route path=path!("/*any") view=NotFound />
                </Routes>
            </main>
        </Router>
    }
}/// Renders a user bubble (right-aligned, gray background).
#[component]
fn UserMessageView(content: String) -> impl IntoView {
    view! {
        <div class="user-message-container flex justify-end w-full gap-4">
            <div class="flex flex-col items-end max-w-[85%]">
                <div class="user-message-header text-[11px] text-gray-400 dark:text-gray-500 mb-1 font-medium select-none">
                    "You"
                </div>
                <div class="user-message-bubble bg-gray-100 dark:bg-[#2f2f2f] text-[#0d0d0d] dark:text-[#ececec] rounded-2xl px-4 py-2.5 text-sm leading-relaxed whitespace-pre-wrap border border-gray-200/50 dark:border-gray-700/30 shadow-sm">
                    {content}
                </div>
            </div>
            <div class="flex-shrink-0 w-8 h-8 rounded-full bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 flex items-center justify-center text-sm font-semibold select-none shadow-sm">
                "U"
            </div>
        </div>
    }
}

/// Renders assistant markdown response (left-aligned).
#[component]
fn AssistantMessageView(content: String) -> impl IntoView {
    let ws = use_context::<ReadSignal<Option<String>>>()
        .map(|sig| sig.get())
        .flatten();
    let agent_url = get_agent_server_url_any();
    let html_content = render_markdown(&content, ws.as_deref(), &agent_url);
    view! {
        <div class="assistant-message-container flex justify-start w-full gap-4">
            <div class="flex-shrink-0 w-8 h-8 rounded-full bg-black dark:bg-white text-white dark:text-black flex items-center justify-center text-sm font-semibold select-none shadow-md">
                "A"
            </div>
            <div class="flex flex-col items-start max-w-[85%] w-full">
                <div class="assistant-message-header text-[11px] text-gray-400 dark:text-gray-500 mb-1 font-medium select-none">
                    "Agent"
                </div>
                <div 
                    class="markdown-content text-[#0d0d0d] dark:text-[#ececec] leading-relaxed text-sm w-full"
                    inner_html=html_content
                />
            </div>
        </div>
    }
}

/// Collapsible details block showing the reasoning process.
#[component]
fn ThinkingView(content: String, is_streaming: bool) -> impl IntoView {
    if content.trim().is_empty() {
        return ().into_any();
    }
    
    // Render the thought content as markdown so headers, code blocks, bullets etc.
    // all display properly inside the collapsible panel.
    let ws = use_context::<ReadSignal<Option<String>>>()
        .map(|sig| sig.get())
        .flatten();
    let agent_url = get_agent_server_url_any();
    let html_content = render_markdown(&content, ws.as_deref(), &agent_url);
    let word_count = content.split_whitespace().count();
    let char_count = content.len();
    let stats = format!("{} chars \u{00B7} {} words", char_count, word_count);

    view! {
        <details 
            class="thinking-details w-full mb-4 border border-amber-200/50 dark:border-amber-950/20 bg-amber-50/10 dark:bg-amber-950/5 rounded-xl overflow-hidden shadow-sm transition-all duration-300"
            open=is_streaming
        >
            <summary class="flex items-center justify-between px-4 py-2.5 cursor-pointer font-medium text-xs text-amber-600 dark:text-amber-400 bg-amber-50/40 dark:bg-amber-950/10 hover:bg-amber-100/50 dark:hover:bg-amber-950/20 transition-colors select-none">
                <div class="flex items-center gap-2">
                    {if is_streaming {
                        view! {
                            <svg class="w-3.5 h-3.5 flex-shrink-0 animate-spin text-amber-600 dark:text-amber-400" fill="none" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                        }.into_any()
                    } else {
                        view! {
                            <span class="thinking-icon text-sm select-none">
                                "\u{1F9E0}"
                            </span>
                        }.into_any()
                    }}
                    <span class="tracking-wide uppercase font-semibold text-[10px]">
                        {if is_streaming { "Thinking process..." } else { "Thought process" }}
                    </span>
                </div>
                <span class="text-[10px] opacity-75 font-mono">
                    {move || if is_streaming { "active".to_string() } else { stats.clone() }}
                </span>
            </summary>
            <div 
                class="p-4 text-xs text-gray-600 dark:text-gray-400 border-t border-amber-100/50 dark:border-amber-950/10 leading-relaxed max-h-[350px] overflow-y-auto markdown-content"
                inner_html=html_content
            />
        </details>
    }.into_any()
}

/// Shortens a file path for display: strips file:// prefix, replaces home dir with ~,
/// and truncates the middle of long paths keeping both ends visible.
fn shorten_path(path: &str) -> String {
    // Strip file:// or file:/// URI prefix
    let path = if let Some(p) = path.strip_prefix("file:///") {
        format!("/{p}")
    } else if let Some(p) = path.strip_prefix("file://") {
        p.to_string()
    } else {
        path.to_string()
    };
    // Replace home directory with ~
    let home = std::env::var("HOME").unwrap_or_default();
    let path = if !home.is_empty() && path.starts_with(&home) {
        format!("~{}", &path[home.len()..])
    } else {
        path
    };
    // Truncate very long paths keeping first 24 and last 28 chars
    if path.len() > 55 {
        format!("{}...{}", &path[..24], &path[path.len() - 28..])
    } else {
        path
    }
}

/// Expandable tool call card.
///
/// - `open=true`  when Running  (details visible while agent works)
/// - `open=false` when Done/Error (collapses; label becomes the collapsed title)
/// The `label` comes from the agent's step description (e.g. "Change Directory").
#[component]
fn ToolCallView(
    id: u64,
    call_id: String,
    name: String,
    args: serde_json::Value,
    status: ToolCallStatus,
    canonical_path: Option<String>,
    label: Option<String>,
    subagent_blocks: Vec<MessageBlock>,
    on_answer: Callback<(u64, Vec<QuestionResponse>, bool)>,
) -> impl IntoView {
    let _ = id;
    let _ = call_id;
    let args_str = serde_json::to_string_pretty(&args).unwrap_or_else(|_| args.to_string());

    // Prefer the human-readable label; fall back to raw tool name.
    let display_title = label
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(&name)
        .to_string();

    let tool_icon = match name.as_str() {
        "LIST_DIR" => view! {
            <svg class="w-3.5 h-3.5 flex-shrink-0 opacity-60" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
            </svg>
        }.into_any(),
        "FIND_FILE" | "SEARCH_DIR" => view! {
            <svg class="w-3.5 h-3.5 flex-shrink-0 opacity-60" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" />
            </svg>
        }.into_any(),
        "RUN_COMMAND" => view! {
            <svg class="w-3.5 h-3.5 flex-shrink-0 opacity-60" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.348a1.125 1.125 0 010 1.971l-11.54 6.347a1.125 1.125 0 01-1.667-.985V5.653z" />
            </svg>
        }.into_any(),
        "VIEW_FILE" => view! {
            <svg class="w-3.5 h-3.5 flex-shrink-0 opacity-60" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
            </svg>
        }.into_any(),
        "EDIT_FILE" | "CREATE_FILE" => view! {
            <svg class="w-3.5 h-3.5 flex-shrink-0 opacity-60" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L6.832 19.82a4.5 4.5 0 01-1.897 1.13l-2.685.8.8-2.685a4.5 4.5 0 011.13-1.897L16.863 4.487zm0 0L19.5 7.125" />
            </svg>
        }.into_any(),
        "START_SUBAGENT" => view! {
            <svg class="w-3.5 h-3.5 flex-shrink-0 opacity-60" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 3v1.5M12 3v1.5m3.75-1.5v1.5M19.5 8.25h-1.5M19.5 12h-1.5m1.5 3.75h-1.5m-14.25-3.75h-1.5m1.5-3.75h-1.5m1.5 7.5h-1.5m3 3V21M12 19.5V21m3.75-1.5V21m-9-15h10.5a1.5 1.5 0 011.5 1.5v10.5a1.5 1.5 0 01-1.5 1.5H6.75a1.5 1.5 0 01-1.5-1.5V6.75a1.5 1.5 0 011.5-1.5zm2.25 4.5h4.5v4.5h-4.5v-4.5z" />
            </svg>
        }.into_any(),
        "GENERATE_IMAGE" => view! {
            <svg class="w-3.5 h-3.5 flex-shrink-0 opacity-60" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 001.5-1.5V6a1.5 1.5 0 00-1.5-1.5H3.75A1.5 1.5 0 002.25 6v12a1.5 1.5 0 001.5 1.5zm10.5-11.25h.008v.008h-.008V8.25zm.375 0a.375.375 0 11-.75 0 .375.375 0 01.75 0z" />
            </svg>
        }.into_any(),
        _ => view! {
            <svg class="w-3.5 h-3.5 flex-shrink-0 opacity-60" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M21.75 6.75a4.5 4.5 0 01-4.884 4.484c-1.076-.091-2.264.071-2.95.904l-7.152 8.684a2.548 2.548 0 11-3.586-3.586l8.684-7.152c.833-.686.995-1.874.904-2.95a4.5 4.5 0 016.336-4.486l-3.276 3.276a3.004 3.004 0 002.25 2.25l3.276-3.276c.256.565.398 1.192.398 1.852z" />
            </svg>
        }.into_any(),
    };

    // `<details>` is open while the tool is actively running so the user can
    // watch the arguments in real time; it collapses automatically when done.
    let is_open = matches!(status, ToolCallStatus::Running);    // Theme tokens per status
    let (border_cls, summary_cls, badge_bg, badge_text) = match status {
        ToolCallStatus::Running => (
            "border-amber-300/70 dark:border-amber-800/50",
            "bg-amber-50/60 dark:bg-amber-950/20 text-amber-800 dark:text-amber-300 hover:bg-amber-100/60 dark:hover:bg-amber-950/30",
            "bg-amber-500",
            "Running",
        ),
        ToolCallStatus::Done => (
            "border-emerald-200/70 dark:border-emerald-900/40",
            "bg-emerald-50/40 dark:bg-emerald-950/15 text-emerald-800 dark:text-emerald-300 hover:bg-emerald-50/70 dark:hover:bg-emerald-950/25",
            "bg-emerald-500",
            "Done",
        ),
        ToolCallStatus::Error => (
            "border-red-300/70 dark:border-red-900/50",
            "bg-red-50/60 dark:bg-[#251515]/40 text-red-700 dark:text-red-400 hover:bg-red-100/60 dark:hover:bg-red-950/30",
            "bg-red-500",
            "Failed",
        ),
    };

    let status_icon = match status {
        ToolCallStatus::Running => view! {
            <svg class="w-3.5 h-3.5 flex-shrink-0 animate-spin" fill="none" stroke="currentColor" stroke-width="2.5" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99" />
            </svg>
        }.into_any(),
        ToolCallStatus::Done => view! {
            <svg class="w-3.5 h-3.5 flex-shrink-0" fill="none" stroke="currentColor" stroke-width="3" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
            </svg>
        }.into_any(),
        ToolCallStatus::Error => view! {
            <svg class="w-3.5 h-3.5 flex-shrink-0" fill="none" stroke="currentColor" stroke-width="3" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
        }.into_any(),
    };

    // Compact tool-name chip so the raw identifier is always visible
    let name_chip_cls = match status {
        ToolCallStatus::Running => "bg-amber-100 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300 border-amber-200 dark:border-amber-800/40",
        ToolCallStatus::Done    => "bg-emerald-100 dark:bg-emerald-950/30 text-emerald-700 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800/30",
        ToolCallStatus::Error   => "bg-red-100 dark:bg-red-950/30 text-red-700 dark:text-red-400 border-red-200 dark:border-red-800/30",
    };
    let name_chip = name.clone();

    // Contextual subtitle: query (for SEARCH_DIR), command (for RUN_COMMAND), or path (for file tools)
    let subtitle = if name == "SEARCH_DIR" {
        // Prefer showing the search query — more informative than the directory path
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .map(|s| if s.len() > 40 { format!("\"{}…\"", &s[..40]) } else { format!("\"{}\"", s) });
        if let Some(q_str) = query {
            view! { <span class="font-mono text-[10px] opacity-60 truncate max-w-[220px]">{q_str}</span> }.into_any()
        } else { ().into_any() }
    } else if let Some(ref path) = canonical_path {
        let short = shorten_path(path);
        view! {
            <span class="font-mono text-[10px] opacity-70 truncate max-w-[220px]" title=path.clone()>
                {short}
            </span>
        }.into_any()
    } else if name == "RUN_COMMAND" {
        let cmd = args.get("command_line")
            .or_else(|| args.get("CommandLine"))
            .and_then(|v| v.as_str())
            .map(|s| if s.len() > 40 { format!("{}…", &s[..40]) } else { s.to_string() });
        if let Some(cmd_str) = cmd {
            view! { <span class="font-mono text-[10px] opacity-60 truncate max-w-[220px]">{cmd_str}</span> }.into_any()
        } else { ().into_any() }
    } else if name == "GENERATE_IMAGE" {
        let prompt = args.get("prompt")
            .and_then(|v| v.as_str())
            .map(|s| if s.len() > 50 { format!("\"{}…\"", &s[..50]) } else { format!("\"{}\"", s) });
        if let Some(p_str) = prompt {
            view! { <span class="font-mono text-[10px] opacity-60 truncate max-w-[220px]">{p_str}</span> }.into_any()
        } else { ().into_any() }
    } else { ().into_any() };

    view! {
        <details
            open=is_open
            class=format!("tool-call-details w-full mb-2 border rounded-xl overflow-hidden shadow-sm {}", border_cls)
        >
            <summary class=format!(
                "flex items-center justify-between px-3 py-2 cursor-pointer text-xs select-none transition-colors {}",
                summary_cls
            )>
                // Left: wrench icon + status icon + human-readable title + raw tool chip + subtitle
                <div class="flex items-center gap-2 flex-1 min-w-0">
                    // Dynamic tool call icon
                    {tool_icon}
                    {status_icon}
                    // Primary: intent label ("Change Directory") not raw name
                    <span class="font-semibold truncate">{display_title}</span>
                    // Secondary: raw tool name chip
                    <span class=format!(
                        "font-mono text-[9px] px-1.5 py-0.5 rounded border font-bold uppercase tracking-wide flex-shrink-0 {}",
                        name_chip_cls
                    )>
                        {name_chip}
                    </span>
                    {subtitle}
                </div>
                // Right: status badge
                <span class=format!(
                    "flex-shrink-0 text-[9px] px-2 py-0.5 rounded-full text-white font-bold {}",
                    badge_bg
                )>
                    {badge_text}
                </span>
            </summary>
            // Arguments panel — visible when open
            <div class="p-3 text-xs font-mono border-t border-gray-200/40 dark:border-gray-800/30 bg-gray-50/30 dark:bg-[#181818]/10 overflow-x-auto">
                <div class="text-[10px] text-gray-400 dark:text-gray-500 mb-1.5 font-semibold uppercase tracking-wider">"Arguments:"</div>
                <pre class="whitespace-pre overflow-x-auto leading-relaxed text-gray-700 dark:text-gray-300">{args_str}</pre>
            </div>

            // Subagent trajectory container (nested timelines)
            {if !subagent_blocks.is_empty() {
                let sub_blocks = subagent_blocks.clone();
                let on_answer_cb = on_answer.clone();
                view! {
                    <div class="px-3 pb-3 border-t border-gray-200/40 dark:border-gray-800/30 bg-white/5">
                        <div class="mt-3 pl-3 border-l border-dashed border-gray-300 dark:border-zinc-700/60 space-y-3">
                            <div class="text-[10px] text-gray-400 dark:text-zinc-500 font-semibold uppercase tracking-wider mb-1 flex items-center gap-1.5">
                                <span class="w-1.5 h-1.5 rounded-full bg-blue-500 animate-pulse"></span>
                                "Subagent Trajectory"
                            </div>
                            {sub_blocks.into_iter().map(|block| {
                                let cb = on_answer_cb.clone();
                                view! {
                                    <MessageBlockView block=block on_answer=cb />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }.into_any()
            } else {
                ().into_any()
            }}
        </details>
    }
}

/// Renders file diff output with syntax-highlighted +/- lines.
#[component]
fn DiffView(name: String, content: String) -> impl IntoView {
    let lines = content.lines().map(|s| s.to_string()).collect::<Vec<String>>();
    view! {
        <div class="diff-view-wrapper mb-4 border border-gray-200 dark:border-gray-800 rounded-xl overflow-hidden shadow-sm">
            <div class="diff-header flex items-center justify-between px-4 py-2 bg-[#eaeaea] dark:bg-[#252525] border-b border-gray-200 dark:border-gray-800 text-xs font-mono text-gray-500 dark:text-gray-400 select-none">
                <span class="font-bold tracking-wide text-[10px] uppercase">"File Changes (Diff)"</span>
                <span class="text-[9px] px-1.5 py-0.5 rounded bg-gray-200 dark:bg-gray-800 text-gray-600 dark:text-gray-400 font-semibold">{name}</span>
            </div>
            <div class="diff-lines-container font-mono text-[11px] leading-normal overflow-x-auto max-h-[350px] bg-[#fdfdfd] dark:bg-[#151515]">
                {lines.into_iter().enumerate().map(|(idx, line)| {
                    let is_addition = line.starts_with('+');
                    let is_deletion = line.starts_with('-');
                    let row_cls = if is_addition {
                        "bg-green-50 dark:bg-green-950/20 text-green-800 dark:text-green-305 border-l-4 border-green-500"
                    } else if is_deletion {
                        "bg-red-50 dark:bg-red-950/20 text-red-800 dark:text-red-305 border-l-4 border-red-500"
                    } else {
                        "text-gray-500 dark:text-gray-400 border-l-4 border-transparent"
                    };
                    view! {
                        <div class=format!("diff-line flex w-full hover:bg-gray-100/50 dark:hover:bg-gray-900/50 px-4 py-0.5 {}", row_cls)>
                            <span class="w-8 flex-shrink-0 text-[9px] opacity-40 text-right pr-3 select-none">{idx + 1}</span>
                            <span class="whitespace-pre">{line}</span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

/// Collapsible output panel shown below a completed ToolCallView.
#[component]
fn ToolResultView(
    name: String,
    result: Option<serde_json::Value>,
    error: Option<String>,
) -> impl IntoView {
    let ws = use_context::<ReadSignal<Option<String>>>()
        .map(|sig| sig.get())
        .flatten();

    // ── GENERATE_IMAGE: extract image_paths for inline rendering ────────
    let image_paths: Vec<String> = if name == "GENERATE_IMAGE" {
        extract_image_paths_from_result(result.as_ref())
    } else {
        Vec::new()
    };

    // Extract the most human-readable output from the result JSON.
    // For RUN_COMMAND: combined_output (stdout/stderr) + exit code.
    // For file tools: success note or content preview.
    // Fallback: pretty-printed JSON.
    let result_str = if let Some(ref res) = result {
        if name == "RUN_COMMAND" {
            // combined_output is the merged stdout+stderr from the harness
            let output = res.get("combined_output")
                .or_else(|| res.get("CombinedOutput"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let exit_code = res.get("exit_code")
                .or_else(|| res.get("ExitCode"))
                .and_then(|v| v.as_i64())
                .map(|c| format!(" (exit {})", c))
                .unwrap_or_default();
            if output.is_empty() {
                format!("(no output{})", exit_code)
            } else {
                format!("{}{}", output.trim_end(), exit_code)
            }
        } else if name == "SEARCH_DIR" {
            // output is the grep results text from the harness, packed into args.output
            res.get("output")
                .and_then(|v| v.as_str())
                .unwrap_or("(no results)")
                .to_string()
        } else if name == "VIEW_FILE" {
            res.get("content")
                .or_else(|| res.get("Content"))
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| res.as_str().unwrap_or(""))
                .to_string()
        } else if matches!(name.as_str(), "EDIT_FILE" | "CREATE_FILE") {
            // File write tools return minimal confirmation; content shown as diff
            if let Some(s) = res.as_str() {
                s.to_string()
            } else {
                serde_json::to_string_pretty(res).unwrap_or_else(|_| res.to_string())
            }
        } else if name == "GENERATE_IMAGE" {
            // Image rendering is handled separately below; show image_name as text
            let img_name = res.get("image_name")
                .and_then(|v| v.as_str())
                .unwrap_or("image");
            let count = image_paths.len();
            if count > 0 {
                format!("Generated {} image(s): {}", count, img_name)
            } else {
                "(no images generated)".to_string()
            }
        } else if let Some(s) = res.as_str() {
            s.to_string()
        } else {
            serde_json::to_string_pretty(res).unwrap_or_else(|_| res.to_string())
        }
    } else {
        String::new()
    };

    let is_diff = (name == "EDIT_FILE" || name == "replace_file_content" || name == "multi_replace_file_content" || name == "write_to_file")
        && (result_str.contains("\n+") || result_str.contains("\n-"));

    let has_error = error.is_some();
    let content_cls = if has_error {
        "text-red-750 dark:text-red-400 bg-red-50/50 dark:bg-[#251515]/20 border-t border-red-200 dark:border-red-900/30"
    } else {
        "text-gray-700 dark:text-gray-300 bg-gray-50/50 dark:bg-[#1a1a1a]/20 border-t border-gray-200 dark:border-gray-800/30"
    };

    // ── GENERATE_IMAGE with images: render an inline image gallery ──────
    let is_image_result = name == "GENERATE_IMAGE" && !image_paths.is_empty() && !has_error;

    view! {
        {if is_image_result {
            let paths = image_paths.clone();
            view! {
                <div class="generate-image-result w-full mb-4 border border-violet-200/60 dark:border-violet-900/40 rounded-2xl overflow-hidden shadow-md">
                    // Header
                    <div class="flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-violet-50 to-fuchsia-50 dark:from-violet-950/20 dark:to-fuchsia-950/20 border-b border-violet-200/40 dark:border-violet-800/30 select-none">
                        <svg class="w-4 h-4 text-violet-500 dark:text-violet-400" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 001.5-1.5V6a1.5 1.5 0 00-1.5-1.5H3.75A1.5 1.5 0 002.25 6v12a1.5 1.5 0 001.5 1.5zm10.5-11.25h.008v.008h-.008V8.25zm.375 0a.375.375 0 11-.75 0 .375.375 0 01.75 0z" />
                        </svg>
                        <span class="font-bold tracking-wide text-[10px] uppercase text-violet-700 dark:text-violet-300">"Generated Image"</span>
                        <span class="text-[9px] px-1.5 py-0.5 rounded bg-violet-100 dark:bg-violet-900/40 text-violet-600 dark:text-violet-400 font-semibold">
                            {format!("{} file(s)", paths.len())}
                        </span>
                    </div>
                    // Image grid
                    <div class="p-4 bg-gray-50/30 dark:bg-[#141414]/40">
                        <div class="grid grid-cols-1 gap-4">
                            {paths.into_iter().map(|path| {
                                // Build the image source URL via agent server /media endpoint.
                                let abs_path = if !path.starts_with('/') {
                                    if let Some(ref ws_str) = ws {
                                        format!("{}/{}", ws_str.trim_end_matches('/'), path)
                                    } else {
                                        path.clone()
                                    }
                                } else {
                                    path.clone()
                                };

                                let base = get_agent_server_url_any();
                                let encoded = percent_encode_path(&abs_path);
                                let img_src = format!("{}/media?path={}", base, encoded);

                                let file_name = path.rsplit('/').next().unwrap_or(&path).to_string();

                                view! {
                                    <div class="group relative rounded-xl overflow-hidden border border-gray-200/60 dark:border-gray-800/50 shadow-sm hover:shadow-lg transition-all duration-300 bg-white dark:bg-[#1a1a1a]">
                                        <img
                                            src=img_src.clone()
                                            alt=file_name.clone()
                                            class="w-full h-auto max-h-[500px] object-contain bg-[#f5f5f5] dark:bg-[#0d0d0d]"
                                            loading="lazy"
                                        />
                                        // Overlay with filename and actions on hover
                                        <div class="absolute bottom-0 left-0 right-0 flex items-center justify-between px-3 py-2 bg-gradient-to-t from-black/60 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                                            <span class="text-[10px] font-mono text-white/90 truncate max-w-[60%]">{file_name.clone()}</span>
                                            <div class="flex items-center gap-2">
                                                <a
                                                    href=img_src
                                                    target="_blank"
                                                    rel="noopener"
                                                    class="text-[9px] px-2 py-1 rounded bg-white/20 text-white hover:bg-white/30 transition-colors font-semibold"
                                                >
                                                    "Open"
                                                </a>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>
            }.into_any()
        } else if is_diff {
            view! {
                <DiffView name=name.clone() content=result_str.clone() />
            }.into_any()
        } else {
            view! {
                <details 
                    class=format!("tool-result-details w-full mb-4 border rounded-xl overflow-hidden shadow-sm transition-all duration-300 {}",
                        if has_error { "border-red-200 dark:border-red-900/40" } else { "border-gray-200 dark:border-gray-800" }
                    )
                >
                    <summary 
                        class=format!("flex items-center gap-2 px-4 py-2 cursor-pointer font-medium text-xs select-none {}",
                            if has_error { "bg-red-50/40 dark:bg-[#251515] text-red-700 dark:text-red-400" } else { "bg-gray-50 dark:bg-[#1c1c1c] text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-[#252525]" }
                        )
                    >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" stroke-width="2.5" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v6m3-3H9m12 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <span class="font-bold tracking-wide text-[10px] uppercase">
                            {if has_error { "Output (Failed)" } else { "Output (Success)" }}
                        </span>
                    </summary>
                    <div class=format!("p-4 text-xs font-mono whitespace-pre-wrap max-h-[300px] overflow-y-auto leading-relaxed {}", content_cls)>
                        {if has_error {
                            error.clone().unwrap_or_default()
                        } else {
                            result_str
                        }}
                    </div>
                </details>
            }.into_any()
        }}
    }
}


/// Interactive form that pauses the stream for agent questions.
#[component]
#[allow(unused_variables)]
fn QuestionView(
    id: u64,
    trajectory_id: String,
    step_index: u32,
    questions: Vec<AskQuestionEntry>,
    answered: bool,
    on_answer: Callback<(u64, Vec<QuestionResponse>, bool)>,
) -> impl IntoView {
    let (selections, set_selections) = signal(
        questions.iter().map(|_| {
            (Vec::<String>::new(), String::new())
        }).collect::<Vec<_>>()
    );

    let on_submit = Callback::new({
        move |_: leptos::web_sys::MouseEvent| {
            let current = selections.get();
            let responses = current.into_iter().map(|(opt_ids, freeform)| {
                QuestionResponse {
                    selected_option_ids: if opt_ids.is_empty() { None } else { Some(opt_ids) },
                    freeform_response: freeform,
                    skipped: false,
                }
            }).collect::<Vec<_>>();
            on_answer.run((id, responses, false));
        }
    });

    let on_skip = Callback::new({
        let questions = questions.clone();
        move |_: leptos::web_sys::MouseEvent| {
            let responses = questions.iter().map(|_| {
                QuestionResponse {
                    selected_option_ids: None,
                    freeform_response: String::new(),
                    skipped: true,
                }
            }).collect::<Vec<_>>();
            on_answer.run((id, responses, true));
        }
    });

    view! {
        <div class="question-view-wrapper flex justify-start w-full gap-4 mb-6">
            <div class="w-8 flex-shrink-0"></div>
            <div class="flex-1 max-w-[85%]">
                <div class="question-card border border-blue-200 dark:border-blue-900/50 bg-blue-50/5 dark:bg-blue-955/5 rounded-2xl shadow-md overflow-hidden">
                    <div class="flex items-center gap-2 px-5 py-3 bg-blue-50 dark:bg-blue-950/20 border-b border-blue-200 dark:border-blue-900/30 text-blue-700 dark:text-blue-400 select-none">
                        <span class="text-sm">"💬"</span>
                        <span class="font-bold tracking-wide text-xs uppercase">"Agent Question Required"</span>
                    </div>

                    <div class="p-5 space-y-6">
                        {questions.into_iter().enumerate().map(|(q_idx, q)| {
                            let is_multi = q.is_multi_select;
                            let options = q.options.clone();
                            let question_text = q.question.clone();

                            view! {
                                <div class="space-y-3">
                                    <h3 class="text-sm font-semibold text-gray-800 dark:text-gray-200 leading-snug">
                                        {question_text}
                                    </h3>

                                    {if answered {
                                        view! {
                                            <div class="text-xs text-gray-500 dark:text-gray-400 italic">
                                                "Option selections recorded."
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="space-y-2">
                                                {options.into_iter().map(|opt| {
                                                    let opt_id = opt.id.clone();
                                                    let opt_text = opt.text.clone();
                                                    let is_checked = move || {
                                                        selections.get().get(q_idx).map(|s| s.0.contains(&opt_id)).unwrap_or(false)
                                                    };
                                                    
                                                    let toggle_opt = {
                                                        let opt_id = opt.id.clone();
                                                        move |_| {
                                                            set_selections.update(|sels| {
                                                                if let Some(entry) = sels.get_mut(q_idx) {
                                                                    if is_multi {
                                                                        if let Some(pos) = entry.0.iter().position(|x| x == &opt_id) {
                                                                            entry.0.remove(pos);
                                                                        } else {
                                                                            entry.0.push(opt_id.clone());
                                                                        }
                                                                    } else {
                                                                        entry.0.clear();
                                                                        entry.0.push(opt_id.clone());
                                                                    }
                                                                }
                                                             });
                                                         }
                                                     };

                                                     view! {
                                                         <label class="flex items-start gap-3 p-2.5 rounded-lg border border-gray-200 dark:border-gray-800 hover:bg-gray-50 dark:hover:bg-gray-900/30 cursor-pointer transition-colors text-xs font-medium text-gray-700 dark:text-gray-300">
                                                             <input 
                                                                 type=if is_multi { "checkbox" } else { "radio" }
                                                                 name=format!("q_{}_{}", id, q_idx)
                                                                 checked=is_checked()
                                                                 on:change=toggle_opt
                                                                 class="mt-0.5 h-3.5 w-3.5 text-blue-600 rounded border-gray-300 focus:ring-blue-500 dark:bg-gray-800 dark:border-gray-700"
                                                             />
                                                             <span>{opt_text}</span>
                                                         </label>
                                                     }
                                                 }).collect::<Vec<_>>()}

                                                 <div class="mt-4 space-y-1">
                                                     <label class="text-[10px] uppercase font-bold text-gray-400 dark:text-gray-500">
                                                         "Response Explanation / Notes"
                                                     </label>
                                                     <textarea 
                                                         placeholder="Type your response or additional notes..."
                                                         rows="2"
                                                         on:input=move |ev| {
                                                             let val = event_target_value(&ev);
                                                             set_selections.update(|sels| {
                                                                 if let Some(entry) = sels.get_mut(q_idx) {
                                                                     entry.1 = val.clone();
                                                                 }
                                                             });
                                                         }
                                                         class="w-full text-xs p-3 rounded-lg border border-gray-200 dark:border-gray-800 bg-transparent outline-none focus:border-blue-500 dark:focus:border-blue-500 placeholder-gray-400 dark:placeholder-gray-600"
                                                     />
                                                 </div>
                                             </div>
                                         }.into_any()
                                     }}
                                 </div>
                             }
                         }).collect::<Vec<_>>()}

                         <Show when=move || !answered>
                             <div class="flex items-center justify-between pt-4 border-t border-gray-200/50 dark:border-gray-800/50">
                                 <button 
                                     on:click=move |ev| on_skip.run(ev)
                                     class="px-4 py-2 rounded-lg text-xs font-semibold border border-gray-300 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-600 dark:text-gray-300 transition-colors"
                                 >
                                     "Skip"
                                 </button>
                                 <button 
                                     on:click=move |ev| on_submit.run(ev)
                                     class="px-4 py-2 rounded-lg text-xs font-semibold bg-blue-600 hover:bg-blue-700 text-white shadow-md transition-colors"
                                 >
                                     "Submit Answer"
                                 </button>
                             </div>
                         </Show>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Tool approval dialog — inline decided chip only.
/// Pending state is handled by the floating panel above the input; this component
/// only renders the collapsed accepted/denied chip in the chat history.
#[component]
fn ConfirmationView(
    _id: u64,
    _trajectory_id: String,
    _step_index: u32,
    tool_call: ClientToolCall,
    decision: Option<bool>,
) -> impl IntoView {
    let args_str = serde_json::to_string_pretty(&tool_call.args).unwrap_or_else(|_| tool_call.args.to_string());
    let path_display = tool_call.canonical_path.as_deref().map(shorten_path);

    if let Some(accepted) = decision {
        // ── Decided: collapse to a compact summary ─────────────────────────
        let (border_cls, summary_cls, badge_bg, badge_text, icon_svg) = if accepted {
            (
                "border-emerald-200/60 dark:border-emerald-900/30",
                "bg-emerald-50/40 dark:bg-emerald-950/10 text-emerald-700 dark:text-emerald-400 hover:bg-emerald-50/60 dark:hover:bg-emerald-950/20",
                "bg-emerald-500",
                "Accepted",
                view! {
                    <svg class="w-3 h-3 flex-shrink-0" fill="none" stroke="currentColor" stroke-width="3" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                    </svg>
                }.into_any(),
            )
        } else {
            (
                "border-red-200/60 dark:border-red-900/30",
                "bg-red-50/40 dark:bg-red-950/10 text-red-700 dark:text-red-400 hover:bg-red-50/60 dark:hover:bg-red-950/20",
                "bg-red-500",
                "Denied",
                view! {
                    <svg class="w-3 h-3 flex-shrink-0" fill="none" stroke="currentColor" stroke-width="3" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                }.into_any(),
            )
        };
        let tool_name = tool_call.name.clone();
        return view! {
            <details class=format!("w-full mb-2 border rounded-xl overflow-hidden shadow-sm {}", border_cls)>
                <summary class=format!(
                    "flex items-center justify-between px-3 py-2 cursor-pointer text-xs select-none transition-colors {}",
                    summary_cls
                )>
                    <div class="flex items-center gap-2 flex-1 min-w-0">
                        {icon_svg}
                        <span class="font-semibold">"Permission"</span>
                        // Tool name chip
                        <span class=format!(
                            "font-mono text-[9px] px-1.5 py-0.5 rounded border font-bold uppercase tracking-wide flex-shrink-0 {}",
                            if accepted {
                                "bg-emerald-100 dark:bg-emerald-950/30 text-emerald-700 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800/30"
                            } else {
                                "bg-red-100 dark:bg-red-950/30 text-red-700 dark:text-red-400 border-red-200 dark:border-red-800/30"
                            }
                        )>
                            {tool_name}
                        </span>
                        {path_display.map(|p| view! {
                            <span class="font-mono text-[10px] opacity-60 truncate max-w-[200px]">{p}</span>
                        })}
                    </div>
                    <span class=format!("flex-shrink-0 text-[9px] px-2 py-0.5 rounded-full text-white font-bold {}", badge_bg)>
                        {badge_text}
                    </span>
                </summary>
                // Expandable args for reference
                <div class="p-3 text-xs font-mono border-t border-gray-200/40 dark:border-gray-800/30 bg-gray-50/30 dark:bg-[#181818]/10 overflow-x-auto">
                    <div class="text-[10px] text-gray-400 dark:text-gray-500 mb-1.5 font-semibold uppercase tracking-wider">"Arguments:"</div>
                    <pre class="whitespace-pre overflow-x-auto text-gray-700 dark:text-gray-300">{args_str}</pre>
                </div>
            </details>
        }.into_any();
    }

    // ── Pending: this case should no longer occur inline (floating panel handles it)
    // but guard defensively to avoid empty renders
    view! {
        <div class="confirmation-view-wrapper w-full mb-2">
            <span class="text-xs text-gray-400">{"(pending confirmation — see panel below input)"}</span>
        </div>
    }.into_any()
}

/// Subtle timeline separator indicating context was compressed.
#[component]
fn CompactionBannerView(step_index: u32) -> impl IntoView {
    view! {
        <div class="compaction-banner flex items-center justify-center py-4 w-full select-none">
            <div class="h-px bg-gray-200 dark:bg-gray-800 flex-1 max-w-[200px]"></div>
            <span class="text-[10px] font-mono text-gray-400 dark:text-gray-500 px-4 uppercase tracking-wider font-semibold">
                {format!("Context Compacted (Step {})", step_index)}
            </span>
            <div class="h-px bg-gray-200 dark:bg-gray-800 flex-1 max-w-[200px]"></div>
        </div>
    }
}

/// Compact token usage indicator shown at the end of each turn.
#[component]
fn UsageBadgeView(
    prompt_tokens: i32,
    output_tokens: i32,
    thinking_tokens: i32,
) -> impl IntoView {
    let format_num = |n: i32| {
        let s = n.to_string();
        let mut formatted = String::new();
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                formatted.push(',');
            }
            formatted.push(c);
        }
        formatted.chars().rev().collect::<String>()
    };

    let prompt_str = format_num(prompt_tokens);
    let output_str = format_num(output_tokens);
    let thinking_str = format_num(thinking_tokens);
    let total_str = format_num(prompt_tokens + output_tokens + thinking_tokens);

    view! {
        <div class="usage-badge w-full mb-2 select-none text-[10px] font-mono text-gray-400 dark:text-gray-500 flex flex-wrap gap-x-3 gap-y-1">
            <span>"📊 Usage:"</span>
            <span>{prompt_str}" in"</span>
            <span>"·"</span>
            <span>{output_str}" out"</span>
            <span>"·"</span>
            <span>{thinking_str}" thinking"</span>
            <span class="font-bold">"(" {total_str} " total)"</span>
        </div>
    }
}

/// Error block for stream failures.
#[component]
fn ErrorBlockView(message: String, http_code: Option<u32>) -> impl IntoView {
    view! {
        <div class="error-block flex justify-start w-full gap-4 mb-4">
            <div class="flex-shrink-0 w-8 h-8 rounded-full bg-red-600 text-white flex items-center justify-center text-sm font-semibold select-none shadow-md">
                "E"
            </div>
            <div class="flex flex-col items-start max-w-[85%] w-full">
                <div class="error-header text-[11px] text-red-500 mb-1 font-medium select-none">
                    "Stream Error"
                </div>
                <div class="error-bubble bg-red-50 dark:bg-red-950/20 border border-red-200 dark:border-red-900/50 rounded-2xl px-4 py-2.5 text-sm text-red-800 dark:text-red-200 whitespace-pre-wrap leading-relaxed shadow-sm">
                    {message}
                    {if let Some(code) = http_code {
                        format!(" (HTTP {})", code)
                    } else {
                        String::new()
                    }}
                </div>
            </div>
        </div>
    }
}

/// Compact task completion badge.
#[component]
fn FinishBlockView(structured_output: Option<serde_json::Value>) -> impl IntoView {
    view! {
        <div class="finish-block w-full mb-4 select-none">
                <div class="flex items-center gap-2 p-3 bg-emerald-50/50 dark:bg-emerald-950/10 border border-emerald-200 dark:border-emerald-900/30 rounded-xl shadow-sm text-emerald-800 dark:text-emerald-300">
                    <svg class="w-4 h-4 text-emerald-500 flex-shrink-0" fill="none" stroke="currentColor" stroke-width="3" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    <span class="text-xs font-semibold">"Task Complete"</span>
                </div>
                
                {if let Some(ref val) = structured_output {
                    let val_str = serde_json::to_string_pretty(val).unwrap_or_default();
                    view! {
                        <details class="mt-2 border border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden bg-gray-50/30 dark:bg-[#1a1a1a]/20">
                            <summary class="px-3 py-1.5 text-[10px] font-mono font-bold text-gray-500 dark:text-gray-400 cursor-pointer select-none">
                                "Structured Output"
                            </summary>
                            <pre class="p-3 text-[10px] font-mono whitespace-pre overflow-x-auto border-t border-gray-200 dark:border-gray-800 text-gray-700 dark:text-gray-300">{val_str}</pre>
                        </details>
                    }.into_any()
                } else {
                    ().into_any()
                }}
        </div>
    }
}


#[derive(Clone, PartialEq, Debug)]
struct SubagentInfo {
    trajectory_id: String,
    prompt: String,
    status: ToolCallStatus,
    step_count: usize,
    error: Option<String>,
    subagents: Vec<SubagentInfo>,
}

fn collect_subagents_recursive(blocks: &[MessageBlock]) -> Vec<SubagentInfo> {
    let mut result = Vec::new();
    for block in blocks {
        if let MessageBlock::ToolCall {
            name,
            args,
            status,
            subagent_trajectory_id,
            subagent_blocks,
            ..
        } = block {
            if name == "START_SUBAGENT" {
                if let Some(traj_id) = subagent_trajectory_id {
                    let prompt = args["prompt"].as_str().unwrap_or("Unknown Subagent").to_string();
                    
                    let mut step_count = 0;
                    let mut error = None;
                    for sb in subagent_blocks {
                        match sb {
                            MessageBlock::Thinking { .. } | MessageBlock::AssistantMessage { .. } => {
                                step_count += 1;
                            }
                            MessageBlock::Error { message, .. } => {
                                error = Some(message.clone());
                            }
                            _ => {}
                        }
                    }

                    let children = collect_subagents_recursive(subagent_blocks);
                    
                    result.push(SubagentInfo {
                        trajectory_id: traj_id.clone(),
                        prompt,
                        status: status.clone(),
                        step_count,
                        error,
                        subagents: children,
                    });
                }
            } else {
                result.extend(collect_subagents_recursive(subagent_blocks));
            }
        }
    }
    result
}

#[component]
fn SubagentStatusItem(item: SubagentInfo, depth: usize) -> AnyView {
    let status_class = match item.status {
        ToolCallStatus::Running => "bg-blue-500/20 text-blue-400 border-blue-500/30",
        ToolCallStatus::Done => "bg-emerald-500/20 text-emerald-400 border-emerald-500/30",
        ToolCallStatus::Error => "bg-rose-500/20 text-rose-400 border-rose-500/30",
    };
    
    let status_text = match item.status {
        ToolCallStatus::Running => "RUNNING",
        ToolCallStatus::Done => "COMPLETED",
        ToolCallStatus::Error => "FAILED",
    };

    let error_view = item.error.map(|err| {
        view! {
            <div class="mt-1.5 text-xs text-rose-400 bg-rose-500/10 border border-rose-500/20 rounded p-1.5 font-mono break-all">
                {err}
            </div>
        }
    });

    view! {
        <div class="flex flex-col space-y-1.5" style=format!("padding-left: {}px", depth * 12)>
            <div class="p-3 rounded-lg border border-gray-200/10 bg-white/5 dark:bg-[#202020]/40 backdrop-blur-md shadow-sm transition-all duration-300 hover:border-gray-200/20">
                <div class="flex items-start justify-between gap-2">
                    <div class="flex flex-col min-w-0">
                        <span class="text-[10px] text-gray-500 font-mono tracking-wider">
                            {item.trajectory_id}
                        </span>
                        <span class="text-xs text-gray-800 dark:text-gray-200 font-medium truncate mt-0.5 max-w-[200px]">
                            {item.prompt}
                        </span>
                    </div>
                    <span class=format!("text-[8px] font-bold px-1.5 py-0.5 rounded border tracking-wider {}", status_class)>
                        {status_text}
                    </span>
                </div>
                
                <div class="flex items-center gap-3 mt-2 text-[10px] text-gray-500 font-medium">
                    <div class="flex items-center gap-1">
                        <svg class="w-3.5 h-3.5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2" />
                        </svg>
                        <span>{item.step_count} " steps"</span>
                    </div>
                </div>

                {error_view}
            </div>

            {item.subagents.into_iter().map(|child| {
                view! {
                    <SubagentStatusItem item=child depth=depth + 1 />
                }
            }).collect::<Vec<_>>()}
        </div>
    }.into_any()
}

/// Top-level Block Dispatcher. Renders the appropriate component based on MessageBlock variant.
#[component]
fn MessageBlockView(
    block: MessageBlock,
    on_answer: Callback<(u64, Vec<QuestionResponse>, bool)>,
) -> impl IntoView {
    match block {
        MessageBlock::UserMessage { content, .. } => {
            view! { <UserMessageView content /> }.into_any()
        }
        MessageBlock::AssistantMessage { content, .. } => {
            view! { <AssistantMessageView content /> }.into_any()
        }
        MessageBlock::Thinking { content, is_streaming, .. } => {
            view! { <ThinkingView content is_streaming /> }.into_any()
        }
        MessageBlock::ToolCall {
            id,
            call_id,
            name,
            args,
            status,
            canonical_path,
            label,
            subagent_blocks,
            ..
        } => {
            let on_answer_cb = on_answer.clone();
            view! {
                <ToolCallView
                    id=id
                    call_id=call_id
                    name=name
                    args=args
                    status=status
                    canonical_path=canonical_path
                    label=label
                    subagent_blocks=subagent_blocks
                    on_answer=on_answer_cb
                />
            }.into_any()
        }
        MessageBlock::ToolResult { name, result, error, .. } => {
            view! { <ToolResultView name result error /> }.into_any()
        }
        MessageBlock::Question { id, questions, answered, trajectory_id, step_index, .. } => {
            view! { <QuestionView id questions answered trajectory_id step_index on_answer /> }.into_any()
        }
        MessageBlock::Confirmation { id, tool_call, decision, trajectory_id, step_index, .. } => {
            view! { <ConfirmationView _id=id tool_call decision _trajectory_id=trajectory_id _step_index=step_index /> }.into_any()
        }
        MessageBlock::UsageSummary { prompt_tokens, output_tokens, thinking_tokens, .. } => {
            view! { <UsageBadgeView prompt_tokens output_tokens thinking_tokens /> }.into_any()
        }
        MessageBlock::Compaction { step_index, .. } => {
            view! { <CompactionBannerView step_index /> }.into_any()
        }
        MessageBlock::Error { message, http_code, .. } => {
            view! { <ErrorBlockView message http_code /> }.into_any()
        }
        MessageBlock::Finish { structured_output, .. } => {
            view! { <FinishBlockView structured_output /> }.into_any()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PendingQuestion {
    trajectory_id: String,
    step_index: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct PendingConfirm {
    trajectory_id: String,
    step_index: u32,
    tool_name: String,
    // Full tool call so the floating panel can show args without a separate block.
    tool_call: ClientToolCall,
}

fn get_current_time() -> u64 {
    #[cfg(feature = "hydrate")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
    #[cfg(not(feature = "hydrate"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
struct TokenEvent {
    step_index: u32,
    text: String,
    #[serde(default)]
    trajectory_id: Option<String>,
}

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
struct ThoughtEvent {
    step_index: u32,
    text: String,
    #[serde(default)]
    trajectory_id: Option<String>,
}

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct StatusEvent {
    step_index: u32,
    status: String,
    #[serde(default)]
    trajectory_id: Option<String>,
}

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
struct ToolStartEvent {
    id: String,
    name: String,
    args: serde_json::Value,
    canonical_path: Option<String>,
    /// Human-readable label from the agent's step description.
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    trajectory_id: Option<String>,
}

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
struct ToolResultEvent {
    id: String,
    name: String,
    result: Option<serde_json::Value>,
    error: Option<String>,
    #[serde(default)]
    trajectory_id: Option<String>,
}

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
struct QuestionEvent {
    trajectory_id: String,
    step_index: u32,
    questions: Vec<AskQuestionEntry>,
}

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
struct ConfirmEvent {
    /// May be absent when emitted from the concurrent confirm-task (no step context available).
    #[serde(default)]
    trajectory_id: String,
    #[serde(default)]
    step_index: u32,
    tool_call: ClientToolCall,
}

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
struct UsageEvent {
    prompt_token_count: i32,
    candidates_token_count: i32,
    thoughts_token_count: i32,
}

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
struct CompactionEvent {
    step_index: u32,
    #[serde(default)]
    trajectory_id: Option<String>,
}

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
struct FinishEvent {
    structured_output: Option<serde_json::Value>,
    #[serde(default)]
    trajectory_id: Option<String>,
}

#[cfg(feature = "hydrate")]
#[derive(serde::Deserialize)]
struct ErrorEvent {
    message: String,
    http_code: Option<u32>,
}

#[cfg(feature = "hydrate")]
fn update_streaming_block<F>(
    set_blocks: WriteSignal<Vec<MessageBlock>>,
    block_id: Option<u64>,
    f: F,
) where
    F: FnOnce(&mut MessageBlock),
{
    if let Some(id) = block_id {
        set_blocks.update(|bs| {
            if let Some(b) = bs.iter_mut().find(|b| match b {
                MessageBlock::Thinking { id: bid, .. } => *bid == id,
                MessageBlock::AssistantMessage { id: bid, .. } => *bid == id,
                _ => false,
            }) {
                f(b);
            }
        });
    }
}

#[cfg(feature = "hydrate")]
struct SubagentStreamState {
    streaming_assistant_id: Option<u64>,
    streaming_thinking_id: Option<u64>,
    current_text_step: u32,
    current_think_step: u32,
    seen_tool_ids: std::collections::HashSet<String>,
}

#[cfg(feature = "hydrate")]
impl Default for SubagentStreamState {
    fn default() -> Self {
        Self {
            streaming_assistant_id: None,
            streaming_thinking_id: None,
            current_text_step: u32::MAX,
            current_think_step: u32::MAX,
            seen_tool_ids: std::collections::HashSet::new(),
        }
    }
}

#[cfg(feature = "hydrate")]
fn find_and_associate_subagent(blocks: &mut Vec<MessageBlock>, traj_id: &str) -> bool {
    for block in blocks.iter_mut().rev() {
        if let MessageBlock::ToolCall {
            name,
            subagent_trajectory_id,
            subagent_blocks,
            ..
        } = block {
            if find_and_associate_subagent(subagent_blocks, traj_id) {
                return true;
            }
            if name == "START_SUBAGENT" && subagent_trajectory_id.is_none() {
                *subagent_trajectory_id = Some(traj_id.to_string());
                return true;
            }
        }
    }
    false
}

#[cfg(feature = "hydrate")]
fn update_subagent_blocks_impl(
    blocks: &mut Vec<MessageBlock>,
    traj_id: &str,
    f: &mut dyn FnMut(&mut Vec<MessageBlock>),
) -> bool {
    for block in blocks.iter_mut() {
        if let MessageBlock::ToolCall {
            subagent_trajectory_id,
            subagent_blocks,
            ..
        } = block {
            if let Some(tid) = subagent_trajectory_id.as_ref() {
                if tid == traj_id {
                    f(subagent_blocks);
                    return true;
                }
            }
            if update_subagent_blocks_impl(subagent_blocks, traj_id, f) {
                return true;
            }
        }
    }
    false
}

#[cfg(feature = "hydrate")]
fn update_subagent_blocks<F>(blocks: &mut Vec<MessageBlock>, traj_id: &str, mut f: F) -> bool
where
    F: FnMut(&mut Vec<MessageBlock>),
{
    update_subagent_blocks_impl(blocks, traj_id, &mut f)
}

#[cfg(feature = "hydrate")]
fn ensure_and_update_subagent_blocks_impl(
    blocks: &mut Vec<MessageBlock>,
    traj_id: &str,
    f: &mut dyn FnMut(&mut Vec<MessageBlock>),
) {
    let updated = update_subagent_blocks_impl(blocks, traj_id, f);
    if !updated {
        if find_and_associate_subagent(blocks, traj_id) {
            update_subagent_blocks_impl(blocks, traj_id, f);
        }
    }
}

#[cfg(feature = "hydrate")]
fn ensure_and_update_subagent_blocks<F>(blocks: &mut Vec<MessageBlock>, traj_id: &str, mut f: F)
where
    F: FnMut(&mut Vec<MessageBlock>),
{
    ensure_and_update_subagent_blocks_impl(blocks, traj_id, &mut f);
}

#[cfg(feature = "hydrate")]
fn update_block_by_id_impl(
    blocks: &mut Vec<MessageBlock>,
    id: u64,
    f: &mut dyn FnMut(&mut MessageBlock),
) -> bool {
    for block in blocks.iter_mut() {
        if block.id() == id {
            f(block);
            return true;
        }
        if let MessageBlock::ToolCall { subagent_blocks, .. } = block {
            if update_block_by_id_impl(subagent_blocks, id, f) {
                return true;
            }
        }
    }
    false
}

#[cfg(feature = "hydrate")]
fn mark_all_thinking_done(blocks: &mut [MessageBlock]) {
    for block in blocks {
        match block {
            MessageBlock::Thinking { is_streaming, .. } => {
                *is_streaming = false;
            }
            MessageBlock::ToolCall { subagent_blocks, .. } => {
                mark_all_thinking_done(subagent_blocks);
            }
            _ => {}
        }
    }
}

/// Chat page component
#[component]
fn ChatPage() -> impl IntoView {
    // Session list and active session tracking
    let (sessions, set_sessions) = signal(Vec::<SessionMeta>::new());
    let (active_session_id, set_active_session_id) = signal(Option::<String>::None);
    let (blocks, set_blocks) = signal(Vec::<MessageBlock>::new());

    // Rename/Edit signals
    let (editing_session_id, set_editing_session_id) = signal(Option::<String>::None);
    let (edit_title_input, set_edit_title_input) = signal(String::new());
    let edit_input_ref = NodeRef::<leptos::html::Input>::new();

    let (input_text, set_input_text) = signal(String::new());
    let (error_text, set_error_text) = signal(Option::<String>::None);
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();
    let messages_container_ref = NodeRef::<leptos::html::Div>::new();

    let (dark_mode, set_dark_mode) = signal(true);
    let (sidebar_open, set_sidebar_open) = signal(false);
    let (hub_open, set_hub_open) = signal(false);

    // Monotonic block ID counter
    let (block_id_counter, set_block_id_counter) = signal(0u64);
    let (is_streaming, set_is_streaming) = signal(false);

    // Accumulating buffers for streaming UI elements
    let (stream_text_buf, set_stream_text_buf) = signal(String::new());
    let (stream_think_buf, set_stream_think_buf) = signal(String::new());
    let (_stream_tool_id, set_stream_tool_id) = signal(Option::<String>::None);

    // Tracking state for interactive flows waiting for user choice
    let (pending_question, set_pending_question) = signal(Option::<PendingQuestion>::None);
    let (pending_confirm, set_pending_confirm) = signal(Option::<PendingConfirm>::None);

    // Open Folder modal signals
    let (show_open_folder, set_show_open_folder) = signal(false);
    let (open_folder_input, set_open_folder_input) = signal(String::new());
    let (open_folder_error, set_open_folder_error) = signal(Option::<String>::None);
    let (workspace_path, set_workspace_path) = signal(Option::<String>::None);
    provide_context(workspace_path);
    // Silence SSR unused-variable warnings for signals only read/written inside #[cfg(feature = "hydrate")]
    let _ = &open_folder_input;
    let _ = &set_workspace_path;

    // Stored references for SSE connections and listeners to prevent memory leaks and GC
    #[cfg(feature = "hydrate")]
    let active_listeners = StoredValue::new_local(Option::<Vec<gloo_events::EventListener>>::None);

    #[cfg(feature = "hydrate")]
    let active_es = StoredValue::new_local(Option::<web_sys::EventSource>::None);

    // Stores the native onerror Closure so it stays alive as long as the EventSource does.
    // Using StoredValue instead of forget() prevents the captured reactive signals from
    // outliving the reactive owner (which would cause "disposed reactive value" panics).
    #[cfg(feature = "hydrate")]
    let active_onerror_closure = StoredValue::new_local(
        Option::<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>::None
    );

    // Cleanup: close the EventSource and drop all listeners when the component is disposed.
    // Without this, the EventSource keeps firing into dead reactive signals.
    #[cfg(feature = "hydrate")]
    {
        let active_es_cleanup = active_es;
        let active_listeners_cleanup = active_listeners;
        let active_onerror_cleanup = active_onerror_closure;
        on_cleanup(move || {
            active_es_cleanup.update_value(|opt| {
                if let Some(es) = opt.take() {
                    es.close();
                }
            });
            active_listeners_cleanup.set_value(None);
            active_onerror_cleanup.set_value(None);
        });
    }

    // Dynamic title tracking
    let active_session_title = move || {
        if let Some(sid) = active_session_id.get() {
            sessions.get().iter()
                .find(|s| s.id == sid)
                .map(|s| s.title.clone())
                .unwrap_or_else(|| "Chat".to_string())
        } else {
            "Antigravity Chat".to_string()
        }
    };

    // Block ID generator
    let next_id = move || {
        let next = block_id_counter.get_untracked() + 1;
        set_block_id_counter.set(next);
        next
    };

    // Load available sessions from backend KV store.
    // On page refresh, tries to restore the last active session from localStorage
    // before falling back to the first session in the list.
    let load_sessions = move || {
        spawn_local(async move {
            if let Ok(sess_list) = list_sessions().await {
                set_sessions.set(sess_list.clone());
                if active_session_id.get_untracked().is_none() {
                    // Try to restore the last-active session from localStorage.
                    #[cfg(feature = "hydrate")]
                    let saved_id: Option<String> = web_sys::window()
                        .and_then(|w| w.local_storage().ok().flatten())
                        .and_then(|ls| ls.get_item("last_active_session").ok().flatten());
                    #[cfg(not(feature = "hydrate"))]
                    let saved_id: Option<String> = None;

                    // Pick: saved session (if still exists) → first session → create new
                    let target_id = saved_id
                        .filter(|id| sess_list.iter().any(|s| &s.id == id))
                        .or_else(|| sess_list.first().map(|s| s.id.clone()));

                    if let Some(sid) = target_id {
                        set_active_session_id.set(Some(sid));
                    } else {
                        // No sessions at all — create a default one.
                        if let Ok(new_id) = create_session(None).await {
                            set_active_session_id.set(Some(new_id));
                            if let Ok(updated_list) = list_sessions().await {
                                set_sessions.set(updated_list);
                            }
                        }
                    }
                }
            }
        });
    };

    // Mount effect: load sessions
    Effect::new(move |_| {
        load_sessions();
    });

    // Persist active session ID to localStorage so we can restore it after a page refresh.
    Effect::new(move |_| {
        if let Some(sid) = active_session_id.get() {
            #[cfg(feature = "hydrate")]
            {
                if let Some(ls) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                {
                    let _ = ls.set_item("last_active_session", &sid);
                }
            }
            let _ = sid; // suppress SSR unused-variable warning
        }
    });

    // Focus and select the edit input when renaming starts
    Effect::new(move |_| {
        if editing_session_id.get().is_some() {
            #[cfg(feature = "hydrate")]
            {
                if let Some(el) = edit_input_ref.get() {
                    let _ = el.focus();
                    el.select();
                }
            }
        }
    });

    // Theme preference recovery
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(win) = web_sys::window() {
                if let Ok(Some(ls)) = win.local_storage() {
                    if let Ok(Some(val)) = ls.get_item("theme") {
                        set_dark_mode.set(val == "dark");
                    }
                }
            }
        }
    });

    // Handle switching between sessions: fetch blocks and set the correct block counter
    Effect::new(move |_| {
        if let Some(sid) = active_session_id.get() {
            let sid_blocks = sid.clone();
            spawn_local(async move {
                if let Ok(b) = get_session_blocks(sid_blocks).await {
                    set_blocks.set(b.clone());
                    // Sync monotonic counter to avoid collisions
                    let max_id = b.iter().map(|block| match block {
                        MessageBlock::UserMessage { id, .. } => *id,
                        MessageBlock::Thinking { id, .. } => *id,
                        MessageBlock::ToolCall { id, .. } => *id,
                        MessageBlock::ToolResult { id, .. } => *id,
                        MessageBlock::AssistantMessage { id, .. } => *id,
                        MessageBlock::Question { id, .. } => *id,
                        MessageBlock::Confirmation { id, .. } => *id,
                        MessageBlock::UsageSummary { id, .. } => *id,
                        MessageBlock::Compaction { id, .. } => *id,
                        MessageBlock::Finish { id, .. } => *id,
                        MessageBlock::Error { id, .. } => *id,
                    }).max().unwrap_or(0);
                    set_block_id_counter.set(max_id);
                }
            });
            // Also sync workspace path from agent server
            #[cfg(feature = "hydrate")]
            {
                let sid_ws = sid.clone();
                spawn_local(async move {
                    let agent_url = get_agent_server_url();
                    let url = format!("{}/workspace?session_id={}", agent_url, sid_ws);
                    if let Ok(resp) = gloo_net::http::Request::get(&url).send().await {
                        if resp.ok() {
                            if let Ok(body) = resp.json::<serde_json::Value>().await {
                                if let Some(ws) = body["workspace"].as_str() {
                                    set_workspace_path.set(Some(ws.to_string()));
                                }
                            }
                        }
                    }
                });
            }
        }
    });

    // One-shot mount Effect: register scroll/wheel/touch listeners on the chat container
    // to track when the user has intentionally scrolled up. Uses a JS global flag
    // `window.__userScrolledUp` so the token-stream Effect can read it cheaply.
    #[cfg(feature = "hydrate")]
    Effect::new(move |first_run: Option<()>| {
        // Only run once on first mount (first_run is None on the first call)
        if first_run.is_some() {
            return;
        }
        let _ = js_sys::eval(r#"
            (function() {
                var container = document.getElementById('chat-messages-container');
                if (!container || container.__agScrollListenersRegistered) return;
                container.__agScrollListenersRegistered = true;
                window.__userScrolledUp = false;

                function checkScrolledUp() {
                    var distFromBottom = container.scrollHeight - container.scrollTop - container.clientHeight;
                    if (distFromBottom > 100) {
                        window.__userScrolledUp = true;
                    }
                }

                container.addEventListener('scroll', checkScrolledUp, { passive: true });
                container.addEventListener('wheel', function(e) {
                    if (e.deltaY < 0) window.__userScrolledUp = true;
                }, { passive: true });
                container.addEventListener('touchstart', function() {
                    window.__userScrolledUp = false;
                }, { passive: true });
                container.addEventListener('touchmove', function(e) {
                    checkScrolledUp();
                }, { passive: true });
            })();
        "#);
    });

    // Auto-scroll messages viewport to bottom and run highlight/mermaid formatting.
    // Only scrolls when the user has NOT manually scrolled up (window.__userScrolledUp = false).
    // Uses a double requestAnimationFrame so the scroll executes AFTER the browser has
    // actually painted the new DOM nodes (single RAF fires before layout is committed).
    Effect::new(move |_| {
        let _ = blocks.get();
        let _ = stream_text_buf.get();
        let _ = stream_think_buf.get();
        // Read is_streaming unconditionally so reactivity tracks it in all build modes.
        let is_stream = is_streaming.get();
        // Also track pending_confirm so a new permission request forces a scroll.
        let _has_pending = pending_confirm.get().is_some();
        #[cfg(feature = "hydrate")]
        {
            let behavior = if is_stream { "auto" } else { "smooth" };
            let js_code = format!(
                r#"
                (function() {{
                    // Double-RAF: first RAF queues after current paint,
                    // second RAF fires after the browser has committed layout for new nodes.
                    requestAnimationFrame(function() {{
                        requestAnimationFrame(function() {{
                            if (window.__userScrolledUp) return;
                            var container = document.getElementById('chat-messages-container');
                            if (container) {{
                                container.scrollTo({{ top: container.scrollHeight, behavior: '{}' }});
                            }}
                        }});
                    }});
                }})();
                "#,
                behavior
            );
            let _ = js_sys::eval(&js_code);

            request_animation_frame(move || {
                let _ = js_sys::eval(r#"
                    document.querySelectorAll('pre code.language-mermaid').forEach((codeEl) => {
                        const preEl = codeEl.parentElement;
                        if (preEl) {
                            const div = document.createElement('div');
                            div.className = 'mermaid';
                            div.textContent = codeEl.textContent;
                            preEl.replaceWith(div);
                        }
                    });
                "#);
                let _ = js_sys::eval("if (window.hljs) { window.hljs.highlightAll(); }");
                let _ = js_sys::eval("if (window.mermaid) { window.mermaid.run(); }");
            });
        }
        // Suppress unused variable warning in SSR builds (is_stream read above for reactivity).
        #[cfg(not(feature = "hydrate"))]
        let _ = is_stream;
    });



    // Toggle dark/light modes
    let toggle_dark_mode = move |_| {
        let next = !dark_mode.get();
        set_dark_mode.set(next);
        #[cfg(feature = "hydrate")]
        {
            if let Some(win) = web_sys::window() {
                if let Ok(Some(ls)) = win.local_storage() {
                    let _ = ls.set_item("theme", if next { "dark" } else { "light" });
                }
            }
        }
    };

    // Clear blocks for active session
    let on_clear = move |_| {
        if let Some(sid) = active_session_id.get_untracked() {
            set_blocks.set(Vec::new());
            set_error_text.set(None);
            set_is_streaming.set(false);
            spawn_local(async move {
                let _ = save_turn_blocks(sid, Vec::new()).await;
            });
        }
    };

    // Delete session
    let do_delete_session = move |sid: String| {
        let active_id = active_session_id.get_untracked();
        spawn_local(async move {
            if delete_session(sid.clone()).await.is_ok() {
                if let Ok(updated_list) = list_sessions().await {
                    set_sessions.set(updated_list.clone());
                    if Some(sid) == active_id {
                        if let Some(first) = updated_list.first() {
                            set_active_session_id.set(Some(first.id.clone()));
                        } else {
                            if let Ok(new_id) = create_session(None).await {
                                set_active_session_id.set(Some(new_id));
                                if let Ok(final_list) = list_sessions().await {
                                    set_sessions.set(final_list);
                                }
                            }
                        }
                    }
                }
            }
        });
    };

    // Create a new session
    let on_new_chat = move |_| {
        spawn_local(async move {
            if let Ok(new_id) = create_session(None).await {
                set_active_session_id.set(Some(new_id));
                if let Ok(updated_list) = list_sessions().await {
                    set_sessions.set(updated_list);
                }
            }
        });
    };

    // Halt/Cancel streaming
    let on_halt = {
        move |_| {
            #[cfg(feature = "hydrate")]
            {
                active_es.update_value(|opt_es| {
                    if let Some(es) = opt_es.take() {
                        es.close();
                    }
                });
                active_listeners.set_value(None);
            }
            set_is_streaming.set(false);
            #[cfg(feature = "hydrate")]
            {
                spawn_local(async move {
                    let agent_url = get_agent_server_url();
                    let url = format!("{}/halt", agent_url);
                    let _ = gloo_net::http::Request::post(&url).send().await;
                });
            }
        }
    };

    // User response callback for agent questions
    let on_answer = Callback::new(move |(block_id, responses, cancelled): (u64, Vec<QuestionResponse>, bool)| {
        let _ = &responses;
        let _ = cancelled;
        set_blocks.update(|bs| {
            if let Some(MessageBlock::Question { answered, .. }) = bs.iter_mut().find(|b| match b {
                MessageBlock::Question { id, .. } => *id == block_id,
                _ => false,
            }) {
                *answered = true;
            }
        });
        if let Some(pending) = pending_question.get_untracked() {
            let _ = &pending;
            set_pending_question.set(None);
            #[cfg(feature = "hydrate")]
            {
                let payload = AnswerPayload {
                    session_id: active_session_id.get_untracked().unwrap_or_default(),
                    trajectory_id: pending.trajectory_id,
                    step_index: pending.step_index,
                    responses,
                    cancelled,
                };
                spawn_local(async move {
                    let agent_url = get_agent_server_url();
                    let url = format!("{}/answer", agent_url);
                    let _ = gloo_net::http::Request::post(&url)
                        .json(&payload)
                        .unwrap()
                        .send()
                        .await;
                });
            }
        }
    });

    // User approval callback for dangerous tool calls.
    // Called from the floating confirmation panel (not from inline block buttons).
    // Pushes a decided Confirmation chip into the chat history, then sends the
    // /confirm HTTP request to the agent server.
    let on_confirm = Callback::new(move |(accepted, allow_for_session): (bool, bool)| {
        if let Some(pending) = pending_confirm.get_untracked() {
            // Push a decided chip so the chat history records what happened.
            let chip_id = next_id();
            set_blocks.update(|bs| bs.push(MessageBlock::Confirmation {
                id: chip_id,
                trajectory_id: pending.trajectory_id.clone(),
                step_index: pending.step_index,
                tool_call: pending.tool_call.clone(),
                decision: Some(accepted),
            }));
            set_pending_confirm.set(None);
            // allow_for_session is only meaningful in the WASM/hydrate build.
            #[cfg(not(feature = "hydrate"))]
            let _ = allow_for_session;
            #[cfg(feature = "hydrate")]
            {
                let sess_id = active_session_id.get_untracked().unwrap_or_default();
                let payload = ConfirmPayload {
                    session_id: sess_id.clone(),
                    trajectory_id: pending.trajectory_id,
                    step_index: pending.step_index,
                    accepted,
                    allow_for_session,
                    tool_name: Some(pending.tool_name),
                };
                spawn_local(async move {
                    let agent_url = get_agent_server_url();
                    // Send deny/accept to the confirm endpoint.
                    let confirm_url = format!("{}/confirm", agent_url);
                    let _ = gloo_net::http::Request::post(&confirm_url)
                        .json(&payload)
                        .unwrap()
                        .send()
                        .await;
                    // If denied, also halt the agent so it stops immediately
                    // instead of continuing after the tool error.
                    if !accepted {
                        let halt_url = format!("{}/halt", agent_url);
                        let halt_body = serde_json::json!({ "session_id": sess_id });
                        let _ = gloo_net::http::Request::post(&halt_url)
                            .json(&halt_body)
                            .unwrap()
                            .send()
                            .await;
                    }
                });
            }
        }
    });

    // Primary request dispatcher
    let do_send = move || {
        let text = input_text.get();
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            return;
        }

        let user_id = next_id();
        let now = get_current_time();
        set_blocks.update(|bs| bs.push(MessageBlock::UserMessage {
            id: user_id,
            content: trimmed.clone(),
            timestamp: now,
        }));

        set_input_text.set(String::new());
        if let Some(el) = textarea_ref.get() {
            el.set_value("");
            #[cfg(feature = "hydrate")]
            {
                use wasm_bindgen::JsCast;
                if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                    let _ = web_sys::HtmlElement::style(html_el).set_property("height", "auto");
                }
            }
        }

        set_error_text.set(None);
        set_stream_text_buf.set(String::new());
        set_stream_think_buf.set(String::new());
        set_stream_tool_id.set(None);
        set_pending_question.set(None);
        set_pending_confirm.set(None);
        set_is_streaming.set(true);

        #[cfg(feature = "hydrate")]
        {
            use gloo_events::EventListener;
            use wasm_bindgen::JsCast;
            use wasm_bindgen::closure::Closure;
            use web_sys::{EventSource, MessageEvent};

            let agent_url = get_agent_server_url();
            let encoded_msg = get_agent_server_url_encoded(&trimmed);
            let session_id_for_url = active_session_id.get_untracked().unwrap_or_default();
            let url = format!("{}/chat/stream?message={}&session_id={}", agent_url, encoded_msg, session_id_for_url);

            match EventSource::new(&url) {
                Ok(es) => {
                    let streaming_assistant_id = StoredValue::new_local(Option::<u64>::None);
                    let streaming_thinking_id = StoredValue::new_local(Option::<u64>::None);
                    // Track which step_index we last saw for text and thought events.
                    // When the step_index changes we reset the streaming block IDs so
                    // a brand-new AssistantMessage / Thinking block is created for each
                    // agent step (enables proper multi-step agentic UI).
                    let current_text_step = Rc::new(Cell::new(u32::MAX));
                    let current_think_step = Rc::new(Cell::new(u32::MAX));
                    // Use Rc<Cell<bool>> for the completion flag so it can be cheaply
                    // cloned into multiple closures (safe: WASM is single-threaded).
                    use std::rc::Rc;
                    use std::cell::Cell;
                    let stream_completed = Rc::new(Cell::new(false));

                    let main_trajectory_id = Rc::new(std::cell::RefCell::new(Option::<String>::None));
                    let subagents_state = Rc::new(std::cell::RefCell::new(std::collections::HashMap::<String, SubagentStreamState>::new()));

                    // Event handlers
                    let on_token = {
                        let main_trajectory_id = main_trajectory_id.clone();
                        let subagents_state = subagents_state.clone();
                        move |event: &web_sys::Event| {
                            if let Ok(msg_event) = event.clone().dyn_into::<MessageEvent>() {
                                if let Some(data_str) = msg_event.data().as_string() {
                                    if let Ok(data) = serde_json::from_str::<TokenEvent>(&data_str) {
                                        let is_subagent = if let Some(ref traj_id) = data.trajectory_id {
                                            let mut main_id_opt = main_trajectory_id.borrow().clone();
                                            if main_id_opt.is_none() {
                                                *main_trajectory_id.borrow_mut() = Some(traj_id.clone());
                                                main_id_opt = Some(traj_id.clone());
                                            }
                                            main_id_opt.as_ref() != Some(traj_id)
                                        } else {
                                            false
                                        };

                                        if is_subagent {
                                            let traj_id = data.trajectory_id.clone().unwrap();
                                            let mut state_map = subagents_state.borrow_mut();
                                            let sub_state = state_map.entry(traj_id.clone()).or_default();
                                            
                                            if sub_state.current_text_step != data.step_index {
                                                sub_state.current_text_step = data.step_index;
                                                sub_state.streaming_assistant_id = None;
                                            }
                                            let mut id_opt = sub_state.streaming_assistant_id;
                                            if id_opt.is_none() {
                                                let new_id = next_id();
                                                sub_state.streaming_assistant_id = Some(new_id);
                                                id_opt = Some(new_id);
                                                set_blocks.update(|bs| {
                                                    ensure_and_update_subagent_blocks(bs, &traj_id, move |sub_blocks| {
                                                        sub_blocks.push(MessageBlock::AssistantMessage {
                                                            id: new_id,
                                                            content: String::new(),
                                                            timestamp: get_current_time(),
                                                        });
                                                    });
                                                });
                                            }
                                            
                                            let target_id = id_opt.unwrap();
                                            let text_to_push = data.text.clone();
                                            set_blocks.update(|bs| {
                                                update_block_by_id_impl(bs, target_id, &mut |b| {
                                                    if let MessageBlock::AssistantMessage { content, .. } = b {
                                                        content.push_str(&text_to_push);
                                                    }
                                                });
                                            });
                                        } else {
                                            // If this token belongs to a NEW step, start a fresh block.
                                            if current_text_step.get() != data.step_index {
                                                current_text_step.set(data.step_index);
                                                streaming_assistant_id.set_value(None);
                                            }
                                            let mut id_opt = streaming_assistant_id.get_value();
                                            if id_opt.is_none() {
                                                let new_id = next_id();
                                                set_blocks.update(|bs| bs.push(MessageBlock::AssistantMessage {
                                                    id: new_id,
                                                    content: String::new(),
                                                    timestamp: get_current_time(),
                                                }));
                                                streaming_assistant_id.set_value(Some(new_id));
                                                id_opt = Some(new_id);
                                            }
                                            set_stream_text_buf.update(|s| s.push_str(&data.text));
                                            update_streaming_block(set_blocks, id_opt, |b| {
                                                if let MessageBlock::AssistantMessage { content, .. } = b {
                                                    content.push_str(&data.text);
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    };

                    let on_thought = {
                        let main_trajectory_id = main_trajectory_id.clone();
                        let subagents_state = subagents_state.clone();
                        move |event: &web_sys::Event| {
                            if let Ok(msg_event) = event.clone().dyn_into::<MessageEvent>() {
                                if let Some(data_str) = msg_event.data().as_string() {
                                    if let Ok(data) = serde_json::from_str::<ThoughtEvent>(&data_str) {
                                        let is_subagent = if let Some(ref traj_id) = data.trajectory_id {
                                            let mut main_id_opt = main_trajectory_id.borrow().clone();
                                            if main_id_opt.is_none() {
                                                *main_trajectory_id.borrow_mut() = Some(traj_id.clone());
                                                main_id_opt = Some(traj_id.clone());
                                            }
                                            main_id_opt.as_ref() != Some(traj_id)
                                        } else {
                                            false
                                        };

                                        if is_subagent {
                                            let traj_id = data.trajectory_id.clone().unwrap();
                                            let mut state_map = subagents_state.borrow_mut();
                                            let sub_state = state_map.entry(traj_id.clone()).or_default();
                                            
                                            if sub_state.current_think_step != data.step_index {
                                                sub_state.current_think_step = data.step_index;
                                                sub_state.streaming_thinking_id = None;
                                            }
                                            let mut id_opt = sub_state.streaming_thinking_id;
                                            if id_opt.is_none() {
                                                let new_id = next_id();
                                                sub_state.streaming_thinking_id = Some(new_id);
                                                id_opt = Some(new_id);
                                                set_blocks.update(|bs| {
                                                    ensure_and_update_subagent_blocks(bs, &traj_id, move |sub_blocks| {
                                                        sub_blocks.push(MessageBlock::Thinking {
                                                            id: new_id,
                                                            content: String::new(),
                                                            is_streaming: true,
                                                        });
                                                    });
                                                });
                                            }
                                            
                                            let target_id = id_opt.unwrap();
                                            let text_to_push = data.text.clone();
                                            set_blocks.update(|bs| {
                                                update_block_by_id_impl(bs, target_id, &mut |b| {
                                                    if let MessageBlock::Thinking { content, .. } = b {
                                                        content.push_str(&text_to_push);
                                                    }
                                                });
                                            });
                                        } else {
                                            // If this thought belongs to a NEW step, start a fresh block.
                                            if current_think_step.get() != data.step_index {
                                                current_think_step.set(data.step_index);
                                                streaming_thinking_id.set_value(None);
                                            }
                                            let mut id_opt = streaming_thinking_id.get_value();
                                            if id_opt.is_none() {
                                                let new_id = next_id();
                                                set_blocks.update(|bs| bs.push(MessageBlock::Thinking {
                                                    id: new_id,
                                                    content: String::new(),
                                                    is_streaming: true,
                                                }));
                                                streaming_thinking_id.set_value(Some(new_id));
                                                id_opt = Some(new_id);
                                            }
                                            set_stream_think_buf.update(|s| s.push_str(&data.text));
                                            update_streaming_block(set_blocks, id_opt, |b| {
                                                if let MessageBlock::Thinking { content, .. } = b {
                                                    content.push_str(&data.text);
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    };

                    let on_tool_start = {
                        let main_trajectory_id = main_trajectory_id.clone();
                        let subagents_state = subagents_state.clone();
                        move |event: &web_sys::Event| {
                            if let Ok(msg_event) = event.clone().dyn_into::<MessageEvent>() {
                                if let Some(data_str) = msg_event.data().as_string() {
                                    if let Ok(data) = serde_json::from_str::<ToolStartEvent>(&data_str) {
                                        let is_subagent = if let Some(ref traj_id) = data.trajectory_id {
                                            let mut main_id_opt = main_trajectory_id.borrow().clone();
                                            if main_id_opt.is_none() {
                                                *main_trajectory_id.borrow_mut() = Some(traj_id.clone());
                                                main_id_opt = Some(traj_id.clone());
                                            }
                                            main_id_opt.as_ref() != Some(traj_id)
                                        } else {
                                            false
                                        };

                                        if is_subagent {
                                            let traj_id = data.trajectory_id.clone().unwrap();
                                            let mut state_map = subagents_state.borrow_mut();
                                            let sub_state = state_map.entry(traj_id.clone()).or_default();
                                            
                                            if !sub_state.seen_tool_ids.contains(&data.id) {
                                                sub_state.seen_tool_ids.insert(data.id.clone());
                                                let id = next_id();
                                                let tool_id = data.id.clone();
                                                let tool_name = data.name.clone();
                                                let tool_args = data.args.clone();
                                                let tool_canonical = data.canonical_path.clone();
                                                let tool_label = data.label.clone();
                                                
                                                set_blocks.update(|bs| {
                                                    ensure_and_update_subagent_blocks(bs, &traj_id, move |sub_blocks| {
                                                        sub_blocks.push(MessageBlock::ToolCall {
                                                            id,
                                                            call_id: tool_id.clone(),
                                                            name: tool_name.clone(),
                                                            args: tool_args.clone(),
                                                            canonical_path: tool_canonical.clone(),
                                                            label: tool_label.clone(),
                                                            status: ToolCallStatus::Running,
                                                            subagent_trajectory_id: None,
                                                            subagent_blocks: Vec::new(),
                                                        });
                                                    });
                                                });
                                            }
                                        } else {
                                            // Belt-and-suspenders dedup: backend already deduplicates,
                                            // but guard here too in case of reconnection replays.
                                            let already_exists = blocks.get_untracked().iter().any(|b| match b {
                                                MessageBlock::ToolCall { call_id, .. } => call_id == &data.id,
                                                _ => false,
                                            });
                                            if !already_exists {
                                                let id = next_id();
                                                set_blocks.update(|bs| bs.push(MessageBlock::ToolCall {
                                                    id,
                                                    call_id: data.id.clone(),
                                                    name: data.name,
                                                    args: data.args,
                                                    canonical_path: data.canonical_path,
                                                    label: data.label,
                                                    status: ToolCallStatus::Running,
                                                    subagent_trajectory_id: None,
                                                    subagent_blocks: Vec::new(),
                                                }));
                                                set_stream_tool_id.set(Some(data.id));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    };

                    let on_tool_result = {
                        let main_trajectory_id = main_trajectory_id.clone();
                        move |event: &web_sys::Event| {
                            if let Ok(msg_event) = event.clone().dyn_into::<MessageEvent>() {
                                if let Some(data_str) = msg_event.data().as_string() {
                                    if let Ok(data) = serde_json::from_str::<ToolResultEvent>(&data_str) {
                                        let is_subagent = if let Some(ref traj_id) = data.trajectory_id {
                                            let mut main_id_opt = main_trajectory_id.borrow().clone();
                                            if main_id_opt.is_none() {
                                                *main_trajectory_id.borrow_mut() = Some(traj_id.clone());
                                                main_id_opt = Some(traj_id.clone());
                                            }
                                            main_id_opt.as_ref() != Some(traj_id)
                                        } else {
                                            false
                                        };

                                        if is_subagent {
                                            let traj_id = data.trajectory_id.clone().unwrap();
                                            let call_id_for_search = data.id.clone();
                                            let is_error = data.error.is_some();
                                            
                                            set_blocks.update(|bs| {
                                                fn update_status_recursive(blocks: &mut Vec<MessageBlock>, cid: &str, err: bool) -> bool {
                                                    for b in blocks.iter_mut() {
                                                        if let MessageBlock::ToolCall { call_id, status, subagent_blocks, .. } = b {
                                                            if call_id == cid {
                                                                *status = if err { ToolCallStatus::Error } else { ToolCallStatus::Done };
                                                                return true;
                                                            }
                                                            if update_status_recursive(subagent_blocks, cid, err) {
                                                                return true;
                                                            }
                                                        }
                                                    }
                                                    false
                                                }
                                                
                                                update_status_recursive(bs, &call_id_for_search, is_error);
                                                
                                                let res_id = next_id();
                                                let tool_id = data.id.clone();
                                                let tool_name = data.name.clone();
                                                let tool_result = data.result.clone();
                                                let tool_error = data.error.clone();
                                                
                                                ensure_and_update_subagent_blocks_impl(bs, &traj_id, &mut |sub_blocks| {
                                                    sub_blocks.push(MessageBlock::ToolResult {
                                                        id: res_id,
                                                        call_id: tool_id.clone(),
                                                        name: tool_name.clone(),
                                                        result: tool_result.clone(),
                                                        error: tool_error.clone(),
                                                    });
                                                });
                                            });
                                        } else {
                                            set_blocks.update(|bs| {
                                                if let Some(MessageBlock::ToolCall { status, .. }) = bs.iter_mut().find(|b| match b {
                                                    MessageBlock::ToolCall { call_id, .. } => call_id == &data.id,
                                                    _ => false,
                                                }) {
                                                    *status = if data.error.is_some() {
                                                        ToolCallStatus::Error
                                                    } else {
                                                        ToolCallStatus::Done
                                                    };
                                                }
                                                let res_id = next_id();
                                                bs.push(MessageBlock::ToolResult {
                                                    id: res_id,
                                                    call_id: data.id,
                                                    name: data.name,
                                                    result: data.result,
                                                    error: data.error,
                                                });
                                            });
                                            set_stream_tool_id.set(None);
                                        }
                                    }
                                }
                            }
                        }
                    };

                    let on_question = {
                        let main_trajectory_id = main_trajectory_id.clone();
                        move |event: &web_sys::Event| {
                            if let Ok(msg_event) = event.clone().dyn_into::<MessageEvent>() {
                                if let Some(data_str) = msg_event.data().as_string() {
                                    if let Ok(data) = serde_json::from_str::<QuestionEvent>(&data_str) {
                                        let is_subagent = {
                                            let mut main_id_opt = main_trajectory_id.borrow().clone();
                                            if main_id_opt.is_none() {
                                                *main_trajectory_id.borrow_mut() = Some(data.trajectory_id.clone());
                                                main_id_opt = Some(data.trajectory_id.clone());
                                            }
                                            main_id_opt.as_ref() != Some(&data.trajectory_id)
                                        };

                                        let id = next_id();
                                        let traj_id = data.trajectory_id.clone();
                                        let step_idx = data.step_index;
                                        let qs = data.questions.clone();

                                        if is_subagent {
                                            set_blocks.update(|bs| {
                                                ensure_and_update_subagent_blocks(bs, &data.trajectory_id, move |sub_blocks| {
                                                    sub_blocks.push(MessageBlock::Question {
                                                        id,
                                                        trajectory_id: traj_id.clone(),
                                                        step_index: step_idx,
                                                        questions: qs.clone(),
                                                        answered: false,
                                                    });
                                                });
                                            });
                                        } else {
                                            set_blocks.update(|bs| bs.push(MessageBlock::Question {
                                                id,
                                                trajectory_id: data.trajectory_id.clone(),
                                                step_index: data.step_index,
                                                questions: data.questions,
                                                answered: false,
                                            }));
                                        }

                                        set_pending_question.set(Some(PendingQuestion {
                                            trajectory_id: data.trajectory_id,
                                            step_index: data.step_index,
                                        }));
                                    }
                                }
                            }
                        }
                    };

                    let on_confirm = move |event: &web_sys::Event| {
                        if let Ok(msg_event) = event.clone().dyn_into::<MessageEvent>() {
                            if let Some(data_str) = msg_event.data().as_string() {
                                if let Ok(data) = serde_json::from_str::<ConfirmEvent>(&data_str) {
                                    let tool_name = data.tool_call.name.clone();
                                    // Store pending data in the signal so the floating panel
                                    // above the input can render it. We do NOT push an inline
                                    // MessageBlock::Confirmation here — the decided chip is
                                    // pushed only after the user acts (in on_confirm callback).
                                    set_pending_confirm.set(Some(PendingConfirm {
                                        trajectory_id: data.trajectory_id,
                                        step_index: data.step_index,
                                        tool_name,
                                        tool_call: data.tool_call,
                                    }));
                                }
                            }
                        }
                    };

                    let on_usage = move |event: &web_sys::Event| {
                        if let Ok(msg_event) = event.clone().dyn_into::<MessageEvent>() {
                            if let Some(data_str) = msg_event.data().as_string() {
                                if let Ok(data) = serde_json::from_str::<UsageEvent>(&data_str) {
                                    let id = next_id();
                                    set_blocks.update(|bs| bs.push(MessageBlock::UsageSummary {
                                        id,
                                        prompt_tokens: data.prompt_token_count,
                                        output_tokens: data.candidates_token_count,
                                        thinking_tokens: data.thoughts_token_count,
                                    }));
                                }
                            }
                        }
                    };

                    let on_compaction = {
                        let main_trajectory_id = main_trajectory_id.clone();
                        move |event: &web_sys::Event| {
                            if let Ok(msg_event) = event.clone().dyn_into::<MessageEvent>() {
                                if let Some(data_str) = msg_event.data().as_string() {
                                    if let Ok(data) = serde_json::from_str::<CompactionEvent>(&data_str) {
                                        let is_subagent = if let Some(ref traj_id) = data.trajectory_id {
                                            let mut main_id_opt = main_trajectory_id.borrow().clone();
                                            if main_id_opt.is_none() {
                                                *main_trajectory_id.borrow_mut() = Some(traj_id.clone());
                                                main_id_opt = Some(traj_id.clone());
                                            }
                                            main_id_opt.as_ref() != Some(traj_id)
                                        } else {
                                            false
                                        };

                                        if is_subagent {
                                            let traj_id = data.trajectory_id.clone().unwrap();
                                            let new_id = next_id();
                                            let step_index = data.step_index;
                                            set_blocks.update(|bs| {
                                                ensure_and_update_subagent_blocks(bs, &traj_id, move |sub_blocks| {
                                                    sub_blocks.push(MessageBlock::Compaction {
                                                        id: new_id,
                                                        step_index,
                                                    });
                                                });
                                            });
                                        } else {
                                            let id = next_id();
                                            set_blocks.update(|bs| bs.push(MessageBlock::Compaction {
                                                id,
                                                step_index: data.step_index,
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                    };

                    // 'status' events track the per-step lifecycle (ACTIVE/DONE/ERROR/…).
                    // We use them to collapse any open streaming-thinking block when the model
                    // finishes a step (DONE) and a tool is about to run. This prevents the
                    // thinking panel from remaining 'open' while the tool result comes in.
                    let on_status = {
                        let main_trajectory_id = main_trajectory_id.clone();
                        move |event: &web_sys::Event| {
                            if let Ok(msg_event) = event.clone().dyn_into::<MessageEvent>() {
                                if let Some(data_str) = msg_event.data().as_string() {
                                    if let Ok(data) = serde_json::from_str::<StatusEvent>(&data_str) {
                                        let is_subagent = if let Some(ref traj_id) = data.trajectory_id {
                                            let mut main_id_opt = main_trajectory_id.borrow().clone();
                                            if main_id_opt.is_none() {
                                                *main_trajectory_id.borrow_mut() = Some(traj_id.clone());
                                                main_id_opt = Some(traj_id.clone());
                                            }
                                            main_id_opt.as_ref() != Some(traj_id)
                                        } else {
                                            false
                                        };

                                        if is_subagent {
                                            let traj_id = data.trajectory_id.clone().unwrap();
                                            if data.status == "DONE" || data.status == "ERROR" {
                                                set_blocks.update(|bs| {
                                                    update_subagent_blocks(bs, &traj_id, |sub_blocks| {
                                                        mark_all_thinking_done(sub_blocks);
                                                    });
                                                });
                                            }
                                        } else {
                                            // When a step completes (DONE/ERROR), mark all open
                                            // Thinking blocks as no longer streaming so they collapse.
                                            if data.status == "DONE" || data.status == "ERROR" {
                                                set_blocks.update(|bs| {
                                                    for b in bs.iter_mut() {
                                                        if let MessageBlock::Thinking { is_streaming, .. } = b {
                                                            *is_streaming = false;
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    };

                    let on_finish = {
                        let main_trajectory_id = main_trajectory_id.clone();
                        move |event: &web_sys::Event| {
                            if let Ok(msg_event) = event.clone().dyn_into::<MessageEvent>() {
                                if let Some(data_str) = msg_event.data().as_string() {
                                    if let Ok(data) = serde_json::from_str::<FinishEvent>(&data_str) {
                                        let is_subagent = if let Some(ref traj_id) = data.trajectory_id {
                                            let mut main_id_opt = main_trajectory_id.borrow().clone();
                                            if main_id_opt.is_none() {
                                                *main_trajectory_id.borrow_mut() = Some(traj_id.clone());
                                                main_id_opt = Some(traj_id.clone());
                                            }
                                            main_id_opt.as_ref() != Some(traj_id)
                                        } else {
                                            false
                                        };

                                        if is_subagent {
                                            let traj_id = data.trajectory_id.clone().unwrap();
                                            let new_id = next_id();
                                            let struct_out = data.structured_output.clone();
                                            set_blocks.update(|bs| {
                                                ensure_and_update_subagent_blocks(bs, &traj_id, move |sub_blocks| {
                                                    sub_blocks.push(MessageBlock::Finish {
                                                        id: new_id,
                                                        structured_output: struct_out.clone(),
                                                    });
                                                });
                                            });
                                        } else {
                                            let id = next_id();
                                            set_blocks.update(|bs| bs.push(MessageBlock::Finish {
                                                id,
                                                structured_output: data.structured_output,
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                    };

                    // Helper: finalise the stream — mark thinking done, clear flags, save to KV.
                    // Called by either 'idle' or 'done'. Idempotent: Rc<Cell<bool>> guards
                    // against double-execution.
                    let stream_completed_fin = stream_completed.clone();
                    let set_sessions2 = set_sessions;
                    let do_finalize = Rc::new(move || {
                        if stream_completed_fin.get() {
                            return; // Already finalized
                        }
                        stream_completed_fin.set(true);

                        set_blocks.update(|bs| {
                            mark_all_thinking_done(bs);
                        });
                        set_is_streaming.set(false);
                        set_stream_text_buf.set(String::new());
                        set_stream_think_buf.set(String::new());
                        // Reset user-scroll-up flag so the next response auto-scrolls again
                        #[cfg(feature = "hydrate")]
                        { let _ = js_sys::eval("window.__userScrolledUp = false;"); }

                        let session_id = active_session_id.get_untracked().unwrap_or_default();
                        let all_blocks = blocks.get_untracked();
                        let set_sessions3 = set_sessions2;
                        // Snapshot local titles before async save so user-renames survive the
                        // KV round-trip (rename_session and save_turn_blocks can race).
                        let local_titles: std::collections::HashMap<String, String> =
                            sessions.get_untracked()
                                .into_iter()
                                .map(|s| (s.id, s.title))
                                .collect();
                        spawn_local(async move {
                            let _ = save_turn_blocks(session_id, all_blocks).await;
                            // Refresh sidebar so the auto-generated title appears after the
                            // first message, but DO NOT overwrite a user-renamed title.
                            if let Ok(mut updated) = list_sessions().await {
                                for meta in &mut updated {
                                    if let Some(local) = local_titles.get(&meta.id) {
                                        // If the user renamed it (local ≠ "New Chat") AND the
                                        // server still shows the same old value or "New Chat",
                                        // keep the local rename.  If the server has a brand-new
                                        // different value (e.g. it was saved by a concurrent
                                        // rename call that already completed), trust the server.
                                        if local != "New Chat" && meta.title == *local {
                                            // Server agrees — nothing to do.
                                        } else if local != "New Chat" && meta.title == "New Chat" {
                                            // Server hasn't caught up — keep local rename.
                                            meta.title = local.clone();
                                        }
                                        // Otherwise (server has a fresh value) trust the server.
                                    }
                                }
                                set_sessions3.set(updated);
                            }
                        });
                    });

                    // 'idle' fires just before 'done' — use it as the primary completion trigger
                    // so we finalise as soon as possible, reducing any timing gap.
                    let do_finalize_idle = do_finalize.clone();
                    let active_es_idle = active_es;
                    let active_listeners_idle = active_listeners;
                    let on_idle = move |_event: &web_sys::Event| {
                        do_finalize_idle();
                        // Close the EventSource so the browser doesn't keep the connection open.
                        active_es_idle.update_value(|opt_es| {
                            if let Some(es) = opt_es.take() {
                                es.close();
                            }
                        });
                        active_listeners_idle.set_value(None);
                    };

                    let active_es_done = active_es;
                    let active_listeners_done = active_listeners;
                    let do_finalize_done = do_finalize.clone();
                    let on_done = move |_event: &web_sys::Event| {
                        // Do NOT close if a confirm is still pending.
                        if pending_confirm.get_untracked().is_some() {
                            return;
                        }
                        do_finalize_done(); // idempotent

                        active_es_done.update_value(|opt_es| {
                            if let Some(es) = opt_es.take() {
                                es.close();
                            }
                        });
                        active_listeners_done.set_value(None);
                    };

                    // Named SSE 'error' event — server explicitly emitted event: error.
                    // Two kinds of errors arrive here:
                    //   1. Tool-step errors (tool denied, tool execution failed) — non-fatal.
                    //      These are accompanied by a tool_result event and do NOT end the stream.
                    //      We render them inline and keep the stream open.
                    //   2. Fatal stream errors (agent crashed, send failed, etc.) — close stream.
                    //      These are followed by idle+done events so stream_completed guards them.
                    // Heuristic: if the error message is the SDK permission-denial string, it is
                    // a non-fatal tool error — skip the stream-tear-down.
                    let active_es_err = active_es;
                    let active_listeners_err = active_listeners;
                    let stream_completed_err = stream_completed.clone();
                    let do_finalize_err = do_finalize.clone();
                    let on_sse_error = move |event: &web_sys::Event| {
                        // Ignore spurious errors after clean completion
                        if stream_completed_err.get() {
                            return;
                        }

                        let msg_event_opt = event.clone().dyn_into::<MessageEvent>();
                        let (message, http_code, is_fatal) = if let Ok(ref msg_event) = msg_event_opt {
                            if let Some(data_str) = msg_event.data().as_string() {
                                if let Ok(data) = serde_json::from_str::<ErrorEvent>(&data_str) {
                                    // Classify: tool-step errors (permission denial, tool execution
                                    // failures) are non-fatal — the stream continues.
                                    let fatal = !data.message.contains("denied permission")
                                        && !data.message.contains("tool call")
                                        && !data.message.contains("Tool execution")
                                        && !data.message.contains("Confirmation channel");
                                    (data.message, data.http_code, fatal)
                                } else {
                                    (data_str, None, true)
                                }
                            } else {
                                ("Error streaming response".to_string(), None, true)
                            }
                        } else {
                            // Plain Event = native browser connection error (non-MessageEvent)
                            // This fires for SSE parse failures — treat as fatal.
                            ("Connection to agent lost".to_string(), None, true)
                        };

                        // Always surface the error as a block for visibility.
                        let err_id = next_id();
                        set_blocks.update(|bs| bs.push(MessageBlock::Error {
                            id: err_id,
                            message,
                            http_code,
                        }));

                        // Only tear down the stream for truly fatal errors.
                        if is_fatal {
                            do_finalize_err();
                            active_es_err.update_value(|opt_es| {
                                if let Some(es) = opt_es.take() {
                                    es.close();
                                }
                            });
                            active_listeners_err.set_value(None);
                            set_is_streaming.set(false);
                        }
                    };

                    // Native onerror is set via the ES property so it does NOT double-fire
                    // with the addEventListener("error") above (which handles named SSE errors).
                    // We only use native onerror as a final safety net.
                    let active_es_native_err = active_es;
                    let active_listeners_native_err = active_listeners;
                    let stream_completed_nat = stream_completed.clone();
                    let do_finalize_nat = do_finalize.clone();
                    let native_err_closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| {
                        // Finalize if the stream wasn't already completed.
                        // We intentionally do NOT push an Error block here — the native
                        // onerror fires on transport-level disconnects AND when we call
                        // es.close() ourselves after a normal idle/done sequence. Any
                        // server-sent errors already arrive via the named "error" SSE
                        // event handler above.  Pushing "Connection to agent lost" from
                        // here causes false positives after every normal completion.
                        if !stream_completed_nat.get() {
                            do_finalize_nat();
                        }
                        active_es_native_err.update_value(|opt_es| {
                            if let Some(es) = opt_es.take() {
                                es.close();
                            }
                        });
                        active_listeners_native_err.set_value(None);
                    });
                    es.set_onerror(Some(native_err_closure.as_ref().unchecked_ref()));
                    // Store the closure in a StoredValue so it lives exactly as long as the
                    // reactive owner, preventing "disposed reactive value" panics from a
                    // leaked (forget()-ed) closure that fires after the owner is dropped.
                    active_onerror_closure.set_value(Some(native_err_closure));

                    let target: &web_sys::EventTarget = es.as_ref();
                    let list = vec![
                        EventListener::new(target, "token", on_token),
                        EventListener::new(target, "thought", on_thought),
                        EventListener::new(target, "status", on_status),
                        EventListener::new(target, "tool_start", on_tool_start),
                        EventListener::new(target, "tool_result", on_tool_result),
                        EventListener::new(target, "question", on_question),
                        EventListener::new(target, "confirm", on_confirm),
                        EventListener::new(target, "usage", on_usage),
                        EventListener::new(target, "compaction", on_compaction),
                        EventListener::new(target, "finish", on_finish),
                        EventListener::new(target, "idle", on_idle),
                        EventListener::new(target, "error", on_sse_error),
                        EventListener::new(target, "done", on_done),
                    ];

                    active_listeners.set_value(Some(list));
                    active_es.set_value(Some(es));
                }
                Err(e) => {
                    set_is_streaming.set(false);
                    set_error_text.set(Some(format!("Failed to connect to agent server: {:?}", e)));
                }
            }
        }
    };

    let on_send = move |_| do_send();

    // Open Folder handler (hydrate only)
    // Resolved paths from a showDirectoryPicker() name lookup
    let (resolve_results, set_resolve_results) = signal(Vec::<String>::new());

    // Apply workspace path: POST /workspace and update UI state
    let apply_workspace = move |_path: String| {
        #[cfg(feature = "hydrate")]
        {
            let path = _path;
            let session_id = active_session_id.get_untracked().unwrap_or_default();
            spawn_local(async move {
                let agent_url = get_agent_server_url();
                match gloo_net::http::Request::post(&format!("{}/workspace", agent_url))
                    .json(&serde_json::json!({ "session_id": session_id, "path": path }))
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if resp.ok() {
                            if let Ok(body) = resp.json::<serde_json::Value>().await {
                                let ws = body["workspace"].as_str().unwrap_or("").to_string();
                                set_workspace_path.set(Some(ws));
                                set_show_open_folder.set(false);
                                set_open_folder_error.set(None);
                                set_resolve_results.set(Vec::new());
                                set_blocks.set(Vec::new());
                            }
                        } else if let Ok(body) = resp.json::<serde_json::Value>().await {
                            let err = body["error"].as_str().unwrap_or("Unknown error").to_string();
                            set_open_folder_error.set(Some(err));
                            set_show_open_folder.set(true);
                        }
                    }
                    Err(e) => {
                        set_open_folder_error.set(Some(format!("Request failed: {e}")));
                        set_show_open_folder.set(true);
                    }
                }
            });
        }
    };

    // Open the browser-native folder picker via showDirectoryPicker(),
    // then resolve the folder name to a full path on the backend.
    let open_native_picker = move || {
        #[cfg(feature = "hydrate")]
        {
            let agent_url = get_agent_server_url();
            spawn_local(async move {
                use wasm_bindgen::prelude::*;
                use wasm_bindgen_futures::JsFuture;

                // Call window.showDirectoryPicker() via JS reflection
                let window = web_sys::window().expect("no global window");
                let picker_fn = js_sys::Reflect::get(&window, &JsValue::from_str("showDirectoryPicker")).ok();
                let Some(picker_fn) = picker_fn else {
                    // Browser doesn't support showDirectoryPicker — show manual modal
                    set_show_open_folder.set(true);
                    return;
                };
                let Ok(func) = picker_fn.dyn_into::<js_sys::Function>() else {
                    set_show_open_folder.set(true);
                    return;
                };

                // Show the OS-native folder picker (this blocks until user picks or cancels)
                let promise = match func.call0(&window).and_then(|v| v.dyn_into::<js_sys::Promise>()) {
                    Ok(p) => p,
                    Err(_) => { set_show_open_folder.set(true); return; }
                };

                let handle = match JsFuture::from(promise).await {
                    Ok(h) => h,
                    Err(_) => return, // User hit Cancel — do nothing
                };

                // Get the folder name from the FileSystemDirectoryHandle
                let name = js_sys::Reflect::get(&handle, &JsValue::from_str("name"))
                    .ok()
                    .and_then(|v| v.as_string())
                    .unwrap_or_default();

                if name.is_empty() {
                    set_show_open_folder.set(true);
                    return;
                }

                // Ask backend to resolve name → full path(s) via `find`
                let name_enc = js_sys::encode_uri_component(&name).as_string().unwrap_or_else(|| name.clone());
                let url = format!("{}/resolve/folder?name={}", agent_url, name_enc);

                match gloo_net::http::Request::get(&url).send().await {
                    Ok(resp) => {
                        if let Ok(body) = resp.json::<serde_json::Value>().await {
                            let paths: Vec<String> = body["paths"]
                                .as_array()
                                .unwrap_or(&vec![])
                                .iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect();

                            match paths.len() {
                                0 => {
                                    // Not found — show modal with the name pre-filled for manual completion
                                    set_open_folder_input.set(format!("/{name}"));
                                    set_open_folder_error.set(Some(format!("Could not locate \"{name}\" under home directory. Enter the full path manually.")));
                                    set_show_open_folder.set(true);
                                }
                                1 => {
                                    // Exactly one match — apply immediately, no confirmation needed
                                    apply_workspace(paths.into_iter().next().unwrap());
                                }
                                _ => {
                                    // Multiple matches — show picker modal with choices
                                    set_resolve_results.set(paths);
                                    set_open_folder_input.set(String::new());
                                    set_open_folder_error.set(None);
                                    set_show_open_folder.set(true);
                                }
                            }
                        }
                    }
                    Err(_) => {
                        set_show_open_folder.set(true);
                    }
                }
            });
        }
    };

    // Sidebar "Open Folder" button
    let on_open_folder_sidebar = move |_| open_native_picker();

    // "Browse..." button inside the modal (same picker, fills the input)
    let on_browse_in_modal = move |_| open_native_picker();

    // "Open" button in the modal (manual path entry)
    let on_open_folder = move |_| {
        #[cfg(feature = "hydrate")]
        {
            let path = open_folder_input.get_untracked();
            if path.trim().is_empty() { return; }
            apply_workspace(path);
        }
    };

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            do_send();
        }
    };

    view! {
        <div class=move || if dark_mode.get() {
            "dark h-screen w-screen flex bg-[#212121] text-[#ececec] overflow-hidden"
        } else {
            "h-screen w-screen flex bg-white text-[#0d0d0d] overflow-hidden"
        }>
            // Left Sidebar
            <aside class=move || format!(
                "w-64 flex-shrink-0 bg-[#f9f9f9] dark:bg-[#171717] border-r border-[#e5e5e7] dark:border-[#2f2f2f] flex flex-col h-full transition-transform duration-300 z-30 \
                 fixed inset-y-0 left-0 md:static md:translate-x-0 {}",
                if sidebar_open.get() { "translate-x-0" } else { "-translate-x-full" }
            )>
                // Sidebar Header
                <div class="p-3 flex items-center justify-between border-b border-[#e5e5e7] dark:border-[#2f2f2f]">
                    <span class="font-semibold text-sm tracking-tight text-gray-800 dark:text-gray-200">
                        "Antigravity"
                    </span>
                    <button
                        on:click=move |_| set_sidebar_open.set(false)
                        class="md:hidden p-1.5 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-800 text-gray-500 dark:text-gray-400"
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                // Sidebar Actions
                <div class="p-3 space-y-2">
                    <button
                        on:click=on_new_chat
                        class="w-full flex items-center gap-2 px-3 py-2 text-sm font-medium rounded-lg border border-[#e5e5e7] dark:border-[#2f2f2f] bg-white dark:bg-[#212121] hover:bg-gray-50 dark:hover:bg-[#2f2f2f] text-gray-800 dark:text-gray-200 transition-colors"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                        </svg>
                        "New chat"
                    </button>
                    // Open Folder button: try native picker first
                    <button
                        id="btn-open-folder"
                        on:click=on_open_folder_sidebar
                        class="flex items-center gap-2 w-full px-3 py-2 rounded-lg text-sm font-medium text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-[#252525] transition-colors"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V7z" />
                        </svg>
                        "Open Folder"
                    </button>
                    // Workspace indicator
                    <Show when=move || workspace_path.get().is_some()>
                        <div
                            class="mx-0 px-2 py-1 rounded text-[10px] font-mono text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950/20 border border-emerald-200 dark:border-emerald-900/30 truncate"
                            title=move || workspace_path.get().unwrap_or_default()
                        >
                            {move || workspace_path.get()
                                .map(|p| {
                                    let path = std::path::Path::new(&p);
                                    path.file_name()
                                        .map(|n| format!("\u{1F4C1} {}", n.to_string_lossy()))
                                        .unwrap_or_else(|| format!("\u{1F4C1} {}", p))
                                })
                                .unwrap_or_default()
                            }
                        </div>
                    </Show>
                </div>

                // Recent Chats Scroll Area
                <div class="flex-1 overflow-y-auto px-3 py-2 space-y-1">
                    <div class="text-xs font-semibold text-gray-400 dark:text-gray-500 px-3 py-2 uppercase tracking-wider">
                        "Recent"
                    </div>
                    <For
                        each=move || sessions.get()
                        key=|sess| sess.id.clone()
                        let:sess
                    >
                        {
                            let sid = sess.id.clone();
                            let sid_click = sess.id.clone();
                            let sid_del = sess.id.clone();
                            let sid_active = sid.clone();
                            let is_active = move || active_session_id.get() == Some(sid_active.clone());
                            


                            view! {
                                <div class=move || format!(
                                    "group flex items-center justify-between gap-2 px-3 py-2 text-sm rounded-lg cursor-pointer sidebar-chat-item {}",
                                    if is_active() { "active" } else { "" }
                                )
                                on:click=move |_| {
                                    if editing_session_id.get_untracked().is_none() {
                                        set_active_session_id.set(Some(sid_click.clone()));
                                    }
                                }
                                >
                                    {move || {
                                        let sid_edit = sid.clone();
                                        let is_editing = editing_session_id.get() == Some(sid_edit.clone());
                                        if is_editing {
                                            let sid_save = sid_edit.clone();
                                            let on_submit = move |new_val: String| {
                                                let new_title = new_val.trim().to_string();
                                                if !new_title.is_empty() {
                                                    let target_sid = sid_save.clone();
                                                    // Optimistic local update
                                                    set_sessions.update(|list| {
                                                        if let Some(pos) = list.iter().position(|s| s.id == target_sid) {
                                                            list[pos].title = new_title.clone();
                                                        }
                                                    });
                                                    // Persist to backend KV store
                                                    spawn_local(async move {
                                                        let _ = rename_session(target_sid, new_title).await;
                                                    });
                                                }
                                                set_editing_session_id.set(None);
                                            };
                                            let on_submit_key = on_submit.clone();
                                            let on_submit_blur = on_submit.clone();
                                            view! {
                                                <div class="flex items-center gap-2 w-full" on:click=move |ev| ev.stop_propagation()>
                                                    <svg class="w-4 h-4 flex-shrink-0 text-blue-500" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                                                    </svg>
                                                    <input
                                                        type="text"
                                                        class="flex-1 bg-white dark:bg-gray-900 border border-blue-500 rounded px-1.5 py-0.5 text-xs outline-none text-gray-900 dark:text-gray-100 font-normal w-full"
                                                        node_ref=edit_input_ref
                                                        prop:value=edit_title_input
                                                        on:input=move |ev| set_edit_title_input.set(event_target_value(&ev))
                                                        on:keydown=move |ev| {
                                                            if ev.key() == "Enter" {
                                                                ev.prevent_default();
                                                                on_submit_key(edit_title_input.get_untracked());
                                                            } else if ev.key() == "Escape" {
                                                                set_editing_session_id.set(None);
                                                            }
                                                        }
                                                        on:blur=move |_| {
                                                            on_submit_blur(edit_title_input.get_untracked());
                                                        }
                                                    />
                                                </div>
                                            }.into_any()
                                        } else {
                                            let sid_btn_edit = sid_edit.clone();
                                            let sid_title = sid_edit.clone();
                                            let title = move || {
                                                sessions.with(|list| {
                                                    list.iter()
                                                        .find(|s| s.id == sid_title)
                                                        .map(|s| s.title.clone())
                                                        .unwrap_or_else(|| "".to_string())
                                                })
                                            };
                                            let cur_title_fn = title.clone();
                                            let do_start_edit = move |ev: leptos::web_sys::MouseEvent| {
                                                ev.stop_propagation();
                                                set_edit_title_input.set(cur_title_fn());
                                                set_editing_session_id.set(Some(sid_btn_edit.clone()));
                                            };
                                            let do_start_edit_dbl = do_start_edit.clone();
                                            let sid_del_btn = sid_del.clone();
                                            
                                            view! {
                                                <div class="flex items-center gap-2 truncate w-full" on:dblclick=do_start_edit_dbl>
                                                    <svg class="w-4 h-4 flex-shrink-0 text-gray-400 dark:text-gray-500 group-hover:text-gray-500 dark:group-hover:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                                                    </svg>
                                                    <span class="truncate">{title}</span>
                                                </div>
                                                <div class="flex items-center gap-0.5">
                                                    <button
                                                        on:click=do_start_edit
                                                        class="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-400 hover:text-blue-500 transition-all"
                                                        title="Rename chat"
                                                    >
                                                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                                                        </svg>
                                                    </button>
                                                    <button
                                                        on:click=move |ev| {
                                                            ev.stop_propagation();
                                                            do_delete_session(sid_del_btn.clone());
                                                        }
                                                        class="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-400 hover:text-red-500 transition-all"
                                                        title="Delete chat"
                                                    >
                                                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                                        </svg>
                                                    </button>
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                            }
                        }
                    </For>
                </div>

                // Sidebar Footer Settings
                <div class="p-3 border-t border-[#e5e5e7] dark:border-[#2f2f2f] space-y-1">
                    <button
                        on:click=toggle_dark_mode
                        class="w-full flex items-center gap-3 px-3 py-2.5 text-sm font-medium rounded-lg text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                    >
                        {move || if dark_mode.get() {
                            view! {
                                <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364-6.364l-.707.707M6.343 17.657l-.707.707m0-11.314l.707.707m11.314 11.314l.707-.707M12 7a5 5 0 100 10 5 5 0 000-10z" />
                                </svg>
                                <span>"Light theme"</span>
                            }.into_view()
                        } else {
                            view! {
                                <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
                                </svg>
                                <span>"Dark theme"</span>
                            }.into_view()
                        }}
                    </button>

                    <button
                        on:click=on_clear
                        class="w-full flex items-center gap-3 px-3 py-2.5 text-sm font-medium rounded-lg text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/20 transition-colors"
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                        <span>"Clear conversation"</span>
                    </button>
                </div>
            </aside>

            // Mobile Overlay Backdrop
            <Show when=move || sidebar_open.get()>
                <div
                    on:click=move |_| set_sidebar_open.set(false)
                    class="fixed inset-0 bg-black/40 z-20 md:hidden"
                ></div>
            </Show>

            // Main Chat Area
            <div class="flex-1 flex flex-row h-full overflow-hidden bg-white dark:bg-[#212121] transition-colors duration-200">
                <div class="flex-1 flex flex-col h-full overflow-hidden relative">
                    // Top Bar
                    <header class="h-14 border-b border-[#e5e5e7] dark:border-[#2f2f2f] flex items-center px-4 justify-between bg-white dark:bg-[#212121] flex-shrink-0 z-10">
                        <div class="flex items-center gap-2">
                            <button
                                on:click=move |_| set_sidebar_open.update(|o| *o = !*o)
                                class="md:hidden p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-500 dark:text-gray-400"
                            >
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                                </svg>
                            </button>

                            <div class="flex items-center gap-1.5 font-semibold text-base text-gray-900 dark:text-gray-100 select-none">
                                {active_session_title}
                                <span class="text-xs font-normal text-gray-400 dark:text-gray-500">
                                    "v0.1.4"
                                </span>
                            </div>
                        </div>

                        <div class="flex items-center gap-3">
                            <button
                                on:click=move |_| set_hub_open.update(|open| *open = !*open)
                                class=move || {
                                    let active = if hub_open.get() {
                                        "text-blue-500 bg-blue-500/10 border-blue-500/20"
                                    } else {
                                        "text-gray-500 dark:text-zinc-400 hover:bg-gray-100 dark:hover:bg-zinc-800 border-transparent"
                                    };
                                    format!("p-2 rounded-lg border text-xs font-medium transition-all duration-200 flex items-center gap-1.5 {}", active)
                                }
                                title="Agent Status Hub"
                            >
                                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2" />
                                </svg>
                                <span class="hidden sm:inline">"Status Hub"</span>
                            </button>

                            <div class="flex items-center gap-2">
                                <div class={move || {
                                    if is_streaming.get() {
                                        "w-2 h-2 rounded-full bg-[#10a37f] animate-pulse"
                                    } else {
                                        "w-2 h-2 rounded-full bg-[#10a37f]"
                                    }
                                }}></div>
                                <span class="text-xs text-gray-500 dark:text-gray-400 font-medium">
                                    {move || {
                                        if is_streaming.get() {
                                            "Streaming"
                                        } else {
                                            "Ready"
                                        }
                                    }}
                                </span>
                            </div>
                        </div>
                    </header>

                // Messages Viewport
                <div class="flex-1 overflow-y-auto" id="chat-messages-container" node_ref=messages_container_ref>
                    // Empty State
                    <Show when=move || blocks.get().is_empty() && !is_streaming.get()>
                        <div class="max-w-3xl mx-auto w-full px-4 h-full flex flex-col items-center justify-center text-center py-20">
                            <h2 class="text-[#0d0d0d] dark:text-white text-3xl font-semibold mb-8 tracking-tight">
                                "What's on the agenda today?"
                            </h2>

                            // Prompt Pills
                            <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 w-full max-w-2xl px-4">
                                <button
                                    on:click=move |_| {
                                        set_input_text.set("Search the codebase for all usages of `unsafe` blocks.".to_string());
                                        if let Some(el) = textarea_ref.get() {
                                            el.set_value("Search the codebase for all usages of `unsafe` blocks.");
                                            let _ = el.focus();
                                        }
                                    }
                                    class="p-4 rounded-2xl border border-[#e5e5e7] dark:border-[#2f2f2f] text-left hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors text-sm"
                                >
                                    <div class="font-medium text-gray-800 dark:text-gray-200 mb-1">
                                        "Audit Codebase"
                                    </div>
                                    <div class="text-xs text-gray-500 dark:text-gray-400 line-clamp-2">
                                        "Search the codebase for all usages of `unsafe` blocks."
                                    </div>
                                </button>

                                <button
                                    on:click=move |_| {
                                        set_input_text.set("Run the tests and explain any failures.".to_string());
                                        if let Some(el) = textarea_ref.get() {
                                            el.set_value("Run the tests and explain any failures.");
                                            let _ = el.focus();
                                        }
                                    }
                                    class="p-4 rounded-2xl border border-[#e5e5e7] dark:border-[#2f2f2f] text-left hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors text-sm"
                                >
                                    <div class="font-medium text-gray-800 dark:text-gray-200 mb-1">
                                        "Test Suite"
                                    </div>
                                    <div class="text-xs text-gray-500 dark:text-gray-400 line-clamp-2">
                                        "Run the tests and explain any failures."
                                    </div>
                                </button>

                                <button
                                    on:click=move |_| {
                                        set_input_text.set("Write a migration guide from Leptos 0.6 to 0.8.".to_string());
                                        if let Some(el) = textarea_ref.get() {
                                            el.set_value("Write a migration guide from Leptos 0.6 to 0.8.");
                                            let _ = el.focus();
                                        }
                                    }
                                    class="p-4 rounded-2xl border border-[#e5e5e7] dark:border-[#2f2f2f] text-left hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors text-sm"
                                >
                                    <div class="font-medium text-gray-800 dark:text-gray-200 mb-1">
                                        "Technical Writing"
                                    </div>
                                    <div class="text-xs text-gray-500 dark:text-gray-400 line-clamp-2">
                                        "Write a migration guide from Leptos 0.6 to 0.8."
                                    </div>
                                </button>
                            </div>
                        </div>
                    </Show>

                    // Active Conversation Messages (unified blocks timeline)
                    <div class="max-w-3xl mx-auto w-full px-4 py-8 flex flex-col gap-8">
                        // Direct reactive iteration: re-evaluates on every `blocks` update.
                        // We intentionally do NOT use <For key=id> here because streaming
                        // mutates block content (tokens appending, is_streaming toggling) without
                        // changing the id key — which would cause <For> to reuse stale DOM nodes.
                        // Chat conversations have a small number of blocks so full re-render is fine.
                        // Filter out short AssistantMessages that match a tool label
                        // (the agent emits e.g. "Change Directory" as both a text step
                        // and as the step.content for the ToolCall — we use it as the
                        // card title and suppress the redundant standalone bubble).
                        {move || {
                            let mut all_blocks = blocks.get();
                            // Reorder: ensure Thinking blocks appear before adjacent
                            // AssistantMessage blocks.  The server may emit text tokens
                            // before thinking deltas for the same step, causing
                            // AssistantMessage to be pushed before Thinking in the vec.
                            {
                                let mut i = 1;
                                while i < all_blocks.len() {
                                    if matches!(all_blocks[i], MessageBlock::Thinking { .. })
                                        && matches!(all_blocks[i - 1], MessageBlock::AssistantMessage { .. })
                                    {
                                        all_blocks.swap(i - 1, i);
                                    }
                                    i += 1;
                                }
                            }
                            // Collect all non-empty short labels from ToolCall blocks
                            let tool_labels: std::collections::HashSet<String> = all_blocks.iter()
                                .filter_map(|b| match b {
                                    MessageBlock::ToolCall { label, .. } => {
                                        label.as_ref()
                                            .map(|l| l.trim().to_lowercase())
                                            .filter(|l| !l.is_empty() && l.len() < 60)
                                    }
                                    _ => None,
                                })
                                .collect();
                            all_blocks.into_iter()
                                .filter(|block| match block {
                                    // Hide short AssistantMessages whose text is used
                                    // as a tool card label — they'd be redundant.
                                    MessageBlock::AssistantMessage { content, .. } => {
                                        let t = content.trim().to_lowercase();
                                        !(t.len() < 60 && tool_labels.contains(&t))
                                    }
                                    _ => true,
                                })
                                .map(|block| view! {
                                    <MessageBlockView block=block.clone() on_answer=on_answer />
                                })
                                .collect_view()
                        }}

                        // Bouncing dots loading indicator
                        <Show when=move || {
                            is_streaming.get() && !blocks.get().iter().any(|b| {
                                matches!(b, MessageBlock::Thinking { .. } | MessageBlock::AssistantMessage { .. } | MessageBlock::ToolCall { .. })
                            })
                        }>
                            <div class="flex w-full gap-4 justify-start">
                                <div class="flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold bg-black dark:bg-white text-white dark:text-black select-none">
                                    "A"
                                </div>
                                <div class="flex flex-col items-start max-w-[85%] w-full">
                                    <div class="text-[11px] text-gray-400 dark:text-gray-500 mb-1 font-medium select-none">
                                        "Agent"
                                    </div>
                                    <div class="text-sm text-gray-500 dark:text-gray-400 flex items-center gap-2">
                                        <div class="flex gap-1 py-1">
                                            <div class="w-1.5 h-1.5 bg-[#10a37f] rounded-full animate-bounce" style="animation-delay: 0ms"></div>
                                            <div class="w-1.5 h-1.5 bg-[#10a37f] rounded-full animate-bounce" style="animation-delay: 150ms"></div>
                                            <div class="w-1.5 h-1.5 bg-[#10a37f] rounded-full animate-bounce" style="animation-delay: 300ms"></div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </Show>

                        // Error toast formatted beautifully
                        <Show when=move || error_text.get().is_some()>
                            <div class="flex w-full gap-4 justify-start">
                                <div class="flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold bg-red-600 text-white select-none">
                                    "E"
                                </div>
                                <div class="flex flex-col items-start max-w-[85%]">
                                    <div class="text-[11px] text-red-500 mb-1 font-medium select-none">
                                        "Error"
                                    </div>
                                    <div class="bg-red-50 dark:bg-red-950/20 border border-red-200 dark:border-red-900/50 rounded-2xl px-4 py-2.5 text-sm text-red-800 dark:text-red-200 whitespace-pre-wrap">
                                        {move || error_text.get().unwrap_or_default()}
                                    </div>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>

                // Bottom Input Area
                <div class="border-t border-[#e5e5e7] dark:border-[#2f2f2f] bg-white dark:bg-[#212121] py-4">
                    <div class="max-w-3xl mx-auto w-full px-4">

                        // ── Floating Permission Panel ──────────────────────────────
                        // Shown above the input when the agent requests a confirmation.
                        // Replaces the old inline-in-chat pending card. Dismissed
                        // automatically when the user accepts or denies.
                        <Show when=move || pending_confirm.get().is_some()>
                            {move || {
                                if let Some(ref pc) = pending_confirm.get() {
                                    let tool_name = pc.tool_call.name.clone();
                                    let args_str = serde_json::to_string_pretty(&pc.tool_call.args)
                                        .unwrap_or_else(|_| pc.tool_call.args.to_string());
                                    let path_display = pc.tool_call.canonical_path.as_deref()
                                        .map(shorten_path);
                                    let on_confirm_deny = on_confirm;
                                    let on_confirm_once = on_confirm;
                                    let on_confirm_sess = on_confirm;
                                    view! {
                                        <div class="mb-3 animate-in slide-in-from-bottom-2 duration-200">
                                            <div class="border border-amber-300 dark:border-amber-700/50 bg-amber-50 dark:bg-amber-950/20 rounded-2xl shadow-lg overflow-hidden">
                                                // Header
                                                <div class="flex items-center gap-2 px-4 py-2.5 bg-amber-100/80 dark:bg-amber-900/20 border-b border-amber-200 dark:border-amber-800/30">
                                                    <span class="text-sm">{"🔒"}</span>
                                                    <span class="font-bold text-xs uppercase tracking-wide text-amber-800 dark:text-amber-300">{"Permission Required"}</span>
                                                    <span class="ml-1 font-mono text-[10px] px-1.5 py-0.5 rounded bg-amber-200 dark:bg-amber-800/40 text-amber-900 dark:text-amber-200 uppercase font-bold">
                                                        {tool_name}
                                                    </span>
                                                    {path_display.map(|p| view! {
                                                        <span class="font-mono text-[10px] opacity-60 truncate max-w-[220px]">{p}</span>
                                                    })}
                                                </div>
                                                // Args preview
                                                <div class="px-4 py-2.5">
                                                    <pre class="text-[11px] font-mono text-gray-600 dark:text-gray-400 overflow-x-auto max-h-[80px] whitespace-pre">
                                                        {args_str}
                                                    </pre>
                                                </div>
                                                // Actions
                                                <div class="flex items-center justify-between gap-2 px-4 py-2.5 border-t border-amber-200/60 dark:border-amber-800/30 bg-amber-50/50 dark:bg-amber-950/10">
                                                    <button
                                                        on:click=move |_| on_confirm_deny.run((false, false))
                                                        class="px-4 py-1.5 rounded-lg text-xs font-semibold border border-red-200 dark:border-red-800/40 hover:bg-red-50 dark:hover:bg-red-950/20 text-red-600 dark:text-red-400 transition-colors"
                                                    >
                                                        {"Deny"}
                                                    </button>
                                                    <div class="flex items-center gap-2">
                                                        <button
                                                            on:click=move |_| on_confirm_once.run((true, false))
                                                            class="px-4 py-1.5 rounded-lg text-xs font-semibold bg-amber-600 hover:bg-amber-700 text-white shadow transition-colors"
                                                        >
                                                            {"Allow Once"}
                                                        </button>
                                                        <button
                                                            on:click=move |_| on_confirm_sess.run((true, true))
                                                            class="px-4 py-1.5 rounded-lg text-xs font-semibold bg-emerald-600 hover:bg-emerald-700 text-white shadow transition-colors"
                                                        >
                                                            {"Allow for Session"}
                                                        </button>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    ().into_any()
                                }
                            }}
                        </Show>

                        <div class="relative flex items-end bg-[#f4f4f4] dark:bg-[#2f2f2f] rounded-3xl p-2 border border-transparent focus-within:border-gray-300 dark:focus-within:border-gray-700 transition-colors">
                            <textarea
                                node_ref=textarea_ref
                                prop:value=move || input_text.get()
                                on:input=move |ev| {
                                    set_input_text.set(event_target_value(&ev));
                                    #[cfg(feature = "hydrate")]
                                    {
                                        use wasm_bindgen::JsCast;
                                        if let Some(target) = ev.target() {
                                            if let Ok(el) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                                                if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                                                    let _ = web_sys::HtmlElement::style(html_el).set_property("height", "auto");
                                                    let scroll_height = el.scroll_height();
                                                    let _ = web_sys::HtmlElement::style(html_el).set_property("height", &format!("{scroll_height}px"));
                                                }
                                            }
                                        }
                                    }
                                }
                                on:keydown=on_keydown
                                placeholder="Message Antigravity..."
                                rows="1"
                                class="flex-1 bg-transparent resize-none outline-none text-[#0d0d0d] dark:text-[#ececec] placeholder-gray-500 dark:placeholder-gray-400 text-base py-2 px-3 overflow-y-auto max-h-48"
                            ></textarea>
                            {move || if is_streaming.get() {
                                view! {
                                    <button
                                        on:click=on_halt
                                        class="flex items-center justify-center w-8 h-8 rounded-full bg-red-600 hover:bg-red-700 text-white shadow-md transition-all ml-2 mb-1"
                                        title="Stop stream"
                                    >
                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2.5" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                                        </svg>
                                    </button>
                                }.into_any()
                            } else {
                                view! {
                                    <button
                                        on:click=on_send
                                        disabled=move || input_text.get().trim().is_empty()
                                        class="flex items-center justify-center w-8 h-8 rounded-full bg-black dark:bg-[#ececec] text-white dark:text-[#212121] disabled:opacity-20 disabled:cursor-not-allowed hover:opacity-85 active:scale-95 transition-all ml-2 mb-1"
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4">
                                            <line x1="12" y1="19" x2="12" y2="5"></line>
                                            <polyline points="5 12 12 5 19 12"></polyline>
                                        </svg>
                                    </button>
                                }.into_any()
                            }}
                        </div>
                        <div class="text-[11px] text-center text-gray-500 dark:text-gray-400 mt-2 select-none">
                            "Antigravity Chat is powered by Gemini and Spin WASI. Messages are stored locally in KV."
                        </div>
                    </div>
                </div>
            </div>

            // Open Folder Modal — shown when multiple matches found, or as manual entry fallback
            <Show when=move || show_open_folder.get()>
                <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
                    <div class="bg-white dark:bg-[#1c1c1c] border border-gray-200 dark:border-[#2f2f2f] rounded-2xl shadow-2xl w-full max-w-md mx-4 p-6 space-y-4">
                        // Header
                        <div class="flex items-center justify-between">
                            <h2 class="text-base font-bold text-gray-900 dark:text-gray-100">"Open Workspace Folder"</h2>
                            <button
                                on:click=move |_| {
                                    set_show_open_folder.set(false);
                                    set_open_folder_error.set(None);
                                    set_resolve_results.set(Vec::new());
                                }
                                class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
                            >
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        </div>

                        // Multiple matches — show a clickable list to choose from
                        <Show when=move || !resolve_results.get().is_empty()>
                            <p class="text-sm text-gray-500 dark:text-gray-400">
                                "Multiple folders found with that name. Pick one:"
                            </p>
                            <div class="space-y-1 max-h-48 overflow-y-auto">
                                <For
                                    each=move || resolve_results.get()
                                    key=|p| p.clone()
                                    children=move |path| {
                                        let p = path.clone();
                                        let t = path.clone();
                                        view! {
                                            <button
                                                on:click=move |_| apply_workspace(p.clone())
                                                class="w-full text-left px-3 py-2.5 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-400 dark:hover:border-blue-500 hover:bg-blue-50 dark:hover:bg-blue-950/20 text-sm font-mono text-gray-800 dark:text-gray-200 transition-colors truncate"
                                                title=t
                                            >
                                                {path}
                                            </button>
                                        }
                                    }
                                />
                            </div>
                            // Divider
                            <div class="flex items-center gap-3">
                                <div class="flex-1 h-px bg-gray-200 dark:bg-gray-700" />
                                <span class="text-xs text-gray-400 dark:text-gray-500">"or pick a different folder"</span>
                                <div class="flex-1 h-px bg-gray-200 dark:bg-gray-700" />
                            </div>
                        </Show>

                        // Browse button (opens native picker again)
                        <button
                            on:click=on_browse_in_modal
                            class="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-xl border-2 border-dashed border-gray-300 dark:border-gray-600 hover:border-blue-400 dark:hover:border-blue-500 hover:bg-blue-50 dark:hover:bg-blue-950/20 text-gray-600 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 text-sm font-medium transition-colors group"
                        >
                            <svg class="w-5 h-5 group-hover:scale-110 transition-transform" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V7z" />
                            </svg>
                            "Browse\u{2026}"
                        </button>

                        // Manual path entry
                        <div class="flex items-center gap-3">
                            <div class="flex-1 h-px bg-gray-200 dark:bg-gray-700" />
                            <span class="text-xs text-gray-400 dark:text-gray-500">"or enter path manually"</span>
                            <div class="flex-1 h-px bg-gray-200 dark:bg-gray-700" />
                        </div>
                        <input
                            type="text"
                            id="open-folder-input"
                            placeholder="/Users/you/my-project"
                            prop:value=move || open_folder_input.get()
                            on:input=move |ev| set_open_folder_input.set(event_target_value(&ev))
                            class="w-full px-3 py-2.5 rounded-lg border border-gray-300 dark:border-gray-700 bg-gray-50 dark:bg-[#252525] text-sm text-gray-900 dark:text-gray-100 font-mono placeholder-gray-400 dark:placeholder-gray-600 outline-none focus:border-blue-500 dark:focus:border-blue-400 transition-colors"
                        />
                        <Show when=move || open_folder_error.get().is_some()>
                            <p class="text-xs text-red-500 dark:text-red-400">
                                {move || open_folder_error.get().unwrap_or_default()}
                            </p>
                        </Show>
                        <div class="flex items-center justify-end gap-3 pt-2">
                            <button
                                on:click=move |_| {
                                    set_show_open_folder.set(false);
                                    set_open_folder_error.set(None);
                                    set_resolve_results.set(Vec::new());
                                }
                                class="px-4 py-2 rounded-lg text-sm font-medium border border-gray-300 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-[#2a2a2a] text-gray-700 dark:text-gray-300 transition-colors"
                            >
                                "Cancel"
                            </button>
                            <button
                                on:click=on_open_folder
                                class="px-4 py-2 rounded-lg text-sm font-bold bg-blue-600 hover:bg-blue-700 text-white shadow-sm transition-colors"
                            >
                                "Open"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
            </div>

            // Glassmorphic status hub sidebar
            <aside class=move || {
                let width = if hub_open.get() {
                    "w-80 border-l border-gray-200/10 dark:border-white/5 opacity-100 visible"
                } else {
                    "w-0 opacity-0 invisible"
                };
                format!("h-full flex flex-col transition-all duration-300 ease-in-out bg-white/60 dark:bg-[#151515]/60 backdrop-blur-xl overflow-hidden shrink-0 {}", width)
            }>
                <div class="p-4 border-b border-gray-200/10 dark:border-white/5 flex items-center justify-between">
                    <h3 class="text-sm font-semibold text-gray-800 dark:text-gray-200">
                        "Agent Status Hub"
                    </h3>
                    <button
                        on:click=move |_| set_hub_open.set(false)
                        class="p-1 rounded-md text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
                    >
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <div class="flex-1 overflow-y-auto p-4 space-y-4 custom-scrollbar">
                    {move || {
                        let subagents = collect_subagents_recursive(&blocks.get());
                        if subagents.is_empty() {
                            view! {
                                <div class="flex flex-col items-center justify-center h-48 text-center text-gray-400 dark:text-zinc-500">
                                    <svg class="w-8 h-8 mb-2 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                                    </svg>
                                    <span class="text-xs">"No active subagents"</span>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="flex flex-col space-y-4">
                                    {subagents.into_iter().map(|item| {
                                        view! {
                                            <SubagentStatusItem item=item depth=0 />
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </aside>
        </div>
    }
}


/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        if let Some(resp) = use_context::<leptos_wasi::response::ResponseOptions>() {
            resp.set_status(leptos_wasi::prelude::StatusCode::NOT_FOUND);
        }
    }

    view! { <h1 class="text-[#0d0d0d] dark:text-[#ececec] text-center py-20 text-2xl">"Not Found"</h1> }
}

#[cfg(feature = "ssr")]
async fn send_wasi_http_post(
    scheme: &str,
    authority: &str,
    path: &str,
    headers: &[(String, Vec<u8>)],
    body: &[u8],
) -> Result<(u16, Vec<u8>), String> {
    struct SimpleBody(Option<bytes::Bytes>);

    impl http_body::Body for SimpleBody {
        type Data = bytes::Bytes;
        type Error = std::io::Error;

        fn poll_frame(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
            std::task::Poll::Ready(self.0.take().map(|data| Ok(http_body::Frame::data(data))))
        }
    }

    let mut builder = http::Request::builder()
        .method(http::Method::POST)
        .uri(format!("{}://{}{}", scheme, authority, path));

    for (k, v) in headers {
        builder = builder.header(k, &v[..]);
    }

    let http_req = builder
        .body(SimpleBody(Some(bytes::Bytes::copy_from_slice(body))))
        .map_err(|e| format!("Failed to build request: {e:?}"))?;

    let wasi_req = wasip3::http_compat::http_into_wasi_request(http_req)
        .map_err(|e| format!("Failed to convert request: {e:?}"))?;

    let wasi_resp = wasip3::http::client::send(wasi_req)
        .await
        .map_err(|e| format!("Failed to send request: {e:?}"))?;

    let status = wasi_resp.get_status_code();

    let http_resp = wasip3::http_compat::http_from_wasi_response(wasi_resp)
        .map_err(|e| format!("Failed to convert response: {e:?}"))?;

    use futures::future::poll_fn;
    use http_body::Body;

    let mut incoming_body = std::pin::pin!(http_resp.into_body());
    let mut body_bytes = Vec::new();
    while let Some(frame) = poll_fn(|cx| incoming_body.as_mut().poll_frame(cx)).await {
        match frame {
            Ok(f) => {
                if let Some(data) = f.data_ref() {
                    body_bytes.extend_from_slice(data);
                }
            }
            Err(e) => return Err(format!("Failed to read response frame: {e:?}")),
        }
    }

    Ok((status, body_bytes))
}
/// Send a message via the antigravity-sdk-rust Agent sidecar server (full SDK)
///
/// Calls `POST /chat` on the `agent_server` binary which runs the full SDK with
/// localharness, tools, hooks, and policies.
#[server(prefix = "/api")]
pub async fn send_message(message: String) -> Result<ChatMessage, ServerFnError<String>> {
    let agent_server_url = spin_sdk::variables::get("agent_server_url")
        .await
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

    // Load chat history from KV store
    let store = spin_sdk::key_value::Store::open_default().await.map_err(|e| e.to_string())?;
    let history: Vec<ChatMessage> = match store.get_json::<Vec<ChatMessage>>("chat_messages").await {
        Ok(Some(msgs)) => msgs,
        _ => Vec::new(),
    };

    // ── Sidecar Mode: Full SDK via agent_server ──
    let body = serde_json::to_vec(&serde_json::json!({ "message": message }))
        .map_err(|e| e.to_string())?;

    let headers = vec![("content-type".to_string(), b"application/json".to_vec())];

    // Parse the sidecar URL to extract scheme/authority
    let (scheme, authority) = parse_url_parts(&agent_server_url);

    let (status, resp_bytes) =
        send_wasi_http_post(&scheme, &authority, "/chat", &headers, &body).await?;

    if status != 200 {
        let err_text = String::from_utf8_lossy(&resp_bytes);
        return Err(ServerFnError::ServerError(format!(
            "Agent sidecar error ({}): {}",
            status, err_text
        )));
    }

    // Parse sidecar response: { "text": "...", "conversation_id": "..." }
    let resp_json: serde_json::Value =
        serde_json::from_slice(&resp_bytes).map_err(|e| e.to_string())?;

    let text = resp_json["text"]
        .as_str()
        .unwrap_or("(No response)")
        .to_string();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Persist messages to KV store with sequential IDs
    let mut messages = history;
    let next_id = messages.iter().map(|m| m.id).max().unwrap_or(0) + 1;

    messages.push(ChatMessage {
        id: next_id,
        role: "user".to_string(),
        content: message,
        timestamp: now.saturating_sub(1),
        thinking: None,
        tool_calls: None,
    });

    let assistant_msg = ChatMessage {
        id: next_id + 1,
        role: "assistant".to_string(),
        content: text,
        timestamp: now,
        thinking: None,
        tool_calls: None,
    };

    messages.push(assistant_msg.clone());

    store
        .set_json("chat_messages", &messages)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(assistant_msg)
}

/// Parse a URL string into (scheme, authority) components for `wasi::http`.
#[cfg(feature = "ssr")]
fn parse_url_parts(url: &str) -> (String, String) {
    // Simple URL parsing without pulling in the `url` crate
    let (scheme, rest) = if let Some(stripped) = url.strip_prefix("https://") {
        ("https".to_string(), stripped)
    } else if let Some(stripped) = url.strip_prefix("http://") {
        ("http".to_string(), stripped)
    } else {
        ("http".to_string(), url)
    };

    // Authority is everything up to the first /
    let authority = rest.split('/').next().unwrap_or("127.0.0.1:8080").to_string();

    (scheme, authority)
}

/// Get all chat messages from the KV store.
#[server(prefix = "/api")]
pub async fn get_messages() -> Result<Vec<ChatMessage>, ServerFnError<String>> {
    let store = spin_sdk::key_value::Store::open_default().await.map_err(|e| e.to_string())?;
    match store.get_json::<Vec<ChatMessage>>("chat_messages").await {
        Ok(Some(msgs)) => Ok(msgs),
        Ok(None) => Ok(Vec::new()),
        Err(e) => {
            eprintln!("Error loading messages: {e}");
            Ok(Vec::new())
        }
    }
}

/// Clear all chat messages from the KV store.
#[server(prefix = "/api")]
pub async fn clear_messages() -> Result<(), ServerFnError<String>> {
    let store = spin_sdk::key_value::Store::open_default().await.map_err(|e| e.to_string())?;
    let _ = store.delete("chat_messages").await;
    Ok(())
}

/// Parse markdown text into HTML string.
/// When `workspace_path` is provided, relative image URLs are rewritten to
/// proxy through the agent server's /media endpoint.
/// Absolute filesystem paths (starting with `/`) are also routed through /media
/// since the browser cannot access the local filesystem directly.
fn render_markdown(markdown: &str, workspace_path: Option<&str>, agent_server_url: &str) -> String {
    let parser = pulldown_cmark::Parser::new_ext(markdown, pulldown_cmark::Options::all());

    let mapped = parser.map(|event| {
        match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image {
                link_type, dest_url, title, id,
            }) => {
                let url_str = dest_url.as_ref();
                let rewritten = if url_str.starts_with("http://") || url_str.starts_with("https://") {
                    // Fully qualified URL → leave alone
                    dest_url
                } else if url_str.starts_with('/') {
                    // Absolute filesystem path → route through /media directly
                    let enc = percent_encode_path(url_str);
                    format!("{}/media?path={}", agent_server_url.trim_end_matches('/'), enc)
                        .into()
                } else {
                    // Relative path → prepend workspace, then route through /media
                    if let Some(ws) = workspace_path {
                        let abs = format!("{}/{}", ws.trim_end_matches('/'), url_str);
                        let enc = percent_encode_path(&abs);
                        format!("{}/media?path={}", agent_server_url.trim_end_matches('/'), enc)
                            .into()
                    } else {
                        dest_url
                    }
                };
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image {
                    link_type, dest_url: rewritten, title, id,
                })
            }
            other => other,
        }
    });

    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, mapped);
    html_output
}

/// Extract image file paths from a GENERATE_IMAGE tool result.
///
/// Strategy:
/// 1. Try `result["image_paths"]` array (primary — populated by harness on Done step)
/// 2. Fallback: scan the result string value for absolute paths ending in image extensions
fn extract_image_paths_from_result(result: Option<&serde_json::Value>) -> Vec<String> {
    let res = match result {
        Some(v) => v,
        None => return Vec::new(),
    };

    // Strategy 1: structured image_paths array
    if let Some(arr) = res.get("image_paths").and_then(|v| v.as_array()) {
        let paths: Vec<String> = arr.iter()
            .filter_map(|p| p.as_str().map(String::from))
            .filter(|p| !p.is_empty())
            .collect();
        if !paths.is_empty() {
            return paths;
        }
    }

    // Strategy 2: scan text for absolute paths ending in image extensions
    let text = res.as_str()
        .or_else(|| res.get("text").and_then(|v| v.as_str()))
        .unwrap_or("");

    if text.is_empty() {
        return Vec::new();
    }

    let image_extensions = [".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg"];
    text.split_whitespace()
        .filter(|word| {
            let w = word.trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || c == '[' || c == ']');
            w.starts_with('/')
                && image_extensions.iter().any(|ext| w.to_lowercase().ends_with(ext))
        })
        .map(|word| word.trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || c == '[' || c == ']').to_string())
        .collect()
}

/// Save a single completed turn (user message & assistant response) to KV.
#[server(prefix = "/api")]
pub async fn save_chat_turn(
    user_msg: String,
    assistant_msg: String,
    thinking: Option<String>,
    tool_calls: Option<Vec<ClientToolCall>>,
) -> Result<(), ServerFnError<String>> {
    let store = spin_sdk::key_value::Store::open_default().await.map_err(|e| e.to_string())?;
    let mut history: Vec<ChatMessage> = match store.get_json::<Vec<ChatMessage>>("chat_messages").await {
        Ok(Some(msgs)) => msgs,
        _ => Vec::new(),
    };

    let next_id = history.iter().map(|m| m.id).max().unwrap_or(0) + 1;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    history.push(ChatMessage {
        id: next_id,
        role: "user".to_string(),
        content: user_msg,
        timestamp: now.saturating_sub(1),
        thinking: None,
        tool_calls: None,
    });

    history.push(ChatMessage {
        id: next_id + 1,
        role: "assistant".to_string(),
        content: assistant_msg,
        timestamp: now,
        thinking,
        tool_calls,
    });

    store
        .set_json("chat_messages", &history)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

/// List all chat sessions.
#[server(prefix = "/api")]
pub async fn list_sessions() -> Result<Vec<SessionMeta>, ServerFnError<String>> {
    let store = spin_sdk::key_value::Store::open_default().await.map_err(|e| e.to_string())?;

    // Check if session_index exists, if not, migrate legacy
    let index_exists = store.exists("session_index").await.unwrap_or(false);
    if !index_exists {
        let legacy_exists = store.exists("chat_messages").await.unwrap_or(false);
        if legacy_exists {
            migrate_legacy_messages(&store).await?;
        } else {
            let empty = SessionIndex { sessions: Vec::new() };
            store.set_json("session_index", &empty).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        }
    }

    match store.get_json::<SessionIndex>("session_index").await {
        Ok(Some(idx)) => Ok(idx.sessions),
        _ => Ok(Vec::new()),
    }
}

#[cfg(feature = "ssr")]
async fn migrate_legacy_messages(
    store: &spin_sdk::key_value::Store,
) -> Result<(), ServerFnError<String>> {
    let legacy: Vec<ChatMessage> = match store.get_json::<Vec<ChatMessage>>("chat_messages").await {
        Ok(Some(msgs)) => msgs,
        _ => Vec::new(),
    };

    let mut session = ChatSession::new("default".to_string());
    session.title = "Default Session".to_string();

    for msg in legacy {
        let id = session.next_id();
        let block = match msg.role.as_str() {
            "user" => MessageBlock::UserMessage {
                id,
                content: msg.content,
                timestamp: msg.timestamp,
            },
            _ => MessageBlock::AssistantMessage {
                id,
                content: msg.content,
                timestamp: msg.timestamp,
            },
        };
        session.blocks.push(block);
    }

    let meta = SessionMeta {
        id: session.id.clone(),
        title: session.title.clone(),
        created_at: session.created_at,
        updated_at: session.updated_at,
    };

    store
        .set_json(format!("session_{}", &session.id), &session)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let index = SessionIndex {
        sessions: vec![meta],
    };
    store
        .set_json("session_index", &index)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let _ = store.delete("chat_messages").await;
    Ok(())
}

/// Create a new session.
#[server(prefix = "/api")]
pub async fn create_session(title: Option<String>) -> Result<String, ServerFnError<String>> {
    let store = spin_sdk::key_value::Store::open_default().await.map_err(|e| e.to_string())?;
    let mut idx = match store.get_json::<SessionIndex>("session_index").await {
        Ok(Some(idx)) => idx,
        _ => SessionIndex { sessions: Vec::new() },
    };

    let session_id = format!("sess_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis());

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let meta = SessionMeta {
        id: session_id.clone(),
        title: title.unwrap_or_else(|| "New Chat".to_string()),
        created_at: now,
        updated_at: now,
    };

    idx.sessions.insert(0, meta);
    store.set_json("session_index", &idx).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let session = ChatSession::new(session_id.clone());
    store.set_json(format!("session_{}", session_id), &session).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(session_id)
}

/// Delete a session.
#[server(prefix = "/api")]
pub async fn delete_session(session_id: String) -> Result<(), ServerFnError<String>> {
    let store = spin_sdk::key_value::Store::open_default().await.map_err(|e| e.to_string())?;
    let mut idx = match store.get_json::<SessionIndex>("session_index").await {
        Ok(Some(idx)) => idx,
        _ => SessionIndex { sessions: Vec::new() },
    };

    idx.sessions.retain(|s| s.id != session_id);
    store.set_json("session_index", &idx).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let _ = store.delete(&format!("session_{}", session_id)).await;
    Ok(())
}

/// Rename a session.
#[server(prefix = "/api")]
pub async fn rename_session(session_id: String, new_title: String) -> Result<(), ServerFnError<String>> {
    let store = spin_sdk::key_value::Store::open_default().await.map_err(|e| e.to_string())?;

    // Update the session detail key
    let session_key = format!("session_{}", session_id);
    if let Ok(Some(mut sess)) = store.get_json::<ChatSession>(&session_key).await {
        sess.title = new_title.clone();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        sess.updated_at = now;
        store
            .set_json(&session_key, &sess)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    }

    // Update the session meta inside the index
    let mut idx = match store.get_json::<SessionIndex>("session_index").await {
        Ok(Some(idx)) => idx,
        _ => SessionIndex { sessions: Vec::new() },
    };

    if let Some(pos) = idx.sessions.iter().position(|s| s.id == session_id) {
        idx.sessions[pos].title = new_title;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        idx.sessions[pos].updated_at = now;
        store
            .set_json("session_index", &idx)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    }

    Ok(())
}


/// Get all message blocks for a session.
#[server(prefix = "/api")]
pub async fn get_session_blocks(session_id: String) -> Result<Vec<MessageBlock>, ServerFnError<String>> {
    let store = spin_sdk::key_value::Store::open_default().await.map_err(|e| e.to_string())?;
    match store.get_json::<ChatSession>(&format!("session_{}", session_id)).await {
        Ok(Some(sess)) => Ok(sess.blocks),
        _ => Ok(Vec::new()),
    }
}

/// Save/update all blocks for a session, and auto-update title.
#[server(prefix = "/api", input = leptos::server_fn::codec::Json)]
pub async fn save_turn_blocks(session_id: String, blocks: Vec<MessageBlock>) -> Result<(), ServerFnError<String>> {
    let store = spin_sdk::key_value::Store::open_default().await.map_err(|e| e.to_string())?;
    let mut sess = match store.get_json::<ChatSession>(&format!("session_{}", session_id)).await {
        Ok(Some(s)) => s,
        _ => ChatSession::new(session_id.clone()),
    };

    sess.blocks = blocks;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    sess.updated_at = now;

    // Auto-update title if it's "New Chat" and we have a user message.
    // Use the first non-empty line of the first UserMessage, truncated to 50
    // characters using char-boundary-safe slicing.
    let mut new_title = None;
    if sess.title == "New Chat" {
        for b in &sess.blocks {
            if let MessageBlock::UserMessage { content, .. } = b {
                // Skip past any leading blank lines
                let first_line = content
                    .lines()
                    .map(|l| l.trim())
                    .find(|l| !l.is_empty())
                    .unwrap_or("");

                let mut title: String = first_line.chars().take(50).collect();
                if first_line.chars().count() > 50 {
                    title.push_str("...");
                }
                if title.is_empty() {
                    title = "Chat".to_string();
                }
                sess.title = title.clone();
                new_title = Some(title);
                break;
            }
        }
    }

    store.set_json(format!("session_{}", session_id), &sess).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    // Also update session_index
    let mut idx = match store.get_json::<SessionIndex>("session_index").await {
        Ok(Some(idx)) => idx,
        _ => SessionIndex { sessions: Vec::new() },
    };

    if let Some(pos) = idx.sessions.iter().position(|s| s.id == session_id) {
        if let Some(t) = new_title {
            idx.sessions[pos].title = t;
        }
        idx.sessions[pos].updated_at = now;
        // Move to the top (most recently updated)
        let meta = idx.sessions.remove(pos);
        idx.sessions.insert(0, meta);
    } else {
        // Fallback: create index entry if missing
        idx.sessions.insert(0, SessionMeta {
            id: session_id,
            title: sess.title.clone(),
            created_at: sess.created_at,
            updated_at: now,
        });
    }

    store.set_json("session_index", &idx).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

#[cfg(feature = "hydrate")]
fn get_agent_server_url() -> String {
    use wasm_bindgen::JsCast;
    if let Some(win) = web_sys::window() {
        if let Some(doc) = win.document() {
            if let Ok(Some(meta)) = doc.query_selector("meta[name='agent-server-url']") {
                if let Ok(meta_el) = meta.dyn_into::<web_sys::HtmlMetaElement>() {
                    return meta_el.content();
                }
            }
        }
    }
    "http://127.0.0.1:8080".to_string()
}

#[cfg(feature = "hydrate")]
fn get_agent_server_url_encoded(val: &str) -> String {
    js_sys::encode_uri_component(val).as_string().unwrap_or_default()
}

/// Returns the agent server URL, compiling on both SSR and hydrate targets.
fn get_agent_server_url_any() -> String {
    #[cfg(feature = "ssr")]
    {
        leptos::prelude::use_context::<crate::types::AgentServerUrl>()
            .map(|url| url.0)
            .unwrap_or_else(|| "http://127.0.0.1:8080".to_string())
    }
    #[cfg(not(feature = "ssr"))]
    {
        #[cfg(feature = "hydrate")]
        {
            get_agent_server_url()
        }
        #[cfg(not(feature = "hydrate"))]
        {
            "http://127.0.0.1:8080".to_string()
        }
    }
}

/// Pure-Rust percent-encoding for file paths used in query parameters.
fn percent_encode_path(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' | b'/' => {
                encoded.push(*byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_encode_path() {
        assert_eq!(percent_encode_path("foo/bar.png"), "foo/bar.png");
        assert_eq!(percent_encode_path("foo space/bar.png"), "foo%20space/bar.png");
    }

    #[test]
    fn test_render_markdown_rewrite_relative() {
        let markdown = "![Cyber Dragon](dragon_cyberpunk_123.png)";
        let workspace = Some("/Users/uriah/workspace");
        let agent_url = "http://127.0.0.1:8080";
        let html = render_markdown(markdown, workspace, agent_url);
        // Relative path should be rewritten with workspace prepended
        assert!(html.contains("src=\"http://127.0.0.1:8080/media?path=/Users/uriah/workspace/dragon_cyberpunk_123.png\""));
    }

    #[test]
    fn test_render_markdown_rewrite_absolute_filesystem() {
        let markdown = "![Core Dragon](/Users/uriah/.gemini/antigravity/brain/abc123/dragon_logo_core.png)";
        let workspace = Some("/Users/uriah/workspace");
        let agent_url = "http://127.0.0.1:8080";
        let html = render_markdown(markdown, workspace, agent_url);
        // Absolute filesystem path should be routed through /media WITHOUT workspace prepend
        assert!(html.contains("src=\"http://127.0.0.1:8080/media?path=/Users/uriah/.gemini/antigravity/brain/abc123/dragon_logo_core.png\""));
    }

    #[test]
    fn test_render_markdown_leaves_external_urls() {
        let markdown = "![External](https://example.com/foo.png)";
        let html = render_markdown(markdown, Some("/ws"), "http://127.0.0.1:8080");
        // External URLs should NOT be rewritten
        assert!(html.contains("src=\"https://example.com/foo.png\""));
    }

    #[test]
    fn test_extract_image_paths_structured() {
        let result = serde_json::json!({
            "prompt": "a dragon",
            "image_paths": ["/Users/uriah/brain/dragon.png"],
            "image_name": "dragon"
        });
        let paths = extract_image_paths_from_result(Some(&result));
        assert_eq!(paths, vec!["/Users/uriah/brain/dragon.png"]);
    }

    #[test]
    fn test_extract_image_paths_empty_array_fallback_text() {
        // When image_paths is empty, fall back to scanning the string value
        let result = serde_json::Value::String(
            "Generated image saved to /Users/uriah/brain/dragon.png successfully".to_string()
        );
        let paths = extract_image_paths_from_result(Some(&result));
        assert_eq!(paths, vec!["/Users/uriah/brain/dragon.png"]);
    }

    #[test]
    fn test_extract_image_paths_empty_array_no_text() {
        let result = serde_json::json!({
            "prompt": "a dragon",
            "image_paths": [],
            "image_name": "dragon"
        });
        let paths = extract_image_paths_from_result(Some(&result));
        assert!(paths.is_empty());
    }

    #[test]
    fn test_extract_image_paths_none() {
        let paths = extract_image_paths_from_result(None);
        assert!(paths.is_empty());
    }
}
