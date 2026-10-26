mod bootstrap;
mod cli;
mod git;
mod initialize;
mod metadata;
mod signing;

use std::process::ExitCode;

use anyhow::{Result, ensure};

fn run() -> Result<()> {
    match cli::parse(std::env::args_os().skip(1).collect())? {
        cli::Command::Help => println!("{}", env!("BIP_COMMIT_HASH")),
        cli::Command::Greeting => println!("bip!"),
        cli::Command::Init { signing_key } => {
            let repo = git::Repository::discover()?;
            println!("{}", initialize::initialize(&repo, &signing_key)?);
        }
        cli::Command::Verify { expected_id } => {
            let repo = git::Repository::discover()?;
            if let Some(expected) = &expected_id {
                ensure!(
                    repo.valid_oid(expected),
                    "Expected identity must be a full hexadecimal object ID; abbreviated IDs and revision expressions are invalid"
                );
            }
            let oid = bootstrap::discover(&repo)?;
            let signers = bootstrap::validate(&repo, &oid)?;
            signing::verify(&repo, &oid, &signers)?;
            if let Some(expected) = expected_id {
                ensure!(
                    oid.eq_ignore_ascii_case(&expected),
                    "Repository identity mismatch: expected {expected}, found {oid}"
                );
                println!("Repository identity: {oid}\nExpected identity matched.");
            } else {
                println!(
                    "Repository identity: {oid}\nNo expected identity was supplied or checked."
                );
            }
            println!(
                "Initialization verified; subsequent ownership changes and governance history were not checked."
            );
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bip: {error:#}");
            ExitCode::FAILURE
        }
    }
}
