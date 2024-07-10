use std::env;

use xshell::{cmd, Shell};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
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

    let pin_versions = [
        ("cc".into(), "1.0.105"),
        ("chrono-tz".into(), "0.8.3"),
        ("chrono".into(), "0.4.23"),
        ("deadpool-runtime".into(), "0.1.3"),
        ("regex".into(), "1.9.6"),
        (latest_version(&sh, "serde_with")?, "3.1.0"),
        (latest_version(&sh, "time")?, "0.3.20"),
    ];

    cmd!(sh, "rm {lockfile}").run_if(dry_run)?;

    if !pin_versions.is_empty() {
        let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        for (krate, version) in pin_versions {
            cmd!(sh, "{cargo} update --package {krate} --precise {version}").run_if(dry_run)?;
        }
    }

    cmd!(sh, "cargo +1.63.0 test --no-run --all-features").run_if(dry_run)?;

    cmd!(sh, "cp {lockfile} {ci_dir}/Cargo.lock.msrv").run_if(dry_run)?;

    return Ok(());

    fn latest_version(sh: &Shell, krate: &str) -> Result<String> {
        let index = match krate.len() {
            1 => format!("https://index.crates.io/1/{}", krate),
            2 => format!("https://index.crates.io/2/{}", krate),
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

        Ok(format!("{krate}@{version}"))
    }
}

fn update_min_lock() -> Result {
    let Env {
        lockfile,
        ci_dir,
        dry_run,
        ..
    } = task_env();

    let sh = Shell::new()?;

    cmd!(sh, "rm {lockfile}").run_if(dry_run)?;

    cmd!(
        sh,
        "cargo +nightly -Z minimal-versions test --no-run --all-features"
    )
    .env("RUST_LOG", "debug")
    .run_if(dry_run)?;

    cmd!(sh, "cp {lockfile} {ci_dir}/Cargo.lock.min").run_if(dry_run)?;

    Ok(())
}

struct Env {
    lockfile: String,
    ci_dir: String,
    dry_run: bool,
}

fn task_env() -> Env {
    let dry_run = env::var("DRY_RUN")
        .ok()
        .and_then(|o| o.parse().ok())
        .unwrap_or(false);

    let workspace = env::var("WORKSPACE_ROOT");

    let (lockfile, ci_dir) = match workspace {
        Ok(ws) => (format!("{}/Cargo.lock", ws), format!("{}/ci", ws)),
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
            eprintln!("DRY_RUN: {}", self);
        } else {
            self.run()?;
        }
        Ok(())
    }
}
