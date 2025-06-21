mod driver_config;
use system_info;
use driver_config::DriverConfig;

mod hardware_driver;

mod config_file;

use std::fs::File;
use std::process::exit;

#[cfg(not(debug_assertions))]
use nix::unistd::Uid;

#[cfg(debug_assertions)]
const PATH_FILE: &str = "./winteros-hardware.nix";

#[cfg(not(debug_assertions))]
const PATH_FILE: &str = "/etc/nixos/winteros-hardware.nix";

fn main() {
    #[cfg(not(debug_assertions))]
    if !Uid::effective().is_root() {
        println!("You must run this executable with root permissions");
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
