use std::{
    fmt,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use chrono::Utc;

/// Guards process-wide stdout/stderr redirection for the server lifetime.
pub struct ProcessLogGuards {
    stdout: Option<gag::Redirect<File>>,
    stderr: Option<gag::Redirect<File>>,
    stop: Arc<AtomicBool>,
    tailers: Vec<JoinHandle<()>>,
}

impl fmt::Debug for ProcessLogGuards {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("ProcessLogGuards").finish()
    }
}

impl Drop for ProcessLogGuards {
    fn drop(&mut self) {
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
        drop(self.stdout.take());
        drop(self.stderr.take());
        self.stop.store(true, Ordering::SeqCst);
        for tailer in self.tailers.drain(..) {
            let _ = tailer.join();
        }
    }
}

/// Redirects process stdout/stderr and panic output into `trapo-server.log`.
///
/// # Errors
///
/// Returns an error when the log directory or redirect file handles cannot be
/// created.
pub fn install_process_logging(log_dir: &Path) -> std::io::Result<ProcessLogGuards> {
    std::fs::create_dir_all(log_dir)?;
    let path = log_dir.join("trapo-server.log");
    let stdout_path = log_dir.join("trapo-server.stdout.raw.log");
    let stderr_path = log_dir.join("trapo-server.stderr.raw.log");
    truncate_file(&stdout_path)?;
    truncate_file(&stderr_path)?;
    install_panic_hook(path.clone());
    append_process_log_line(&path, "INFO", "process", "stdio capture initialized")?;
    let stdout = gag::Redirect::stdout(open_process_log_file(&stdout_path)?)?;
    let stderr = gag::Redirect::stderr(open_process_log_file(&stderr_path)?)?;
    let stop = Arc::new(AtomicBool::new(false));
    let tailers = vec![
        spawn_log_tailer(
            stdout_path,
            path.clone(),
            "INFO",
            "native-stdout",
            stop.clone(),
        ),
        spawn_log_tailer(stderr_path, path, "WARN", "native-stderr", stop.clone()),
    ];
    Ok(ProcessLogGuards {
        stdout: Some(stdout),
        stderr: Some(stderr),
        stop,
        tailers,
    })
}

fn open_process_log_file(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path) // skylos: ignore[SKY-D215] path is the configured app log file.
}

fn truncate_file(path: &Path) -> std::io::Result<()> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path) // skylos: ignore[SKY-D215] path is a raw stdio capture file under the configured app log root.
        .map(drop)
}

fn append_process_log_line(
    path: &Path,
    level: &str,
    component: &str,
    message: &str,
) -> std::io::Result<()> {
    let mut file = open_process_log_file(path)?;
    writeln!(
        file,
        "{} {} {} {}",
        Utc::now().to_rfc3339(),
        level,
        component,
        super::single_line(message)
    )
}

fn spawn_log_tailer(
    raw_path: PathBuf,
    log_path: PathBuf,
    level: &'static str,
    component: &'static str,
    stop: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let Ok(file) = File::open(&raw_path) else {
            // skylos: ignore[SKY-D215] raw_path is the stdio capture file created under the app log root.
            return;
        };
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(80));
                }
                Ok(_) => {
                    let message = line.trim_end_matches(['\r', '\n']);
                    if !message.trim().is_empty() {
                        let _ = append_process_log_line(&log_path, level, component, message);
                    }
                }
                Err(_) => break,
            }
        }
    })
}

fn install_panic_hook(path: PathBuf) {
    static INSTALLED: AtomicBool = AtomicBool::new(false);
    if INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let message = panic_message(panic_info);
        let location = panic_info.location().map_or_else(
            || "unknown location".to_string(),
            |location| format!("{}:{}", location.file(), location.line()),
        );
        let _ = append_process_log_line(
            &path,
            "ERROR",
            "panic",
            &format!("panic at {location}: {message}"),
        );
        previous(panic_info);
    }));
}

fn panic_message(panic_info: &std::panic::PanicHookInfo<'_>) -> String {
    panic_info.payload().downcast_ref::<&str>().map_or_else(
        || {
            panic_info
                .payload()
                .downcast_ref::<String>()
                .map_or_else(|| "non-string panic payload".to_string(), Clone::clone)
        },
        |message| (*message).to_string(),
    )
}
