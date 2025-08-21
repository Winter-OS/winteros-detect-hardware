mod driver_config;
use driver_config::DriverConfig;
use system_info;

mod hardware_driver;

mod config_file;

use std::fs::File;
use std::process::exit;

#[cfg(not(debug_assertions))]
use std::process::Command;

#[cfg(not(debug_assertions))]
use nix::unistd::Uid;

#[cfg(debug_assertions)]
const PATH_FILE: &str = "./winteros-hardware.nix";

#[cfg(not(debug_assertions))]
const PATH_FILE: &str = "/etc/nixos/winteros-hardware.nix";

fn main() {
    #[cfg(not(debug_assertions))]
    if !Uid::effective().is_root() {
        let exe = std::env::current_exe().unwrap();
        let args: Vec<_> = std::env::args().skip(1).collect();

        let status = Command::new("pkexec")
            .arg(exe)
            .args(args)
            .status()
            .expect("Impossible de lancer pkexec");
        exit(13);
    }
    let config = match DriverConfig::new() {
        Ok(conf) => conf,
        Err(err) => {
            println!("Impossible to get hardware config : {}", err);
            exit(2);
        }
    };
    println!(
        "{:?}, Fingerprint : {}, IIO sensor : {}",
        config.get_module(),
        config.get_fingerprint(),
        config.get_iio_sensor()
    );

    let mut config_file = File::create(PATH_FILE).unwrap();
    let _ = match config_file::write_config(&config, &mut config_file) {
        Ok(_) => (),
        Err(err) => {
            println!("Impossible to write config file : {}", err.to_string());
            exit(1);
        }
    };
}
