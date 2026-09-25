use macroquad::prelude::*;
use sysinfo::{Pid, ProcessesToUpdate, System};

pub const DEFAULT_MAP_WIDTH: usize = 20;
pub const DEFAULT_MAP_HEIGHT: usize = 15;
pub const TILE_DISPLAY_SIZE: f32 = 40.0;

pub fn print_mem_mb(system: &mut System, label: &str) {
    let pid = Pid::from_u32(std::process::id());
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]));
    if let Some(process) = system.process(pid) {
        println!(
            "[{}] RAM: {:.2} MB",
            label,
            process.memory() as f64 / 1024.0 / 1024.0
        );
    }
}

pub struct TileInfo {
    pub id: usize,
    pub name: String,
    pub texture: Texture2D,
    pub collision: bool,
}

pub struct MapEditor {
    pub map_width: usize,
    pub map_height: usize,
    pub tiles: Vec<i32>,
    pub selected_tile: i32,
    pub loaded_tiles: Vec<TileInfo>,
    pub camera_offset: Vec2,
    pub zoom: f32,
    pub status_message: String,
}

impl MapEditor {
    pub fn new() -> Self {
        let map_width = DEFAULT_MAP_WIDTH;
        let map_height = DEFAULT_MAP_HEIGHT;
        let tiles = vec![-1; map_width * map_height];

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

    pub fn resize_map(&mut self, new_width: usize, new_height: usize) {
        let mut new_tiles = vec![-1; new_width * new_height];
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

    pub fn load_tiles_from_folder(&mut self) {
        if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
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

    pub fn save_project(&mut self) {
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

    pub fn load_map_from_file(&mut self) {
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

    // 1. Заполнить всю карту выбранным тайлом
    pub fn fill_entire_map(&mut self) {
        if self.selected_tile >= -1 {
            for tile in &mut self.tiles {
                *tile = self.selected_tile;
            }
            self.status_message = format!("Вся карта заполнена тайлом ID: {}", self.selected_tile);
        }
    }

    // 2. Алгоритм заливки области (Flood Fill / Ведро)
    pub fn flood_fill(&mut self, start_x: usize, start_y: usize) {
        let target_tile = self.tiles[start_y * self.map_width + start_x];
        let replacement_tile = self.selected_tile;

        // Если целевой тайл уже равен выбранному, ничего делать не нужно
        if target_tile == replacement_tile {
            return;
        }

        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start_x, start_y));

        while let Some((x, y)) = queue.pop_front() {
            let idx = y * self.map_width + x;
            if self.tiles[idx] == target_tile {
                self.tiles[idx] = replacement_tile;

                // Проверяем соседей (вверх, вниз, влево, вправо)
                if x > 0 {
                    queue.push_back((x - 1, y));
                }
                if x + 1 < self.map_width {
                    queue.push_back((x + 1, y));
                }
                if y > 0 {
                    queue.push_back((x, y - 1));
                }
                if y + 1 < self.map_height {
                    queue.push_back((x, y + 1));
                }
            }
        }
        self.status_message = format!("Заливка области выполнена тайлом ID: {}", replacement_tile);
    }
}
