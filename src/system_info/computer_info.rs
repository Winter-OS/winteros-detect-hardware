use std::fs;
use std::path::Path;
use std::process::Command;

pub fn get_vendor() -> Result<String, String> {
    match fs::read_to_string("/sys/devices/virtual/dmi/id/sys_vendor") {
        Ok(vendor) => Ok(vendor.trim().to_string()),
        Err(e) => Err(format!("Impossible to get vendor : {}", e.to_string())),
    }
}

pub fn get_product_name() -> Result<String, String> {
    match fs::read_to_string("/sys/devices/virtual/dmi/id/product_name") {
        Ok(vendor) => Ok(vendor.trim().to_string()),
        Err(e) => Err(format!(
            "Impossible to get product name : {}",
            e.to_string()
        )),
    }
}

pub fn get_product_family() -> Result<String, String> {
    match fs::read_to_string("/sys/devices/virtual/dmi/id/product_family") {
        Ok(vendor) => Ok(vendor.trim().to_string()),
        Err(e) => Err(format!(
            "Impossible to get product family : {}",
            e.to_string()
        )),
    }
}

pub fn has_iio_device() -> bool {
    let path = Path::new("/sys/bus/iio/devices");
    if path.exists()
        && path.is_dir()
        && match path.read_dir() {
            Ok(read_dir) => read_dir,
            Err(_) => return false,
        }
        .next()
        .is_some()
    {
        return true;
    }
    return false;
}

pub fn has_fingerprint_device() -> bool {
    let output = match Command::new("lsusb").output() {
        Ok(out) => out,
        Err(_) => return false,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().split('\n').collect();

    for line in lines {
        if line.to_lowercase().contains("fingerprint") {
            return true;
        }
    }
    return false;
}
