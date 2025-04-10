mod system_info;
use system_info::VgaInfo;

mod hardware_driver;
use hardware_driver::HardwareModule;

use std::sync::mpsc;
use std::thread;

fn main() {
    let (tvi, rvi) = mpsc::channel();
    let (thm, rhm) = mpsc::channel();

    thread::spawn(move || {
        let vga_info = VgaInfo::new().expect("Impossible to read vga info");
        tvi.send(vga_info).unwrap();
    });
    thread::spawn(move || {
        let hardware_module = HardwareModule::new().expect("immposible to get available module");
        thm.send(hardware_module).unwrap();
    });

    let vga_info = rvi.recv().unwrap();
    let hardware_module = rhm.recv().unwrap();

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
        "Architechture Poenix ? {}",
        vga_info.match_archtecture_codename("phoenix")
    );

    println!("Ambiant light sensor ? {}", system_info::has_iio_device());
    println!(
        "Fingerprint sensor ? {}",
        system_info::has_fingerprint_device()
    );
    println!("{:#?}", system_info::ComputerInfo::new().unwrap());

    println!("Available module : {:#?}", hardware_module);
}
