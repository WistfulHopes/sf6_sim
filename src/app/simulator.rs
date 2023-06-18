mod action_names;

use eframe::egui;
use eframe::egui::{Color32, ComboBox, Frame, Sense, Slider};
use eframe::emath::{Pos2, Rect};
use eframe::epaint::Stroke;
use num_derive::FromPrimitive;
use sf6_rsz_parser::fchar::{CharacterAsset, DataId};
use sf6_rsz_parser::parse_fchar;
use sf6_rsz_parser::rsz::json_parser::parse_json;
use sf6_rsz_parser::rsz::RSZValue;
use bitvec::vec::BitVec;
use include_bytes_zstd::include_bytes_zstd;

#[derive(Default)]
pub enum Character {
    #[default]
    Common,
    Ryu,
    Luke,
    Kimberly,
    ChunLi,
    Manon,
    Zangief,
    JP,
    Dhalsim,
    Cammy,
    Ken,
    DeeJay,
    Lily,
    Blanka,
    Juri,
    Marisa,
    Guile,
    EHonda,
    Jamie,
}

#[derive(Default, FromPrimitive, PartialEq, Eq, Clone)]
enum SteerOperationType {
    #[default]
    Nop = 0,
    Set = 1,
    Add = 2,
    Multiply = 3,
    SetSign = 4,
    AddSign = 5,
    SetNegativeX = 6,
    SetNegativeY = 7,
    SetNegativeZ = 8,
    SetMinimum = 9,
    SetMaximum = 10,
    SetIgnore = 11,
    SetInherit = 12,
    SetTarget = 13,
    SetHomingValue = 14,
    SetHomingTime = 15,
    SetInheritXYZ = 16,
}

#[derive(Default, FromPrimitive, PartialEq, Eq, Clone)]
enum SteerValueType {
    #[default]
    VelocityX = 0,
    VelocityY = 1,
    VelocityZ = 2,
    AccelerationX = 3,
    AccelerationY = 4,
    AccelerationZ = 5,
}

#[derive(Default, Clone)]
struct CollisionBox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Default)]
struct PushCollisionKey {
    condition: u8,
    attribute: u16,
    pushbox: CollisionBox,
}

#[derive(Default)]
struct DamageCollisionKey {
    condition: u8,
    collision_type: u8,
    immune: u8,
    extend: u8,
    level: u8,
    type_flag: u32,
    boxes: Vec<CollisionBox>,
}

#[derive(Default)]
struct AttackCollisionKey {
    condition: u8,
    collision_type: u8,
    hit_id: i8,
    guard_bit: u8,
    kind_flag: u32,
    hit_offset: [i32; 2],
    boxes: Vec<CollisionBox>,
}

#[derive(Default, Ord, PartialOrd, Eq, PartialEq)]
struct Trigger {
    action: i32,
    condition_flag: u32,
}

#[derive(Default)]
struct Vector3f {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Default)]
struct ActionInfo {
    first_active_frame: i32,
    recovery_frame: i32,
    end_frame: i32,
    loop_count: i32,
}

pub struct Viewer {
    pub asset: Option<CharacterAsset>,
    pub character: Character,
    selected_index: i32,
    action_index: i32,
    action_index_string: String,
    current_frame: usize,
    action_info: ActionInfo,
    push_collision_keys: Vec<PushCollisionKey>,
    damage_collision_keys: Vec<DamageCollisionKey>,
    attack_collision_keys: Vec<AttackCollisionKey>,
    triggers: Vec<Trigger>,
    position: Vector3f,
    velocity: Vector3f,
    acceleration: Vector3f,
    prev_position: Vector3f,
    prev_velocity: Vector3f,
    prev_acceleration: Vector3f,
    root_motion: Vector3f,
    offset_x: f32,
    offset_y: f32,
    last_cursor_pos: Pos2,
    should_update: bool,
}

impl Default for Viewer {
    fn default() -> Self {
        Self {
            asset: None,
            character: Character::Common,
            selected_index: -1,
            action_index: 0,
            action_index_string: "".to_string(),
            current_frame: 0,
            action_info: Default::default(),
            push_collision_keys: vec![],
            damage_collision_keys: vec![],
            attack_collision_keys: vec![],
            triggers: vec![],
            position: Default::default(),
            velocity: Default::default(),
            acceleration: Default::default(),
            prev_position: Default::default(),
            prev_velocity: Default::default(),
            prev_acceleration: Default::default(),
            root_motion: Default::default(),
            offset_x: 90.0,
            offset_y: 300.0,
            last_cursor_pos: Default::default(),
            should_update: false,
        }
    }
}
fn steer_key_to_value(
    op_type: SteerOperationType,
    in_value: f32,
    prev_value: f32,
    modify_value: f32,
) -> f32 {
    let mut value = in_value;
    match op_type {
        SteerOperationType::Nop => {}
        SteerOperationType::Set => value = modify_value,
        SteerOperationType::Add => value += modify_value,
        SteerOperationType::Multiply => value *= modify_value,
        SteerOperationType::SetSign => {}
        SteerOperationType::AddSign => {}
        SteerOperationType::SetNegativeX => {
            if value < 0f32 && prev_value > 0f32 {
                value = modify_value;
            } else if value > 0f32 && prev_value < 0f32 {
                value = modify_value;
            }
        }
        SteerOperationType::SetNegativeY => {
            if value < 0f32 && prev_value > 0f32 {
                value = modify_value;
            } else if value > 0f32 && prev_value < 0f32 {
                value = modify_value;
            }
        }
        SteerOperationType::SetNegativeZ => {
            if value < 0f32 && prev_value > 0f32 {
                value = modify_value;
            } else if value > 0f32 && prev_value < 0f32 {
                value = modify_value;
            }
        }
        SteerOperationType::SetMinimum => {
            if value > modify_value {
                value = modify_value;
            }
        }
        SteerOperationType::SetMaximum => {
            if value < modify_value {
                value = modify_value;
            }
        }
        SteerOperationType::SetIgnore => {}
        SteerOperationType::SetInherit => {}
        SteerOperationType::SetTarget => {}
        SteerOperationType::SetHomingValue => {}
        SteerOperationType::SetHomingTime => {}
        SteerOperationType::SetInheritXYZ => {}
    }
    value
}

impl Viewer {
    pub fn open_fchar(&mut self, buffer: Vec<u8>) -> bool {
        parse_json(include_bytes_zstd!("assets/rszsf6.json", 9)).unwrap();
        let fchar = parse_fchar(&buffer);
        match fchar {
            Ok(fchar) => {
                self.asset = Some(fchar.1);
                self.selected_index = -1;
                self.action_index = 0;
                self.current_frame = 1;
                true
            }
            Err(_) => false,
        }
    }

    fn get_action_name(&self, action_index: i32) -> String {
        match self.character {
            Character::Common => {
                let action_name: action_names::CommonActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::CommonActions::Common_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Ryu => {
                let action_name: action_names::RyuActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::RyuActions::Ryu_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Luke => {
                let action_name: action_names::LukeActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::LukeActions::Luke_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Kimberly => {
                let action_name: action_names::KimberlyActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::KimberlyActions::Kimberly_INVALID_ID => {
                        format!("{}", action_index)
                    }
                    _ => action_name.to_string(),
                }
            }
            Character::ChunLi => {
                let action_name: action_names::ChunLiActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::ChunLiActions::ChunLi_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Manon => {
                let action_name: action_names::ManonActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::ManonActions::Manon_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Zangief => {
                let action_name: action_names::ZangiefActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::ZangiefActions::Zangief_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::JP => {
                let action_name: action_names::JPActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::JPActions::JP_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Dhalsim => {
                let action_name: action_names::DhalsimActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::DhalsimActions::Dhalsim_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Cammy => {
                let action_name: action_names::CammyActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::CammyActions::Cammy_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Ken => {
                let action_name: action_names::KenActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::KenActions::Ken_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::DeeJay => {
                let action_name: action_names::DeeJayActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::DeeJayActions::DeeJay_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Lily => {
                let action_name: action_names::LilyActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::LilyActions::Lily_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Blanka => {
                let action_name: action_names::BlankaActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::BlankaActions::Blanka_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Juri => {
                let action_name: action_names::JuriActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::JuriActions::Juri_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Marisa => {
                let action_name: action_names::MarisaActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::MarisaActions::Marisa_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Guile => {
                let action_name: action_names::GuileActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::GuileActions::Guile_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::EHonda => {
                let action_name: action_names::EHondaActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::EHondaActions::EHonda_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
            Character::Jamie => {
                let action_name: action_names::JamieActions =
                    num::FromPrimitive::from_i32(action_index).unwrap_or_default();
                match action_name {
                    action_names::JamieActions::Jamie_INVALID_ID => format!("{}", action_index),
                    _ => action_name.to_string(),
                }
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut action_label: String = format!(
            "Action #{}: {}",
            self.selected_index,
            self.get_action_name(self.action_index)
        );
        if self.selected_index == -1 {
            action_label = "Select an action".to_owned();
        }
        ComboBox::from_label("Action List")
            .selected_text(action_label)
            .width(300.0)
            .show_ui(ui, |ui| match &self.asset {
                Some(fchar) => {
                    for (index, action) in fchar.action_list.iter().enumerate() {
                        let action_index = &action.info.action_data.action_id;
                        if ui
                            .selectable_label(
                                true,
                                format!(
                                    "Action #{}: {}",
                                    index,
                                    self.get_action_name(action_index.clone())
                                ),
                            )
                            .clicked()
                        {
                            self.selected_index = index as i32;
                            self.action_index = action_index.clone();
                            self.should_update = true;
                            self.current_frame = 1;
                            self.action_index_string = "".to_string();
                        }
                    }
                }
                None => (),
            });
        ui.label("Search by action index");
        let textedit_response = ui.add(egui::TextEdit::singleline(&mut self.action_index_string));
        if textedit_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            match &self.asset {
                Some(fchar) => {
                    for (index, action) in fchar.action_list.iter().enumerate() {
                        let action_index = action.info.action_data.action_id.clone();
                        if self.action_index_string != "" {
                            let action_string = self.action_index_string.parse::<i32>();
                            match action_string {
                                Ok(parsed_action_index) => {
                                    if parsed_action_index == action_index {
                                        self.selected_index = index as i32;
                                        self.action_index = action_index.clone();
                                        self.should_update = true;
                                        self.current_frame = 1;
                                    }
                                }
                                Err(_) => ()
                            }
                        }
                    }
                }
                None => (),
            }
            self.action_index_string = "".to_string();
        }

        if self.selected_index != -1 {
            if self.should_update {
                self.action_info = Default::default();
                self.get_action_info();
                self.position = Default::default();
                self.velocity = Default::default();
                self.acceleration = Default::default();
                self.prev_position = Default::default();
                self.prev_velocity = Default::default();
                self.prev_acceleration = Default::default();
                self.root_motion = Default::default();
                for frame in 0..self.current_frame - 1 {
                    self.update_position(frame as i32);
                }
                self.get_boxes();
                self.get_trigger_keys();
                self.should_update = false;
            }
            ui.horizontal(|ui| match &self.asset {
                Some(fchar) => {
                    let action = &fchar.action_list[self.selected_index.clone() as usize];
                    let temp_frame = self.current_frame;
                    ui.add(
                        Slider::new(
                            &mut self.current_frame,
                            1..=action.info.action_data.frames as usize,
                        )
                        .clamp_to_range(true)
                        .smart_aim(true)
                        .orientation(egui::SliderOrientation::Horizontal)
                        .text("Current Frame"),
                    );
                    if temp_frame != self.current_frame {
                        self.should_update = true;
                    }
                }
                None => (),
            })
            .response;
            egui::ScrollArea::vertical().show(ui, |ui| {
                Frame::canvas(ui.style()).show(ui, |ui| self.render_boxes(ui));
                ui.collapsing("Action info", |ui| {
                    let mut first_active_frame: String = format!(
                        "First active frame: {}",
                        self.action_info.first_active_frame + 1
                    );
                    if self.action_info.first_active_frame == -1 {
                        first_active_frame = "First active frame: N/A".to_owned();
                    }
                    ui.label(first_active_frame);
                    let mut recovery_frame: String =
                        format!("Recovery frame: {}", self.action_info.recovery_frame + 1);
                    if self.action_info.recovery_frame == -1 {
                        recovery_frame = "Recovery frame: N/A".to_owned();
                    }
                    ui.label(recovery_frame);
                    ui.label(format!(
                        "First actionable frame: {}",
                        self.action_info.end_frame + 1
                    ));
                    let mut loop_count: String =
                        format!("Loop count: {}", self.action_info.loop_count);
                    if self.action_info.loop_count == -1 {
                        loop_count = "Loop count: infinite".to_owned();
                    }
                    ui.label(loop_count);
                });
                ui.collapsing("Cancel list", |ui| {
                    for trigger in &self.triggers {
                        ui.horizontal(|ui| {
                            ui.label(format!("Action {}", self.get_action_name(trigger.action)));
                            let mut cancel_flags: String = "".to_owned();
                            if trigger.condition_flag & 0b1 > 0 {
                                cancel_flags.push_str("Hit | ")
                            }
                            if trigger.condition_flag & 0b10 > 0 {
                                cancel_flags.push_str("Guard | ")
                            }
                            if trigger.condition_flag & 0b100 > 0 {
                                cancel_flags.push_str("Whiff | ")
                            }
                            if trigger.condition_flag & 0b010000000000 > 0 {
                                cancel_flags.push_str("Counter | ")
                            }
                            if trigger.condition_flag & 0b0001000000000000 > 0 {
                                cancel_flags.push_str("Parry | ")
                            }
                            if trigger.condition_flag & 0b0010000000000000 > 0 {
                                cancel_flags.push_str("Just | ")
                            }
                            if trigger.condition_flag & 0b100000000000 > 0 {
                                cancel_flags.push_str("Strike | ")
                            }
                            if trigger.condition_flag & 0b1000 > 0 {
                                cancel_flags.push_str("Armor | ")
                            }
                            if trigger.condition_flag & 0b00010000 > 0 {
                                cancel_flags.push_str("Jump | ")
                            }
                            if trigger.condition_flag & 0b00100000 > 0 {
                                cancel_flags.push_str("SuperJump | ")
                            }
                            if trigger.condition_flag & 0b10000000 > 0 {
                                cancel_flags.push_str("Fly | ")
                            }
                            if trigger.condition_flag & 0b000100000000 > 0 {
                                cancel_flags.push_str("WallBk | ")
                            }
                            if trigger.condition_flag & 0b01000000000000000000 > 0 {
                                cancel_flags.push_str("VJump | ")
                            }
                            if trigger.condition_flag & 0b10000000000000000000 > 0 {
                                cancel_flags.push_str("FJump | ")
                            }
                            if trigger.condition_flag & 0b000100000000000000000000 > 0 {
                                cancel_flags.push_str("BJump | ")
                            }
                            if trigger.condition_flag & 0b001000000000000000000000 > 0 {
                                cancel_flags.push_str("Throw | ")
                            }
                            if trigger.condition_flag & 0b0100000000000000 > 0 {
                                cancel_flags.push_str("Normal | ")
                            }
                            if trigger.condition_flag & 0b1000000000000000 > 0 {
                                cancel_flags.push_str("Easy | ")
                            }
                            if trigger.condition_flag & 0b00010000000000000000 > 0 {
                                cancel_flags.push_str("Extra | ")
                            }
                            if trigger.condition_flag & 0b01000000 > 0 {
                                cancel_flags.push_str("Defer | ")
                            }
                            if trigger.condition_flag & 0b00100000000000000000 > 0 {
                                cancel_flags.push_str("Inhibit | ")
                            }
                            if trigger.condition_flag & 0b010000000000000000000000 > 0 {
                                cancel_flags.push_str("Terminator | ")
                            }
                            if cancel_flags.len() > 3 {
                                cancel_flags = cancel_flags[0..cancel_flags.len() - 3].to_owned();
                            }
                            ui.label(format!("Cancel flags: {}", cancel_flags));
                        });
                    }
                });
            });

            ui.horizontal(|ui| {
                ui.label("");
            })
            .response
        } else {
            ui.horizontal(|ui| {
                ui.label("Select an action from the action list!");
            })
            .response
        }
    }

    fn get_action_info(&mut self) {
        match &self.asset {
            Some(fchar) => {
                let action = &fchar.action_list[self.selected_index.clone() as usize];
                let action_frame = &action.action.data[0];
                let first_active_frame = &action_frame.fields[0].value;
                match first_active_frame {
                    RSZValue::Int32(frame) => self.action_info.first_active_frame = frame.clone(),
                    _ => (),
                }
                let recovery_frame = &action_frame.fields[1].value;
                match recovery_frame {
                    RSZValue::Int32(frame) => self.action_info.recovery_frame = frame.clone(),
                    _ => (),
                }
                let end_frame = &action_frame.fields[2].value;
                match end_frame {
                    RSZValue::Int32(frame) => self.action_info.end_frame = frame.clone(),
                    _ => (),
                }
                let action_state = &action.action.data[1];
                let loop_count = &action_state.fields[0].value;
                match loop_count {
                    RSZValue::Int32(count) => self.action_info.loop_count = count.clone(),
                    _ => (),
                }
            }
            None => (),
        }
    }

    fn update_position(&mut self, frame: i32) {
        self.velocity.x += self.acceleration.x;
        self.velocity.y += self.acceleration.y;
        self.velocity.z += self.acceleration.z;
        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;
        self.position.z += self.velocity.z;
        match &self.asset {
            Some(fchar) => {
                let action = &fchar.action_list[self.selected_index.clone() as usize];
                for object in &action.objects {
                    for (index, object_index) in object.action.object_table.iter().enumerate() {
                        if object.info.object_data.key_data[index].key_start_frame <= frame
                            && object.info.object_data.key_data[index].key_end_frame > frame
                        {
                            let data = &object.action.data[object_index.clone() as usize - 1];
                            match data.name.as_str() {
                                "CharacterAsset.SteerKey" => {
                                    let op_value = &data.fields[0].value;
                                    let mut op_type: SteerOperationType = Default::default();
                                    match op_value {
                                        RSZValue::UInt8(ubyte) => {
                                            op_type =
                                                num::FromPrimitive::from_u8(ubyte.clone()).unwrap();
                                        }
                                        _ => (),
                                    }
                                    let value = &data.fields[1].value;
                                    let mut value_type: SteerValueType = Default::default();
                                    match value {
                                        RSZValue::UInt8(ubyte) => {
                                            value_type =
                                                num::FromPrimitive::from_u8(ubyte.clone()).unwrap();
                                        }
                                        _ => (),
                                    }
                                    let modify_type = &data.fields[4].value;
                                    let mut modify_value = 0f32;
                                    match modify_type {
                                        RSZValue::Float(float) => {
                                            modify_value = float.clone();
                                        }
                                        _ => (),
                                    }
                                    match value_type {
                                        SteerValueType::VelocityX => {
                                            self.velocity.x = steer_key_to_value(
                                                op_type.clone(),
                                                self.velocity.x,
                                                self.prev_velocity.x,
                                                modify_value,
                                            )
                                        }
                                        SteerValueType::VelocityY => {
                                            self.velocity.y = steer_key_to_value(
                                                op_type.clone(),
                                                self.velocity.y,
                                                self.prev_velocity.y,
                                                modify_value,
                                            )
                                        }
                                        SteerValueType::VelocityZ => {
                                            self.velocity.z = steer_key_to_value(
                                                op_type.clone(),
                                                self.velocity.z,
                                                self.prev_velocity.z,
                                                modify_value,
                                            )
                                        }
                                        SteerValueType::AccelerationX => {
                                            self.acceleration.x = steer_key_to_value(
                                                op_type.clone(),
                                                self.acceleration.x,
                                                self.prev_acceleration.x,
                                                modify_value,
                                            )
                                        }
                                        SteerValueType::AccelerationY => {
                                            self.acceleration.y = steer_key_to_value(
                                                op_type.clone(),
                                                self.acceleration.y,
                                                self.prev_acceleration.y,
                                                modify_value,
                                            )
                                        }
                                        SteerValueType::AccelerationZ => {
                                            self.acceleration.z = steer_key_to_value(
                                                op_type.clone(),
                                                self.acceleration.z,
                                                self.prev_acceleration.z,
                                                modify_value,
                                            )
                                        }
                                    }
                                    match op_type {
                                        SteerOperationType::SetNegativeX => {
                                            if self.velocity.x == 0f32 {
                                                self.acceleration.x = 0f32;
                                            }
                                        }
                                        SteerOperationType::SetNegativeY => {
                                            if self.velocity.y == 0f32 {
                                                self.acceleration.y = 0f32;
                                            }
                                        }
                                        SteerOperationType::SetNegativeZ => {
                                            if self.velocity.z == 0f32 {
                                                self.acceleration.z = 0f32;
                                            }
                                        }
                                        _ => (),
                                    }
                                }
                                "CharacterAsset.PlaceKey" => {
                                    let mut pos_list: Vec<&RSZValue> = vec![];
                                    let pos_list_value = &data.fields[3].value;
                                    match pos_list_value {
                                        RSZValue::List(list) => {
                                            for value in list {
                                                pos_list.push(value);
                                            }
                                        }
                                        _ => (),
                                    }
                                    
                                    if frame >= pos_list.len() as i32 {
                                        return;
                                    }

                                    let axis = &data.fields[1].value;
                                    match axis {
                                        RSZValue::UInt8(byte) => match byte {
                                            0 => match pos_list[frame as usize] {
                                                RSZValue::Float(float) => {
                                                    self.root_motion.x = float.clone()
                                                }
                                                _ => (),
                                            },
                                            1 => match pos_list[frame as usize] {
                                                RSZValue::Float(float) => {
                                                    self.root_motion.y = float.clone()
                                                }
                                                _ => (),
                                            },
                                            2 => match pos_list[frame as usize] {
                                                RSZValue::Float(float) => {
                                                    self.root_motion.z = float.clone()
                                                }
                                                _ => (),
                                            },
                                            _ => (),
                                        },
                                        _ => (),
                                    };
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
            None => (),
        }
        self.prev_acceleration.x = self.acceleration.x;
        self.prev_acceleration.y = self.acceleration.y;
        self.prev_acceleration.z = self.acceleration.z;
        self.prev_velocity.x = self.velocity.x;
        self.prev_velocity.y = self.velocity.y;
        self.prev_velocity.z = self.velocity.z;
        self.prev_position.x = self.position.x;
        self.prev_position.y = self.position.y;
        self.prev_position.z = self.position.z;

        if self.position.y < 0f32 {
            self.position.y = 0f32;
            self.velocity.y = 0f32;
            self.acceleration.y = 0f32;
        }
    }

    fn get_triggers(&mut self, group: i32, condition_flag: u32) {
        match &self.asset {
            Some(fchar) => {
                let mut data_index: usize = 0;
                for (n, data_id) in fchar.data_id_table.iter().enumerate() {
                    match data_id {
                        DataId::TriggerGroup => data_index = n,
                        _ => (),
                    }
                }
                let mut index: usize = 0;
                for (n, value) in fchar.data_list_table[data_index]
                    .data_ids
                    .iter()
                    .enumerate()
                {
                    if value.clone() == group as u32 {
                        index = n;
                    }
                }
                if index == 0 {
                    return;
                }
                let mut triggers: Vec<u64> = vec![];
                let trigger_group = &fchar.data_list_table[data_index].data_rsz.data[index];
                match &trigger_group.fields[1].value {
                    RSZValue::List(list) => {
                        for select_trigger in list {
                            match select_trigger {
                                RSZValue::UInt64(ulong) => {
                                    triggers.push(ulong.clone());
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
                for (trigger_index, select_trigger) in triggers.iter().enumerate() {
                    let bits: BitVec = BitVec::from_element(select_trigger.clone() as usize);
                    for (bit_index, bit) in bits.iter().enumerate() {
                        if bit == false {
                            continue;
                        }
                        let mut data_index: usize = 0;
                        for (n, data_id) in fchar.data_id_table.iter().enumerate() {
                            match data_id {
                                DataId::Trigger => data_index = n,
                                _ => (),
                            }
                        }
                        let mut index: usize = 0;
                        for (n, value) in fchar.data_list_table[data_index]
                            .data_ids
                            .iter()
                            .enumerate()
                        {
                            if value.clone() == (bit_index + trigger_index * 64) as u32 {
                                index = n + 1;
                            }
                        }
                        let mut stored_trigger: Trigger = Default::default();
                        stored_trigger.condition_flag = condition_flag;
                        let trigger =
                            &fchar.data_list_table[data_index].data_rsz.data[index * 17 - 1];
                        match &trigger.fields[5].value {
                            RSZValue::Int32(action) => {
                                stored_trigger.action = action.clone();
                            }
                            _ => (),
                        }
                        self.triggers.push(stored_trigger);
                    }
                }
            }
            None => (),
        }
    }

    fn get_trigger_keys(&mut self) {
        self.triggers.clear();
        let mut groups: Vec<i32> = vec![];
        let mut condition_flags: Vec<u32> = vec![];
        match &self.asset {
            Some(fchar) => {
                let action = &fchar.action_list[self.selected_index.clone() as usize];
                for object in &action.objects {
                    for (index, object_index) in object.action.object_table.iter().enumerate() {
                        if object.info.object_data.key_data[index].key_start_frame
                            <= self.current_frame as i32 - 1
                            && object.info.object_data.key_data[index].key_end_frame
                                > self.current_frame as i32 - 1
                        {
                            let data = &object.action.data[object_index.clone() as usize - 1];
                            match data.name.as_str() {
                                "CharacterAsset.TriggerKey" => {
                                    let group = &data.fields[0].value;
                                    match group {
                                        RSZValue::Int32(group) => {
                                            groups.push(group.clone());
                                        }
                                        _ => (),
                                    }
                                    let condition_flag = &data.fields[1].value;
                                    match condition_flag {
                                        RSZValue::UInt32(condition_flag) => {
                                            condition_flags.push(condition_flag.clone());
                                        }
                                        _ => (),
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
            None => (),
        }
        for (index, group) in groups.iter().enumerate() {
            self.get_triggers(group.clone(), condition_flags[index]);
        }
        self.triggers.sort_unstable();
        self.triggers.dedup();
    }

    fn index_to_box(
        &self,
        fchar: &CharacterAsset,
        int: i32,
        data_type: DataId,
        boxes: &mut Vec<CollisionBox>,
    ) {
        let mut data_index: usize = 0;
        for (n, data_id) in fchar.data_id_table.iter().enumerate() {
            if data_id.clone() == data_type {
                data_index = n;
            }
        }
        let mut index: usize = 0;
        for (n, value) in fchar.data_list_table[data_index]
            .data_ids
            .iter()
            .enumerate()
        {
            if value.clone() == int as u32 {
                index = n + 1;
            }
        }
        if index == 0 {
            return;
        }
        let data = &fchar.data_list_table[data_index].data_rsz.data[index * 6 - 1];
        let x_field = &data.fields[0].value;
        let mut x = 0f32;
        match x_field {
            RSZValue::Int16(short) => x = short.clone() as f32,
            _ => (),
        }
        let y_field = &data.fields[1].value;
        let mut y = 0f32;
        match y_field {
            RSZValue::Int16(short) => y = short.clone() as f32,
            _ => (),
        }
        let width_field = &data.fields[2].value;
        let mut width = 0f32;
        match width_field {
            RSZValue::Int16(short) => width = short.clone() as f32,
            _ => (),
        }
        let height_field = &data.fields[3].value;
        let mut height = 0f32;
        match height_field {
            RSZValue::Int16(short) => height = short.clone() as f32,
            _ => (),
        }
        let collision_box: CollisionBox = CollisionBox {
            x,
            y,
            width,
            height,
        };
        boxes.push(collision_box);
    }

    fn get_boxes(&mut self) {
        self.push_collision_keys.clear();
        self.damage_collision_keys.clear();
        self.attack_collision_keys.clear();
        match &self.asset {
            Some(fchar) => {
                let action = &fchar.action_list[self.selected_index.clone() as usize];
                for object in &action.objects {
                    for (index, object_index) in object.action.object_table.iter().enumerate() {
                        if object.info.object_data.key_data[index].key_start_frame
                            <= self.current_frame as i32 - 1
                            && object.info.object_data.key_data[index].key_end_frame
                                > self.current_frame as i32 - 1
                        {
                            let data = &object.action.data[object_index.clone() as usize - 1];
                            match data.name.as_str() {
                                "CharacterAsset.PushCollisionKey" => {
                                    let mut boxes: Vec<CollisionBox> = vec![];
                                    let mut pushbox: CollisionBox = Default::default();

                                    let mut condition = 0u8;
                                    match &data.fields[0].value {
                                        RSZValue::UInt8(ubyte) => condition = ubyte.clone(),
                                        _ => (),
                                    }
                                    let mut attribute = 0u16;
                                    match &data.fields[1].value {
                                        RSZValue::UInt16(ushort) => attribute = ushort.clone(),
                                        _ => (),
                                    }
                                    match &data.fields[2].value {
                                        RSZValue::Int32(int) => {
                                            self.index_to_box(
                                                &fchar,
                                                int.clone(),
                                                DataId::ThrowHurtBox,
                                                &mut boxes,
                                            );
                                        }
                                        _ => (),
                                    }

                                    if boxes.len() > 0 {
                                        pushbox = boxes[0].clone();
                                    }

                                    let push_collision = PushCollisionKey {
                                        condition,
                                        attribute,
                                        pushbox,
                                    };
                                    self.push_collision_keys.push(push_collision)
                                }
                                "CharacterAsset.DamageCollisionKey" => {
                                    let mut boxes: Vec<CollisionBox> = vec![];

                                    let mut head_list: &Vec<RSZValue> = &vec![];
                                    match &data.fields[9].value {
                                        RSZValue::List(list) => head_list = list,
                                        _ => (),
                                    }
                                    for head_index in head_list {
                                        match head_index {
                                            RSZValue::Int32(int) => {
                                                self.index_to_box(
                                                    &fchar,
                                                    int.clone(),
                                                    DataId::HurtBox,
                                                    &mut boxes,
                                                );
                                            }
                                            _ => (),
                                        }
                                    }
                                    let mut body_list: &Vec<RSZValue> = &vec![];
                                    match &data.fields[10].value {
                                        RSZValue::List(list) => body_list = list,
                                        _ => (),
                                    }
                                    for body_index in body_list {
                                        match body_index {
                                            RSZValue::Int32(int) => {
                                                self.index_to_box(
                                                    &fchar,
                                                    int.clone(),
                                                    DataId::HurtBox,
                                                    &mut boxes,
                                                );
                                            }
                                            _ => (),
                                        }
                                    }
                                    let mut leg_list: &Vec<RSZValue> = &vec![];
                                    match &data.fields[11].value {
                                        RSZValue::List(list) => leg_list = list,
                                        _ => (),
                                    }
                                    for leg_index in leg_list {
                                        match leg_index {
                                            RSZValue::Int32(int) => {
                                                self.index_to_box(
                                                    &fchar,
                                                    int.clone(),
                                                    DataId::HurtBox,
                                                    &mut boxes,
                                                );
                                            }
                                            _ => (),
                                        }
                                    }
                                    let mut throw_list: &Vec<RSZValue> = &vec![];
                                    match &data.fields[12].value {
                                        RSZValue::List(list) => throw_list = list,
                                        _ => (),
                                    }

                                    let mut condition = 0u8;
                                    match &data.fields[0].value {
                                        RSZValue::UInt8(ubyte) => condition = ubyte.clone(),
                                        _ => (),
                                    }
                                    let mut collision_type = 0u8;
                                    match &data.fields[1].value {
                                        RSZValue::UInt8(ubyte) => collision_type = ubyte.clone(),
                                        _ => (),
                                    }
                                    let mut immune = 0u8;
                                    match &data.fields[2].value {
                                        RSZValue::UInt8(ubyte) => immune = ubyte.clone(),
                                        _ => (),
                                    }
                                    let mut extend = 0u8;
                                    match &data.fields[3].value {
                                        RSZValue::UInt8(ubyte) => extend = ubyte.clone(),
                                        _ => (),
                                    }
                                    let mut level = 0u8;
                                    match &data.fields[4].value {
                                        RSZValue::UInt8(ubyte) => level = ubyte.clone(),
                                        _ => (),
                                    }
                                    let mut type_flag = 0u32;
                                    match &data.fields[5].value {
                                        RSZValue::UInt32(uint) => type_flag = uint.clone(),
                                        _ => (),
                                    }

                                    let damage_collision = DamageCollisionKey {
                                        condition,
                                        collision_type,
                                        immune,
                                        extend,
                                        level,
                                        type_flag,
                                        boxes,
                                    };
                                    self.damage_collision_keys.push(damage_collision)
                                }
                                "CharacterAsset.AttackCollisionKey" => {
                                    let mut boxes: Vec<CollisionBox> = vec![];

                                    let mut condition = 0u8;
                                    match &data.fields[0].value {
                                        RSZValue::UInt8(ubyte) => condition = ubyte.clone(),
                                        _ => (),
                                    }
                                    let mut collision_type = 0u8;
                                    match &data.fields[1].value {
                                        RSZValue::UInt8(ubyte) => collision_type = ubyte.clone(),
                                        _ => (),
                                    }
                                    let mut hit_id = 0i8;
                                    match &data.fields[2].value {
                                        RSZValue::Int8(byte) => hit_id = byte.clone(),
                                        _ => (),
                                    }
                                    let mut guard_bit = 0u8;
                                    match &data.fields[3].value {
                                        RSZValue::UInt8(ubyte) => guard_bit = ubyte.clone(),
                                        _ => (),
                                    }
                                    let mut kind_flag = 0u32;
                                    match &data.fields[4].value {
                                        RSZValue::UInt32(uint) => kind_flag = uint.clone(),
                                        _ => (),
                                    }
                                    let mut hit_offset = [0; 2];
                                    match &data.fields[4].value {
                                        RSZValue::Int2(int2) => {
                                            hit_offset[0] = int2.x.clone();
                                            hit_offset[1] = int2.y.clone();
                                        }
                                        _ => (),
                                    }

                                    let mut box_list: &Vec<RSZValue> = &vec![];
                                    match &data.fields[11].value {
                                        RSZValue::List(list) => box_list = list,
                                        _ => (),
                                    }
                                    for index in box_list {
                                        match index {
                                            RSZValue::Int32(int) => {
                                                if collision_type == 3 {
                                                    self.index_to_box(
                                                        &fchar,
                                                        int.clone(),
                                                        DataId::ProximityBox,
                                                        &mut boxes,
                                                    );
                                                } else {
                                                    self.index_to_box(
                                                        &fchar,
                                                        int.clone(),
                                                        DataId::StrikeBox,
                                                        &mut boxes,
                                                    );
                                                }
                                            }
                                            _ => (),
                                        }
                                    }

                                    let attack_collision = AttackCollisionKey {
                                        condition,
                                        collision_type,
                                        hit_id,
                                        guard_bit,
                                        kind_flag,
                                        hit_offset,
                                        boxes,
                                    };
                                    self.attack_collision_keys.push(attack_collision)
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
            None => (),
        }
    }

    fn render_boxes(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (mut response, painter) = ui.allocate_painter(
            eframe::emath::Vec2 {
                x: (ui.available_width()),
                y: (ui.available_height() - 150.0),
            },
            Sense::click_and_drag(),
        );
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if self.last_cursor_pos != Default::default() {
                let pointer_delta = pointer_pos - self.last_cursor_pos;
                self.offset_x += pointer_delta.x;
                self.offset_y += pointer_delta.y;
                response.mark_changed();
            }
            self.last_cursor_pos = pointer_pos;
        } else {
            self.last_cursor_pos = Default::default();
        }
        if response.clicked_by(egui::PointerButton::Secondary) {
            self.offset_x = 90.0;
            self.offset_y = 300.0;
        }
        for push_collision_key in &self.push_collision_keys {
            painter.rect(
                Rect {
                    min: Pos2 {
                        x: push_collision_key.pushbox.x.clone()
                            - push_collision_key.pushbox.width.clone()
                            + self.offset_x.clone()
                            + 0.5
                            + self.position.x
                            + self.root_motion.x,
                        y: -push_collision_key.pushbox.y.clone()
                            - push_collision_key.pushbox.height.clone()
                            + self.offset_y.clone()
                            + 0.5
                            - self.position.y
                            + self.root_motion.y,
                    },
                    max: Pos2 {
                        x: push_collision_key.pushbox.x.clone()
                            + push_collision_key.pushbox.width.clone()
                            + self.offset_x.clone()
                            - 0.5
                            + self.position.x
                            + self.root_motion.x,
                        y: -push_collision_key.pushbox.y.clone()
                            + push_collision_key.pushbox.height.clone()
                            + self.offset_y.clone()
                            - 0.5
                            - self.position.y
                            + self.root_motion.y,
                    },
                },
                0.0,
                egui::Rgba::from_rgba_unmultiplied(0.8, 0.8, 0.0, 0.25),
                Stroke {
                    width: 1.0,
                    color: Color32::YELLOW,
                },
            );
        }
        for damage_collision_key in &self.damage_collision_keys {
            for hurtbox in &damage_collision_key.boxes {
                painter.rect(
                    Rect {
                        min: Pos2 {
                            x: hurtbox.x.clone() - hurtbox.width.clone()
                                + self.offset_x.clone()
                                + 0.5
                                + self.position.x
                                + self.root_motion.x,
                            y: -hurtbox.y.clone() - hurtbox.height.clone()
                                + self.offset_y.clone()
                                + 0.5
                                - self.position.y
                                + self.root_motion.y,
                        },
                        max: Pos2 {
                            x: hurtbox.x.clone() + hurtbox.width.clone() + self.offset_x.clone()
                                - 0.5
                                + self.position.x
                                + self.root_motion.x,
                            y: -hurtbox.y.clone() + hurtbox.height.clone() + self.offset_y.clone()
                                - 0.5
                                - self.position.y
                                + self.root_motion.y,
                        },
                    },
                    0.0,
                    egui::Rgba::from_rgba_unmultiplied(0.0, 0.8, 0.0, 0.25),
                    Stroke {
                        width: 1.0,
                        color: Color32::GREEN,
                    },
                );
            }
        }
        for attack_collision_key in &self.attack_collision_keys {
            for hitbox in &attack_collision_key.boxes {
                if attack_collision_key.collision_type == 3 {
                    painter.rect(
                        Rect {
                            min: Pos2 {
                                x: hitbox.x.clone() - hitbox.width.clone()
                                    + self.offset_x.clone()
                                    + 0.5,
                                y: -hitbox.y.clone() - hitbox.height.clone()
                                    + self.offset_y.clone()
                                    + 0.5,
                            },
                            max: Pos2 {
                                x: hitbox.x.clone() + hitbox.width.clone() + self.offset_x.clone()
                                    - 0.5,
                                y: -hitbox.y.clone()
                                    + hitbox.height.clone()
                                    + self.offset_y.clone()
                                    - 0.5,
                            },
                        },
                        0.0,
                        egui::Rgba::from_rgba_unmultiplied(0.5, 0.5, 0.5, 0.25),
                        Stroke {
                            width: 1.0,
                            color: Color32::GRAY,
                        },
                    );
                } else {
                    painter.rect(
                        Rect {
                            min: Pos2 {
                                x: hitbox.x.clone() - hitbox.width.clone()
                                    + self.offset_x.clone()
                                    + 0.5
                                    + self.position.x
                                    + self.root_motion.x,
                                y: -hitbox.y.clone() - hitbox.height.clone()
                                    + self.offset_y.clone()
                                    + 0.5
                                    - self.position.y
                                    + self.root_motion.y,
                            },
                            max: Pos2 {
                                x: hitbox.x.clone() + hitbox.width.clone() + self.offset_x.clone()
                                    - 0.5
                                    + self.position.x
                                    + self.root_motion.x,
                                y: -hitbox.y.clone()
                                    + hitbox.height.clone()
                                    + self.offset_y.clone()
                                    - 0.5
                                    - self.position.y
                                    + self.root_motion.y,
                            },
                        },
                        0.0,
                        egui::Rgba::from_rgba_unmultiplied(0.8, 0.0, 0.0, 0.25),
                        Stroke {
                            width: 1.0,
                            color: Color32::RED,
                        },
                    )
                }
            }
        }
        let mut visuals = ui.ctx().style().visuals.clone();
        if visuals.dark_mode {
            painter.circle(
                Pos2 {
                    x: self.position.x + self.offset_x.clone(),
                    y: -self.position.y + self.offset_y.clone(),
                },
                5f32,
                Color32::GRAY,
                Stroke {
                    width: 1.0,
                    color: Color32::WHITE,
                },
            );
        } else {
            painter.circle(
                Pos2 {
                    x: self.position.x + self.offset_x.clone(),
                    y: -self.position.y + self.offset_y.clone(),
                },
                5f32,
                Color32::GRAY,
                Stroke {
                    width: 1.0,
                    color: Color32::BLACK,
                },
            );
        }

        response
    }
}
