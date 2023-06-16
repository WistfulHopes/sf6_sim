use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use eframe::egui;
use eframe::egui::{Color32, ComboBox, Frame, Sense, Slider};
use eframe::emath::{Pos2, Rect};
use eframe::epaint::Stroke;
use sf6_rsz_parser::CharacterAsset;
use sf6_rsz_parser::parse_fchar;
use sf6_rsz_parser::rsz::json_parser::parse_json;
use sf6_rsz_parser::rsz::RSZValue;

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
    boxes: Vec<CollisionBox>
}

#[derive(Default)]
struct AttackCollisionKey {
    condition: u8,
    collision_type: u8,
    hit_id: i8,
    guard_bit: u8,
    kind_flag: u32,
    hit_offset: [i32; 2],
    boxes: Vec<CollisionBox>
}

pub struct Viewer {
    pub asset: Option<CharacterAsset>,
    selected_index: i32,
    action_index: i32,
    current_frame: usize,
    push_collision_keys: Vec<PushCollisionKey>,
    damage_collision_keys: Vec<DamageCollisionKey>,
    attack_collision_keys: Vec<AttackCollisionKey>,
    offset_x: f32,
    offset_y: f32,
    last_cursor_pos: Pos2,
    should_update: bool,
}

impl Default for Viewer {
    fn default() -> Self {
        Self {
            asset: None,
            selected_index: -1,
            action_index: 0,
            current_frame: 0,
            push_collision_keys: vec![],
            damage_collision_keys: vec![],
            attack_collision_keys: vec![],
            offset_x: 640.0,
            offset_y: 360.0,
            last_cursor_pos: Default::default(),
            should_update: false,
        }
    }
}

impl Viewer {
    pub fn open_fchar(&mut self, path: &PathBuf) -> bool {
        let file = File::open(&path);
        match file {
            Ok(mut file) => {
                parse_json("rszsf6.json".to_string()).unwrap();
                let mut buffer: Vec<u8> = Vec::new();
                file.read_to_end(&mut buffer).unwrap();
                let fchar = parse_fchar(&buffer);
                match fchar {
                    Ok(fchar) => {
                        self.asset = Some(fchar.1);
                        true
                    }   
                    Err(_) => false
                }
            }
            Err(_) => false
        }
    }
    
    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ComboBox::from_label("Action List")
            .selected_text(format!("Action #{}: {}", self.selected_index, self.action_index))
            .width(150.0)
            .show_ui(ui, |ui| {
                match &self.asset {
                    Some(fchar) => {
                        for (index, action) in fchar.action_list.iter().enumerate() {
                            let action_index = &action.info.action_data.action_id;
                            if ui.selectable_label(true, format!("Action #{}: {}", index, action_index)).clicked()
                            {
                                self.selected_index = index as i32;
                                self.action_index = action_index.clone();
                                self.should_update = true;
                            }
                        }
                    }
                    None => ()
                }
            });
        if self.selected_index != -1 {
            if self.should_update {
                self.get_boxes();
                self.should_update = false;
            }
            Frame::canvas(ui.style()).show(ui, |ui| {
                self.render_boxes(ui)
            });
            
            ui.horizontal(|ui| {
                match &self.asset {
                    Some(fchar) => {
                        let action = &fchar.action_list[self.selected_index.clone() as usize];
                        let temp_frame = self.current_frame;
                        ui.add(
                            Slider::new(&mut self.current_frame, 0..=action.info.action_data.frames as usize - 1)
                                .clamp_to_range(true)
                                .smart_aim(true)
                                .orientation(egui::SliderOrientation::Horizontal)
                                .text("Current Frame")
                        );
                        if temp_frame != self.current_frame {
                            self.get_boxes();
                        }
                    }
                    None => ()
                }
            }).response
        }
        else {
            ui.horizontal(|ui| {
                ui.label("Select an action from the action list!");
            }).response
        }
    }
    
    fn index_to_box(&self, fchar: &CharacterAsset, int: i32, data_index: i32, boxes: &mut Vec<CollisionBox>)
    {
        let mut index: usize = 0;
        for (n, value) in fchar.data_list_table[data_index as usize].data_ids.iter().enumerate()
        {
            if value.clone() == int as u32 {
                index = n + 1;
            }
        }
        if index == 0 {
            return;
        }
        let data = &fchar.data_list_table[data_index as usize]
            .data_rsz.data[index * 6 - 1];
        let x_field = &data.fields[0].value;
        let mut x = 0f32;
        match x_field {
            RSZValue::Int16(short) => x = short.clone() as f32,
            _ => ()
        }
        let y_field = &data.fields[1].value;
        let mut y = 0f32;
        match y_field {
            RSZValue::Int16(short) => y = short.clone() as f32,
            _ => ()
        }
        let width_field = &data.fields[2].value;
        let mut width = 0f32;
        match width_field {
            RSZValue::Int16(short) => width = short.clone() as f32,
            _ => ()
        }
        let height_field = &data.fields[3].value;
        let mut height = 0f32;
        match height_field {
            RSZValue::Int16(short) => height = short.clone() as f32,
            _ => ()
        }
        let collision_box: CollisionBox = CollisionBox {
            x,
            y,
            width,
            height,
        };
        boxes.push(collision_box);
    }
    
    fn get_boxes(&mut self)
    {
        self.push_collision_keys.clear();
        self.damage_collision_keys.clear();
        self.attack_collision_keys.clear();
        match &self.asset {
            Some(fchar) => {
                let action = &fchar.action_list[self.selected_index.clone() as usize];
                for object in &action.objects {
                    for (index, object_index) in object.action.object_table.iter().enumerate() {
                        if object.info.object_data.key_data[index].key_start_frame <= self.current_frame as i32
                            && object.info.object_data.key_data[index].key_end_frame > self.current_frame as i32 {
                            let data = &object.action.data[object_index.clone() as usize - 1];
                            match data.name.as_str() {
                                "CharacterAsset.PushCollisionKey" => {
                                    let mut boxes: Vec<CollisionBox> = vec![];
                                    let mut pushbox: CollisionBox = Default::default();

                                    let mut condition = 0u8;
                                    match &data.fields[0].value {
                                        RSZValue::UInt8(ubyte) => condition = ubyte.clone(),
                                        _ => ()
                                    }
                                    let mut attribute = 0u16;
                                    match &data.fields[1].value {
                                        RSZValue::UInt16(ushort) => attribute = ushort.clone(),
                                        _ => ()
                                    }
                                    match &data.fields[2].value {
                                        RSZValue::Int32(int) => {
                                            self.index_to_box(&fchar, int.clone(), 16, &mut boxes);
                                        }
                                        _ => ()
                                    }

                                    pushbox = boxes[0].clone();

                                    let push_collision = PushCollisionKey {
                                        condition,
                                        attribute,
                                        pushbox,
                                    };
                                    self.push_collision_keys.push(push_collision)
                                },
                                "CharacterAsset.DamageCollisionKey" => {
                                    let mut boxes: Vec<CollisionBox> = vec![];

                                    let mut head_list: &Vec<RSZValue> = &vec![];
                                    match &data.fields[9].value {
                                        RSZValue::List(list) => head_list = list,
                                        _ => ()
                                    }
                                    for head_index in head_list {
                                        match head_index {
                                            RSZValue::Int32(int) => {
                                                self.index_to_box(&fchar, int.clone(), 17, &mut boxes);
                                            }
                                            _ => ()
                                        }
                                    }
                                    let mut body_list: &Vec<RSZValue> = &vec![];
                                    match &data.fields[10].value {
                                        RSZValue::List(list) => body_list = list,
                                        _ => ()
                                    }
                                    for body_index in body_list {
                                        match body_index {
                                            RSZValue::Int32(int) => {
                                                self.index_to_box(&fchar, int.clone(), 17, &mut boxes);
                                            }
                                            _ => ()
                                        }
                                    }
                                    let mut leg_list: &Vec<RSZValue> = &vec![];
                                    match &data.fields[11].value {
                                        RSZValue::List(list) => leg_list = list,
                                        _ => ()
                                    }
                                    for leg_index in leg_list {
                                        match leg_index {
                                            RSZValue::Int32(int) => {
                                                self.index_to_box(&fchar, int.clone(), 17, &mut boxes);
                                            }
                                            _ => ()
                                        }
                                    }
                                    let mut throw_list: &Vec<RSZValue> = &vec![];
                                    match &data.fields[12].value {
                                        RSZValue::List(list) => throw_list = list,
                                        _ => ()
                                    }

                                    let mut condition = 0u8;
                                    match &data.fields[0].value {
                                        RSZValue::UInt8(ubyte) => condition = ubyte.clone(),
                                        _ => ()
                                    }
                                    let mut collision_type = 0u8;
                                    match &data.fields[1].value {
                                        RSZValue::UInt8(ubyte) => collision_type = ubyte.clone(),
                                        _ => ()
                                    }
                                    let mut immune = 0u8;
                                    match &data.fields[2].value {
                                        RSZValue::UInt8(ubyte) => immune = ubyte.clone(),
                                        _ => ()
                                    }
                                    let mut extend = 0u8;
                                    match &data.fields[3].value {
                                        RSZValue::UInt8(ubyte) => extend = ubyte.clone(),
                                        _ => ()
                                    }
                                    let mut level = 0u8;
                                    match &data.fields[4].value {
                                        RSZValue::UInt8(ubyte) => level = ubyte.clone(),
                                        _ => ()
                                    }
                                    let mut type_flag = 0u32;
                                    match &data.fields[5].value {
                                        RSZValue::UInt32(uint) => type_flag = uint.clone(),
                                        _ => ()
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
                                },
                                "CharacterAsset.AttackCollisionKey" => {
                                    let mut boxes: Vec<CollisionBox> = vec![];

                                    let mut box_list: &Vec<RSZValue> = &vec![];
                                    match &data.fields[11].value {
                                        RSZValue::List(list) => box_list = list,
                                        _ => ()
                                    }
                                    for index in box_list {
                                        match index {
                                            RSZValue::Int32(int) => {
                                                self.index_to_box(&fchar, int.clone(), 9, &mut boxes);
                                            }
                                            _ => ()
                                        }
                                    }

                                    let mut condition = 0u8;
                                    match &data.fields[0].value {
                                        RSZValue::UInt8(ubyte) => condition = ubyte.clone(),
                                        _ => ()
                                    }
                                    let mut collision_type = 0u8;
                                    match &data.fields[1].value {
                                        RSZValue::UInt8(ubyte) => collision_type = ubyte.clone(),
                                        _ => ()
                                    }
                                    let mut hit_id = 0i8;
                                    match &data.fields[2].value {
                                        RSZValue::Int8(byte) => hit_id = byte.clone(),
                                        _ => ()
                                    }
                                    let mut guard_bit = 0u8;
                                    match &data.fields[3].value {
                                        RSZValue::UInt8(ubyte) => guard_bit = ubyte.clone(),
                                        _ => ()
                                    }
                                    let mut kind_flag = 0u32;
                                    match &data.fields[4].value {
                                        RSZValue::UInt32(uint) => kind_flag = uint.clone(),
                                        _ => ()
                                    }
                                    let mut hit_offset = [0; 2];
                                    match &data.fields[4].value {
                                        RSZValue::Int2(int2) => {
                                            hit_offset[0] = int2.x.clone();
                                            hit_offset[1] = int2.y.clone();
                                        },
                                        _ => ()
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
                                },
                                _ => ()
                            }
                        }
                    }
                }
            }
            None => ()
        }
    }
    
    fn render_boxes(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (mut response, painter) = ui.allocate_painter(
            eframe::emath::Vec2 {
                x: (ui.available_width()),
                y: (ui.available_height() - 150.0)
            },
            Sense::click_and_drag()
        );
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if self.last_cursor_pos != Default::default()
            {
                let pointer_delta = pointer_pos - self.last_cursor_pos;
                self.offset_x += pointer_delta.x;
                self.offset_y += pointer_delta.y;
                response.mark_changed();
            }
            self.last_cursor_pos = pointer_pos;
        }
        else {
            self.last_cursor_pos = Default::default();
        }
        if response.clicked_by(egui::PointerButton::Secondary)
        {
            self.offset_x = 640.0;
            self.offset_y = 360.0;
        }
        for push_collision_key in &self.push_collision_keys
        {
            painter.rect_stroke(
                Rect {
                    min: Pos2{x: -(push_collision_key.pushbox.x.clone() + push_collision_key.pushbox.width.clone()) + self.offset_x.clone() + 1.5,
                        y: -(push_collision_key.pushbox.y.clone() + push_collision_key.pushbox.height.clone()) + self.offset_y.clone() + 1.5},
                    max: Pos2{x: -(push_collision_key.pushbox.x.clone() - push_collision_key.pushbox.width.clone()) + self.offset_x.clone() - 1.5,
                        y: -(push_collision_key.pushbox.y.clone() - push_collision_key.pushbox.height.clone()) + self.offset_y.clone() - 1.5},
                },
                0.0,
                Stroke{width: 3.0, color: Color32::YELLOW},
            );
        }
        for damage_collision_key in &self.damage_collision_keys
        {
            for hurtbox in &damage_collision_key.boxes {
                painter.rect_stroke(
                    Rect {
                        min: Pos2{x: -(hurtbox.x.clone() + hurtbox.width.clone()) + self.offset_x.clone() + 1.5,
                            y: -(hurtbox.y.clone() + hurtbox.height.clone()) + self.offset_y.clone() + 1.5},
                        max: Pos2{x: -(hurtbox.x.clone() - hurtbox.width.clone()) + self.offset_x.clone() - 1.5,
                            y: -(hurtbox.y.clone() - hurtbox.height.clone()) + self.offset_y.clone() - 1.5},
                    },
                    0.0,
                    Stroke{width: 3.0, color: Color32::GREEN},
                );
            }
        }
        for attack_collision_key in &self.attack_collision_keys
        {
            for hitbox in &attack_collision_key.boxes {
                if attack_collision_key.collision_type == 3 {
                    painter.rect_stroke(
                        Rect {
                            min: Pos2{x: -(hitbox.x.clone() + hitbox.width.clone()) + self.offset_x.clone() + 1.5,
                                y: -(hitbox.y.clone() + hitbox.height.clone()) + self.offset_y.clone() + 1.5},
                            max: Pos2{x: -(hitbox.x.clone() - hitbox.width.clone()) + self.offset_x.clone() - 1.5,
                                y: -(hitbox.y.clone() - hitbox.height.clone()) + self.offset_y.clone() - 1.5},
                        },
                        0.0,
                        Stroke{width: 3.0, color: Color32::GRAY},
                    );
                }
                else {
                    painter.rect_stroke(
                        Rect {
                            min: Pos2{x: -(hitbox.x.clone() + hitbox.width.clone()) + self.offset_x.clone() + 1.5,
                                y: -(hitbox.y.clone() + hitbox.height.clone()) + self.offset_y.clone() + 1.5},
                            max: Pos2{x: -(hitbox.x.clone() - hitbox.width.clone()) + self.offset_x.clone() - 1.5,
                                y: -(hitbox.y.clone() - hitbox.height.clone()) + self.offset_y.clone() - 1.5},
                        },
                        0.0,
                        Stroke{width: 3.0, color: Color32::RED},
                    );
                }
            }
        }

        response
    }
}