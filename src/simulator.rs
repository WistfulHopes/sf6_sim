use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use bitvec::order::Lsb0;
use bitvec::view::BitView;
use eframe::egui;
use eframe::egui::{Color32, ComboBox, Frame, Sense, Slider};
use eframe::emath::{Pos2, Rect};
use eframe::epaint::Stroke;
use sf6_rsz_parser::fchar::{CharacterAsset, DataId};
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

#[derive(Default, Ord, PartialOrd, Eq, PartialEq)]
struct Trigger {
    action: i32,
    condition_flag: u32,
}

pub struct Viewer {
    pub asset: Option<CharacterAsset>,
    selected_index: i32,
    action_index: i32,
    current_frame: usize,
    push_collision_keys: Vec<PushCollisionKey>,
    damage_collision_keys: Vec<DamageCollisionKey>,
    attack_collision_keys: Vec<AttackCollisionKey>,
    triggers: Vec<Trigger>,
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
            triggers: vec![],
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
                self.get_trigger_keys();
                self.should_update = false;
            }
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
                            self.should_update = true;
                        }
                    }
                    None => ()
                }
            }).response;
            egui::ScrollArea::vertical()
                .show(ui, |ui| {
                Frame::canvas(ui.style()).show(ui, |ui| {
                    self.render_boxes(ui)
                });
                ui.label("Cancel list:");
                for trigger in &self.triggers {
                    ui.horizontal(|ui| {
                        ui.label(format!("Action {}", trigger.action));
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
                            cancel_flags.push_str("TERMINATOR | ")
                        }
                        if cancel_flags.len() > 3 {
                            cancel_flags = cancel_flags[0..cancel_flags.len() - 3].to_owned();
                        }
                        ui.label(format!("Cancel flags: {}", cancel_flags));
                    });
                }
            });
            ui.horizontal(|ui| {
                ui.label("");
            }).response
        }
        else {
            ui.horizontal(|ui| {
                ui.label("Select an action from the action list!");
            }).response
        }
    }
    
    fn get_triggers(&mut self, group: i32, condition_flag: u32)
    {
        match &self.asset {
            Some(fchar) => {
                let mut data_index: usize = 0;
                for (n, data_id) in fchar.data_id_table.iter().enumerate() {
                    match data_id {
                        DataId::TriggerGroup => data_index = n,
                        _ => ()
                    }
                }
                let mut index: usize = 0;
                for (n, value) in fchar.data_list_table[data_index].data_ids.iter().enumerate()
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
                                _ => ()
                            }
                        }
                    }
                    _ => ()
                }
                for (trigger_index, select_trigger) in triggers.iter().enumerate() {
                    let bits = select_trigger.view_bits::<Lsb0>();
                    for (bit_index, bit) in bits.iter().enumerate() {
                        if bit == false {
                            continue;
                        }
                        let mut data_index: usize = 0;
                        for (n, data_id) in fchar.data_id_table.iter().enumerate() {
                            match data_id {
                                DataId::Trigger => data_index = n,
                                _ => ()
                            }
                        }
                        let mut index: usize = 0;
                        for (n, value) in fchar.data_list_table[data_index].data_ids.iter().enumerate()
                        {
                            if value.clone() == (bit_index + trigger_index * 64) as u32 {
                                index = n  + 1;
                            }
                        }
                        let mut stored_trigger: Trigger = Default::default();
                        stored_trigger.condition_flag = condition_flag;
                        let trigger = &fchar.data_list_table[data_index].data_rsz.data[index * 17 - 1];
                        match &trigger.fields[5].value
                        {
                            RSZValue::Int32(action) => {
                                stored_trigger.action = action.clone();
                            }
                            _ => ()
                        }
                        self.triggers.push(stored_trigger);
                    }
                }
            }
            None => ()
        }
    }
    
    fn get_trigger_keys(&mut self)
    {
        self.triggers.clear();
        let mut groups: Vec<i32> = vec![];
        let mut condition_flags: Vec<u32> = vec![];
        match &self.asset {
            Some(fchar) => {
                let action = &fchar.action_list[self.selected_index.clone() as usize];
                for object in &action.objects {
                    for (index, object_index) in object.action.object_table.iter().enumerate() {
                        if object.info.object_data.key_data[index].key_start_frame <= self.current_frame as i32
                            && object.info.object_data.key_data[index].key_end_frame > self.current_frame as i32 {
                            let data = &object.action.data[object_index.clone() as usize - 1];
                            match data.name.as_str() {
                                "CharacterAsset.TriggerKey" => {
                                    let group = &data.fields[0].value;
                                    match group {
                                        RSZValue::Int32(group) => {
                                            groups.push(group.clone());
                                        }
                                        _ => ()
                                    }
                                    let condition_flag = &data.fields[1].value;
                                    match condition_flag {
                                        RSZValue::UInt32(condition_flag) => {
                                            condition_flags.push(condition_flag.clone());
                                        }
                                        _ => ()
                                    }
                                }
                                _ => ()
                            }
                        }
                    }
                }
            }
            None => ()
        }
        for (index, group) in groups.iter().enumerate() {
            self.get_triggers(group.clone(), condition_flags[index]);
        }
        self.triggers.sort_unstable();
        self.triggers.dedup();
    }
    
    fn index_to_box(&self, fchar: &CharacterAsset, int: i32, data_type: DataId, boxes: &mut Vec<CollisionBox>)
    {
        let mut data_index: usize = 0;
        for (n, data_id) in fchar.data_id_table.iter().enumerate() {
            if data_id.clone() == data_type {
                data_index = n;
            }
        }
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
                                            self.index_to_box(&fchar, int.clone(), DataId::ThrowHurtBox, &mut boxes);
                                        }
                                        _ => ()
                                    }
                                    
                                    if boxes.len() > 0
                                    {
                                        pushbox = boxes[0].clone();
                                    }
                                    
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
                                                self.index_to_box(&fchar, int.clone(), DataId::HurtBox, &mut boxes);
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
                                                self.index_to_box(&fchar, int.clone(), DataId::HurtBox, &mut boxes);
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
                                                self.index_to_box(&fchar, int.clone(), DataId::HurtBox, &mut boxes);
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

                                    let mut box_list: &Vec<RSZValue> = &vec![];
                                    match &data.fields[11].value {
                                        RSZValue::List(list) => box_list = list,
                                        _ => ()
                                    }
                                    for index in box_list {
                                        match index {
                                            RSZValue::Int32(int) => {
                                                if collision_type == 3 {
                                                    self.index_to_box(&fchar, int.clone(), DataId::ProximityBox, &mut boxes);                                                    
                                                }
                                                else {
                                                    self.index_to_box(&fchar, int.clone(), DataId::StrikeBox, &mut boxes);                                                    
                                                }
                                            }
                                            _ => ()
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
            painter.rect(
                Rect {
                    min: Pos2{x: -(push_collision_key.pushbox.x.clone() + push_collision_key.pushbox.width.clone()) + self.offset_x.clone() + 0.5,
                        y: -(push_collision_key.pushbox.y.clone() + push_collision_key.pushbox.height.clone()) + self.offset_y.clone() + 0.5},
                    max: Pos2{x: -(push_collision_key.pushbox.x.clone() - push_collision_key.pushbox.width.clone()) + self.offset_x.clone() - 0.5,
                        y: -(push_collision_key.pushbox.y.clone() - push_collision_key.pushbox.height.clone()) + self.offset_y.clone() - 0.5},
                },
                0.0,
                egui::Rgba::from_rgba_unmultiplied(0.8,0.8,0.0,0.25),
                Stroke{width: 1.0, color: Color32::YELLOW},
            );
        }
        for damage_collision_key in &self.damage_collision_keys
        {
            for hurtbox in &damage_collision_key.boxes {
                painter.rect(
                    Rect {
                        min: Pos2{x: -(hurtbox.x.clone() + hurtbox.width.clone()) + self.offset_x.clone() + 0.5,
                            y: -(hurtbox.y.clone() + hurtbox.height.clone()) + self.offset_y.clone() + 0.5},
                        max: Pos2{x: -(hurtbox.x.clone() - hurtbox.width.clone()) + self.offset_x.clone() - 0.5,
                            y: -(hurtbox.y.clone() - hurtbox.height.clone()) + self.offset_y.clone() - 0.5},
                    },
                    0.0,
                    egui::Rgba::from_rgba_unmultiplied(0.0,0.8,0.0,0.25),
                    Stroke{width: 1.0, color: Color32::GREEN},
                );
            }
        }
        for attack_collision_key in &self.attack_collision_keys
        {
            for hitbox in &attack_collision_key.boxes {
                if attack_collision_key.collision_type == 3 {
                    painter.rect(
                        Rect {
                            min: Pos2{x: -(hitbox.x.clone() + hitbox.width.clone()) + self.offset_x.clone() + 0.5,
                                y: -(hitbox.y.clone() + hitbox.height.clone()) + self.offset_y.clone() + 0.5},
                            max: Pos2{x: -(hitbox.x.clone() - hitbox.width.clone()) + self.offset_x.clone() - 0.5,
                                y: -(hitbox.y.clone() - hitbox.height.clone()) + self.offset_y.clone() - 0.5},
                        },
                        0.0,
                        egui::Rgba::from_rgba_unmultiplied(0.5,0.5,0.5,0.25),
                        Stroke{width: 1.0, color: Color32::GRAY},
                    );
                }
                else {
                    painter.rect(
                        Rect {
                            min: Pos2{x: -(hitbox.x.clone() + hitbox.width.clone()) + self.offset_x.clone() + 0.5,
                                y: -(hitbox.y.clone() + hitbox.height.clone()) + self.offset_y.clone() + 0.5},
                            max: Pos2{x: -(hitbox.x.clone() - hitbox.width.clone()) + self.offset_x.clone() - 0.5,
                                y: -(hitbox.y.clone() - hitbox.height.clone()) + self.offset_y.clone() - 0.5},
                        },
                        0.0,
                        egui::Rgba::from_rgba_unmultiplied(0.8,0.0,0.0,0.25),
                        Stroke{width: 1.0, color: Color32::RED},
                    )
                }
            }
        }
        response
    }
}