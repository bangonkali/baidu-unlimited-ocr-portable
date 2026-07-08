use std::{env, path::PathBuf};

#[derive(Debug)]
pub(crate) struct Args {
    pub(crate) image: Option<PathBuf>,
    pub(crate) self_check: bool,
    pub(crate) help: bool,
}

impl Args {
    pub(crate) fn parse() -> Result<Self, String> {
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
                "-h" | "--help" => {
                    return Ok(Self {
                        image: None,
                        self_check: false,
                        help: true,
                    });
                }
                value => return Err(format!("unknown argument: {value}\n{}", usage())),
            }
        }
        if self_check || image.is_some() {
            return Ok(Self {
                image,
                self_check,
                help: false,
            });
        }
        Err(usage())
    }
}

pub(crate) fn usage() -> String {
    "usage: trapo-pp-ocrv6-runner --image <page.png> [--format text] [--self-check]".to_string()
}
