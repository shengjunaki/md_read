use std::{
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    thread,
    time::Duration,
};

use crate::cli;

const IPC_ADDR: &str = "127.0.0.1:45173";

pub struct InstanceGuard;

pub enum InstanceRole {
    Primary(InstanceGuard),
    Forwarded,
}

pub fn acquire_or_forward() -> InstanceRole {
    cleanup_legacy_lock();

    match TcpListener::bind(IPC_ADDR) {
        Ok(listener) => {
            spawn_ipc_listener(listener);
            InstanceRole::Primary(InstanceGuard)
        }
        Err(_) => {
            if forward_startup_path() {
                InstanceRole::Forwarded
            } else {
                InstanceRole::Primary(InstanceGuard)
            }
        }
    }
}

pub fn consume_pending_path() -> Option<PathBuf> {
    let path = pending_path();
    let raw = fs::read_to_string(&path).ok()?;
    let _ = fs::remove_file(path);

    let candidate = PathBuf::from(raw.trim());
    if cli::is_markdown_path(&candidate) {
        Some(candidate)
    } else {
        None
    }
}

fn spawn_ipc_listener(listener: TcpListener) {
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            receive_forwarded_path(stream);
        }
    });
}

fn receive_forwarded_path(mut stream: TcpStream) {
    let mut raw = String::new();
    if stream.read_to_string(&mut raw).is_err() {
        return;
    }

    let candidate = PathBuf::from(raw.trim());
    if !cli::is_markdown_path(&candidate) {
        return;
    }

    let _ = fs::create_dir_all(ipc_dir());
    let _ = fs::write(pending_path(), candidate.display().to_string());
}

fn forward_startup_path() -> bool {
    let Some(path) = env::args_os().nth(1).map(PathBuf::from) else {
        return false;
    };
    if !cli::is_markdown_path(&path) {
        return false;
    }

    let Ok(mut stream) = TcpStream::connect_timeout(
        &IPC_ADDR
            .parse()
            .expect("hardcoded IPC address should parse"),
        Duration::from_millis(120),
    ) else {
        return false;
    };

    stream
        .write_all(path.display().to_string().as_bytes())
        .is_ok()
}

fn pending_path() -> PathBuf {
    ipc_dir().join("pending.mdpath")
}

fn ipc_dir() -> PathBuf {
    env::temp_dir().join("md-reader")
}

fn cleanup_legacy_lock() {
    let _ = fs::remove_file(ipc_dir().join("md-reader.lock"));
}
