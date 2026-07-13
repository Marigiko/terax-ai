// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

fn parse_cli() -> (bool, Option<String>) {
    let args: Vec<String> = env::args().collect();
    let no_block = args.contains(&"--no-block".to_string());
    let serve = args
        .iter()
        .find_map(|a| a.strip_prefix("--serve="))
        .map(String::from);
    (no_block, serve)
}

fn should_detach(no_block: bool) -> bool {
    if !no_block {
        return false;
    }
    #[cfg(windows)]
    {
        let parent = env::var("TERAX_DETACHED").unwrap_or_default();
        parent != "1"
    }
    #[cfg(not(windows))]
    {
        unsafe { libc::fork() > 0 }
    }
}

fn main() {
    let (no_block, _serve) = parse_cli();

    if should_detach(no_block) {
        #[cfg(unix)]
        {
            let current = env::current_exe().expect("cannot resolve exe path");
            let _ = std::process::Command::new(current)
                .env("TERAX_DETACHED", "1")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("Failed to detach process");
            std::process::exit(0);
        }
        #[cfg(windows)]
        {
            let current = env::current_exe().expect("cannot resolve exe path");
            let _ = std::process::Command::new(current)
                .env("TERAX_DETACHED", "1")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("Failed to detach process");
            std::process::exit(0);
        }
    }

    #[cfg(target_os = "macos")]
    {
        use objc2::msg_send;
        use objc2_foundation::{ns_string, NSUserDefaults};
        unsafe {
            let defaults = NSUserDefaults::standardUserDefaults();
            let key = ns_string!("ApplePressAndHoldEnabled");
            let _: () = msg_send![&defaults, setBool: false, forKey: key];
        }
    }

    terax_lib::run()
}
