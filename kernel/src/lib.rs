#![no_std]
mod font;
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};
use core::str;

#[panic_handler] fn panic(_info: &PanicInfo) -> ! { loop { core::hint::spin_loop(); } }
#[no_mangle] pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 { for i in 0..n { *dest.add(i) = c as u8; } dest }
#[no_mangle] pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 { for i in 0..n { *dest.add(i) = *src.add(i); } dest }

fn draw_rect(fb: *mut u32, fb_w: u32, x: u32, y: u32, w: u32, h: u32, color: u32) {
    for iy in y..(y+h) {
        for ix in x..(x+w) { unsafe { write_volatile(fb.add((iy * fb_w + ix) as usize), color); } }
    }
}

fn draw_char(fb: *mut u32, fb_w: u32, x: u32, y: u32, c: char, color: u32, scale: u32) {
    let bitmap = font::get_char_bitmap(c);
    for row in 0..8 {
        for col in 0..8 {
            if (bitmap[row] & (1 << (7 - col))) != 0 {
                draw_rect(fb, fb_w, x + col * scale, y + (row as u32) * scale, scale, scale, color);
            }
        }
    }
}

fn draw_text(fb: *mut u32, fb_w: u32, x: u32, y: u32, text: &str, color: u32, scale: u32) {
    let mut cx = x;
    for c in text.chars() { draw_char(fb, fb_w, cx, y, c, color, scale); cx += 8 * scale; }
}

fn get_val(s: &str, r: &[i32; 26]) -> i32 {
    let b = s.as_bytes();
    if b.len() == 1 && b[0] >= b'a' && b[0] <= b'z' { return r[(b[0] - b'a') as usize]; }
    let mut v = 0; let mut sign = 1;
    for (i, &ch) in b.iter().enumerate() {
        if i==0 && ch==b'-' { sign = -1; continue; }
        if ch >= b'0' && ch <= b'9' { v = v*10 + (ch-b'0') as i32; }
    }
    v * sign
}

fn run_script(fb: *mut u32, kb: *mut u8, w: u32, h: u32, code: &str) {
    let mut lines = [""; 64]; let mut line_count = 0;
    for l in code.split('\n') {
        let t = l.trim();
        if t.len() > 0 && !t.starts_with("//") { if line_count < 64 { lines[line_count] = t; line_count += 1; } }
    }
    let mut mem = [0i32; 1024]; let mut regs = [0i32; 26];
    let mut pc = 0; let mut seed = 12345u32;
    let colors = [0xFF0D0D14, 0xFFFFFFFF, 0xFF4AF626, 0xFFF38BA8, 0xFF89B4FA, 0xFFF9E2AF, 0xFF94E2D5];

    while pc < line_count {
        let key = unsafe { read_volatile(kb) };
        if key != 0 { unsafe { write_volatile(kb, 0); } }
        let mut exit = false; let mut jumped = false;
        for statement in lines[pc].split(';') {
            let stmt = statement.trim();
            if stmt.len() == 0 { continue; }
            let mut words = [""; 15]; let mut wc = 0;
            for word in stmt.split(' ') { if word.len() > 0 && wc < 15 { words[wc] = word; wc += 1; } }
            let mut cur = &words[0..wc];
            loop {
                if cur.len() == 0 { break; }
                let cmd = cur[0];
                if cmd == "if" && cur.len() > 4 {
                    let v1 = get_val(cur[1], &regs); let op = cur[2]; let v2 = get_val(cur[3], &regs);
                    let cond = if op == "==" { v1 == v2 } else if op == "!=" { v1 != v2 } else if op == ">" { v1 > v2 } else if op == "<" { v1 < v2 } else { false };
                    if cond { cur = &cur[4..]; continue; } else { break; }
                } else if cmd == "set" && cur.len() > 2 {
                    let var = cur[1].as_bytes()[0]; if var >= b'a' && var <= b'z' { regs[(var - b'a') as usize] = get_val(cur[2], &regs); } break;
                } else if cmd == "op" && cur.len() > 3 {
                    let var = cur[1].as_bytes()[0];
                    if var >= b'a' && var <= b'z' {
                        let idx = (var - b'a') as usize; let v2 = get_val(cur[3], &regs);
                        if cur[2] == "add" { regs[idx] += v2; } else if cur[2] == "sub" { regs[idx] -= v2; } else if cur[2] == "mul" { regs[idx] *= v2; }
                    } break;
                } else if cmd == "arr_set" && cur.len() > 3 {
                    let base = get_val(cur[1], &regs) as usize; let idx = get_val(cur[2], &regs) as usize; let val = get_val(cur[3], &regs);
                    if base+idx < 1024 { mem[base+idx] = val; } break;
                } else if cmd == "arr_get" && cur.len() > 3 {
                    let var = cur[1].as_bytes()[0];
                    if var >= b'a' && var <= b'z' {
                        let base = get_val(cur[2], &regs) as usize; let idx = get_val(cur[3], &regs) as usize;
                        if base+idx < 1024 { regs[(var-b'a') as usize] = mem[base+idx]; }
                    } break;
                } else if cmd == "key" && cur.len() > 1 {
                    let var = cur[1].as_bytes()[0]; if var >= b'a' && var <= b'z' { regs[(var - b'a') as usize] = key as i32; } break;
                } else if cmd == "rand" && cur.len() > 2 {
                    let var = cur[1].as_bytes()[0];
                    if var >= b'a' && var <= b'z' {
                        seed = (seed.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
                        regs[(var - b'a') as usize] = (seed % (get_val(cur[2], &regs) as u32)) as i32;
                    } break;
                } else if cmd == "clear" && cur.len() > 1 {
                    let c = get_val(cur[1], &regs) as usize; draw_rect(fb, w, 0, 0, w, h, colors[c % colors.len()]); break;
                } else if cmd == "grid" && cur.len() > 3 {
                    let rx = get_val(cur[1], &regs) as u32 * 20; let ry = get_val(cur[2], &regs) as u32 * 20;
                    let rc = get_val(cur[3], &regs) as usize; draw_rect(fb, w, rx, ry, 20, 20, colors[rc % colors.len()]); break;
                } else if cmd == "text" && cur.len() > 4 {
                    let rx = get_val(cur[1], &regs) as u32; let ry = get_val(cur[2], &regs) as u32;
                    let rc = get_val(cur[3], &regs) as usize; 
                    let mut tmp = [0u8; 64]; let mut ti = 0;
                    for b in cur[4].as_bytes() { if ti < 64 { tmp[ti] = if *b==b'_' { b' ' } else { *b }; ti+=1; } }
                    if let Ok(s) = str::from_utf8(&tmp[..ti]) { draw_text(fb, w, rx, ry, s, colors[rc % colors.len()], 2); } break;
                } else if cmd == "delay" && cur.len() > 1 {
                    let ms = get_val(cur[1], &regs); for _ in 0..(ms * 50000) { core::hint::spin_loop(); } break;
                } else if cmd == "jmp" && cur.len() > 1 {
                    pc = get_val(cur[1], &regs) as usize; jumped = true; break;
                } else if cmd == "exit" { exit = true; break; } else { break; }
            }
            if exit || jumped { break; }
        }
        if exit { break; }
        if !jumped { pc += 1; }
    }
}

struct VFile { name: &'static str, content: &'static str }
const FILESYSTEM: [VFile; 3] = [
    VFile { name: "sysinfo.ae", content: "clear 0\ntext 20 20 4 AETHERIA_OS\ntext 20 80 1 Kernel_:_Rust\nkey k\nif k == 113 exit 0\njmp 6" },
    VFile { name: "snake.ae", content: "set l 3; arr_set 0 0 15; arr_set 256 0 10; clear 0\nkey k\nif k == 113 exit 0\njmp 1" },
    VFile { name: "bounce.ae", content: "set x 10; set y 10; set a 1; set b 1; clear 0; grid x y 4; jmp 1" }
];

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn aetheria_main(fb_addr: *mut u32, kb_addr: *mut u8, width: u32, height: u32) -> ! {
    let bg = 0xFF0D0D14u32; let green = 0xFF4AF626u32;
    draw_rect(fb_addr, width, 0, 0, width, height, bg);
    draw_text(fb_addr, width, 20, 20, "AetheriaOS v0.5 Active", green, 2);
    loop {
        let key = unsafe { read_volatile(kb_addr) };
        if key == 10 { // Enter
            run_script(fb_addr, kb_addr, width, height, FILESYSTEM[0].content);
            draw_rect(fb_addr, width, 0, 0, width, height, bg);
        }
        core::hint::spin_loop();
    }
}
