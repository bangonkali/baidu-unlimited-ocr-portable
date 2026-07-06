use std::{
    env,
    path::Path,
    process::{Command, ExitCode},
};

use trapo_server::{
    install_process_logging, run_embedding_worker, validate_ffi_library, validate_llama_library,
};

pub(crate) fn handle_early_command(args: &[String]) -> Option<ExitCode> {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Some(ExitCode::SUCCESS);
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        print_version();
        return Some(ExitCode::SUCCESS);
    }
    if let Some(index) = args.iter().position(|arg| arg == "--check-ocr-runtime") {
        return Some(
            required_arg(args, index, "--check-ocr-runtime", "a path to uocr-ffi")
                .map_or_else(|| ExitCode::from(2), check_ocr_runtime),
        );
    }
    if let Some(index) = args
        .iter()
        .position(|arg| arg == "--check-embedding-runtime")
    {
        return Some(
            required_arg(
                args,
                index,
                "--check-embedding-runtime",
                "a path to llama.cpp",
            )
            .map_or_else(|| ExitCode::from(2), check_embedding_runtime),
        );
    }
    if let Some(index) = args.iter().position(|arg| arg == "--embedding-worker") {
        return Some(embedding_worker(args.get(index + 1), args.get(index + 2)));
    }
    handle_self_test_command(args)
}

fn handle_self_test_command(args: &[String]) -> Option<ExitCode> {
    if let Some(index) = args.iter().position(|arg| arg == "--self-test-log-stdio") {
        return Some(
            required_arg(args, index, "--self-test-log-stdio", "a log directory")
                .map_or_else(|| ExitCode::from(2), self_test_log_stdio),
        );
    }
    if let Some(index) = args.iter().position(|arg| arg == "--self-test-log-panic") {
        return Some(
            required_arg(args, index, "--self-test-log-panic", "a log directory")
                .map_or_else(|| ExitCode::from(2), self_test_log_panic),
        );
    }
    if let Some(index) = args
        .iter()
        .position(|arg| arg == "--self-test-log-child-stderr")
    {
        return Some(
            required_arg(
                args,
                index,
                "--self-test-log-child-stderr",
                "a log directory",
            )
            .map_or_else(|| ExitCode::from(2), self_test_log_child_stderr),
        );
    }
    if args.iter().any(|arg| arg == "--self-test-child-stderr") {
        eprintln!("self-test child stderr marker");
        return Some(ExitCode::SUCCESS);
    }
    None
}

fn required_arg<'a>(
    args: &'a [String],
    index: usize,
    flag: &str,
    description: &str,
) -> Option<&'a str> {
    args.get(index + 1)
        .filter(|value| !value.is_empty())
        .map(String::as_str)
        .or_else(|| {
            eprintln!("{flag} requires {description}");
            None
        })
}

fn print_help() {
    println!(
        "trapo-server\n\nOptions:\n  --port <PORT>                       Listen port (default 8765)\n  --no-browser                        Do not open a browser window\n  --check-ocr-runtime <PATH>          Validate a uocr-ffi runtime library\n  --check-embedding-runtime <PATH>    Validate a llama.cpp runtime library\n  --version                           Print version"
    );
}

fn print_version() {
    println!(
        "trapo-server {} git_tag={} git_sha={}",
        env!("CARGO_PKG_VERSION"),
        option_env!("TRAPO_GIT_TAG").unwrap_or("dev"),
        option_env!("TRAPO_GIT_SHA").unwrap_or("unknown")
    );
}

fn check_ocr_runtime(path: impl AsRef<str>) -> ExitCode {
    let path = path.as_ref();
    match validate_ffi_library(Path::new(path)) {
        Ok(()) => {
            println!("uocr-ffi runtime loaded: {path}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn check_embedding_runtime(path: impl AsRef<str>) -> ExitCode {
    let path = path.as_ref();
    match validate_llama_library(Path::new(path)) {
        Ok(()) => {
            println!("llama.cpp runtime loaded: {path}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn embedding_worker(request_path: Option<&String>, response_path: Option<&String>) -> ExitCode {
    let (Some(request_path), Some(response_path)) = (request_path, response_path) else {
        eprintln!("--embedding-worker requires request and response paths");
        return ExitCode::from(2);
    };
    match run_embedding_worker(Path::new(request_path), Path::new(response_path)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("embedding worker failed: {error}");
            ExitCode::from(1)
        }
    }
}

fn self_test_log_stdio(path: impl AsRef<str>) -> ExitCode {
    let path = path.as_ref();
    let Ok(_guards) = install_process_logging(Path::new(path)) else {
        eprintln!("failed to install self-test process logging");
        return ExitCode::from(2);
    };
    println!("self-test stdout marker");
    eprintln!("self-test stderr marker");
    ExitCode::SUCCESS
}

#[allow(
    clippy::panic,
    reason = "hidden integration-test command verifies panic hook logging"
)]
fn self_test_log_panic(path: impl AsRef<str>) -> ExitCode {
    let path = path.as_ref();
    let Ok(_guards) = install_process_logging(Path::new(path)) else {
        eprintln!("failed to install self-test process logging");
        return ExitCode::from(2);
    };
    panic!("self-test panic marker");
}

fn self_test_log_child_stderr(path: impl AsRef<str>) -> ExitCode {
    let path = path.as_ref();
    let Ok(_guards) = install_process_logging(Path::new(path)) else {
        eprintln!("failed to install self-test process logging");
        return ExitCode::from(2);
    };
    let Ok(exe) = env::current_exe() else {
        eprintln!("failed to resolve self-test executable path");
        return ExitCode::from(1);
    };
    let status = Command::new(exe).arg("--self-test-child-stderr").status();
    match status {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(status) => {
            eprintln!("self-test child exited with {status}");
            ExitCode::from(1)
        }
        Err(error) => {
            eprintln!("failed to start self-test child: {error}");
            ExitCode::from(1)
        }
    }
}
