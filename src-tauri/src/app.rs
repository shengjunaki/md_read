use crate::{cli, fs::document, instance, markdown};
use std::process::Command;

#[tauri::command]
fn load_startup_document() -> Result<Option<markdown::RenderedDocument>, String> {
    let Some(path) = cli::startup_markdown_path().map_err(|err| err.to_string())? else {
        return Ok(None);
    };

    let source = document::read_markdown(&path).map_err(|err| err.to_string())?;
    Ok(Some(markdown::render_document(&source)))
}

#[tauri::command]
fn consume_pending_document() -> Result<Option<markdown::RenderedDocument>, String> {
    let Some(path) = instance::consume_pending_path() else {
        return Ok(None);
    };

    let source = document::read_markdown(&path).map_err(|err| err.to_string())?;
    Ok(Some(markdown::render_document(&source)))
}

#[tauri::command]
fn minimize_window(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|err| err.to_string())
}

#[tauri::command]
fn toggle_maximize_window(window: tauri::Window) -> Result<(), String> {
    if window.is_maximized().map_err(|err| err.to_string())? {
        window.unmaximize().map_err(|err| err.to_string())
    } else {
        window.maximize().map_err(|err| err.to_string())
    }
}

#[tauri::command]
fn close_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|err| err.to_string())
}

#[tauri::command]
fn start_dragging_window(window: tauri::Window) -> Result<(), String> {
    window.start_dragging().map_err(|err| err.to_string())
}

#[tauri::command]
fn open_external_link(url: String) -> Result<(), String> {
    if !is_allowed_external_url(&url) {
        return Err("Only http, https, and mailto links can be opened externally.".to_string());
    }

    #[cfg(windows)]
    {
        Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", &url])
            .spawn()
            .map(|_| ())
            .map_err(|err| err.to_string())
    }

    #[cfg(not(windows))]
    {
        let _ = url;
        Err("External links are only implemented for Windows in this MVP.".to_string())
    }
}

fn is_allowed_external_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with("mailto:")
}

pub fn run() {
    let _guard = match instance::acquire_or_forward() {
        instance::InstanceRole::Primary(guard) => guard,
        instance::InstanceRole::Forwarded => return,
    };

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_startup_document,
            consume_pending_document,
            minimize_window,
            toggle_maximize_window,
            close_window,
            start_dragging_window,
            open_external_link
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Markdown Reader");
}
