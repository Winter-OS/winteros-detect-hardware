use std::fs;

#[derive(Debug)]
pub struct ComputerInfo {
    vendor: String,
    product_name: String,
    product_family: String,
}
impl ComputerInfo {
    fn get_vendor() -> Result<String, String> {
        match fs::read_to_string("/sys/devices/virtual/dmi/id/sys_vendor") {
            Ok(vendor) => Ok(vendor.trim().to_string()),
            Err(e) => Err(format!("Impossible to get vendor : {}", e.to_string())),
        }
    }

    fn get_product_name() -> Result<String, String> {
        match fs::read_to_string("/sys/devices/virtual/dmi/id/product_name") {
            Ok(vendor) => Ok(vendor.trim().to_string()),
            Err(e) => Err(format!(
                "Impossible to get product name : {}",
                e.to_string()
            )),
        }
    }

    fn get_product_family() -> Result<String, String> {
        match fs::read_to_string("/sys/devices/virtual/dmi/id/product_family") {
            Ok(vendor) => Ok(vendor.trim().to_string()),
            Err(e) => Err(format!(
                "Impossible to get product family : {}",
                e.to_string()
            )),
        }
    }

    pub fn new() -> Result<ComputerInfo, String> {
        Ok(ComputerInfo {
            vendor: Self::get_vendor()?,
            product_name: Self::get_product_name()?,
            product_family: Self::get_product_family()?,
        })
    }
}
