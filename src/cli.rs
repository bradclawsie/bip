use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{Result, bail};

const USAGE: &str =
    "Usage: bip init --signing-key <path> | bip verify [--expected-id <commit-hash>]";

#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Greeting,
    Init { signing_key: PathBuf },
    Verify { expected_id: Option<String> },
}

pub fn parse(args: Vec<OsString>) -> Result<Command> {
    if args.iter().any(|arg| arg == "--help") {
        return Ok(Command::Help);
    }
    match args.as_slice() {
        [] => Ok(Command::Greeting),
        [command, flag, path] if command == "init" && flag == "--signing-key" => {
            Ok(Command::Init {
                signing_key: path.into(),
            })
        }
        [command] if command == "verify" => Ok(Command::Verify { expected_id: None }),
        [command, flag, id] if command == "verify" && flag == "--expected-id" => {
            let id = id
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Expected identity must be hexadecimal"))?;
            Ok(Command::Verify {
                expected_id: Some(id.to_owned()),
            })
        }
        _ => bail!("{USAGE}"),
    }
}
