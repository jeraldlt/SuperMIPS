#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

mod errors;
use errors::SuperMipsError;

use mimic_emulator::mips32::assembler;
use mimic_emulator::mips32::core::Core;

use clap::{Parser, Subcommand};
use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use termimad::crossterm::style::{Attribute::*, Color::*};
use termimad::*;

use std::io::prelude::*;
use std::io::stdout;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Assemble a MIPS assembly file and run it
    Run {
        /// A MIPS assembly file that the SuperMIPS emulator will assemble and run.
        file: PathBuf,
    },
    /// Print a document file that details a characteristic of SuperMIPS
    ///
    /// Available docs are: instructions, syscalls, memory
    Docs {
        /// Name of the doc to print
        doc: String,
    },

    /// Run an example game
    ///
    /// Available example games are: bouncy
    Example {
        /// Name of the example game
        example: String,
    },

    /// Print the source of an available example game
    Source {
        /// Name of the example game
        example: String,

        /// Optional file to save the game source to (will print to stdout if this is not included)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Clone)]
enum Docs {
    Instructions,
    SystemCalls,
    Memory,
}

impl Docs {
    fn from_string(input: String) -> Option<Self> {
        match input.to_lowercase().as_str() {
            "instructions" => Some(Docs::Instructions),
            "syscalls" => Some(Docs::SystemCalls),
            "memory" => Some(Docs::Memory),
            _ => None,
        }
    }
}

const HEADER: &str = r"
   ____                  __  __________  ____
  / __/_ _____  ___ ____/  |/  /  _/ _ \/ __/
 _\ \/ // / _ \/ -_) __/ /|_/ // // ___/\ \  
/___/\_,_/ .__/\__/_/ /_/  /_/___/_/  /___/  
        /_/     ";
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

pub fn main() -> Result<(), SuperMipsError> {
    println!("{}Version {}\n", HEADER, env!("CARGO_PKG_VERSION"));
    let args = Args::parse();

    return match args.command {
        Commands::Run { file } => run_from_file(file),
        Commands::Docs { doc } => print_doc(doc),
        Commands::Example { example } => match example.to_lowercase().as_str() {
            "bouncy" => run_from_string(include_str!("../example_games/bouncy.asm").to_owned()),

            _ => Ok(()), // throw error,
        },
        Commands::Source { example, output } => {
            let contents = match example.to_lowercase().as_str() {
                "bouncy" => {
                    include_str!("../example_games/bouncy.asm")
                }

                _ => "", // throw error,
            };

            if let Some(pathbuf) = output {
                std::fs::write(pathbuf, contents).unwrap();
            } else {
                println!("{}", contents);
            }

            Ok(())
        }
    };
}

fn print_doc(doc: String) -> Result<(), SuperMipsError> {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(Yellow);
    // skin.paragraph.set_fgbg(Magenta, rgb(30, 30, 40));
    skin.italic.add_attr(Underlined);

    if let Some(doc) = Docs::from_string(doc) {
        match doc {
            Docs::Instructions => skin.print_text(include_str!("../docs/instructions.md")),
            Docs::SystemCalls => skin.print_text(include_str!("../docs/syscalls.md")),
            Docs::Memory => skin.print_text(include_str!("../docs/memory.md")),
        }
    } else {
        // return error
    }
    Ok(())
}

fn run_from_file(path: PathBuf) -> Result<(), SuperMipsError> {
    let (text_bytes, data_bytes) = assembler::assemble_from_file(path.as_path())?;

    run(text_bytes, data_bytes)
}

fn run_from_string(contents: String) -> Result<(), SuperMipsError> {
    let (text_bytes, data_bytes) = assembler::assemble_from_string(contents)?;

    run(text_bytes, data_bytes)
}

fn run(text_bytes: Vec<u8>, data_bytes: Vec<u8>) -> Result<(), SuperMipsError> {
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

        if let Some(d) = core.clone_data_as_needed(&mut data_generation) {
            data = d;
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

        core.tick(handle_syscall)?;

        // canvas.present();
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
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
