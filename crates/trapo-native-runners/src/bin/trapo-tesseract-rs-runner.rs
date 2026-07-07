//! Process-contract wrapper for the Trapo Tesseract OCR engine.

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
    let command_line = default_command();
    let (program, fixed_args) = command_line
        .split_first()
        .ok_or_else(|| "TRAPO_TESSERACT_COMMAND was empty".to_string())?;
    let mut command = Command::new(program);
    command.args(fixed_args);
    if let Some(home) = embedded_engine_home() {
        command.env("TESSDATA_PREFIX", home.join("tessdata"));
        prepend_path(&mut command, home.join("bin"));
    }
    if args.self_check {
        command.arg("--list-langs");
    } else {
        command
            .arg(&args.image)
            .arg("stdout")
            .arg("-l")
            .arg(args.language)
            .arg("--psm")
            .arg(args.page_segmentation_mode);
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = command
        .output()
        .map_err(|error| format!("failed to start tesseract: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() {
        if args.self_check && !stdout.contains("eng") {
            return Err(format!(
                "tesseract self-check did not list eng tessdata: {stdout}"
            ));
        }
        return Ok(stdout);
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(if stderr.is_empty() { stdout } else { stderr })
}

struct Args {
    image: PathBuf,
    language: String,
    page_segmentation_mode: String,
    self_check: bool,
    help: bool,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let mut image = None;
        let mut language = "eng".to_string();
        let mut page_segmentation_mode = "6".to_string();
        let mut self_check = false;
        let mut args = env::args_os().skip(1);
        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--image" => image = args.next().map(PathBuf::from),
                "--self-check" => self_check = true,
                "--language" | "-l" => {
                    language = next_value(&mut args, "--language")?;
                }
                "--psm" | "--page-segmentation-mode" => {
                    page_segmentation_mode = next_value(&mut args, "--psm")?;
                }
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
                language,
                page_segmentation_mode,
                self_check,
                help: false,
            });
        }
        let image = image.ok_or_else(usage)?;
        Ok(Self {
            image,
            language,
            page_segmentation_mode,
            self_check,
            help: false,
        })
    }

    fn help() -> Self {
        Self {
            image: PathBuf::new(),
            language: "eng".to_string(),
            page_segmentation_mode: "6".to_string(),
            self_check: false,
            help: true,
        }
    }
}

fn next_value(args: &mut impl Iterator<Item = OsString>, name: &str) -> Result<String, String> {
    args.next()
        .map(|value| value.to_string_lossy().to_string())
        .ok_or_else(|| format!("{name} requires a value"))
}

fn tesseract_name() -> OsString {
    OsString::from(if cfg!(windows) {
        "tesseract.exe"
    } else {
        "tesseract"
    })
}

fn default_command() -> Vec<OsString> {
    if let Some(command) = env::var_os("TRAPO_TESSERACT_COMMAND") {
        return vec![command];
    }
    if let Some(home) = embedded_engine_home() {
        let command = home.join("bin").join(PathBuf::from(tesseract_name()));
        if command.is_file() {
            return vec![command.into_os_string()];
        }
    }
    vec![tesseract_name()]
}

fn embedded_engine_home() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let bin_dir = exe.parent()?;
    [
        bin_dir.parent()?.join("tesseract"),
        bin_dir.join("tesseract"),
        env::current_dir()
            .ok()?
            .join("thirdparty")
            .join("tesseract-runtime"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_dir())
}

fn prepend_path(command: &mut Command, dir: PathBuf) {
    let mut paths = vec![dir];
    if let Some(current) = env::var_os("PATH") {
        paths.extend(env::split_paths(&current));
    }
    if let Ok(joined) = env::join_paths(paths) {
        command.env("PATH", joined);
    }
}

fn usage() -> String {
    "usage: trapo-tesseract-rs-runner --image <page.png> [--language eng] [--psm 6] [--format text] [--self-check]"
        .to_string()
}
