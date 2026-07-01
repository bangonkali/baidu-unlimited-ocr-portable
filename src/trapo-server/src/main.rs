use std::{env, net::SocketAddr, process::ExitCode, time::Duration};

use trapo_server::{AppState, ServerConfig, build_router};

#[tokio::main]
async fn main() -> ExitCode {
    if env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return ExitCode::SUCCESS;
    }
    if env::args().any(|arg| arg == "--version" || arg == "-V") {
        println!(
            "trapo-server {} git_tag={} git_sha={}",
            env!("CARGO_PKG_VERSION"),
            option_env!("TRAPO_GIT_TAG").unwrap_or("dev"),
            option_env!("TRAPO_GIT_SHA").unwrap_or("unknown")
        );
        return ExitCode::SUCCESS;
    }

    let config = ServerConfig::from_env_and_args(env::args());
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
    if let Err(error) = axum::serve(listener, build_router(state)).await {
        eprintln!("trapo-server failed: {error}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn print_help() {
    println!(
        "trapo-server\n\nOptions:\n  --port <PORT>       Listen port (default 8890)\n  --no-browser        Do not open a browser window\n  --version           Print version"
    );
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
        .spawn();
}
