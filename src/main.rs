mod system_info;
use system_info::ComputerInfo;
use system_info::CpuInfo;
use system_info::VgaInfo;

mod hardware_driver;
use hardware_driver::HardwareModule;

use std::sync::mpsc;
use std::thread;

mod list_hardware_module;
use list_hardware_module::DriverConfig;

fn main() {
    let (tvi, rvi) = mpsc::channel();
    let (thm, rhm) = mpsc::channel();
    let (tci, rci) = mpsc::channel();

    thread::spawn(move || {
        let vga_info = VgaInfo::new().expect("Impossible to read vga info");
        tvi.send(vga_info).unwrap();
    });
    thread::spawn(move || {
        let hardware_module = HardwareModule::new().expect("immposible to get available module");
        thm.send(hardware_module).unwrap();
    });
    thread::spawn(move || {
        let computer_info = ComputerInfo::new().expect("immposible to get computer info");
        tci.send(computer_info).unwrap();
    });

    let vga_info = rvi.recv().unwrap();
    let hardware_module = rhm.recv().unwrap();
    let computer_info = rci.recv().unwrap();

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

    println!(
        "Ambiant light sensor ? {}",
        system_info::computer_info::ComputerInfo::has_iio_device()
    );
    println!(
        "Fingerprint sensor ? {}",
        system_info::computer_info::ComputerInfo::has_fingerprint_device()
    );
    println!("Product vendor : {}", computer_info.get_vendor());
    println!("Product family : {}", computer_info.get_product_family());
    println!("Product name : {}", computer_info.get_product_name());

    println!(
        "Computer module : {}",
        match DriverConfig::get_computer_hardware_module(
            &hardware_module,
            &computer_info,
            &vga_info
        ) {
            Some(m) => m,
            None => "no module",
        }
    );

    let cpu_info = CpuInfo::new().unwrap();

    println!(
        "Hardware module : {:#?}",
        DriverConfig::get_common_hardware_module(
            &hardware_module,
            &computer_info,
            &vga_info,
            &cpu_info
        )
    );

    println!("Is laptop : {}", ComputerInfo::is_laptop());

    println!("{}:{}", cpu_info.get_constructor(), cpu_info.get_codename());
}
