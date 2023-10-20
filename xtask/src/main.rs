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
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("msrv") => update_msrv_lock()?,
        Some("min") => update_min_lock()?,
        _ => print_help(),
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
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let sh = Shell::new()?;

    let pin_versions = [
        ("chrono".to_owned(), "0.4.23"),
        ("regex".to_owned(), "1.9.6"),
        (latest_version(&sh, "serde_with")?, "3.1.0"),
        (latest_version(&sh, "time")?, "0.3.20"),
    ];

    cmd!(sh, "rm Cargo.lock").run()?;

    for (krate, version) in pin_versions {
        cmd!(sh, "{cargo} update --package {krate} --precise {version}").run()?;
    }

    cmd!(sh, "cargo +1.63.0 test --no-run").run()?;
    cmd!(sh, "cp Cargo.lock ci/Cargo.lock.msrv").run()?;

    return Ok(());

    fn latest_version(sh: &Shell, krate: &str) -> Result<String> {
        let dir = match krate.len() {
            1 => "1".to_owned(),
            2 => "2".to_owned(),
            3 => format!("3/{}", &krate[..1]),
            _ => format!("{}/{}", &krate[..2], &krate[2..4]),
        };

        // let index = cmd!(sh, "curl --silent https://index.crates.io/{dir}/{krate}").read()?;
        let index = cmd!(sh, "curl https://index.crates.io/{dir}/{krate}").read()?;

        let version = cmd!(sh, "jq --slurp --raw-output 'map(.vers) | last'")
            .stdin(index)
            .read()?;

        Ok(format!("{krate}@{version}"))
    }
}

fn update_min_lock() -> Result {
    let sh = Shell::new()?;

    cmd!(sh, "rm Cargo.lock").run()?;

    cmd!(sh, "cargo +nightly -Z minimal-versions test --no-run")
        .env("RUST_LOG", "debug")
        .run()?;

    cmd!(sh, "cp Cargo.lock ci/Cargo.lock.min").run()?;

    Ok(())
}
