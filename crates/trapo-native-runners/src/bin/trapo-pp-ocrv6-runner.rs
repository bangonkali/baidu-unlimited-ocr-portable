//! Process-contract wrapper for the Trapo PP-OCRv6 engine.

use std::{
    env,
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
};

fn main() -> ExitCode {
    match run() {
        Ok(text) => {
            let _ = io::stdout().write_all(text.as_bytes());
            ExitCode::SUCCESS
        }
        Err(error) => {
            let _ = writeln!(io::stderr(), "{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<String, String> {
    let args = Args::parse()?;
    if args.help {
        return Ok(format!("{}\n", usage()));
    }
    let command_line = default_command()?;
    let (program, fixed_args) = command_line
        .split_first()
        .ok_or_else(|| "TRAPO_PP_OCRV6_COMMAND was empty".to_string())?;
    let mut command = Command::new(program);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(home) = embedded_engine_home() {
        command
            .env("TRAPO_PPOCRV6_HOME", &home)
            .env("HF_HOME", home.join("cache").join("huggingface"))
            .env("PADDLE_HOME", home.join("cache").join("paddle"))
            .env("PADDLEX_HOME", home.join(".paddlex"));
        if cfg!(windows) {
            command.env("USERPROFILE", &home);
        } else {
            command.env("HOME", &home);
        }
    }
    command.args(fixed_args);
    if args.self_check {
        command.arg("--self-check");
    } else {
        command.arg("--image").arg(&args.image);
    }
    let output = command
        .output()
        .map_err(|error| format!("failed to start PP-OCRv6 command: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() {
        return Ok(stdout);
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(if stderr.is_empty() { stdout } else { stderr })
}

struct Args {
    image: PathBuf,
    self_check: bool,
    help: bool,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let mut image = None;
        let mut self_check = false;
        let mut args = env::args_os().skip(1);
        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--image" => image = args.next().map(PathBuf::from),
                "--self-check" => self_check = true,
                "--format" => {
                    let _ = args.next();
                }
                "-h" | "--help" => return Ok(Self::help()),
                value => return Err(format!("unknown argument: {value}\n{}", usage())),
            }
        }
        if self_check {
            return Ok(Self {
                image: PathBuf::new(),
                self_check,
                help: false,
            });
        }
        Ok(Self {
            image: image.ok_or_else(usage)?,
            self_check,
            help: false,
        })
    }

    const fn help() -> Self {
        Self {
            image: PathBuf::new(),
            self_check: true,
            help: true,
        }
    }
}

fn default_command() -> Result<Vec<OsString>, String> {
    if let Some(command) = env::var_os("TRAPO_PP_OCRV6_COMMAND") {
        return Ok(vec![command]);
    }
    let home = embedded_engine_home()
        .ok_or_else(|| "packaged PP-OCRv6 engine directory was not found".to_string())?;
    if let Some(engine) = embedded_engine_binary(&home) {
        return Ok(vec![engine.into_os_string()]);
    }
    let script = home.join("trapo_ppocrv6_engine.py");
    if !script.is_file() {
        return Err(format!(
            "PP-OCRv6 engine script is missing: {}",
            script.display()
        ));
    }
    let python = embedded_python(&home).ok_or_else(|| {
        format!(
            "PP-OCRv6 Python runtime is missing under {}",
            home.display()
        )
    })?;
    Ok(vec![python.into_os_string(), script.into_os_string()])
}

fn embedded_engine_home() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let bin_dir = exe.parent()?;
    [
        bin_dir.parent()?.join("ppocrv6"),
        bin_dir.join("ppocrv6"),
        env::current_dir().ok()?.join("thirdparty").join("ppocrv6"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_dir())
}

fn embedded_python(home: &std::path::Path) -> Option<PathBuf> {
    let suffix = if cfg!(windows) {
        ["Scripts", "python.exe"]
    } else {
        ["bin", "python"]
    };
    let python = home.join(".venv").join(suffix[0]).join(suffix[1]);
    python.is_file().then_some(python)
}

fn embedded_engine_binary(home: &std::path::Path) -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "trapo_ppocrv6_engine.exe"
    } else {
        "trapo_ppocrv6_engine"
    };
    let binary = home.join("bin").join(name);
    binary.is_file().then_some(binary)
}

fn usage() -> String {
    "usage: trapo-pp-ocrv6-runner --image <page.png> [--format text] [--self-check]".to_string()
}
