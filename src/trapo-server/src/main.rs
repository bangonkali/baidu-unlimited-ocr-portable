//! Command-line entry point for the local Trapo server.
// The binary prints user-facing status and startup errors before logging is
// initialized, while library code remains covered by print_stdout/stderr denies.
#![allow(clippy::print_stdout, clippy::print_stderr)]

mod cli;

use std::{env, net::SocketAddr, process::ExitCode, time::Duration};

use trapo_server::{
    AppState, ServerConfig, build_router, ensure_runtime_dll_search_paths, install_process_logging,
};

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if let Some(exit_code) = cli::handle_early_command(&args) {
        return exit_code;
    }

    // Make packaged CUDA/cuDNN bins visible even when started without trapo-server.cmd.
    ensure_runtime_dll_search_paths();

    let config = ServerConfig::from_env_and_args(args);
    let _process_logs = match install_process_logging(&config.log_dir) {
        Ok(guards) => guards,
        Err(error) => {
            eprintln!("failed to initialize process logging: {error}");
            return ExitCode::from(2);
        }
    };
    let addr: SocketAddr = match format!("{}:{}", config.host, config.port).parse() {
        Ok(addr) => addr,
        Err(error) => {
            eprintln!("invalid listen address: {error}");
            return ExitCode::from(2);
        }
    };
    let state = match AppState::new(config.clone()).await {
        Ok(state) => state,
        Err(error) => {
            eprintln!("failed to initialize trapo-server: {error}");
            return ExitCode::from(2);
        }
    };
    if !config.client_dist.join("index.html").is_file() {
        eprintln!(
            "warning: React build was not found at {}; API and Scalar will still be served",
            config.client_dist.display()
        );
    }
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("failed to bind {addr}: {error}");
            return ExitCode::from(2);
        }
    };
    let url = format!("http://{addr}");
    println!("trapo-server listening on {url}");
    if config.open_browser {
        tokio::spawn(open_browser_later(url.clone()));
    }
    let shutdown_state = state.clone();
    if let Err(error) = axum::serve(listener, build_router(state.clone()))
        .with_graceful_shutdown(shutdown_signal(shutdown_state))
        .await
    {
        eprintln!("trapo-server failed: {error}");
        return ExitCode::from(1);
    }
    state.complete_shutdown().await;
    ExitCode::SUCCESS
}

async fn open_browser_later(url: String) {
    tokio::time::sleep(Duration::from_millis(600)).await;
    #[cfg(target_os = "windows")]
    let command = ("cmd", ["/C", "start", "", &url]);
    #[cfg(target_os = "macos")]
    let command = ("open", ["", "", "", &url]);
    #[cfg(all(unix, not(target_os = "macos")))]
    let command = ("xdg-open", ["", "", "", &url]);
    let _ = std::process::Command::new(command.0)
        .args(command.1.into_iter().filter(|arg| !arg.is_empty()))
        .spawn(); // skylos: ignore[SKY-D212] command is a fixed OS opener and url is local server origin.
}

async fn shutdown_signal(state: AppState) {
    let token = state.shutdown_token();
    tokio::select! {
        () = token.cancelled() => {}
        source = wait_for_shutdown_signal() => {
            state.request_signal_shutdown(source).await;
        }
    }
}

#[cfg(target_os = "windows")]
async fn wait_for_shutdown_signal() -> &'static str {
    let mut ctrl_c = tokio::signal::windows::ctrl_c().ok();
    let mut ctrl_break = tokio::signal::windows::ctrl_break().ok();
    let mut ctrl_close = tokio::signal::windows::ctrl_close().ok();
    let mut ctrl_logoff = tokio::signal::windows::ctrl_logoff().ok();
    let mut ctrl_shutdown = tokio::signal::windows::ctrl_shutdown().ok();
    tokio::select! {
        () = recv_windows_ctrl_c(&mut ctrl_c) => "ctrl_c",
        () = recv_windows_ctrl_break(&mut ctrl_break) => "ctrl_break",
        () = recv_windows_ctrl_close(&mut ctrl_close) => "ctrl_close",
        () = recv_windows_ctrl_logoff(&mut ctrl_logoff) => "ctrl_logoff",
        () = recv_windows_ctrl_shutdown(&mut ctrl_shutdown) => "ctrl_shutdown",
    }
}

#[cfg(target_os = "windows")]
async fn recv_windows_ctrl_c(signal: &mut Option<tokio::signal::windows::CtrlC>) {
    if let Some(signal) = signal {
        signal.recv().await;
    } else {
        std::future::pending::<()>().await;
    }
}

#[cfg(target_os = "windows")]
async fn recv_windows_ctrl_break(signal: &mut Option<tokio::signal::windows::CtrlBreak>) {
    if let Some(signal) = signal {
        signal.recv().await;
    } else {
        std::future::pending::<()>().await;
    }
}

#[cfg(target_os = "windows")]
async fn recv_windows_ctrl_close(signal: &mut Option<tokio::signal::windows::CtrlClose>) {
    if let Some(signal) = signal {
        signal.recv().await;
    } else {
        std::future::pending::<()>().await;
    }
}

#[cfg(target_os = "windows")]
async fn recv_windows_ctrl_logoff(signal: &mut Option<tokio::signal::windows::CtrlLogoff>) {
    if let Some(signal) = signal {
        signal.recv().await;
    } else {
        std::future::pending::<()>().await;
    }
}

#[cfg(target_os = "windows")]
async fn recv_windows_ctrl_shutdown(signal: &mut Option<tokio::signal::windows::CtrlShutdown>) {
    if let Some(signal) = signal {
        signal.recv().await;
    } else {
        std::future::pending::<()>().await;
    }
}

#[cfg(all(unix, not(target_os = "windows")))]
async fn wait_for_shutdown_signal() -> &'static str {
    let mut terminate =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).ok();
    tokio::select! {
        result = tokio::signal::ctrl_c() => {
            if let Err(error) = result {
                eprintln!("failed to listen for Ctrl+C: {error}");
            }
            "ctrl_c"
        }
        () = recv_unix_signal(&mut terminate) => "sigterm",
    }
}

#[cfg(all(unix, not(target_os = "windows")))]
async fn recv_unix_signal(signal: &mut Option<tokio::signal::unix::Signal>) {
    if let Some(signal) = signal {
        signal.recv().await;
    } else {
        std::future::pending::<()>().await;
    }
}

#[cfg(not(any(unix, target_os = "windows")))]
async fn wait_for_shutdown_signal() -> &'static str {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("failed to listen for Ctrl+C: {error}");
    }
    "ctrl_c"
}
