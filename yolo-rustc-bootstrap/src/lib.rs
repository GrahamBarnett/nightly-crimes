use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

fn is_quiet() -> bool {
    static QUIET: OnceLock<bool> = OnceLock::new();
    *QUIET.get_or_init(|| {
        std::env::var_os("NIGHTLY_CRIMES_QUIET").is_some()
            || matches!(std::env::var("CARGO_TERM_QUIET").as_deref(), Ok("true"))
    })
}

macro_rules! say {
    ($($t:tt)*) => {
        if !is_quiet() {
            eprintln!($($t)*);
        }
    };
}

fn probe_version(exe: &OsStr) -> Option<String> {
    static CACHE: OnceLock<Mutex<HashMap<OsString, Option<String>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(map) = cache.lock() {
        if let Some(v) = map.get(exe) {
            return v.clone();
        }
    }
    let result = (|| {
        let out = std::process::Command::new(exe).arg("--version").output().ok()?;
        let stdout = String::from_utf8(out.stdout).ok()?;
        stdout.split_whitespace().nth(1).map(str::to_owned)
    })();
    if let Ok(mut map) = cache.lock() {
        map.insert(exe.to_owned(), result.clone());
    }
    result
}

fn is_rust_analyzer(exe: &OsStr) -> bool {
    let Some(name) = Path::new(exe).file_name() else {
        return false;
    };
    let b = name.as_encoded_bytes();
    let stem = b.strip_suffix(b".exe").unwrap_or(b);
    stem == b"rust-analyzer-proc-macro-srv"
}

#[proc_macro]
pub fn do_crimes(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if std::env::args_os().any(|a| a == "--cfg=yolo_rustc_bootstrap") {
        say!("\x1b[1;32m  Destroying\x1b[m stability guarantees");
        return Default::default();
    }

    let mut args = std::env::args_os();
    let Some(exe) = args.next() else {
        eprintln!("nightly-crimes: missing argv[0]");
        std::process::exit(101);
    };
    let mut args: Vec<OsString> = args.collect();

    if is_rust_analyzer(&exe) {
        return Default::default();
    }

    let version = probe_version(&exe);
    let trick = version.as_deref().is_some_and(|v| !v.ends_with("-nightly"));

    if trick {
        say!("\x1b[1;32m   Hijacking\x1b[m this rustc process");
        say!("\x1b[1;32m     Abusing\x1b[m proc macros");
        say!("\x1b[1;32m    Enabling\x1b[m the forbidden environment variable");
        if let Some(v) = &version {
            say!("\x1b[1;32m    Tricking\x1b[m rustc {}", v);
        }
        if let (Ok(c), Ok(v)) = (
            std::env::var("CARGO_PKG_NAME"),
            std::env::var("CARGO_PKG_VERSION"),
        ) {
            say!("\x1b[1;32m Recompiling\x1b[m {} v{}", c, v);
        } else {
            say!("\x1b[1;32m Recompiling\x1b[m your crate");
        }
    }

    let insert_at = args
        .iter()
        .position(|a| a.as_encoded_bytes().starts_with(b"--"))
        .unwrap_or(args.len());
    args.insert(insert_at, "--check-cfg=cfg(yolo_rustc_bootstrap)".into());
    args.insert(insert_at, "--cfg=yolo_rustc_bootstrap".into());

    let mut cmd = std::process::Command::new(&exe);
    cmd.args(&args).env("RUSTC_BOOTSTRAP", "1");
    say!("\x1b[1;32m    Running trick cmd: {:?}", cmd);

    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("nightly-crimes: failed to spawn {:?}: {}", exe, e);
            std::process::exit(101);
        }
    };

    if !status.success() {
        say!(
            "\x1b[1;33m    Finished cmd {:?} with error {:?}",
            std::env::args_os(),
            status
        );
    } else if trick {
        say!("\x1b[1;32m    Finished\x1b[m the dirty work");
        say!("\x1b[1;32m      Hiding\x1b[m all the evidence");
        say!("\x1b[1;32m  Continuing\x1b[m as if nothing happened");
    }

    let code = status.code().unwrap_or_else(|| {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            if let Some(sig) = status.signal() {
                return 128 + sig;
            }
        }
        101
    });
    std::process::exit(code);
}
