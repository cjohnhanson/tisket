//! `tisket tui` without a terminal refuses before it touches the screen.
//!
//! Without a terminal on stdout, crossterm sizes the screen by running
//! `tput`, a program this crate does not spawn; the refusal is what keeps
//! that fallback unreachable.
#![cfg(feature = "tui")]

use std::process::{Command, Stdio};

#[test]
fn tui_without_a_terminal_refuses_and_names_the_script_command() {
    let dir = std::env::temp_dir().join(format!("tisket-tui-noterm-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let config = dir.join("config.yml");
    std::fs::write(&config, "").unwrap();
    let init = Command::new(env!("CARGO_BIN_EXE_tisket"))
        .arg("--root")
        .arg(&dir)
        .arg("--user-config")
        .arg(&config)
        .arg("init")
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");
    let out = Command::new(env!("CARGO_BIN_EXE_tisket"))
        .arg("--root")
        .arg(&dir)
        .arg("--user-config")
        .arg(&config)
        .arg("tui")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("tisket tui needs an interactive terminal"),
        "{stderr}"
    );
    assert!(out.stdout.is_empty(), "nothing drawn: {:?}", out.stdout);
    let _ = std::fs::remove_dir_all(&dir);
}
