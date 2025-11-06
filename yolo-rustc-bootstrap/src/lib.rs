#[proc_macro]
pub fn do_crimes(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if !std::env::args_os().any(|arg| arg == "--cfg=yolo_rustc_bootstrap") {
        let version = String::from_utf8(
            std::process::Command::new(std::env::args_os().next().unwrap())
                .arg("--version")
                .output()
                .unwrap()
                .stdout,
        )
        .expect("version failed");
        let version = version.split(' ').nth(1).unwrap();
        let mut trick = !version.ends_with("-nightly");

        let mut args = std::env::args_os();
        let exe = args.next().expect("get first arg");
        let mut args: Vec<std::ffi::OsString> = args.collect();

        let rust_analyzer = exe.clone().into_string().unwrap().ends_with("rust-analyzer-proc-macro-srv");
        if trick && rust_analyzer {
            trick = false;
        }

        if trick {
            println!("\x1b[1;32m   Hijacking\x1b[m this rustc process");
            println!("\x1b[1;32m     Abusing\x1b[m proc macros");
            println!("\x1b[1;32m    Enabling\x1b[m the forbidden environment variable");
            println!("\x1b[1;32m    Tricking\x1b[m rustc {}", version);
            if let (Ok(c), Ok(v)) = (
                std::env::var("CARGO_PKG_NAME"),
                std::env::var("CARGO_PKG_VERSION"),
            ) {
                println!("\x1b[1;32m Recompiling\x1b[m {} v{}", c, v);
            } else {
                println!("\x1b[1;32m Recompiling\x1b[m your crate");
            }
        }

        let mut cmd = std::process::Command::new(&exe);
        let status = if rust_analyzer {
            cmd.args(args).env("RUSTC_BOOTSTRAP", "1");

            println!("\x1b[1;32m    Skipping cmd: {:?}", cmd);

            return Default::default()
        } else if !trick {
            cmd.args(args).env("RUSTC_BOOTSTRAP", "1");

            println!("\x1b[1;32m    Running cmd: {:?}", cmd);

            cmd.status().unwrap()
        } else {
            let mut insert_at = 0;
            for (idx, a) in args.iter().enumerate() {
                if a.clone().into_string().unwrap().starts_with("--") {
                    insert_at = idx;
                    break;
                }
            }
            args.insert(insert_at, "--check-cfg=cfg(yolo_rustc_bootstrap)".into());
            args.insert(insert_at, "--cfg=yolo_rustc_bootstrap".into());
            cmd.args(args).env("RUSTC_BOOTSTRAP", "1");

            println!("\x1b[1;32m    Running trick cmd: {:?}", cmd);

            cmd.status().unwrap()
        };

        if trick {
            if status.success() {
                println!("\x1b[1;32m    Finished\x1b[m the dirty work");
                println!("\x1b[1;32m      Hiding\x1b[m all the evidence");
                println!("\x1b[1;32m  Continuing\x1b[m as if nothing happened");
            } else {
                println!("\x1b[1;33m    Finished cmd {:?} with error {:?}", std::env::args_os(), status);
            }
        }

        std::process::exit(status.code().unwrap_or(101));
    }

    println!("\x1b[1;32m  Destroying\x1b[m stability guarantees");

    Default::default()
}
