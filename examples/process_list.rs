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
        let (mut ctx, _) = ctx_ret.unwrap();
        println!("VMRead initialized!");

        println!("Process List:\nPID\tVIRT\t\t\tPHYS\t\tBASE\t\tNAME");
        for i in &(ctx.refresh_processes().process_list) {
           println!("{:#4x}\t{:#16x}\t{:#9x}\t{:#9x}\t{}", i.proc.pid, i.proc.process, i.proc.physProcess, i.proc.dirBase, i.name); 
        }
    } else {
        let (eval, estr) = ctx_ret.err().unwrap();
        println!("Initialization error {}: {}", eval, estr);
    }
}
