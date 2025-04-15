use crate::ComputerInfo;
use crate::CpuInfo;
use crate::HardwareModule;
use crate::VgaInfo;

pub struct DriverConfig<'a> {
    impoted_module: Vec<&'a str>,
    fingerprint: bool,
    iio_sensor: bool,
}
impl DriverConfig<'_> {
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

    pub fn get_computer_hardware_module<'a>(
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

    pub fn get_common_hardware_module<'a>(
        hardware_module: &'a HardwareModule,
        computer_info: &ComputerInfo,
        vga_info: &VgaInfo,
        cpu_info: &CpuInfo,
    ) -> Vec<String> {
        let mut all_module: Vec<String> = vec![];

        let common_module = hardware_module.get_common_module();

        // CPU
        let path_cpu = format!("common/cpu/{}/", cpu_info.get_constructor());
        let b_cpu = common_module
            .iter()
            .position(|s| s.starts_with(&path_cpu))
            .unwrap();
        let e_cpu = common_module[b_cpu..]
            .iter()
            .position(|s| !s.starts_with(&path_cpu))
            .unwrap()
            + b_cpu;
        let constructor_module = &common_module[b_cpu..e_cpu];

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
                Some(p) => constructor_module[p].clone(),
                None => format!("{}default.nix", path_cpu),
            },
        );

        // GPU
        if vga_info.has_nvidia_device() {
            // GPU Nvidia&
            match vga_info.get_nvidia_generation() {
                Ok(arch) => {
                    let b_nv = common_module
                        .iter()
                        .position(|s| s.starts_with("common/gpu/nvidia/"))
                        .unwrap();
                    let e_nv = common_module[b_nv..]
                        .iter()
                        .position(|s| !s.starts_with("common/gpu/nvidia/"))
                        .unwrap()
                        + b_nv;
                    let nvidia_module = &common_module[b_nv..e_nv];
                    if let Some(p) = nvidia_module
                        .iter()
                        .position(|s| s.split('/').collect::<Vec<&str>>()[3].eq(arch))
                    {
                        all_module.push(nvidia_module[p].clone());
                    } else {
                        all_module.push(String::from("common/gpu/nvidia/default.nix"));
                    }
                }
                Err(_) => all_module.push(String::from("common/gpu/nvidia/default.nix")),
            }
        }

        all_module
    }
}

// pub fn get_hardware_module()
