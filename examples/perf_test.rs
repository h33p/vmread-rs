extern crate vmread;
extern crate rand;

use std::time::{Duration, Instant};
use rand::{thread_rng, Rng};
use std::process::Command;

fn rwtest(ctx: &vmread::sys::WinCtx, proc: &vmread::WinProcess, start_range: u64, end_range: u64, chunk_sizes: &[usize], chunk_counts: &[usize], read_size: usize) {
    let mut rng = thread_rng();

    for i in chunk_sizes {
        print!("0x{:x}", *i);
        for o in chunk_counts {
            let mut rwlist = proc.get_rwlist(ctx);
            let mut done_size = 0 as usize;
            let mut total_dur = Duration::new(0, 0);
            let mut calls = 0;
            let mut buf = vec![[0 as u8; 0x10000]; *o];

            while done_size < read_size {
                let now = Instant::now();
                {
                    let mut rws = rwlist.start();

                    let base_addr = rng.gen_range(start_range, end_range - (*i as u64 + 0x2000));
                
                    for u in buf.iter_mut() {
                        rws = rws.read_arr(base_addr + rng.gen_range(0, 0x2000), &mut u[..*i]);
                    }
                }
                total_dur += now.elapsed();
                done_size += *i * *o;
                calls += 1;
            }

            let total_time = total_dur.as_micros() as f64;

            print!(", {:.2}, {:.2}", (done_size / 0x100000) as f64 / (total_time / 10e5) as f64, calls as f64 / (total_time / 10e5) as f64);
        }
        println!("");
    }
}

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

        let mut rng = thread_rng();

        loop {
            ctx.refresh_processes();
            let plen = ctx.process_list.len();
            let proc = ctx.process_list[rng.gen_range(0, plen)].refresh_modules(c_ctx);

            let avail_mods = proc.module_list.iter().filter(|&x| x.info.sizeOfModule > 0x400000).collect::<Vec<&vmread::WinDll>>();

            if avail_mods.len() > 0 {
                let tmod = avail_mods[rng.gen_range(0, avail_mods.len())].clone();
                println!("Found test module {} ({:x}) in {}", tmod.name, tmod.info.sizeOfModule, proc.name);
                rwtest(&c_ctx, proc, tmod.info.baseAddress, tmod.info.baseAddress + tmod.info.sizeOfModule,
                    &[
                        0x10000 as usize,
                        0x1000,
                        0x100,
                        0x10,
                        0x8
                    ],
                    &[
                        32 as usize,
                        8,
                        1
                    ], 0x100000 * 64);
                break;
            }
        }
    } else {
        let (eval, estr) = ctx_ret.err().unwrap();
        println!("Initialization error {}: {}", eval, estr);
    }
}
