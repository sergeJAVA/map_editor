#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod editor;
mod renderer;
mod ui;

use editor::{MapEditor, print_mem_mb};
use macroquad::prelude::*;
use sysinfo::System;

#[macroquad::main("2D Custom Tilemap Editor")]
async fn main() {
    let mut editor = MapEditor::new();
    let mut last_mouse_pos = mouse_position();
    let mut sys = System::new_all();
    print_mem_mb(&mut sys, "старт");

    loop {
        clear_background(Color::new(0.15, 0.15, 0.18, 1.0));

        let current_mouse_pos = mouse_position();
        let mouse_delta = Vec2::new(
            current_mouse_pos.0 - last_mouse_pos.0,
            current_mouse_pos.1 - last_mouse_pos.1,
        );
        last_mouse_pos = current_mouse_pos;

        // Отрисовка UI и проверка, перехватывает ли egui мышь
        let egui_wants_pointer = ui::draw_ui(&mut editor);

        // Отрисовка холста карты, тайлов и обработка рисования
        renderer::render_and_handle_canvas(
            &mut editor,
            current_mouse_pos,
            mouse_delta,
            egui_wants_pointer,
        );

        // Замер памяти по клавише M
        if is_key_pressed(KeyCode::M) {
            print_mem_mb(&mut sys, "замер");
        }

        egui_macroquad::draw();

        next_frame().await;
    }
}
