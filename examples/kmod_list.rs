extern crate vmread;

use std::process::Command;

fn main() {
    let pid = std::str::from_utf8(Command::new("sh")
        .arg("-c")
        .arg("pidof qemu-system-x86_64")
        .output()
        .unwrap()
        .stdout
        .as_slice())
        .unwrap()
        .trim()
        .to_string()
        .parse::<i32>()
        .unwrap_or(0);
    
    let ctx_ret = vmread::create_context(pid);

    if ctx_ret.is_ok() {
        let (mut ctx, c_ctx) = ctx_ret.unwrap();
        println!("VMRead initialized!");

        println!("Kernel module list");
        println!("{:#18} {:#18} {:#8} {:#6} {}", "BASE ADDRESS", "ENTRY POINT", "SIZE", "LOADC", "NAME");
        for i in &(ctx.refresh_kmods(c_ctx).kmod_list) {
            println!("{:#18x} {:#18x} {:#8x} {:#6x} {}", i.info.baseAddress, i.info.entryPoint, i.info.sizeOfModule, i.info.loadCount, i.name);
        }
    } else {
        let (eval, estr) = ctx_ret.err().unwrap();
        println!("Initialization error {}: {}", eval, estr);
    }
}
