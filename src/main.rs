mod driver_config;
use driver_config::DriverConfig;
use system_info;

mod hardware_driver;

mod config_file;

use std::fs::File;
use std::io::Write;
use std::process::Stdio;
use std::process::exit;

use std::process::Command;

#[cfg(not(debug_assertions))]
use nix::unistd::Uid;

#[cfg(debug_assertions)]
const PATH_FILE: &str = "./winteros-hardware.nix";

#[cfg(not(debug_assertions))]
const PATH_FILE: &str = "/etc/nixos/winteros-hardware.nix";

fn main() {
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


    #[cfg(not(debug_assertions))]
    let is_root = !Uid::effective().is_root();
    #[cfg(debug_assertions)]
    let is_root = false;

    if is_root {
        let mut child = Command::new("pkexec")
            .arg("tee") // tee écrit dans un fichier même en root
            .arg(PATH_FILE)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .expect("Impossible de lancer pkexec");

            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(config.to_config_file()
                     .as_bytes())
                     .expect("fail to write config file");
            }
    } else {
        let mut config_file = File::create(PATH_FILE).unwrap();
        let _ = match config_file::write_config(&config, &mut config_file) {
            Ok(_) => (),
            Err(err) => {
                println!("Impossible to write config file : {}", err.to_string());
                exit(1);
            }
        };
    }

}
