mod system_info;
use system_info::VgaInfo;

fn main() {
    let vga_info = VgaInfo::new().expect("Impossible to read vga info");
    println!("{:?}", vga_info.has_nvidia_device());
    println!("{:?}", vga_info.has_nvidia_laptop());
    println!("{:?}", vga_info.get_nvidia_generation());
    println!("{:#?}", vga_info);
}
