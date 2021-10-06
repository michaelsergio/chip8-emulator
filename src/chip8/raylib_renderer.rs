use raylib::prelude::*;

pub fn run() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("CHIP-8 EMULATOR")
        .build();

    while !rl.window_should_close() {
        let coord = rl.get_mouse_position();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);
        d.draw_rectangle(0, 0, 640, 50, Color::GREEN);
        d.draw_circle(coord.x as i32, coord.y as i32, 50.0, Color::BLUE);
        d.draw_text(
            format!("Hello {}, {}", coord.x, coord.y).as_str(),
             12, 12, 20, Color::BLACK);
    }
}
