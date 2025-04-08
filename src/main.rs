mod system_info;
use system_info::VgaInfo;
mod hardware_driver;

fn main() {
    let vga_info = VgaInfo::new().expect("Impossible to read vga info");
    println!("NVIDIA device ? {}", vga_info.has_nvidia_device());
    println!(
        "Laptop with NVIDIA device ? {}",
        vga_info.has_nvidia_laptop()
    );
    println!(
        "NVIDIA generation device : {}",
        match vga_info.get_nvidia_generation() {
            Ok(gen) => gen,
            Err(_) => "none",
        }
    );
    println!("{:#?}", vga_info);

    println!(
        "{:#?}",
        match hardware_driver::get_git_tree("NixOS", "nixos-hardware") {
            Ok(ret) => ret,
            Err(err) => panic!("{}", err),
        }
    );
}
