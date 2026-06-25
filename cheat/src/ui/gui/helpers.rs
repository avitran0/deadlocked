use std::hash::Hash;

use egui::{CollapsingHeader, Color32, DragValue, Event, Sense, Ui, Widget};

use crate::cs2::key_codes::KeyCode;

pub fn collapsing_open(ui: &mut Ui, title: &str, add_body: impl FnOnce(&mut Ui)) {
    CollapsingHeader::new(title)
        .default_open(true)
        .show(ui, add_body);
}

pub fn scroll(ui: &mut Ui, id: &str, add_content: impl FnOnce(&mut Ui)) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .id_salt(id)
        .show(ui, add_content);
}

pub fn checkbox(ui: &mut Ui, label: &str, value: &mut bool) -> bool {
    ui.checkbox(value, label).changed()
}

pub fn checkbox_hover(ui: &mut Ui, label: &str, hover_text: &str, value: &mut bool) -> bool {
    ui.checkbox(value, label)
        .on_hover_text(hover_text)
        .changed()
}

pub fn drag(ui: &mut Ui, label: &str, drag: DragValue) -> bool {
    ui.horizontal(|ui| {
        let res = ui.add(drag);
        ui.label(label);
        res
    })
    .inner
    .changed()
}

pub fn combo_box<T: std::fmt::Debug + strum::IntoEnumIterator + PartialEq>(
    ui: &mut Ui,
    id: &str,
    label: &str,
    value: &mut T,
) -> bool {
    let mut changed = false;
    egui::ComboBox::new(id, label)
        .selected_text(format!("{:?}", *value))
        .show_ui(ui, |ui| {
            for mode in T::iter() {
                let text = format!("{:?}", &mode);
                if ui.selectable_value(value, mode, text).clicked() {
                    changed = true;
                }
            }
        });
    changed
}

pub fn color_picker(ui: &mut Ui, label: &str, color: &mut Color32) -> bool {
    let [mut r, mut g, mut b, mut a] = color.to_srgba_unmultiplied();
    let res = ui
        .horizontal(|ui| {
            let (response, painter) =
                ui.allocate_painter(ui.spacing().interact_size, Sense::hover());
            painter.rect_filled(
                response.rect,
                ui.style().visuals.widgets.inactive.corner_radius,
                *color,
            );
            let mut res = ui.add(DragValue::new(&mut r).prefix("r: "));
            res = res.union(ui.add(DragValue::new(&mut g).prefix("g: ")));
            res = res.union(ui.add(DragValue::new(&mut b).prefix("b: ")));
            res = res.union(ui.add(DragValue::new(&mut a).prefix("a: ")));
            ui.label(label);
            res
        })
        .inner;

    let changed = res.changed();
    if changed {
        *color = Color32::from_rgba_premultiplied(r, g, b, a);
    }

    changed
}

pub fn keybind(ui: &mut Ui, id: &str, label: &str, keycode: &mut KeyCode) -> bool {
    ui.horizontal(|ui| {
        let res = ui.add(Keybind::new(keycode, id));
        ui.label(label);
        res
    })
    .inner
    .changed()
}

pub struct Keybind<'gui> {
    keycode: &'gui mut KeyCode,
    id: egui::Id,
}

impl<'gui> Keybind<'gui> {
    pub fn new(keycode: &'gui mut KeyCode, id: impl Hash) -> Self {
        Self {
            keycode,
            id: egui::Id::new(id),
        }
    }
}

impl<'gui> Widget for Keybind<'gui> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let listening_id = ui.make_persistent_id(self.id);

        let mut listening = {
            let ctx = ui.ctx();
            ctx.memory(|mem| mem.data.get_temp::<bool>(listening_id).unwrap_or(false))
        };

        let text = if listening {
            "...".to_string()
        } else {
            format!("{:?}", self.keycode)
        };

        let mut response = ui.button(text);

        if response.clicked() {
            listening = !listening;
        }

        if response.secondary_clicked() {
            listening = false;
        }

        if listening {
            let input = ui.input(|i| {
                for event in &i.events {
                    if let Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        if *key == egui::Key::F35 {
                            return KeyCode::from_egui_modifiers(*modifiers);
                        } else {
                            return KeyCode::from_egui(*key);
                        }
                    }

                    if let Event::PointerButton {
                        button,
                        pressed: true,
                        ..
                    } = event
                    {
                        return Some(KeyCode::from_egui_mouse(*button));
                    }
                }
                None
            });

            if let Some(input) = input {
                if input == KeyCode::Escape {
                    *self.keycode = KeyCode::None;
                    response.mark_changed();
                } else {
                    *self.keycode = input;
                    response.mark_changed();
                }
                listening = false;
            }
        }

        let ctx = ui.ctx();
        ctx.memory_mut(|mem| mem.data.insert_temp(listening_id, listening));

        response
    }
}
