#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

mod core;
mod memory;
mod registers;
// mod syscalls;

use core::Core;

use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};

use std::io::prelude::*;
use std::io::stdout;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

#[rustfmt::skip] // Key numbers lined up nicely
static KEYMAP: [(Scancode, u32); 14] = [
    (Scancode::Escape,    0),
    (Scancode::W,         1),
    (Scancode::A,         2),
    (Scancode::S,         3),
    (Scancode::D,         4),
    (Scancode::Up,        5),
    (Scancode::Left,      6),
    (Scancode::Down,      7),
    (Scancode::Right,     8),
    (Scancode::Space,     9),
    (Scancode::Return,   10),
    (Scancode::KpEnter,  10),
    (Scancode::LShift,   11),
    (Scancode::RShift,   11),
];

fn vecu8_to_vecu32(bytes: Vec<u8>) -> Vec<u32> {
    let mut words: Vec<u32> = Vec::new();
    let mut count = 0;
    let mut val: u32 = 0;

    for byte in bytes {
        val |= (byte as u32) << (8 * count);
        count += 1;
        if count == 4 {
            words.push(val);
            val = 0;
            count = 0;
        }
    }
    if count > 0 {
        words.push(val);
    }

    words
}

fn read_binary_file(path: String) -> Vec<u32> {
    vecu8_to_vecu32(std::fs::read(path).unwrap())
}

pub fn main() {
    let (text_bytes, data_bytes) = mips_assembler::assemble("test.asm");
    let text = vecu8_to_vecu32(text_bytes);
    let mut data = vecu8_to_vecu32(data_bytes);

    // let text = read_binary_file("test.bin".into());
    // let mut data = read_binary_file("test.data".into());

    // for (i, word) in (&text).iter().enumerate() {
    // println!("{i}: {:#04X}", word);
    // }
    // let test_program = read_binary_file("test.bin".into());
    // let test_data = read_binary_file("test.data".into());
    // let test_program: Vec<u32> = vec![
    //     0x24020005, 0x3c011001, 0x34240003, 0x0000000c, 0x24020020, 0x3c017777, 0x342477ff,
    //     0x0000000c, 0x24020022, 0x3c01ff00, 0x342400ff, 0x3c010040, 0x34250040, 0x3c010080,
    //     0x34260080, 0x0000000c, 0x24020001, 0x0000000c, 0x24020002, 0x0000000c, 0x08100004,
    // ];
    // let test_data: Vec<u32> = vec![0x6e756f42, 0x53207963, 0x72617571, 0x00002165];
    // let mut data = test_data.clone();

    let mut core = Core::new_mips_default();
    core.load_text(text.clone());
    core.load_data(data.clone());

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("SuperMIPS", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let ev = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let framerate = std::time::Duration::from_secs_f32(1.0 / 30.0);
    let mut last_frame = std::time::Instant::now();
    // let mut running = true;
    let mut keys_down: u32 = 0;
    let mut keys_up: u32 = 0;
    let mut keys_pressed: u32 = 0;
    let mut data_generation: u32 = 0;

    let mut rng = thread_rng();

    'running: loop {
        // i = (i + 1) % 255;
        // canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        // canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::SPACE),
                    ..
                } => canvas.window_mut().set_title("It worked!").unwrap(),
                _ => {}
            }
        }

        if data_generation != core.memory.data_generation {
            data_generation = core.memory.data_generation;
            data = core.memory.data.clone();
        }
        // The rest of the game loop goes here...
        let handle_syscall = |_inst: u32, regs: [u32; 32]| -> [u32; 32] {
            let mut new_regs = regs.clone();

            let v0 = regs[2];
            match v0 {
                0x00 => ev.push_event(Event::Quit { timestamp: 0 }).unwrap(),
                0x01 => canvas.present(),
                0x02 => {
                    let keys_pressed_tmp = get_keys_pressed(&event_pump);
                    keys_down = (keys_pressed ^ keys_pressed_tmp) & keys_pressed_tmp;
                    keys_up = (keys_pressed ^ keys_pressed_tmp) & keys_pressed;
                    keys_pressed = keys_pressed_tmp;

                    // println!("Keys down: {keys_down}; Keys up: {keys_up}; Keys pressed: {keys_pressed}");

                    let cur_time = std::time::Instant::now();
                    let delta_t = cur_time.duration_since(last_frame);
                    last_frame = cur_time;

                    spin_sleep::sleep(framerate.saturating_sub(delta_t));
                    // std::thread::sleep(framerate - delta_t);
                }
                0x03 => {
                    let msg = get_string_at_address(&data, regs[4]);
                    print!("{msg}");
                    stdout().flush().unwrap();
                }
                0x04 => match regs[5] {
                    0 => print!("{}", regs[4]),
                    1 => print!("{}", regs[4]),
                    2 => print!("{:#04X}", regs[4]),
                    3 => print!("{}", (regs[4] & 0xFF) as u8 as char),
                    _ => {}
                },
                0x05 => {
                    let title = get_string_at_address(&data, regs[4]);
                    canvas.window_mut().set_title(title.as_str()).unwrap();
                }
                0x08 => todo!(),
                0x09 => {
                    rng = thread_rng();
                }
                0x0A => {
                    new_regs[2] = rng.next_u32();
                }
                0x0B => todo!(),
                0x10 => {
                    new_regs[2] = keys_down;
                }
                0x11 => {
                    new_regs[2] = keys_up;
                }
                0x12 => {
                    new_regs[2] = keys_pressed;
                }
                0x20 => {
                    // syscalls::fill_screen(pixels.frame_mut(), regs[4]);
                    canvas.set_draw_color(u32_to_color(regs[4]));
                    canvas.clear();
                }
                0x21 => todo!(),
                0x22 => {
                    // syscalls::draw_rect(pixels.frame_mut(), regs[4], regs[5], regs[6]);
                    canvas.set_draw_color(u32_to_color(regs[4]));
                    canvas.fill_rect(u32s_to_rect(regs[5], regs[6])).unwrap();
                }
                0x23 => todo!(),
                0x24 => todo!(),
                0x30 => todo!(),
                0x31 => todo!(),
                0x32 => todo!(),
                0x33 => todo!(),

                _ => {
                    panic!("Unrecognized syscall {v0:#04X}")
                }
            }

            new_regs
        };

        // println!("Keys held: {}", syscalls::get_keys_held(&input));
        // println!("PC: {}", core.pc);

        core.tick(handle_syscall);

        // canvas.present();
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn u32_to_color(color: u32) -> Color {
    let r = ((color & 0xFF000000) >> 24) as u8;
    let g = ((color & 0x00FF0000) >> 16) as u8;
    let b = ((color & 0x0000FF00) >> 8) as u8;
    let a = ((color & 0x000000FF) >> 0) as u8;

    Color::RGBA(r, g, b, a)
}

fn u32_to_point(point: u32) -> Point {
    let x = (point & 0xFFFF0000) >> 16;
    let y = point & 0x0000FFFF;

    Point::new(x as i32, y as i32)
}

fn u32s_to_rect(upper_left: u32, lower_right: u32) -> Rect {
    let start_x = (upper_left & 0xFFFF0000) >> 16;
    let start_y = upper_left & 0x0000FFFF;

    let end_x = (lower_right & 0xFFFF0000) >> 16;
    let end_y = lower_right & 0x0000FFFF;

    Rect::new(
        start_x as i32,
        start_y as i32,
        end_x - start_x,
        end_y - start_y,
    )
}

fn get_keys_pressed(e: &sdl2::EventPump) -> u32 {
    let mut val: u32 = 0;
    let keyboard_state = e.keyboard_state();
    for (scancode, shift) in KEYMAP {
        if keyboard_state.is_scancode_pressed(scancode) {
            val |= 1 << shift;
        }
    }

    val
}

fn get_string_at_address(data: &Vec<u32>, address: u32) -> String {
    let mut index = (address - 0x10010000) / 4;
    let mut offset = address % 4;

    let mut bytes: Vec<u8> = Vec::new();

    let mut cur_word = data.get(index as usize).unwrap();
    let mut cur_byte;

    loop {
        cur_byte = ((cur_word >> (8 * offset)) & 0xFF) as u8;
        if cur_byte == 0x00 {
            break;
        }
        bytes.push(cur_byte);

        offset += 1;
        if offset >= 4 {
            offset = 0;
            index += 1;
            cur_word = data.get(index as usize).unwrap();
        }
    }

    String::from_utf8(bytes).unwrap()
}

// /// Representation of the application state. In this example, a box will bounce around the screen.
// struct World {
//     box_x: i16,
//     box_y: i16,
//     velocity_x: i16,
//     velocity_y: i16,
// }

// fn main() -> Result<(), Error> {
//     let test_program: Vec<u32> = vec![
//         0x24020020, 0x2404ffff, 0x0000000c, 0x24020022, 0x3c01ff00, 0x342400ff, 0x3c010040,
//         0x34250400, 0x3c010080, 0x34260080, 0x0000000c, 0x24020001, 0x0000000c, 0x24020002,
//         0x0000000c, 0x08100000,
//     ];

//     env_logger::init();
//     let event_loop = EventLoop::new().unwrap();
//     let mut input = WinitInputHelper::new();
//     let window = {
//         let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
//         WindowBuilder::new()
//             .with_title("Hello Pixels")
//             .with_inner_size(size)
//             .with_min_inner_size(size)
//             .with_resizable(false)
//             .build(&event_loop)
//             .unwrap()
//     };

//     let mut pixels = {
//         let window_size = window.inner_size();
//         let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
//         Pixels::new(WIDTH, HEIGHT, surface_texture)?
//     };

//     // let mut world = World::new();

//     let mut core = Core::new_mips_default();
//     core.load_text(test_program);

//     let mut running = true;

//     let framerate = std::time::Duration::from_secs_f32(1.0 / 30.0);
//     let mut last_frame = std::time::Instant::now();
//     loop {
//         let handle_syscall = |_inst: u32, regs: [u32; 32]| -> [u32; 32] {
//             let mut new_regs = regs.clone();

//             let v0 = regs[2];
//             match v0 {
//                 0x00 => running = false,
//                 0x01 => {
//                     if let Err(err) = pixels.render() {
//                         log_error("pixels.render", err);
//                         running = false;
//                         return [0; 32];
//                     }
//                 }
//                 0x02 => {
//                     let cur_time = std::time::Instant::now();
//                     let delta_t = cur_time.duration_since(last_frame);
//                     last_frame = cur_time;

//                     // std::thread::sleep(framerate - delta_t);
//                 }
//                 0x10 => {
//                     new_regs[2] = syscalls::get_keys_pressed(&input);
//                 }
//                 0x11 => {
//                     new_regs[2] = syscalls::get_keys_held(&input);
//                 }
//                 0x20 => {
//                     syscalls::fill_screen(pixels.frame_mut(), regs[4]);
//                 }
//                 0x21 => todo!(),
//                 0x22 => {
//                     syscalls::draw_rect(pixels.frame_mut(), regs[4], regs[5], regs[6]);
//                 }
//                 0x23 => todo!(),
//                 0x24 => todo!(),
//                 0x30 => todo!(),
//                 0x31 => todo!(),
//                 0x32 => todo!(),
//                 0x33 => todo!(),

//                 _ => {
//                     panic!("Unrecognized syscall {v0:#04X}")
//                 }
//             }

//             new_regs
//         };

//         // println!("Keys held: {}", syscalls::get_keys_held(&input));

//         core.tick(handle_syscall);

//         // std::thread::sleep(framerate);
//     }

//     // Draw the current frame
//     // if let Event::WindowEvent { ref event, .. } = event {
//     //     match event {
//     //         WindowEvent::RedrawRequested => {
//     //             // world.draw(pixels.frame_mut());
//     //             if let Err(err) = pixels.render() {
//     //                 log_error("pixels.render", err);
//     //                 win.exit();
//     //                 return;
//     //             }
//     //         }
//     //         _ => {}
//     //     }
//     // }

//     // // Handle input events
//     // if input.update(&event) {
//     //     // Close events
//     //     if input.close_requested() {
//     //         win.exit();
//     //         return;
//     //     }

//     //     // Resize the window
//     //     if let Some(size) = input.window_resized() {
//     //         if let Err(err) = pixels.resize_surface(size.width, size.height) {
//     //             log_error("pixels.resize_surface", err);
//     //             win.exit();
//     //             return;
//     //         }
//     //     }

//     // Update internal state and request a redraw
//     // world.update();
//     // window.request_redraw();
//     // }
//     Ok(())
// }

// fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
//     error!("{method_name}() failed: {err}");
//     for source in err.sources().skip(1) {
//         error!("  Caused by: {source}");
//     }
// }

// impl World {
//     /// Create a new `World` instance that can draw a moving box.
//     fn new() -> Self {
//         Self {
//             box_x: 24,
//             box_y: 16,
//             velocity_x: 1,
//             velocity_y: 1,
//         }
//     }

//     /// Update the `World` internal state; bounce the box around the screen.
//     fn update(&mut self) {
//         if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
//             self.velocity_x *= -1;
//         }
//         if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
//             self.velocity_y *= -1;
//         }

//         self.box_x += self.velocity_x;
//         self.box_y += self.velocity_y;
//     }

//     /// Draw the `World` state to the frame buffer.
//     ///
//     /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
//     fn draw(&self, frame: &mut [u8]) {
//         for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
//             let x = (i % WIDTH as usize) as i16;
//             let y = (i / WIDTH as usize) as i16;

//             let inside_the_box = x >= self.box_x
//                 && x < self.box_x + BOX_SIZE
//                 && y >= self.box_y
//                 && y < self.box_y + BOX_SIZE;

//             let rgba = if inside_the_box {
//                 [0x5e, 0x48, 0xe8, 0xff]
//             } else {
//                 [0x48, 0xb2, 0xe8, 0xff]
//             };

//             pixel.copy_from_slice(&rgba);
//         }
//     }
// }
