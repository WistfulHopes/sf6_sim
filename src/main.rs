#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;

use eframe::emath::Vec2;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2 {
            x: 1280.0,
            y: 720.0,
        }),
        ..Default::default()
    };
    eframe::run_native(
        "Street Fighter 6 Simulator",
        options,
        Box::new(|cc| Box::new(app::SF6Simulator::new(cc))),
    )
    .expect("Failed to start GUI!");
}


// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "sf6sim",
                web_options,
                Box::new(|cc| Box::new(app::SF6Simulator::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}