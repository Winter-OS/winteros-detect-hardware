use crate::hardware_driver::HardwareModule;
use crate::system_info::ComputerInfo;
use crate::system_info::CpuInfo;
use crate::system_info::VgaInfo;

#[derive(Debug)]
pub struct DriverConfig {
    impoted_module: Vec<String>,
    fingerprint: bool,
    iio_sensor: bool,
}
impl DriverConfig {
    fn get_computer_hardware_module_rec<'a>(
        hardware_module: &'a [String],
        computer_info: &ComputerInfo,
        vga_info: &VgaInfo,
        depth: usize,
    ) -> Option<&'a str> {
        if hardware_module.len() == 1 {
            return Some(hardware_module[0].strip_suffix("/default.nix")?);
        } else if hardware_module.len() == 0 {
            return None;
        }

        let mut match_module: Option<&str> = None;
        let mut begin: Option<usize> = None;
        let mut end: Option<usize> = None;
        let mut common_b: Option<usize> = None;
        let mut common_e: Option<usize> = None;
        let mut def: Option<usize> = None;
        let mut nvidia: Option<usize> = None;
        let mut amdgpu: Option<usize> = None;

        for i in 0..hardware_module.len() {
            let module = hardware_module[i].split("/").collect::<Vec<&str>>()[depth];
            if module.eq("common") {
                if let None = common_b {
                    common_b = Some(i);
                }
                common_e = Some(i + 1);
                if let Some(_) = begin {
                    end = Some(i);
                }
                continue;
            } else if module.eq("default.nix") {
                def = Some(i);
                if let Some(_) = begin {
                    end = Some(i);
                }
                continue;
            } else if module.eq("amdgpu") {
                amdgpu = Some(i);
                if let Some(_) = begin {
                    end = Some(i);
                }
                continue;
            } else if module.eq("nvidia") {
                nvidia = Some(i);
                if let Some(_) = begin {
                    end = Some(i);
                }
                continue;
            }

            match match_module {
                None if module.split("-").all(|s| {
                    computer_info.get_product_name().contains(s)
                        || computer_info.get_product_family().contains(s)
                }) =>
                {
                    match_module = Some(module);
                    begin = Some(i);
                }
                Some(m) if m.ne(module) => {
                    end = Some(i);
                    break;
                }
                _ => continue,
            };
        }
        if let None = begin {
            if let Some(c) = common_b {
                let e = common_e.unwrap();
                return Self::get_computer_hardware_module_rec(
                    &hardware_module[c..e],
                    computer_info,
                    vga_info,
                    depth + 1,
                );
            } else {
                match nvidia {
                    Some(n) if vga_info.has_nvidia_device() => {
                        return Some(hardware_module[n].strip_suffix("/default.nix")?)
                    }
                    None | Some(_) => match amdgpu {
                        Some(a) if vga_info.match_archtecture_codename("amd") => {
                            return Some(hardware_module[a].strip_suffix("default.nix")?)
                        }
                        None | Some(_) => match def {
                            Some(d) => {
                                println!(
                                    "{}",
                                    hardware_module[d].strip_suffix("/default.nix").unwrap()
                                );
                                return Some(hardware_module[d].strip_suffix("/default.nix")?);
                            }
                            None => (),
                        },
                    },
                };
            }
        }
        if let None = end {
            return Self::get_computer_hardware_module_rec(
                &hardware_module[begin.unwrap()..],
                computer_info,
                vga_info,
                depth + 1,
            );
        }
        return Self::get_computer_hardware_module_rec(
            &hardware_module[begin.unwrap()..end.unwrap()],
            computer_info,
            vga_info,
            depth + 1,
        );
    }

    fn get_computer_hardware_module_family<'a>(
        hardware_module: &'a [String],
        computer_info: &ComputerInfo,
        vga_info: &VgaInfo,
    ) -> Option<&'a str> {
        let mut match_family_module: Option<&str> = None;
        let mut begin: Option<usize> = None;
        let mut end: Option<usize> = None;
        for i in 0..hardware_module.len() {
            let family_module = hardware_module[i].split("/").collect::<Vec<&str>>()[1];
            match match_family_module {
                None if family_module
                    .split("-")
                    .all(|s| computer_info.get_product_family().contains(s)) =>
                {
                    match_family_module = Some(family_module);
                    begin = Some(i);
                }
                Some(m) if m.ne(family_module) => {
                    end = Some(i);
                    break;
                }
                _ => continue,
            };
        }
        if let None = begin {
            return None;
        }
        if let None = end {
            return Self::get_computer_hardware_module_rec(
                &hardware_module[begin.unwrap()..],
                computer_info,
                vga_info,
                2,
            );
        }
        return Self::get_computer_hardware_module_rec(
            &hardware_module[begin.unwrap()..end.unwrap()],
            computer_info,
            vga_info,
            2,
        );
    }

    fn list_all_vendor(hardware_module: &HardwareModule) -> Vec<&str> {
        let mut v: Vec<&str> = vec![];
        let mut last: &str = "";
        for hard_mod in hardware_module.get_computer_module() {
            let vendor = hard_mod.split("/").collect::<Vec<&str>>()[0];
            if vendor != last {
                v.push(vendor);
                last = &vendor;
            }
        }
        v
    }

    fn get_computer_hardware_module<'a>(
        hardware_module: &'a HardwareModule,
        computer_info: &ComputerInfo,
        vga_info: &VgaInfo,
    ) -> Option<&'a str> {
        let all_vendor = Self::list_all_vendor(hardware_module);
        let vendor = match all_vendor
            .iter()
            .position(|v| v.contains(computer_info.get_vendor()))
        {
            Some(p) => all_vendor[p],
            None => return None,
        };

        let begin = hardware_module
            .get_computer_module()
            .iter()
            .position(|s| s.starts_with(vendor))?;
        let end = hardware_module.get_computer_module()[begin..]
            .iter()
            .position(|s| !s.starts_with(vendor))?
            + begin;

        Self::get_computer_hardware_module_family(
            &hardware_module.get_computer_module()[begin..end],
            computer_info,
            vga_info,
        )
    }

    fn restrict_range_str<'a>(range: &'a [String], prefix: &str) -> &'a [String] {
        let b = range.iter().position(|s| s.starts_with(prefix)).unwrap();
        let e = range[b..]
            .iter()
            .position(|s| !s.starts_with(prefix))
            .unwrap()
            + b;
        &range[b..e]
    }

    fn get_common_hardware_module<'a>(
        hardware_module: &'a HardwareModule,
        vga_info: &VgaInfo,
        cpu_info: &CpuInfo,
        computer_info: &ComputerInfo,
    ) -> Vec<String> {
        let mut all_module: Vec<String> = vec![];

        let common_module = hardware_module.get_common_module();

        // CPU
        let path_cpu = format!("common/cpu/{}/", cpu_info.get_constructor());
        let constructor_module = Self::restrict_range_str(common_module, &path_cpu);

        all_module.push(
            match constructor_module.iter().position(|s| {
                s.split('/')
                    .collect::<Vec<&str>>()
                    .get(3)
                    .unwrap()
                    .split('-')
                    .collect::<Vec<&str>>()
                    .iter()
                    .all(|s2| cpu_info.get_codename().contains(s2))
            }) {
                Some(p) => constructor_module[p]
                    .strip_suffix("/default.nix")
                    .unwrap()
                    .to_string(),
                None => path_cpu.strip_suffix('/').unwrap().to_string(),
            },
        );

        // GPU
        if vga_info.has_nvidia_device() {
            // GPU Nvidia&
            match vga_info.get_nvidia_generation() {
                Ok(arch) => {
                    let nvidia_module =
                        Self::restrict_range_str(common_module, "common/gpu/nvidia/");
                    if let Some(p) = nvidia_module
                        .iter()
                        .position(|s| s.split('/').collect::<Vec<&str>>()[3].eq(arch))
                    {
                        all_module.push(
                            nvidia_module[p]
                                .strip_suffix("/default.nix")
                                .unwrap()
                                .to_string(),
                        );
                    } else {
                        all_module.push(String::from("common/gpu/nvidia"));
                    }
                }
                Err(_) => all_module.push(String::from("common/gpu/nvidia")),
            }
            if vga_info.has_nvidia_laptop() {
                all_module.push(String::from("common/gpu/nvidia/prime.nix"))
            }
        }
        if vga_info.match_archtecture_codename("amd") {
            // GPU AMD
            let amd_module = Self::restrict_range_str(common_module, "common/gpu/amd/");
            if let Some(s) = match amd_module.iter().position(|s| {
                s.split('/')
                    .collect::<Vec<&str>>()
                    .get(3)
                    .unwrap()
                    .split('-')
                    .collect::<Vec<&str>>()
                    .iter()
                    .all(|s2| vga_info.match_archtecture_codename(s2))
            }) {
                Some(p) => Some(
                    amd_module[p]
                        .strip_suffix("/default.nix")
                        .unwrap()
                        .to_string(),
                ),
                None => None,
            } {
                all_module.push(s);
            }
            all_module.push(String::from("common/gpu/amd/"));
        }
        if vga_info.match_archtecture_codename("intel") {
            // GPU intel
            all_module.push(String::from("common/gpu/intel/"));
        }

        if ComputerInfo::is_laptop() {
            all_module.push(String::from("common/pc/laptop/"));
        } else {
            all_module.push(String::from("common/pc/"));
        }

        if computer_info.has_ssd() {
            all_module.push(String::from("common/pc/ssd"));
        }

        all_module
    }

    pub fn new() -> Result<DriverConfig, String> {
        let vga_info = match VgaInfo::new() {
            Ok(vga) => vga,
            Err(err) => return Err(err.to_string()),
        };
        let hardware_module = HardwareModule::new()?;
        let computer_info = ComputerInfo::new()?;
        let cpu_info = CpuInfo::new()?;
        Ok(DriverConfig {
            impoted_module: match Self::get_computer_hardware_module(
                &hardware_module,
                &computer_info,
                &vga_info,
            ) {
                Some(s) => vec![s.to_string()],
                None => Self::get_common_hardware_module(
                    &hardware_module,
                    &vga_info,
                    &cpu_info,
                    &computer_info,
                ),
            },
            fingerprint: ComputerInfo::has_fingerprint_device(),
            iio_sensor: ComputerInfo::has_iio_device(),
        })
    }

    pub fn get_module(&self) -> &Vec<String> {
        return &self.impoted_module;
    }

    pub fn get_fingerprint(&self) -> bool {
        return self.fingerprint;
    }

    pub fn get_iio_sensor(&self) -> bool {
        return self.iio_sensor;
    }

    pub fn to_config_file(&self) -> String {
        let mut ret =
"# Do not modify this file!  It was generated by 'winteros-generate-config'
# and may be overwritten by future invocations.  Please make changes
# to /etc/nixos/configuration.nix instead.
{ lib, ... }:
let
\tnixos-hardware = builtins.fetchTarball \"https://github.com/NixOS/nixos-hardware/archive/master.tar.gz\";
in
{
\timports = [
".to_owned();

        for module in Self::get_module(self) {
            ret.push_str(format!("\t\t(import \"${{nixos-hardware}}/{}\")\n", module).as_str());
        }
        ret.push_str("\t];\n");
        if Self::get_iio_sensor(self) {
            ret.push_str("\thardware.sensor.iio.enable = lib.mkDefault true;\n");
        }
        if Self::get_fingerprint(self) {
            ret.push_str("\tservices.fprintd.enable = lib.mkDefault true;\n");
        }
        ret.push('}');
        ret
    }
}

// pub fn get_hardware_module()
