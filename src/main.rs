use egui_macroquad::egui;
use macroquad::prelude::*;

const DEFAULT_MAP_WIDTH: usize = 20;
const DEFAULT_MAP_HEIGHT: usize = 15;
const TILE_DISPLAY_SIZE: f32 = 40.0;

struct TileInfo {
    id: usize,
    name: String,
    texture: Texture2D,
    collision: bool,
}

struct MapEditor {
    map_width: usize,
    map_height: usize,
    tiles: Vec<i32>,
    selected_tile: i32,

    loaded_tiles: Vec<TileInfo>,

    camera_offset: Vec2,
    zoom: f32,
    status_message: String,
}

impl MapEditor {
    fn new() -> Self {
        let map_width = DEFAULT_MAP_WIDTH;
        let map_height = DEFAULT_MAP_HEIGHT;
        let tiles = vec![0; map_width * map_height];

        Self {
            map_width,
            map_height,
            tiles,
            selected_tile: -1,
            loaded_tiles: Vec::new(),
            camera_offset: Vec2::ZERO,
            zoom: 1.0,
            status_message: "Загрузите папку с тайлами (.png) для начала работы.".to_string(),
        }
    }

    fn resize_map(&mut self, new_width: usize, new_height: usize) {
        let mut new_tiles = vec![0; new_width * new_height];
        for y in 0..new_height.min(self.map_height) {
            for x in 0..new_width.min(self.map_width) {
                let old_idx = y * self.map_width + x;
                let new_idx = y * new_width + x;
                new_tiles[new_idx] = self.tiles[old_idx];
            }
        }
        self.map_width = new_width;
        self.map_height = new_height;
        self.tiles = new_tiles;
        self.status_message = format!("Карта изменена: {}x{}", new_width, new_height);
    }

    // Загрузка папки с тайлами
    fn load_tiles_from_folder(&mut self) {
        if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
            // Очищаем старые тайлы, чтобы освободить память
            self.loaded_tiles.clear();
            let mut new_tiles = Vec::new();

            if let Ok(entries) = std::fs::read_dir(&folder_path) {
                let mut paths: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| {
                        p.extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext.eq_ignore_ascii_case("png"))
                            .unwrap_or(false)
                    })
                    .collect();

                paths.sort();

                for (id, path) in paths.into_iter().enumerate() {
                    if let Ok(bytes) = std::fs::read(&path) {
                        let img = Image::from_file_with_format(&bytes, None);
                        println!(
                            "Загружен тайл: {:?} (размер: {}x{})",
                            path.file_name(),
                            img.width,
                            img.height
                        );

                        let texture = Texture2D::from_image(&img);
                        texture.set_filter(FilterMode::Nearest);

                        let file_stem = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("tile")
                            .to_string();

                        new_tiles.push(TileInfo {
                            id,
                            name: file_stem,
                            texture,
                            collision: false,
                        });
                    }
                }
            }

            if !new_tiles.is_empty() {
                let count = new_tiles.len();
                self.loaded_tiles = new_tiles;
                self.selected_tile = 0;
                self.status_message = format!("Загружено тайлов из папки: {}", count);
            } else {
                self.status_message = "В выбранной папке не найдено .png файлов.".to_string();
            }
        }
    }

    // Сохранение карты (чистый .txt только с индексами) и .tileset файла
    fn save_project(&mut self) {
        // 1. Сохраняем карту в чистом формате (индексы через пробел, построчно)
        if let Some(map_path) = rfd::FileDialog::new().set_file_name("map.txt").save_file() {
            let mut map_content = String::new();
            for y in 0..self.map_height {
                let row_data: Vec<String> = (0..self.map_width)
                    .map(|x| self.tiles[y * self.map_width + x].to_string())
                    .collect();
                map_content.push_str(&row_data.join(" "));
                map_content.push('\n');
            }
            let _ = std::fs::write(&map_path, map_content);
        }

        // 2. Сохраняем конфигурацию тайлсета (.tileset): [индекс] [имя] [true/false]
        if let Some(tileset_path) = rfd::FileDialog::new()
            .set_file_name("tileset.tileset")
            .save_file()
        {
            let mut ts_content = String::new();
            for tile in &self.loaded_tiles {
                ts_content.push_str(&format!("{} {} {}\n", tile.id, tile.name, tile.collision));
            }
            if let Err(e) = std::fs::write(&tileset_path, ts_content) {
                self.status_message = format!("Ошибка сохранения .tileset: {}", e);
            } else {
                self.status_message = "Карта и .tileset файл успешно сохранены!".to_string();
            }
        }
    }

    fn load_map_from_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let mut new_tiles = Vec::new();
                let mut rows = 0;
                let mut cols = 0;

                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    let row_vals: Vec<i32> = line
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if !row_vals.is_empty() {
                        cols = row_vals.len();
                        new_tiles.extend(row_vals);
                        rows += 1;
                    }
                }

                if rows > 0 && cols > 0 {
                    self.map_width = cols;
                    self.map_height = rows;
                    self.tiles = new_tiles;
                    self.status_message = format!("Карта загружена из файла ({}x{})", cols, rows);
                } else {
                    self.status_message = "Ошибка: не удалось распарсить файл карты.".to_string();
                }
            }
        }
    }
}

#[macroquad::main("2D Custom Tilemap Editor")]
async fn main() {
    let mut editor = MapEditor::new();
    let mut last_mouse_pos = mouse_position();

    loop {
        clear_background(Color::new(0.15, 0.15, 0.18, 1.0));

        let current_mouse_pos = mouse_position();
        let mouse_delta = Vec2::new(
            current_mouse_pos.0 - last_mouse_pos.0,
            current_mouse_pos.1 - last_mouse_pos.1,
        );
        last_mouse_pos = current_mouse_pos;

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

                    ui.separator();
                    ui.heading("📦 Палитра тайлов и Коллизии");
                    ui.label("Кликните на тайл для выбора. Настройте коллизию.");

                    if ui
                        .selectable_label(editor.selected_tile == -1, "🧹 Ластик")
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

        // Навигация (панорамирование и зум)
        if !egui_wants_pointer {
            if is_mouse_button_down(MouseButton::Middle) || is_key_down(KeyCode::Space) {
                editor.camera_offset += mouse_delta;
            }

            let wheel = mouse_wheel().1;
            if wheel != 0.0 {
                if wheel > 0.0 {
                    editor.zoom *= 1.15;
                } else {
                    editor.zoom /= 1.15;
                }
                editor.zoom = editor.zoom.clamp(0.1, 10.0);
            }
        }

        // Рендеринг холста карты
        let map_pixel_width = editor.map_width as f32 * TILE_DISPLAY_SIZE * editor.zoom;
        let map_pixel_height = editor.map_height as f32 * TILE_DISPLAY_SIZE * editor.zoom;

        let start_x = editor.camera_offset.x + screen_width() / 2.0 - map_pixel_width / 2.0;
        let start_y = editor.camera_offset.y + screen_height() / 2.0 - map_pixel_height / 2.0;

        for y in 0..editor.map_height {
            for x in 0..editor.map_width {
                let idx = y * editor.map_width + x;
                let tile_id = editor.tiles[idx];

                let tile_screen_x = start_x + x as f32 * TILE_DISPLAY_SIZE * editor.zoom;
                let tile_screen_y = start_y + y as f32 * TILE_DISPLAY_SIZE * editor.zoom;
                let size = TILE_DISPLAY_SIZE * editor.zoom;

                let mut rendered = false;
                if tile_id >= 0 {
                    if let Some(tile_info) =
                        editor.loaded_tiles.iter().find(|t| t.id as i32 == tile_id)
                    {
                        draw_texture_ex(
                            tile_info.texture,
                            tile_screen_x,
                            tile_screen_y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(Vec2::new(size, size)),
                                ..Default::default()
                            },
                        );
                        rendered = true;
                    }
                }

                if !rendered {
                    let bg_color = if tile_id == -1 {
                        Color::new(0.2, 0.2, 0.25, 1.0)
                    } else {
                        Color::new(0.5, 0.2, 0.2, 1.0)
                    };

                    draw_rectangle(tile_screen_x, tile_screen_y, size, size, bg_color);
                    if tile_id != -1 {
                        draw_text(
                            &tile_id.to_string(),
                            tile_screen_x + 4.0,
                            tile_screen_y + size - 6.0,
                            size * 0.4,
                            WHITE,
                        );
                    }
                }

                draw_rectangle_lines(
                    tile_screen_x,
                    tile_screen_y,
                    size,
                    size,
                    1.0,
                    Color::new(0.3, 0.3, 0.35, 0.5),
                );
            }
        }

        // Рисование / стирание мышкой
        if !egui_wants_pointer {
            let m_x = current_mouse_pos.0;
            let m_y = current_mouse_pos.1;

            if m_x >= start_x && m_y >= start_y {
                let local_x = (m_x - start_x) / (TILE_DISPLAY_SIZE * editor.zoom);
                let local_y = (m_y - start_y) / (TILE_DISPLAY_SIZE * editor.zoom);

                if local_x >= 0.0 && local_y >= 0.0 {
                    let grid_x = local_x as usize;
                    let grid_y = local_y as usize;

                    if grid_x < editor.map_width && grid_y < editor.map_height {
                        let idx = grid_y * editor.map_width + grid_x;

                        if is_mouse_button_down(MouseButton::Left) {
                            editor.tiles[idx] = editor.selected_tile;
                        }
                        if is_mouse_button_down(MouseButton::Right) {
                            editor.tiles[idx] = -1;
                        }
                    }
                }
            }
        }

        egui_macroquad::draw();

        next_frame().await;
    }
}
