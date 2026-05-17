/// Tauri shell for Starbound.
///
/// In desktop mode, Tauri launches the embedded webview pointing at
/// the Vite dev server (dev) or the built frontend (release).
///
/// The Rust game server still runs as a separate process — Tauri's job
/// is only to provide the native window. For a fully offline desktop
/// experience you could embed the server here and spawn it as a child
/// process using `std::process::Command`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
