mod simulator;

use simulator::{Character, Viewer};
use eframe::egui::{ComboBox, Context};
use eframe::{
    egui::{self},
    Frame,
};
use include_bytes_zstd::include_bytes_zstd;

#[derive(Default)]
pub struct SF6Simulator {
    viewer: Viewer,
    character_name: String,
}

impl SF6Simulator {
    pub(crate) fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            viewer: Default::default(),
            character_name: "Select a character".to_string(),
        }
    }
}

impl eframe::App for SF6Simulator {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ComboBox::from_label("Character List")
                    .selected_text(self.character_name.clone())
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(true, "Common").clicked() {
                            self.character_name = "Common".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/000.fchar.17", 9));
                            self.viewer.character = Character::Common;
                        }
                        if ui.selectable_label(true, "Ryu").clicked() {
                            self.character_name = "Ryu".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/001.fchar.17", 9));
                            self.viewer.character = Character::Ryu;
                        }
                        if ui.selectable_label(true, "Luke").clicked() {
                            self.character_name = "Luke".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/002.fchar.17", 9));
                            self.viewer.character = Character::Luke;
                        }
                        if ui.selectable_label(true, "Kimberly").clicked() {
                            self.character_name = "Kimberly".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/003.fchar.17", 9));
                            self.viewer.character = Character::Kimberly;
                        }
                        if ui.selectable_label(true, "Chun-Li").clicked() {
                            self.character_name = "Chun-Li".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/004.fchar.17", 9));
                            self.viewer.character = Character::ChunLi;
                        }
                        if ui.selectable_label(true, "Manon").clicked() {
                            self.character_name = "Manon".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/005.fchar.17", 9));
                            self.viewer.character = Character::Manon;
                        }
                        if ui.selectable_label(true, "Zangief").clicked() {
                            self.character_name = "Zangief".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/006.fchar.17", 9));
                            self.viewer.character = Character::Zangief;
                        }
                        if ui.selectable_label(true, "JP").clicked() {
                            self.character_name = "JP".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/007.fchar.17", 9));
                            self.viewer.character = Character::JP;
                        }
                        if ui.selectable_label(true, "Dhalsim").clicked() {
                            self.character_name = "Dhalsim".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/008.fchar.17", 9));
                            self.viewer.character = Character::Dhalsim;
                        }
                        if ui.selectable_label(true, "Cammy").clicked() {
                            self.character_name = "Cammy".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/009.fchar.17", 9));
                            self.viewer.character = Character::Cammy;
                        }
                        if ui.selectable_label(true, "Ken").clicked() {
                            self.character_name = "Ken".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/010.fchar.17", 9));
                            self.viewer.character = Character::Ken;
                        }
                        if ui.selectable_label(true, "Dee Jay").clicked() {
                            self.character_name = "Dee Jay".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/011.fchar.17", 9));
                            self.viewer.character = Character::DeeJay;
                        }
                        if ui.selectable_label(true, "Lily").clicked() {
                            self.character_name = "Lily".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/012.fchar.17", 9));
                            self.viewer.character = Character::Lily;
                        }
                        if ui.selectable_label(true, "Blanka").clicked() {
                            self.character_name = "Blanka".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/015.fchar.17", 9));
                            self.viewer.character = Character::Blanka;
                        }
                        if ui.selectable_label(true, "Juri").clicked() {
                            self.character_name = "Juri".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/016.fchar.17", 9));
                            self.viewer.character = Character::Juri;
                        }
                        if ui.selectable_label(true, "Marisa").clicked() {
                            self.character_name = "Marisa".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/017.fchar.17", 9));
                            self.viewer.character = Character::Marisa;
                        }
                        if ui.selectable_label(true, "Guile").clicked() {
                            self.character_name = "Guile".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/018.fchar.17", 9));
                            self.viewer.character = Character::Guile;
                        }
                        if ui.selectable_label(true, "E. Honda").clicked() {
                            self.character_name = "E. Honda".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/020.fchar.17", 9));
                            self.viewer.character = Character::EHonda;
                        }
                        if ui.selectable_label(true, "Jamie").clicked() {
                            self.character_name = "Jamie".to_string();
                            self.viewer
                                .open_fchar(include_bytes_zstd!("assets/021.fchar.17", 9));
                            self.viewer.character = Character::Jamie;
                        }
                    });
                let mut visuals = ui.ctx().style().visuals.clone();
                visuals.light_dark_radio_buttons(ui);
                ui.ctx().set_visuals(visuals);
            });
            if self.viewer.asset.is_some() {
                self.viewer.ui(ui);
            }
        });
    }
}