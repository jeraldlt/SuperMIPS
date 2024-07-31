use sdl2::keyboard::Keycode;

// pub fn get_keys_pressed(input: &WinitInputHelper) -> u32 {
//     let mut val: u32 = 0;

//     for (keycode, shift) in KEYMAP {
//         if (input.key_pressed(keycode)) {
//             val |= 1 << shift;
//         }
//     }

//     val
// }

// pub fn get_keys_held(input: &WinitInputHelper) -> u32 {
//     let mut val: u32 = 0;

//     for (keycode, shift) in KEYMAP {
//         if (input.key_held(keycode)) {
//             val |= 1 << shift;
//         }
//     }

//     val
// }

pub fn fill_screen(frame: &mut [u8], color: u32) {
    let rgba: [u8; 4] = [
        ((color & 0xFF000000) >> 24) as u8,
        ((color & 0x00FF0000) >> 16) as u8,
        ((color & 0x0000FF00) >> 8) as u8,
        ((color & 0x000000FF) >> 0) as u8,
    ];

    // println!("{:?}", rgba);

    for (_, pixel) in frame.chunks_exact_mut(4).enumerate() {
        pixel.copy_from_slice(&rgba);
    }
}

pub fn draw_rect(frame: &mut [u8], color: u32, upper_left: u32, lower_right: u32) {
    let rgba: [u8; 4] = [
        ((color & 0xFF000000) >> 24) as u8,
        ((color & 0x00FF0000) >> 16) as u8,
        ((color & 0x0000FF00) >> 8) as u8,
        ((color & 0x000000FF) >> 0) as u8,
    ];

    let start_x = (upper_left & 0xFFFF0000) >> 16;
    // let start_y = upper_left & 0x0000FFFF;
    let start_y = 64;

    let end_x = (lower_right & 0xFFFF0000) >> 16;
    let end_y = lower_right & 0x0000FFFF;

    // println!("start=({start_x}, {start_y}); end=({end_x}, {end_y})");

    for x in start_x..end_x {
        for y in start_y..end_y {
            let index: usize = (y * crate::WIDTH + x * 4) as usize;

            for i in 0..4 {
                frame[index + i] = rgba[i];
            }
        }
    }
}
