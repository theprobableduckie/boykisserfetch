#[cfg(any(target_os = "linux", target_os = "android"))]
use {
    crate::helpers::file::file_open,
    std::sync::Mutex,
    std::ops::Add,
};
use std::process::Command;

#[cfg(target_os = "windows")]
pub fn get_ipaddr() -> String {
    let mut ipaddr = String::new();

    let output = Command::new("ipconfig")
        .args(&["/all"])
        .output()
        .expect("Failed to execute process");

    let output = String::from_utf8_lossy(&output.stdout);

    for line in output.lines() {
        if line.contains("IPv4 Address") {
            ipaddr = line.split_whitespace().last().unwrap().to_string();
        }
    }

    ipaddr
}

#[cfg(target_os = "macos")]
pub fn get_ipaddr() -> String {
    use std::net::UdpSocket;
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("8.8.8.8:80")?;
            s.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "unknown".into())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn get_ipaddr() -> String {
    let final_str: Mutex<String> = Mutex::new(String::new());
    let intr = file_open("/proc/net/route");

    let lines: &Vec<&str> = &intr.lines().collect();
    let mut interface = String::new();

    lines.into_iter().for_each(|line| {
        if line.contains("00000000") {
            interface = line.split("\t").collect::<Vec<&str>>()[0].to_string();
        }
    });

    let output = Command::new("ifconfig")
        .arg(interface.clone())
        .output();
    
    if output.is_err() {
        return String::from("Unknown");
    };

    let unw = output.unwrap().stdout;
    
    let stdout = String::from_utf8_lossy(&unw);

    let lines: &Vec<&str> = &stdout.lines().clone().collect();

    let mut next: bool = false;

    let process_ip = |line: &str| {
        let ip = line.split(" ").collect::<Vec<&str>>()[1].to_string();
        final_str.lock().unwrap().push_str(&ip);
    };

    lines.into_iter().for_each(|line| {
        if next {
            line.replace("\t", "")
                .split("  ")
                .collect::<Vec<&str>>()
                .into_iter()
                .for_each(|item| {
                    if item.contains("inet") {
                        process_ip(item);
                    }
                });

            next = false;
        }

        if line.contains(&interface) {
            next = !next;
        }
    });

    let x = final_str
        .lock()
        .unwrap()
        .to_string()
        .add(format!(" ({})", interface).as_str()); 
        
    x
}
