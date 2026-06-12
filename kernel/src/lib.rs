#![no_std]
#![no_main]

mod font;
mod vga;
mod vm;

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};
use core::str;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }

// Встроенные игры и системные утилиты
struct VFile { name: &'static str, content: &'static str }
const FILESYSTEM: [VFile; 3] = [
    VFile { name: "sysinfo.ae", content: "clear 0\ntext 20 20 4 AETHERIA_SYSTEM\ntext 20 80 1 Kernel_:_Rust_v0.5\ntext 20 110 1 VM_:_AetherScript\ntext 20 280 3 Press_Q_to_exit\nkey k\nif k == 113 exit\ndelay 50\njmp 0" },
    VFile { name: "snake.ae", content: "set l 3; arr_set 0 0 15; arr_set 256 0 10\nclear 0; text 20 10 4 SNAKE_GAME\nkey k\nif k == 113 exit\n// Тут логика змейки...\ndelay 60\njmp 1" },
    VFile { name: "bounce.ae", content: "set x 10; set y 10; set a 1; set b 1\nclear 0; grid x y 4; op x add a; op y add b\nif x > 60 set a -1\nif x < 1 set a 1\nkey k\nif k == 113 exit\ndelay 30\njmp 5" }
];

#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn aetheria_main(fb_addr: *mut u32, kb_addr: *mut u8, width: u32, height: u32) -> ! {
    let bg = 0xFF0D0D14u32; 
    let green = 0xFF4AF626u32; 
    let white = 0xFFFFFFFFu32;

    let mut cursor_y = 40;
    let prompt = "root@aetheria:~# ";
    let mut input_buf = [0u8; 64];
    let mut input_len = 0;

    vga::draw_rect(fb_addr, width, 0, 0, width, height, bg);
    vga::draw_text(fb_addr, width, 20, 10, "AetheriaOS v0.5 Active (Type 'help')", green, 2);
    vga::draw_text(fb_addr, width, 20, cursor_y, prompt, green, 2);

    loop {
        let key = read_volatile(kb_addr);
        if key != 0 {
            write_volatile(kb_addr, 0); // Сброс клавиши

            if key == 10 { // ENTER
                cursor_y += 20;
                if let Ok(cmd_str) = str::from_utf8(&input_buf[..input_len]) {
                    let cmd = cmd_str.trim();
                    if cmd == "help" {
                        vga::draw_text(fb_addr, width, 20, cursor_y, "Commands: ls, run [file], clear, help", white, 2);
                        cursor_y += 20;
                    } else if cmd == "ls" {
                        vga::draw_text(fb_addr, width, 20, cursor_y, "sysinfo.ae  snake.ae  bounce.ae", white, 2);
                        cursor_y += 20;
                    } else if cmd.starts_with("run ") {
                        let file_name = &cmd[4..];
                        for file in FILESYSTEM.iter() {
                            if file.name == file_name {
                                let mut machine = vm::AetherVM::new();
                                machine.run(fb_addr, kb_addr, width, height, file.content);
                                vga::draw_rect(fb_addr, width, 0, 0, width, height, bg);
                                cursor_y = 20;
                                break;
                            }
                        }
                    } else if cmd == "clear" {
                        vga::draw_rect(fb_addr, width, 0, 0, width, height, bg);
                        cursor_y = 20;
                    }
                }
                input_len = 0;
                if cursor_y > height - 40 { vga::draw_rect(fb_addr, width, 0, 0, width, height, bg); cursor_y = 20; }
                vga::draw_text(fb_addr, width, 20, cursor_y, prompt, green, 2);
            } else if key == 8 { // BACKSPACE
                if input_len > 0 {
                    input_len -= 1;
                    vga::draw_rect(fb_addr, width, 20 + (prompt.len() + input_len) as u32 * 16, cursor_y, 16, 16, bg);
                }
            } else if input_len < 64 && key >= 32 && key <= 126 { // Печать символа
                input_buf[input_len] = key;
                vga::draw_char(fb_addr, width, 20 + (prompt.len() + input_len) as u32 * 16, cursor_y, key as char, white, 2);
                input_len += 1;
            }
        }
        core::hint::spin_loop();
    }
                            }
