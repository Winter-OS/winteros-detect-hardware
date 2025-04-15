mod system_info;
use system_info::vga_info;
use system_info::ComputerInfo;
use system_info::VgaInfo;

mod hardware_driver;
use hardware_driver::HardwareModule;

use std::sync::mpsc;
use std::thread;

use regex::Regex;
use std::path::Path;
use std::process::Command;

/*
fn get_fullpath_computer_hardware_module<'a, 'b>(
    hardware_module: &'a HardwareModule,
    vga_info: &'b VgaInfo,
    module_available: &'a [String],
    current_path: &str,
    depth: usize,
) -> &'a [String] {
    let mut v: Vec<&str> = vec![];

    let mut last: &str = "";

    let mut begin: Option<usize> = None;
    let mut end: Option<usize> = None;

    let name = system_info::computer_info::get_product_name().unwrap();

    for i in 0..module_available.len() {
        let module = &module_available[i];

        let depth_info = module.split("/").collect::<Vec<&str>>()[depth];
        println!("{}, {}", module, depth_info);
        if depth_info == last {
            continue;
        } else if let None = begin {
            if depth_info
                .split("-")
                .all(|s| family.contains(s) || name.contains(s))
            {
                begin = Some(i);
            }
            last = depth_info;
        } else {
            end = Some(i)
        }
    }
    if let Some(_) = begin
        && let None = end
    {
        end = Some(module_available.len())
    }

    let b = begin.unwrap();
    let e = end.unwrap();

    module_available.as_ref()[b..e].as_ref()
}



fn get_computer_hardware_module<'a, 'b>(
    hardware_module: &'a HardwareModule,
    vga_info: &'b VgaInfo,
) -> Option<&'a [String]> {
    let list_vendor = list_all_vendor(hardware_module);
    let vendor_name = match system_info::computer_info::get_vendor() {
        Ok(v) => v,
        Err(_) => return None,
    };
    let vendor = match HARDWARE_VENDOR_REPLACMENT
        .iter()
        .position(|(s, _)| s.contains(&vendor_name))
    {
        Some(i) => HARDWARE_VENDOR_REPLACMENT[i].1,
        None => {
            list_vendor[match list_vendor
                .iter()
                .position(|&s| vendor_name.to_lowercase().contains(s))
            {
                Some(index) => index,
                None => return None,
            }]
        }
    };

    let computer_module = hardware_module.get_computer_module();

    let vendor_hardware_module = {
        let b = computer_module
            .iter()
            .position(|s| s.starts_with(&vendor))
            .unwrap();
        let e = computer_module[b..]
            .iter()
            .position(|s| !s.starts_with(&vendor))
            .unwrap()
            + b;
        computer_module[b..e].as_ref()
    };
    println!("{:#?}", vendor_hardware_module);
    Some(get_fullpath_computer_hardware_module(
        hardware_module,
        vga_info,
        vendor_hardware_module,
        vendor,
        1,
    ))
}*/

fn get_hardware_module_path_rec<'a>(
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
            return get_hardware_module_path_rec(
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
        return get_hardware_module_path_rec(
            &hardware_module[begin.unwrap()..],
            computer_info,
            vga_info,
            depth + 1,
        );
    }
    return get_hardware_module_path_rec(
        &hardware_module[begin.unwrap()..end.unwrap()],
        computer_info,
        vga_info,
        depth + 1,
    );
}

fn get_hardware_module_path_family<'a>(
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
        return get_hardware_module_path_rec(
            &hardware_module[begin.unwrap()..],
            computer_info,
            vga_info,
            2,
        );
    }
    return get_hardware_module_path_rec(
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

fn get_hardware_module_path<'a>(
    hardware_module: &'a HardwareModule,
    computer_info: &ComputerInfo,
    vga_info: &VgaInfo,
) -> Option<&'a str> {
    let all_vendor = list_all_vendor(hardware_module);
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

    get_hardware_module_path_family(
        &hardware_module.get_computer_module()[begin..end],
        computer_info,
        vga_info,
    )
}

fn is_laptop() -> bool {
    let path = Path::new("/sys/class/power_supply");
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
        match get_hardware_module_path(&hardware_module, &computer_info, &vga_info) {
            Some(m) => m,
            None => "no module",
        }
    );

    // Exécution de la commande cpuid -1 pour obtenir toutes les couches de identification du CPU
    let output = match Command::new("cpuid").output() {
        Ok(out) => out,
        Err(_) => panic!("Failed to execute lspci command"),
    };

    // Attendre que la commande soit terminée et lire son output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().split('\n').collect();
    // Rechercher les lignes qui contiennent "synth"
    //
    let p = lines
        .iter()
        .rposition(|s| s.starts_with("   (synth)"))
        .unwrap();
    let pattern = Regex::new(r"\(.*?\)").unwrap();

    println!(
        "CPU Codename : {}",
        pattern
            .find(lines[p].strip_prefix("   (synth)").unwrap())
            .unwrap()
            .as_str()
            .strip_prefix("(")
            .unwrap()
            .strip_suffix(")")
            .unwrap()
    );

    println!("Is laptop : {}", is_laptop());
    // let a = vec![
    //     "acer/aspire/4810t/default.nix",
    //     "airis/n990/default.nix",
    //     "aoostar/r1/n100/default.nix",
    //     "apple/default.nix",
    //     "apple/imac/14-2/default.nix",
    //     "apple/imac/18-2/default.nix",
    //     "apple/imac/default.nix",
    //     "apple/macbook-air/3/default.nix",
    //     "apple/macbook-air/4/default.nix",
    //     "apple/macbook-air/6/default.nix",
    //     "apple/macbook-air/7/default.nix",
    //     "apple/macbook-air/default.nix",
    //     "apple/macbook-pro/10-1/default.nix",
    //     "apple/macbook-pro/11-1/default.nix",
    //     "apple/macbook-pro/11-5/default.nix",
    //     "apple/macbook-pro/12-1/default.nix",
    //     "apple/macbook-pro/14-1/default.nix",
    //     "apple/macbook-pro/8-1/default.nix",
    //     "apple/macbook-pro/default.nix",
    //     "apple/macmini/4/default.nix",
    //     "apple/macmini/default.nix",
    //     "apple/t2/default.nix",
    //     "apple/t2/pkgs/brcm-firmware/default.nix",
    //     "apple/t2/pkgs/linux-t2/default.nix",
    //     "asus/ally/rc71l/default.nix",
    //     "asus/fa506ic/default.nix",
    //     "asus/fa507nv/default.nix",
    //     "asus/fa507rm/default.nix",
    //     "asus/flow/gv302x/amdgpu/default.nix",
    //     "asus/flow/gv302x/nvidia/default.nix",
    //     "asus/fx504gd/default.nix",
    //     "asus/fx506hm/default.nix",
    //     "asus/pro-ws-x570-ace/default.nix",
    //     "asus/rog-strix/g513im/default.nix",
    //     "asus/rog-strix/g713ie/default.nix",
    //     "asus/rog-strix/g733qs/default.nix",
    //     "asus/rog-strix/x570e/default.nix",
    //     "asus/zenbook/ux371/default.nix",
    //     "asus/zenbook/ux535/default.nix",
    //     "asus/zephyrus/ga401/default.nix",
    //     "asus/zephyrus/ga402/default.nix",
    //     "asus/zephyrus/ga402x/amdgpu/default.nix",
    //     "asus/zephyrus/ga402x/default.nix",
    //     "asus/zephyrus/ga402x/nvidia/default.nix",
    //     "asus/zephyrus/ga502/default.nix",
    //     "asus/zephyrus/ga503/default.nix",
    //     "asus/zephyrus/gu603h/default.nix",
    //     "asus/zephyrus/gu605my/default.nix",
    //     "beagleboard/pocketbeagle/default.nix",
    //     "chuwi/minibook-x/default.nix",
    //     "deciso/dec/default.nix",
    //     "default.nix",
    //     "dell/e7240/default.nix",
    //     "dell/g3/3579/default.nix",
    //     "dell/g3/3779/default.nix",
    //     "dell/inspiron/14-5420/default.nix",
    //     "dell/inspiron/3442/default.nix",
    //     "dell/inspiron/5509/default.nix",
    //     "dell/inspiron/5515/default.nix",
    //     "dell/inspiron/7405/default.nix",
    //     "dell/inspiron/7460/default.nix",
    //     "dell/inspiron/7559/default.nix",
    //     "dell/latitude/3340/default.nix",
    //     "dell/latitude/3480/default.nix",
    //     "dell/latitude/5490/default.nix",
    //     "dell/latitude/5520/default.nix",
    //     "dell/latitude/7280/default.nix",
    //     "dell/latitude/7390/default.nix",
    //     "dell/latitude/7420/default.nix",
    //     "dell/latitude/7430/default.nix",
    //     "dell/latitude/7490/default.nix",
    //     "dell/latitude/9430/default.nix",
    //     "dell/latitude/e7240/default.nix",
    //     "dell/optiplex/3050/default.nix",
    //     "dell/poweredge/r7515/default.nix",
    //     "dell/precision/3541/default.nix",
    //     "dell/precision/3541/intel/default.nix",
    //     "dell/precision/5490/default.nix",
    //     "dell/precision/5530/default.nix",
    //     "dell/precision/5560/default.nix",
    //     "dell/precision/7520/default.nix",
    //     "dell/xps/13-7390/default.nix",
    //     "dell/xps/13-9300/default.nix",
    //     "dell/xps/13-9310/default.nix",
    //     "dell/xps/13-9315/default.nix",
    //     "dell/xps/13-9333/default.nix",
    //     "dell/xps/13-9343/default.nix",
    //     "dell/xps/13-9350/default.nix",
    //     "dell/xps/13-9360/default.nix",
    //     "dell/xps/13-9370/default.nix",
    //     "dell/xps/13-9380/default.nix",
    //     "dell/xps/15-7590/default.nix",
    //     "dell/xps/15-7590/nvidia/default.nix",
    //     "dell/xps/15-9500/default.nix",
    //     "dell/xps/15-9500/nvidia/default.nix",
    //     "dell/xps/15-9510/default.nix",
    //     "dell/xps/15-9510/nvidia/default.nix",
    //     "dell/xps/15-9520/default.nix",
    //     "dell/xps/15-9520/nvidia/default.nix",
    //     "dell/xps/15-9530/default.nix",
    //     "dell/xps/15-9530/nvidia/default.nix",
    //     "dell/xps/15-9550/default.nix",
    //     "dell/xps/15-9550/nvidia/default.nix",
    //     "dell/xps/15-9560/default.nix",
    //     "dell/xps/15-9560/intel/default.nix",
    //     "dell/xps/15-9560/nvidia/default.nix",
    //     "dell/xps/15-9570/default.nix",
    //     "dell/xps/15-9570/intel/default.nix",
    //     "dell/xps/15-9570/nvidia/default.nix",
    //     "dell/xps/17-9700/intel/default.nix",
    //     "dell/xps/17-9700/nvidia/default.nix",
    //     "dell/xps/17-9710/intel/default.nix",
    //     "dell/xps/sleep-resume/bluetooth/default.nix",
    //     "dell/xps/sleep-resume/i2c-designware/default.nix",
    //     "focus/m2/gen1/default.nix",
    //     "framework/13-inch/11th-gen-intel/default.nix",
    //     "framework/13-inch/12th-gen-intel/default.nix",
    //     "framework/13-inch/13th-gen-intel/default.nix",
    //     "framework/13-inch/7040-amd/default.nix",
    //     "framework/13-inch/common/default.nix",
    //     "framework/13-inch/intel-core-ultra-series1/default.nix",
    //     "framework/16-inch/7040-amd/default.nix",
    //     "framework/16-inch/common/default.nix",
    //     "framework/default.nix",
    //     "friendlyarm/nanopc-t4/default.nix",
    //     "friendlyarm/nanopi-r5s/default.nix",
    //     "gigabyte/b550/default.nix",
    //     "gigabyte/b650/default.nix",
    //     "google/pixelbook/default.nix",
    //     "gpd/micropc/default.nix",
    //     "gpd/p2-max/default.nix",
    //     "gpd/pocket-3/default.nix",
    //     "gpd/pocket-4/default.nix",
    //     "gpd/win-2/default.nix",
    //     "gpd/win-max-2/2023/bmi260/default.nix",
    //     "gpd/win-max-2/2023/default.nix",
    //     "gpd/win-max-2/default.nix",
    //     "gpd/win-mini/2024/default.nix",
    //     "gpd/win-mini/default.nix",
    //     "hardkernel/odroid-h3/default.nix",
    //     "hardkernel/odroid-h4/default.nix",
    //     "hardkernel/odroid-hc4/default.nix",
    //     "hp/elitebook/2560p/default.nix",
    //     "hp/elitebook/830/g6/default.nix",
    //     "hp/elitebook/845/g7/default.nix",
    //     "hp/elitebook/845/g8/default.nix",
    //     "hp/elitebook/845/g9/default.nix",
    //     "hp/laptop/14s-dq2024nf/default.nix",
    //     "hp/notebook/14-df0023/default.nix",
    //     "hp/probook/440G5/default.nix",
    //     "huawei/machc-wa/default.nix",
    //     "intel/nuc/8i7beh/default.nix",
    //     "kobol/helios4/default.nix",
    //     "lenovo/ideacentre/k330/default.nix",
    //     "lenovo/ideapad/15ach6/default.nix",
    //     "lenovo/ideapad/15alc6/default.nix",
    //     "lenovo/ideapad/15arh05/default.nix",
    //     "lenovo/ideapad/16ach6/default.nix",
    //     "lenovo/ideapad/16ahp9/default.nix",
    //     "lenovo/ideapad/16iah8/default.nix",
    //     "lenovo/ideapad/default.nix",
    //     "lenovo/ideapad/s145-15api/default.nix",
    //     "lenovo/ideapad/slim-5/default.nix",
    //     "lenovo/ideapad/z510/default.nix",
    //     "lenovo/legion/15ach6/default.nix",
    //     "lenovo/legion/15ach6h/default.nix",
    //     "lenovo/legion/15ach6h/hybrid/default.nix",
    //     "lenovo/legion/15ach6h/nvidia/default.nix",
    //     "lenovo/legion/15arh05h/default.nix",
    //     "lenovo/legion/15ich/default.nix",
    //     "lenovo/legion/16ach6h/default.nix",
    //     "lenovo/legion/16ach6h/edid/default.nix",
    //     "lenovo/legion/16ach6h/hybrid/default.nix",
    //     "lenovo/legion/16ach6h/nvidia/default.nix",
    //     "lenovo/legion/16achg6/hybrid/default.nix",
    //     "lenovo/legion/16achg6/nvidia/default.nix",
    //     "lenovo/legion/16aph8/default.nix",
    //     "lenovo/legion/16arha7/default.nix",
    //     "lenovo/legion/16irx8h/default.nix",
    //     "lenovo/legion/16irx9h/default.nix",
    //     "lenovo/legion/16ithg6/default.nix",
    //     "lenovo/legion/t526amr5/default.nix",
    //     "lenovo/thinkpad/a475/default.nix",
    //     "lenovo/thinkpad/default.nix",
    //     "lenovo/thinkpad/e14/amd/default.nix",
    //     "lenovo/thinkpad/e14/default.nix",
    //     "lenovo/thinkpad/e14/intel/default.nix",
    //     "lenovo/thinkpad/e15/default.nix",
    //     "lenovo/thinkpad/e15/intel/default.nix",
    //     "lenovo/thinkpad/e470/default.nix",
    //     "lenovo/thinkpad/e495/default.nix",
    //     "lenovo/thinkpad/l13/default.nix",
    //     "lenovo/thinkpad/l13/yoga/default.nix",
    //     "lenovo/thinkpad/l14/amd/default.nix",
    //     "lenovo/thinkpad/l14/default.nix",
    //     "lenovo/thinkpad/l14/intel/default.nix",
    //     "lenovo/thinkpad/l480/default.nix",
    //     "lenovo/thinkpad/p1/3th-gen/default.nix",
    //     "lenovo/thinkpad/p1/default.nix",
    //     "lenovo/thinkpad/p14s/amd/default.nix",
    //     "lenovo/thinkpad/p14s/amd/gen1/default.nix",
    //     "lenovo/thinkpad/p14s/amd/gen2/default.nix",
    //     "lenovo/thinkpad/p14s/amd/gen3/default.nix",
    //     "lenovo/thinkpad/p14s/amd/gen4/default.nix",
    //     "lenovo/thinkpad/p14s/default.nix",
    //     "lenovo/thinkpad/p14s/intel/default.nix",
    //     "lenovo/thinkpad/p14s/intel/gen3/default.nix",
    //     "lenovo/thinkpad/p14s/intel/gen5/default.nix",
    //     "lenovo/thinkpad/p16s/amd/default.nix",
    //     "lenovo/thinkpad/p16s/amd/gen1/default.nix",
    //     "lenovo/thinkpad/p16s/amd/gen2/default.nix",
    //     "lenovo/thinkpad/p16s/default.nix",
    //     "lenovo/thinkpad/p43s/default.nix",
    //     "lenovo/thinkpad/p50/default.nix",
    //     "lenovo/thinkpad/p51/default.nix",
    //     "lenovo/thinkpad/p52/default.nix",
    //     "lenovo/thinkpad/p53/default.nix",
    //     "lenovo/thinkpad/t14/amd/default.nix",
    //     "lenovo/thinkpad/t14/amd/gen1/default.nix",
    //     "lenovo/thinkpad/t14/amd/gen2/default.nix",
    //     "lenovo/thinkpad/t14/amd/gen3/default.nix",
    //     "lenovo/thinkpad/t14/amd/gen4/default.nix",
    //     "lenovo/thinkpad/t14/amd/gen5/default.nix",
    //     "lenovo/thinkpad/t14/default.nix",
    //     "lenovo/thinkpad/t14s/amd/default.nix",
    //     "lenovo/thinkpad/t14s/amd/gen1/default.nix",
    //     "lenovo/thinkpad/t14s/amd/gen4/default.nix",
    //     "lenovo/thinkpad/t14s/default.nix",
    //     "lenovo/thinkpad/t410/default.nix",
    //     "lenovo/thinkpad/t420/default.nix",
    //     "lenovo/thinkpad/t430/default.nix",
    //     "lenovo/thinkpad/t440p/default.nix",
    //     "lenovo/thinkpad/t440s/default.nix",
    //     "lenovo/thinkpad/t450s/default.nix",
    //     "lenovo/thinkpad/t460/default.nix",
    //     "lenovo/thinkpad/t460p/default.nix",
    //     "lenovo/thinkpad/t460s/default.nix",
    //     "lenovo/thinkpad/t470s/default.nix",
    //     "lenovo/thinkpad/t480/default.nix",
    //     "lenovo/thinkpad/t480s/default.nix",
    //     "lenovo/thinkpad/t490/default.nix",
    //     "lenovo/thinkpad/t490s/default.nix",
    //     "lenovo/thinkpad/t495/default.nix",
    //     "lenovo/thinkpad/t520/default.nix",
    //     "lenovo/thinkpad/t550/default.nix",
    //     "lenovo/thinkpad/t590/default.nix",
    //     "lenovo/thinkpad/w520/default.nix",
    //     "lenovo/thinkpad/x1-extreme/default.nix",
    //     "lenovo/thinkpad/x1-extreme/gen2/default.nix",
    //     "lenovo/thinkpad/x1-extreme/gen3/default.nix",
    //     "lenovo/thinkpad/x1-extreme/gen4/default.nix",
    //     "lenovo/thinkpad/x1-nano/default.nix",
    //     "lenovo/thinkpad/x1-nano/gen1/default.nix",
    //     "lenovo/thinkpad/x1/10th-gen/default.nix",
    //     "lenovo/thinkpad/x1/11th-gen/default.nix",
    //     "lenovo/thinkpad/x1/12th-gen/default.nix",
    //     "lenovo/thinkpad/x1/6th-gen/QHD/default.nix",
    //     "lenovo/thinkpad/x1/6th-gen/default.nix",
    //     "lenovo/thinkpad/x1/7th-gen/default.nix",
    //     "lenovo/thinkpad/x1/9th-gen/default.nix",
    //     "lenovo/thinkpad/x1/default.nix",
    //     "lenovo/thinkpad/x1/yoga/7th-gen/default.nix",
    //     "lenovo/thinkpad/x1/yoga/default.nix",
    //     "lenovo/thinkpad/x13-yoga/default.nix",
    //     "lenovo/thinkpad/x13/amd/default.nix",
    //     "lenovo/thinkpad/x13/default.nix",
    //     "lenovo/thinkpad/x13/intel/default.nix",
    //     "lenovo/thinkpad/x13/yoga/3th-gen/default.nix",
    //     "lenovo/thinkpad/x13/yoga/default.nix",
    //     "lenovo/thinkpad/x140e/default.nix",
    //     "lenovo/thinkpad/x200s/default.nix",
    //     "lenovo/thinkpad/x220/default.nix",
    //     "lenovo/thinkpad/x230/default.nix",
    //     "lenovo/thinkpad/x250/default.nix",
    //     "lenovo/thinkpad/x260/default.nix",
    //     "lenovo/thinkpad/x270/default.nix",
    //     "lenovo/thinkpad/x280/default.nix",
    //     "lenovo/thinkpad/x390/default.nix",
    //     "lenovo/thinkpad/z/default.nix",
    //     "lenovo/thinkpad/z/gen1/default.nix",
    //     "lenovo/thinkpad/z/gen1/z13/default.nix",
    //     "lenovo/thinkpad/z/gen2/default.nix",
    //     "lenovo/thinkpad/z/gen2/z13/default.nix",
    //     "lenovo/thinkpad/z13/default.nix",
    //     "lenovo/yoga/6/13ALC6/default.nix",
    //     "lenovo/yoga/7/14ARH7/amdgpu/default.nix",
    //     "lenovo/yoga/7/14ARH7/default.nix",
    //     "lenovo/yoga/7/14ARH7/nvidia/default.nix",
    //     "lenovo/yoga/7/14IAH7/hybrid/default.nix",
    //     "lenovo/yoga/7/14IAH7/integrated/default.nix",
    //     "lenovo/yoga/7/slim/gen8/default.nix",
    //     "malibal/aon/s1/default.nix",
    //     "microchip/icicle-kit/default.nix",
    //     "microsoft/surface-pro/3/default.nix",
    //     "microsoft/surface-pro/9/default.nix",
    //     "microsoft/surface/common/default.nix",
    //     "microsoft/surface/common/kernel/default.nix",
    //     "microsoft/surface/common/kernel/linux-surface/default.nix",
    //     "microsoft/surface/default.nix",
    //     "microsoft/surface/surface-go/default.nix",
    //     "microsoft/surface/surface-go/firmware/ath10k/default.nix",
    //     "microsoft/surface/surface-laptop-amd/default.nix",
    //     "microsoft/surface/surface-pro-intel/default.nix",
    //     "milkv/pioneer/default.nix",
    //     "minisforum/v3/default.nix",
    //     "morefine/m600/default.nix",
    //     "msi/b350-tomahawk/default.nix",
    //     "msi/b550-a-pro/default.nix",
    //     "msi/gl62/default.nix",
    //     "msi/gl65/10SDR-492/default.nix",
    //     "msi/gs60/default.nix",
    //     "nxp/imx8mp-evk/default.nix",
    //     "nxp/imx8mq-evk/default.nix",
    //     "nxp/imx8qm-mek/default.nix",
    //     "olimex/teres_i/default.nix",
    //     "omen/14-fb0798ng/default.nix",
    //     "omen/15-ce002ns/default.nix",
    //     "omen/15-en0002np/default.nix",
    //     "omen/15-en0010ca/default.nix",
    //     "omen/15-en1007sa/default.nix",
    //     "omen/16-n0005ne/default.nix",
    //     "omen/16-n0280nd/default.nix",
    //     "onenetbook/4/default.nix",
    //     "onenetbook/4/goodix-stylus-mastykin/default.nix",
    //     "panasonic/letsnote/cf-lx4/default.nix",
    //     "pcengines/apu/default.nix",
    //     "pine64/pinebook-pro/default.nix",
    //     "pine64/pinebook-pro/keyboard-updater/default.nix",
    //     "pine64/rockpro64/default.nix",
    //     "pine64/star64/default.nix",
    //     "protectli/vp4670/default.nix",
    //     "purism/librem/13v3/default.nix",
    //     "purism/librem/5r4/default.nix",
    //     "purism/librem/5r4/librem5-base/default.nix",
    //     "purism/librem/5r4/u-boot/default.nix",
    //     "radxa/default.nix",
    //     "radxa/rock-4c-plus/default.nix",
    //     "radxa/rock-5b/default.nix",
    //     "radxa/rock-pi-4/default.nix",
    //     "radxa/rock-pi-e/default.nix",
    //     "raspberry-pi/2/default.nix",
    //     "raspberry-pi/3/default.nix",
    //     "raspberry-pi/4/default.nix",
    //     "raspberry-pi/5/default.nix",
    //     "rockchip/default.nix",
    //     "rockchip/rk3328/default.nix",
    //     "rockchip/rk3399/default.nix",
    //     "rockchip/rk3588/default.nix",
    //     "samsung/np900x3c/default.nix",
    //     "slimbook/hero/rpl-rtx/default.nix",
    //     "starfive/visionfive/v1/default.nix",
    //     "starfive/visionfive/v2/default.nix",
    //     "starlabs/starlite/i5/default.nix",
    //     "supermicro/a1sri-2758f/default.nix",
    //     "supermicro/default.nix",
    //     "supermicro/m11sdv-8c-ln4f/default.nix",
    //     "supermicro/x10sll-f/default.nix",
    //     "supermicro/x12scz-tln4f/default.nix",
    //     "system76/darp6/default.nix",
    //     "system76/default.nix",
    //     "system76/galp5-1650/default.nix",
    //     "system76/gaze18/default.nix",
    //     "toshiba/swanky/default.nix",
    //     "tuxedo/aura/15/gen1/default.nix",
    //     "tuxedo/infinitybook/default.nix",
    //     "tuxedo/infinitybook/pro14/gen7/default.nix",
    //     "tuxedo/infinitybook/pro14/gen9-intel/default.nix",
    //     "tuxedo/infinitybook/v4/default.nix",
    //     "tuxedo/pulse/14/gen3/default.nix",
    //     "tuxedo/pulse/15/gen2/default.nix",
    //     "xiaomi/redmibook/16-pro-2024/default.nix",
    // ];
}
