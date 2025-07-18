#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::process::Command;
#[cfg(any(target_os = "linux", target_os = "android"))]
use {
    crate::helpers::file::file_open,
    std::{fs::File, io::Read},
};

#[cfg(target_os = "windows")]
pub fn get_hostname() -> String {
    let mut hostname = String::new();

    let output = Command::new("reg")
        .args(&["query", "HKLM\\SYSTEM\\CurrentControlSet\\Control\\ComputerName\\ComputerName", "/v", "ComputerName"])
        .output()
        .expect("Failed to execute process");

    let output = String::from_utf8_lossy(&output.stdout);

    for line in output.lines() {
        if line.contains("ComputerName") {
            hostname = line.split_whitespace().last().unwrap().to_string();
        }
    }

    hostname
}

#[cfg(target_os = "macos")]
pub fn get_hostname() -> String {
    // macOS-compatible implementation
    use std::process::Command;

    let output = Command::new("scutil")
        .arg("--get")
        .arg("ComputerName")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok());

    output.unwrap_or_else(|| "unknown-macos".to_string()).trim().to_string()
}

#[cfg(target_os = "windows")]
pub fn get_user() -> String {
    let mut user = String::new();

    let output = Command::new("reg")
        .args(&["query", "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList", "/s"])
        .output()
        .expect("Failed to execute process");

    let output = String::from_utf8_lossy(&output.stdout);

    for line in output.lines() {
        if line.contains("ProfileImagePath") {
            user = line.split_whitespace().last().unwrap().to_string();
            user = user.split("\\").last().unwrap().to_string();
        }
    }

    user
}

#[cfg(target_os = "windows")]
pub fn get_shell() -> String {
    let mut shell = String::new();

    let output = Command::new("reg")
        .args(&["query", "HKCU\\Console", "/v", "FaceName"])
        .output()
        .expect("Failed to execute process");

    let output = String::from_utf8_lossy(&output.stdout);

    for line in output.lines() {
        if line.contains("FaceName") {
            shell = line.split_whitespace().last().unwrap().to_string();
        }
    }

    if shell == "Lucida Console" {
        shell = "PowerShell".to_string();
    } else {
        shell = "CMD".to_string();
    }

    shell
}

#[cfg(target_os = "windows")]
pub fn get_resolution() -> String {
    let mut temp_horiz = String::new();
    let mut temp_vert = String::new();

    let output = Command::new("wmic")
        .args(&[
            "path", 
            "Win32_VideoController", 
            "get", 
            "CurrentVerticalResolution,CurrentHorizontalResolution", 
            "/format:value"
        ])
        .output()
        .expect("Failed to execute process");

    let output = String::from_utf8_lossy(&output.stdout);

    let split_by_equals = |item: &str| -> String {
        item.split("=")
            .collect::<Vec<&str>>()
            .iter()
            .nth(1)
            .unwrap()
            .trim()
            .to_string()
    };

    for line in output.lines() {
        if line.contains("CurrentHorizontalResolution") {
            temp_horiz = split_by_equals(line);
        }

        if line.contains("CurrentVerticalResolution") {
            temp_vert = split_by_equals(line);
        }
    }

    format!(
        "{}x{}", 
        temp_horiz, 
        temp_vert
    ).to_string()
}

#[cfg(target_os = "macos")]
pub fn get_resolution() -> String {
    use std::process::Command;
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"Finder\" to get bounds of window of desktop")
        .output();

    match output {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout);
            // Output is: 0, 0, width, height
            let parts: Vec<&str> = out.trim().split(',').collect();
            if parts.len() == 4 {
                let w = parts[2].trim();
                let h = parts[3].trim();
                format!("{}x{}", w, h)
            } else {
                "unknown".to_string()
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn get_hostname() -> String {
    Command::new("hostname")
        .output()
        .expect("Failed to execute process")
        .stdout
        .iter()
        .map(|&c| c as char)
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn get_kernel() -> String {
    let mut kernel = String::new();

    let output = Command::new("uname")
        .args(&["-r"])
        .output()
        .expect("Failed to execute process");

    let output = String::from_utf8_lossy(&output.stdout);

    for line in output.lines() {
        kernel = line.to_string();
    }

    kernel
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn get_user() -> String {
    Command::new("whoami")
        .output()
        .expect("Failed to execute whoami")
        .stdout
        .iter()
        .map(|&c| c as char)
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(target_os = "macos")]
pub fn get_user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "unknown-user".into())
}

#[cfg(any(target_os = "linux", target_os = "android"))] 
pub fn get_distro() -> String {
    use std::rc::Rc;

    let mut distro: Rc<String> = Rc::new(String::new());
    let mut temp_buf: String = String::new();

    let mut file = File::open("/etc/os-release").unwrap();
    file.read_to_string(&mut temp_buf).unwrap();

    let lines: &Vec<&str> = &temp_buf.lines().collect();
    
    lines.into_iter().for_each(|line| {
        if line.contains("PRETTY_NAME") {
            distro = Rc::new(
                line.split("=")
                    .collect::<Vec<&str>>()[1].to_string()
                    .replace("\"", "")
            );
        }

        if line.contains("BUILD_ID") {
            distro = Rc::new(
                format!("{} ({})", distro, 
                    line.split("=")
                        .collect::<Vec<&str>>()[1].to_string()
                        .replace("\"", "")
                )
            );
        }
    });

    distro.to_string()
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn get_shell() -> String {
    let temp_buf: String = file_open("/etc/passwd");
    let mut final_str = String::new();

    let lines: &Vec<&str> = &temp_buf.lines().collect();

    lines.into_iter().for_each(|line| {
        if line.contains(&get_user()) {
            final_str = line.split(":")
                .collect::<Vec<&str>>()[6]
                .to_string();
        }
    });

    final_str
}

#[cfg(target_os = "macos")]
pub fn get_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "unknown".into())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn get_resolution() -> String {
    let mut final_str = String::new();

    let output = Command::new("xrandr")
        .output()
        .expect("Failed to execute xrandr");

    let output = String::from_utf8(output.stdout).unwrap();

    let lines: &Vec<&str> = &output.lines().collect();

    lines.into_iter().for_each(|line| {
        if line.contains(" connected") {
            final_str = line.split(" ")
                .collect::<Vec<&str>>()[2]
                .to_string();
        }
    });

    final_str
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn get_init_system() -> String {
    Command::new("ps")
        .args(&["-p", "1", "-o", "comm="])
        .output()
        .unwrap()
        .stdout
        .iter()
        .map(|&c| c as char)
        .collect::<String>()
        .trim().to_string()
}
