use minifb::{Key, Window, WindowOptions};

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

pub struct Window {
    window :
}
pub fn create_window() -> Window {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });
    return window;
}
