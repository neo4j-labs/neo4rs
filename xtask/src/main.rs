#![allow(dead_code)]

use std::env;

use xshell::{Shell, cmd};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{e}");
        std::process::exit(-1);
    }
}

type DynError = Box<dyn std::error::Error>;
type Result<T = ()> = std::result::Result<T, DynError>;

fn try_main() -> Result {
    let mut tasks = env::args().skip(1);
    while let Some(task) = tasks.next().as_deref() {
        match task {
            "msrv" => update_msrv_lock()?,
            "min" => update_min_lock()?,
            "sync" => sync_lockfiles()?,
            _ => {
                print_help();
                return Ok(());
            }
        }
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:

msrv            Update the MSRV lockfile
min             Update the MIN lockfile
"
    )
}

fn update_msrv_lock() -> Result {
    let Env {
        lockfile,
        ci_dir,
        dry_run,
        ..
    } = task_env();

    let sh = Shell::new()?;
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());

    let msrv = {
        let metadata = cmd!(sh, "{cargo} metadata --no-deps --format-version=1").read()?;
        let package = "neo4rs";

        cmd!(
            sh,
            "jq --raw-output '.packages[] | select(.name == \"'{package}'\") | .rust_version'"
        )
        .stdin(metadata)
        .read()
    }?;

    pin_msrv_versions(dry_run, &sh, &cargo, &lockfile)?;

    cmd!(sh, "cargo +{msrv} test --locked --no-run --all-features").run_if(dry_run)?;

    cmd!(sh, "cp {lockfile} {ci_dir}/Cargo.lock.msrv").run_if(dry_run)?;

    Ok(())
}

fn update_min_lock() -> Result {
    let Env {
        lockfile,
        ci_dir,
        dry_run,
        ..
    } = task_env();

    let sh = Shell::new()?;
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());

    pin_msrv_versions(dry_run, &sh, &cargo, &lockfile)?;

    cmd!(
        sh,
        "cargo +nightly -Z minimal-versions test --locked --package neo4rs --no-run --all-features"
    )
    .env("RUST_LOG", "debug")
    .run_if(dry_run)?;

    cmd!(sh, "cp {lockfile} {ci_dir}/Cargo.lock.min").run_if(dry_run)?;

    Ok(())
}

fn pin_msrv_versions(dry_run: bool, sh: &Shell, cargo: &str, lockfile: &str) -> Result<()> {
    cmd!(sh, "rm {lockfile}").run_if(dry_run)?;

    let versions = &[
        ("idna_adapter", "1.2.0"), // 1.2.2 requires 1.86
        ("time", "0.3.45"),        // 0.3.45 requires 1.83, and 1.88 since 0.3.46
        ("serde_with", "3.17.0"),  // 3.17.0 requires 1.82, 3.18.0 does 1.88
        ("uuid", "1.26.1"),        // 1.27.0 requires 1.88
    ];
    pin_versions(dry_run, sh, cargo, versions)
}

fn pin_versions(dry_run: bool, sh: &Shell, cargo: &str, versions: &[(&str, &str)]) -> Result<()> {
    let mut versions = versions.to_vec();
    let mut next_versions = Vec::<(&str, &str)>::new();
    loop {
        let mut error = None::<DynError>;

        for &(krate, version) in &versions {
            if let Err(err) = pin_version(dry_run, sh, cargo, krate, version) {
                if error.is_none() {
                    error = Some(err);
                }
                next_versions.push((krate, version));
            }
        }

        if next_versions.is_empty() || next_versions.len() == versions.len() {
            return match error {
                Some(err) => Err(err),
                None => Ok(()),
            };
        }

        versions = std::mem::take(&mut next_versions);
    }
}

fn pin_version(dry_run: bool, sh: &Shell, cargo: &str, krate: &str, version: &str) -> Result<()> {
    cmd!(
        sh,
        "{cargo} update --offline --package {krate} --precise {version}"
    )
    .run_if(dry_run)?;
    Ok(())
}

fn latest_version(sh: &Shell, krate: &str) -> Result<&'static str> {
    let index = match krate.len() {
        1 => format!("https://index.crates.io/1/{krate}"),
        2 => format!("https://index.crates.io/2/{krate}"),
        3 => format!("https://index.crates.io/3/{}/{}", &krate[..1], krate),
        _ => format!(
            "https://index.crates.io/{}/{}/{}",
            &krate[..2],
            &krate[2..4],
            krate
        ),
    };

    let index = cmd!(sh, "curl --silent {index}").read()?;

    let version = cmd!(sh, "jq --slurp --raw-output 'map(.vers) | last'")
        .stdin(index)
        .read()?;

    let name = format!("{krate}@{version}");
    let name = Box::leak(name.into_boxed_str());

    Ok(name)
}

fn sync_lockfiles() -> Result {
    let Env {
        lockfile,
        ci_dir,
        dry_run,
        ..
    } = task_env();

    let sh = Shell::new()?;
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());

    cmd!(sh, "mv {lockfile} {lockfile}.backup").run_if(dry_run)?;

    cmd!(sh, "cp {ci_dir}/Cargo.lock.min {lockfile}").run_if(dry_run)?;
    let _ = cmd!(sh, "{cargo} check --keep-going").run_if(dry_run);
    cmd!(sh, "mv {lockfile} {ci_dir}/Cargo.lock.min").run_if(dry_run)?;

    cmd!(sh, "cp {ci_dir}/Cargo.lock.msrv {lockfile}").run_if(dry_run)?;
    let _ = cmd!(sh, "{cargo} check --keep-going").run_if(dry_run);
    cmd!(sh, "mv {lockfile} {ci_dir}/Cargo.lock.msrv").run_if(dry_run)?;

    cmd!(sh, "mv {lockfile}.backup {lockfile}").run_if(dry_run)?;

    Ok(())
}

struct Env {
    lockfile: String,
    ci_dir: String,
    dry_run: bool,
}

fn task_env() -> Env {
    let dry_run = env::var_os("DRY_RUN").is_some_and(|env| env == "1" || env == "true");

    let workspace = env::var("WORKSPACE_ROOT");

    let (lockfile, ci_dir) = match workspace {
        Ok(ws) => (format!("{ws}/Cargo.lock"), format!("{ws}/ci")),
        Err(_) => ("Cargo.lock".into(), "ci".into()),
    };

    Env {
        lockfile,
        ci_dir,
        dry_run,
    }
}

trait DryRun {
    fn run_if(&self, dry_run: bool) -> Result<()>;
}

impl DryRun for xshell::Cmd<'_> {
    fn run_if(&self, dry_run: bool) -> Result<()> {
        if dry_run {
            eprintln!("DRY_RUN: {self}");
        } else {
            self.run()?;
        }
        Ok(())
    }
}
