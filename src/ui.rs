use crate::editor::MapEditor;
use egui_macroquad::egui;

pub fn draw_ui(editor: &mut MapEditor) -> bool {
    let mut egui_wants_pointer = false;

    egui_macroquad::ui(|ctx| {
        egui_wants_pointer = ctx.wants_pointer_input();

        egui::SidePanel::left("control_panel")
            .default_width(340.0)
            .show(ctx, |ui| {
                ui.heading("🗺️ Редактор Карт");
                ui.separator();

                ui.label(&editor.status_message);
                ui.separator();

                ui.collapsing("📁 Проект", |ui| {
                    if ui.button("💾 Сохранить карту и .tileset").clicked() {
                        editor.save_project();
                    }
                    if ui.button("📂 Загрузить карту (.txt)").clicked() {
                        editor.load_map_from_file();
                    }
                });

                ui.collapsing("⚙️ Параметры карты", |ui| {
                    let mut w = editor.map_width as i32;
                    let mut h = editor.map_height as i32;

                    ui.horizontal(|ui| {
                        ui.label("Ширина:");
                        ui.add(egui::DragValue::new(&mut w).speed(1.0).clamp_range(1..=500));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Высота:");
                        ui.add(egui::DragValue::new(&mut h).speed(1.0).clamp_range(1..=500));
                    });

                    if ui.button("Применить размер").clicked() {
                        editor.resize_map(w.max(1) as usize, h.max(1) as usize);
                    }
                });

                ui.collapsing("🎨 Загрузка тайлов", |ui| {
                    if ui.button("📂 Выбрать папку с .png тайлами").clicked()
                    {
                        editor.load_tiles_from_folder();
                    }
                    ui.label(format!("Загружено тайлов: {}", editor.loaded_tiles.len()));
                });

                ui.collapsing("🛠️ Инструменты", |ui| {
                    if ui.button("🌊 Залить всю карту выбранным тайлом").clicked()
                    {
                        editor.fill_entire_map();
                    }
                });

                ui.separator();
                ui.heading("📦 Палитра тайлов и Коллизии");
                ui.label("Кликните на тайл для выбора. Настройте коллизию.");

                if ui
                    .selectable_label(editor.selected_tile == -1, "🧹 Ластик (Пусто: -1)")
                    .clicked()
                {
                    editor.selected_tile = -1;
                }

                egui::ScrollArea::vertical()
                    .max_height(350.0)
                    .show(ui, |ui| {
                        for tile in &mut editor.loaded_tiles {
                            let is_selected = editor.selected_tile == tile.id as i32;

                            ui.horizontal(|ui| {
                                let label_text = format!("#{} [{}]", tile.id, tile.name);
                                if ui.selectable_label(is_selected, label_text).clicked() {
                                    editor.selected_tile = tile.id as i32;
                                }

                                ui.checkbox(&mut tile.collision, "Коллизия");
                            });
                        }
                    });
            });
    });

    egui_wants_pointer
}
