#![allow(non_camel_case_types)]
use helpers::arguments::Arguments;
use helpers::boykissers::get_boykisser;

mod helpers;
mod system;

#[derive(Clone, Copy, Debug)]
pub enum ActionType {
    HostInfo,
    Delimiter,
    Details,
    Colors
}

#[derive(Debug)]
pub struct Action<'a> {
    action_type: ActionType,
    name: Option<&'a str>,
    func: Option<fn() -> String>,
}
fn get_uptime() -> String {
    #[cfg(target_os = "linux")]
    {
        let uptime = std::fs::read_to_string("/proc/uptime")
            .unwrap_or_else(|_| "0.0".to_string());
        let uptime_seconds: f64 = uptime.split_whitespace().next().unwrap_or("0").parse().unwrap();
        let hours = (uptime_seconds / 3600.0).floor();
        let minutes = ((uptime_seconds % 3600.0) / 60.0).floor();
        format!("{} hours, {} minutes", hours, minutes)
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("wmic")
            .arg("os")
            .arg("get")
            .arg("LastBootUpTime")
            .output()
            .expect("Failed to execute command");
        
        let boot_time_str = String::from_utf8_lossy(&output.stdout);
        let boot_time = boot_time_str.lines().nth(1).unwrap_or("0").trim();
        
        // Parse the boot time and calculate uptime
        let boot_time = chrono::DateTime::parse_from_str(boot_time, "%Y%m%d%H%M%S.%f%z").unwrap();
        let uptime = chrono::Utc::now().signed_duration_since(boot_time);
        format!("{} hours, {} minutes", uptime.num_hours(), uptime.num_minutes() % 60)
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sysctl")
            .arg("kern.boottime")
            .output()
            .expect("Failed to execute command");
        
        let boot_time_str = String::from_utf8_lossy(&output.stdout);
        let boot_time = boot_time_str.split_whitespace().nth(3).unwrap_or("0").trim_matches(',');
        
        // Parse the boot time and calculate uptime
        let boot_time = chrono::DateTime::from_utc(
            chrono::NaiveDateTime::from_timestamp(boot_time.parse::<i64>().unwrap(), 0),
            chrono::Utc,
        );
        let uptime = chrono::Utc::now().signed_duration_since(boot_time);
        format!("{} hours, {} minutes", uptime.num_hours(), uptime.num_minutes() % 60)
    }
}

fn get_gpus() -> String {
    let mut gpus = Vec::new();

    #[cfg(target_os = "linux")]
    {
        // Use lspci to list all GPUs
        let output = std::process::Command::new("lspci")
            .arg("-nn")
            .output()
            .expect("Failed to execute command");
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("VGA compatible controller") || line.contains("3D controller") {
                gpus.push(format!("GPU          : {}", line.trim()));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Use wmic to list all GPUs
        let output = std::process::Command::new("wmic")
            .arg("path")
            .arg("win32_VideoController")
            .arg("get")
            .arg("name")
            .output()
            .expect("Failed to execute command");
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines().skip(1) { // Skip the header
            let gpu = line.trim();
            if !gpu.is_empty() {
                gpus.push(format!("GPU          : GPU: {}", gpu));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Use system_profiler to list all GPUs
        let output = std::process::Command::new("system_profiler")
            .arg("SPDisplaysDataType")
            .output()
            .expect("Failed to execute command");
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("Chipset Model:") {
                let gpu = line.replace("Chipset Model:", "").trim();
                gpus.push(format!("GPU          : GPU: {}", gpu));
            }
        }
    }

    // Join all GPU names with a newline character
    gpus.join("\n")
}


const ACTIONS: [Action; 14] = [
    Action {
        action_type: ActionType::HostInfo,
        name: None,
        func: Some(system::host::get_hostname),
    },
    Action {
        action_type: ActionType::Delimiter,
        name: None,
        func: None,
    },
    #[cfg(target_os = "linux")]
    Action {
        action_type: ActionType::Details,
        name: Some("Distro"),
        func: Some(system::host::get_distro),
    },
    #[cfg(target_os = "windows")]
    Action {
        action_type: ActionType::Details,
        name: Some("Product"),
        func: Some(system::specs::get_kernel),
    },
    #[cfg(target_os = "linux")]
    Action {
        action_type: ActionType::Details,
        name: Some("Kernel"),
        func: Some(system::host::get_kernel),
    },
    Action {
        action_type: ActionType::Details,
        name: Some("Arch"),
        func: Some(system::specs::get_arch),
    },
    Action {
        action_type: ActionType::Details,
        name: Some("Shell"),
        func: Some(system::host::get_shell),
    },
    Action {
        action_type: ActionType::Details,
        name: Some("Resolution"),
        func: Some(system::host::get_resolution),
    },
    Action {
        action_type: ActionType::Details,
        name: Some("IP"),
        func: Some(system::net::get_ipaddr),
    },
    Action {
        action_type: ActionType::Details,
        name: Some("CPU"),
        func: Some(system::specs::get_cpu),
    },
    #[cfg(target_os = "windows")]
    Action {
        action_type: ActionType::Details,
        name: Some("Disk usage"),
        func: Some(system::specs::get_disk_usage),
    },

    #[cfg(target_os = "windows")]
    Action {
        action_type: ActionType::Details,
        name: Some("GPU"),
        func: Some(system::specs::get_gpu),
    },
    Action {
        action_type: ActionType::Details,
        name: Some("GPU"),
        func: Some(get_gpus), // Use the updated function defined above
    },
    #[cfg(target_os = "linux")]
    Action {
        action_type: ActionType::Details,
        name: Some("Init System"),
        func: Some(system::host::get_init_system),
    },
    Action {
    action_type: ActionType::Details,
    name: Some("Uptime"),
    func: Some(get_uptime),
    },
    Action {
        action_type: ActionType::Delimiter,
        name: None,
        func: None,
    },
    Action {
        action_type: ActionType::Colors,
        name: None,
        func: None,
    }
];

fn main() {
    let args = Arguments::parse();
    let boykisser = get_boykisser(args.boykisser).unwrap();

    let to_skip = ((boykisser.lines / 2) as f32).floor() - 6.0;

    for i in 0..boykisser.lines {
        helpers::print::print_boykisserline(i, &boykisser.text, &args.color);

        let pad_i = (i as f32 - to_skip).floor();

        if ACTIONS.get(pad_i as usize).is_none() || pad_i < 0.0 {
            println!();
            continue;
        }

        match ACTIONS[pad_i as usize].action_type {
            ActionType::HostInfo => {
                helpers::print::print_detail(
                    &system::host::get_user(),
                    system::host::get_hostname(),
                    ActionType::HostInfo,
                    &args.color
                );
            },
                    
            ActionType::Delimiter => {
                helpers::print::print_detail(
                    "",
                    "".to_string(),
                    ActionType::Delimiter,
                    args.color.as_str()
                );
            },
              
            ActionType::Details => {
                helpers::print::print_detail(
                    ACTIONS[pad_i as usize].name.unwrap(),
                    ACTIONS[pad_i as usize].func.unwrap()(),
                    ACTIONS[pad_i as usize].action_type,
                    args.color.as_str()
                );
            },

            ActionType::Colors => {
                helpers::print::print_detail(
                    "",
                    "".to_string(),
                    ActionType::Colors,
                    args.color.as_str()
                );
            }
        }

        println!();
    }
}
