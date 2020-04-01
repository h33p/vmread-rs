extern crate vmread;

fn main() {
    let ctx_ret = vmread::create_context(0);

    if ctx_ret.is_ok() {
        let (mut ctx, _) = ctx_ret.unwrap();
        println!("VMRead initialized!");

        println!("Kernel module list");
        println!("{:#18} {:#18} {:#8} {:#6} {}", "BASE ADDRESS", "ENTRY POINT", "SIZE", "LOADC", "NAME");
        for i in &ctx.refresh_kmods().kmod_list {
            println!("{:#18x} {:#18x} {:#8x} {:#6x} {}", i.info.baseAddress, i.info.entryPoint, i.info.sizeOfModule, i.info.loadCount, i.name);
        }
    } else {
        let (eval, estr) = ctx_ret.err().unwrap();
        println!("Initialization error {}: {}", eval, estr);
    }
}
