use crate::editor::{MapEditor, TILE_DISPLAY_SIZE};
use macroquad::prelude::*;

pub fn render_and_handle_canvas(
    editor: &mut MapEditor,
    current_mouse_pos: (f32, f32),
    mouse_delta: Vec2,
    egui_wants_pointer: bool,
) {
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

    let map_pixel_width = editor.map_width as f32 * TILE_DISPLAY_SIZE * editor.zoom;
    let map_pixel_height = editor.map_height as f32 * TILE_DISPLAY_SIZE * editor.zoom;

    let start_x = editor.camera_offset.x + screen_width() / 2.0 - map_pixel_width / 2.0;
    let start_y = editor.camera_offset.y + screen_height() / 2.0 - map_pixel_height / 2.0;

    let screen_w = screen_width();
    let screen_h = screen_height();

    let mut hovered_tile_pos = None;

    // Собираем видимые клетки (с учётом Frustum Culling) вместо отрисовки сразу
    struct VisibleCell {
        tile_id: i32,
        screen_x: f32,
        screen_y: f32,
        size: f32,
    }
    let mut visible_cells: Vec<VisibleCell> =
        Vec::with_capacity(editor.map_width * editor.map_height);

    for y in 0..editor.map_height {
        for x in 0..editor.map_width {
            let idx = y * editor.map_width + x;
            let tile_id = editor.tiles[idx];

            let tile_screen_x = start_x + x as f32 * TILE_DISPLAY_SIZE * editor.zoom;
            let tile_screen_y = start_y + y as f32 * TILE_DISPLAY_SIZE * editor.zoom;
            let size = TILE_DISPLAY_SIZE * editor.zoom;

            if tile_screen_x + size < 0.0
                || tile_screen_x > screen_w
                || tile_screen_y + size < 0.0
                || tile_screen_y > screen_h
            {
                continue;
            }

            let mx = current_mouse_pos.0;
            let my = current_mouse_pos.1;
            if !egui_wants_pointer
                && mx >= tile_screen_x
                && mx < tile_screen_x + size
                && my >= tile_screen_y
                && my < tile_screen_y + size
            {
                hovered_tile_pos = Some((tile_id, tile_screen_x, tile_screen_y, size));
            }

            visible_cells.push(VisibleCell {
                tile_id,
                screen_x: tile_screen_x,
                screen_y: tile_screen_y,
                size,
            });
        }
    }

    // Группируем по tile_id — все клетки одной текстуры идут подряд, батч не рвётся между ними
    visible_cells.sort_by_key(|c| c.tile_id);

    for cell in &visible_cells {
        let mut rendered = false;
        if cell.tile_id >= 0 {
            if let Some(tile_info) = editor.loaded_tiles.get(cell.tile_id as usize) {
                draw_texture_ex(
                    tile_info.texture,
                    cell.screen_x,
                    cell.screen_y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(cell.size, cell.size)),
                        ..Default::default()
                    },
                );
                rendered = true;
            }
        }
        if !rendered {
            let bg_color = if cell.tile_id == -1 {
                Color::new(0.2, 0.2, 0.25, 1.0)
            } else {
                Color::new(0.5, 0.2, 0.2, 1.0)
            };
            draw_rectangle(cell.screen_x, cell.screen_y, cell.size, cell.size, bg_color);
        }
    }

    // ПРОХОД 2: Отрисовка сетки отдельно для оптимизации батчинга
    for y in 0..editor.map_height {
        for x in 0..editor.map_width {
            let tile_screen_x = start_x + x as f32 * TILE_DISPLAY_SIZE * editor.zoom;
            let tile_screen_y = start_y + y as f32 * TILE_DISPLAY_SIZE * editor.zoom;
            let size = TILE_DISPLAY_SIZE * editor.zoom;

            if tile_screen_x + size < 0.0
                || tile_screen_x > screen_w
                || tile_screen_y + size < 0.0
                || tile_screen_y > screen_h
            {
                continue;
            }

            draw_rectangle_lines(
                tile_screen_x,
                tile_screen_y,
                size,
                size,
                1.0,
                Color::new(0.3, 0.3, 0.35, 0.3),
            );
        }
    }

    // Подсветка тайла под курсором
    if let Some((tile_id, tx, ty, t_size)) = hovered_tile_pos {
        draw_rectangle_lines(tx, ty, t_size, t_size, 2.0, YELLOW);
        if tile_id != -1 {
            let info_text = format!("ID: {}", tile_id);
            draw_text(
                &info_text,
                tx + 4.0,
                ty + t_size - 6.0,
                (t_size * 0.4).clamp(12.0, 24.0),
                YELLOW,
            );
        }
    }

    // Рисование и стирание мышкой
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
                        if editor.tiles[idx] != editor.selected_tile {
                            editor.tiles[idx] = editor.selected_tile;
                        }
                    }
                    if is_mouse_button_down(MouseButton::Right) {
                        editor.tiles[idx] = -1;
                    }
                }
            }
        }
    }
}
