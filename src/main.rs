use egui_macroquad::egui;
use macroquad::prelude::*;

const DEFAULT_MAP_WIDTH: usize = 20;
const DEFAULT_MAP_HEIGHT: usize = 15;
const TILE_DISPLAY_SIZE: f32 = 32.0;

struct MapEditor {
    map_width: usize,
    map_height: usize,
    tiles: Vec<i32>,
    selected_tile: i32,

    tileset_texture: Option<Texture2D>,
    tileset_image: Option<Image>,
    tile_size_px: usize,
    tiles_per_row: usize,
    total_tiles: usize,

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
            selected_tile: 0,
            tileset_texture: None,
            tileset_image: None,
            tile_size_px: 16,
            tiles_per_row: 1,
            total_tiles: 16,
            camera_offset: Vec2::ZERO,
            zoom: 1.0,
            status_message: "Готово к работе. Загрузите тайлсет или рисуйте тайлами.".to_string(),
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

    fn save_to_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().save_file() {
            let mut content = String::new();
            for y in 0..self.map_height {
                let row_data: Vec<String> = (0..self.map_width)
                    .map(|x| self.tiles[y * self.map_width + x].to_string())
                    .collect();
                content.push_str(&row_data.join(" "));
                content.push('\n');
            }
            if let Err(e) = std::fs::write(&path, content) {
                self.status_message = format!("Ошибка сохранения: {}", e);
            } else {
                self.status_message = format!("Карта сохранена в {:?}", path);
            }
        }
    }

    fn load_from_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
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
                        self.status_message =
                            format!("Карта загружена из {:?} ({}x{})", path, cols, rows);
                    } else {
                        self.status_message =
                            "Ошибка: не удалось распарсить тайлы из файла.".to_string();
                    }
                }
                Err(e) => {
                    self.status_message = format!("Ошибка чтения файла: {}", e);
                }
            }
        }
    }

    fn load_tileset(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg"])
            .pick_file()
        {
            if let Ok(bytes) = std::fs::read(&path) {
                let img = Image::from_file_with_format(&bytes, None);
                let texture = Texture2D::from_image(&img);
                texture.set_filter(FilterMode::Nearest);

                let w = img.width;
                let h = img.height;

                self.tileset_image = Some(img);
                self.tileset_texture = Some(texture);

                if self.tile_size_px > 0 {
                    let cols = w as usize / self.tile_size_px;
                    let rows = h as usize / self.tile_size_px;
                    self.tiles_per_row = cols.max(1);
                    self.total_tiles = cols * rows;
                }
                self.status_message = format!("Тайлсет загружен: {}x{} пикселей", w, h);
            }
        }
    }
}

#[macroquad::main("2D Tilemap Map Editor")]
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
                .default_width(280.0)
                .show(ctx, |ui| {
                    ui.heading("🗺️ Редактор Карт");
                    ui.separator();

                    ui.label(&editor.status_message);
                    ui.separator();

                    ui.collapsing("📁 Файл", |ui| {
                        if ui.button("💾 Сохранить в .txt").clicked() {
                            editor.save_to_file();
                        }
                        if ui.button("📂 Загрузить из .txt").clicked() {
                            editor.load_from_file();
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

                    ui.collapsing("🎨 Тайлсет", |ui| {
                        if ui.button("📂 Загрузить картинку тайлсета").clicked()
                        {
                            editor.load_tileset();
                        }

                        let mut tile_size = editor.tile_size_px as i32;
                        ui.horizontal(|ui| {
                            ui.label("Размер тайла (px):");
                            if ui
                                .add(
                                    egui::DragValue::new(&mut tile_size)
                                        .speed(1.0)
                                        .clamp_range(4..=256),
                                )
                                .changed()
                            {
                                editor.tile_size_px = tile_size.max(4) as usize;
                                if let Some(img) = &editor.tileset_image {
                                    let cols = img.width as usize / editor.tile_size_px;
                                    let rows = img.height as usize / editor.tile_size_px;
                                    editor.tiles_per_row = cols.max(1);
                                    editor.total_tiles = cols * rows;
                                }
                            }
                        });

                        ui.label(format!("Всего тайлов: {}", editor.total_tiles));
                    });

                    ui.separator();
                    ui.heading("📦 Палитра тайлов");
                    ui.label("ЛКМ - рисовать, ПКМ - стереть (-1)");
                    ui.label("Зажатое колёсико или Пробел — двигать карту");

                    if ui
                        .selectable_label(editor.selected_tile == -1, "🧹 Пусто (-1)")
                        .clicked()
                    {
                        editor.selected_tile = -1;
                    }

                    egui::ScrollArea::vertical()
                        .max_height(350.0)
                        .show(ui, |ui| {
                            let available_width = ui.available_width();
                            let item_size = 40.0;
                            let items_per_row =
                                (available_width / (item_size + 6.0)).floor().max(1.0) as usize;

                            let total = editor.total_tiles;
                            let mut current_id = 0;

                            while current_id < total {
                                ui.horizontal(|ui| {
                                    let row_end = (current_id + items_per_row).min(total);
                                    for id in current_id..row_end {
                                        let is_selected = editor.selected_tile == id as i32;
                                        let btn_text = format!("#{}", id);
                                        if ui.selectable_label(is_selected, btn_text).clicked() {
                                            editor.selected_tile = id as i32;
                                        }
                                    }
                                    current_id = row_end;
                                });
                            }
                        });
                });
        });

        // Обработка панорамирования холста
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

        // Рендеринг сетки карты и тайлов
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
                if let Some(tex) = &editor.tileset_texture {
                    if tile_id >= 0 && (tile_id as usize) < editor.total_tiles {
                        let cols = editor.tiles_per_row as i32;
                        if cols > 0 {
                            let tx = (tile_id % cols) as f32 * editor.tile_size_px as f32;
                            let ty = (tile_id / cols) as f32 * editor.tile_size_px as f32;
                            let tw = editor.tile_size_px as f32;
                            let th = editor.tile_size_px as f32;

                            draw_texture_ex(
                                *tex,
                                tile_screen_x,
                                tile_screen_y,
                                WHITE,
                                DrawTextureParams {
                                    dest_size: Some(Vec2::new(size, size)),
                                    source: Some(Rect::new(tx, ty, tw, th)),
                                    ..Default::default()
                                },
                            );
                            rendered = true;
                        }
                    }
                }

                if !rendered {
                    let bg_color = if tile_id == -1 {
                        Color::new(0.2, 0.2, 0.25, 1.0)
                    } else {
                        let hue = ((tile_id * 50) % 360) as f32;
                        Color::from_vec(vec4(
                            (hue / 60.0).sin() * 0.5 + 0.5,
                            ((hue + 120.0) / 60.0).sin() * 0.5 + 0.5,
                            ((hue + 240.0) / 60.0).sin() * 0.5 + 0.5,
                            1.0,
                        ))
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
