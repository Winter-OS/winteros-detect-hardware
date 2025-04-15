mod driver_config;
mod system_info;
use driver_config::DriverConfig;

mod hardware_driver;
use hardware_driver::HardwareModule;

fn main() {
    let driver = DriverConfig::new().unwrap();
    println!(
        "{:?}, Fingerprint : {}, IIO sensor : {}",
        driver.get_module(),
        driver.get_fingerprint(),
        driver.get_iio_sensor()
    );
}
