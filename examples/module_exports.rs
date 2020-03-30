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

        loop {
            println!("Enter process name");
            let mut proc_name = String::new();
            match std::io::stdin().read_line(&mut proc_name) {
                Ok(_) => {
                    match proc_name.trim() {
                        "q" => break,
                        s => {
                            match ctx.refresh_processes().process_list.iter_mut().find(|a| a.name == s) {
                                Some(p) => {
                                    println!("Module list for {}", s);
                                    println!("{:#14} {:#14} {:#8} {:#6} {}", "BASE ADDRESS", "ENTRY POINT", "SIZE", "LOADC", "NAME");
                                    for i in &(p.refresh_modules(c_ctx).module_list) {
                                        println!("{:#14x} {:#14x} {:#8x} {:#6x} {}", i.info.baseAddress, i.info.entryPoint, i.info.sizeOfModule, i.info.loadCount, i.name);
                                    }
                                    
                                    loop {
                                        println!("Enter module name");
                                        let mut mod_name = String::new();
                                        match std::io::stdin().read_line(&mut mod_name) {
                                            Ok(_) => {
                                                match mod_name.trim() {
                                                    "q" => break,
                                                    mn => {
                                                        match p.module_list.iter_mut().find(|a| a.name == mn) {
                                                            Some(m) => {
                                                                println!("Export list for {}:", m.name);
                                                                println!("{:#14}  NAME", "ADDRESS");
                                                                for e in &(m.refresh_exports(&p.proc, c_ctx).export_list) {
                                                                    println!("{:<#14x}  {}", e.address, e.name);
                                                                }
                                                            },
                                                            _ => println!("Module not found!")
                                                        }
                                                    }
                                                }
                                            },
                                            Err(error) => {
                                                println!("error: {}", error);
                                                break;
                                            }
                                        }
                                    }
                                },
                                _ => println!("Process {} not found!", s)
                            }
                        }
                    }
                },
                Err(error) => {
                    println!("error: {}", error);
                    break;
                }
            }
        }
    } else {
        let (eval, estr) = ctx_ret.err().unwrap();
        println!("Initialization error {}: {}", eval, estr);
    }
}
