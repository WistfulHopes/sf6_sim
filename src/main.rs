#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod simulator;

use eframe::{egui::{self}, emath::Vec2, Frame};
use eframe::egui::Context;
use crate::simulator::Viewer;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2{x: 1280.0, y: 720.0}),
        ..Default::default()
    };
    eframe::run_native(
        "Street Fighter 6 Simulator",
        options,
        Box::new(|_cc| Box::new(SF6Simulator::new(_cc))),
    ).expect("Failed to start GUI!");
}

#[derive(Default)]
struct SF6Simulator {
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
    success: bool,
    viewer: Viewer,
}

impl SF6Simulator {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            dropped_files: Default::default(),
            picked_path: None,
            success: false,
            viewer: Default::default(),
        }
    }
}

impl eframe::App for SF6Simulator {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    self.file_menu(ui)
                });
                let mut visuals = ui.ctx().style().visuals.clone();
                visuals.light_dark_radio_buttons(ui);
                ui.ctx().set_visuals(visuals);
            });

            ui.label("Open from the File menu.");

            if let Some(picked_path) = &self.picked_path {
                if self.success == false{
                    ui.horizontal(|ui| {
                        ui.label("Failed to open file!\nMake sure your file is a valid Street Fighter 6 FChar file.");
                    });
                }
                else {
                    ui.horizontal(|ui| {
                        ui.label("Picked file:");
                        ui.monospace(picked_path);
                    });
                    self.viewer.ui(ui);
                }
            }

            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                for file in &self.dropped_files {
                    let &path = &file.path.as_ref().unwrap();
                    self.success = self.viewer.open_fchar(path);
                    self.picked_path = Some(path.display().to_string());
                }
                self.dropped_files.clear();
            }
        });
    }
}

impl SF6Simulator {
    fn file_menu(&mut self, ui: &mut egui::Ui) {
        if ui.button("Open").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("FChar File", &["fchar.17"])
                .pick_file() {
                self.success = self.viewer.open_fchar(&path);
                self.picked_path = Some(path.display().to_string());
            };
            ui.close_menu();
        }
    }
}