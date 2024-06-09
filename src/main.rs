extern crate winreg;
use std::io;
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

fn sort<T: Ord>(mut vec: Vec<T>) -> Vec<T> {
    if vec.len() <= 1 {
        return vec;
    }

    let pivot = vec.remove(0);
    let mut left = vec![];
    let mut right = vec![];

    for item in vec {
        if item <= pivot {
            left.push(item);
        } else {
            right.push(item);
        }
    }

    let mut sorted_left = sort(left);
    let mut sorted_right = sort(right);

    sorted_left.push(pivot);
    sorted_left.append(&mut sorted_right);

    sorted_left
}

fn main() -> io::Result<()> {
    // Define the registry paths to check
    let paths = [
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_CURRENT_USER,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];

    let mut apps = Vec::new();

    for (hive, path) in &paths {
        let reg_key = RegKey::predef(*hive);
        if let Ok(key) = reg_key.open_subkey_with_flags(path, KEY_READ) {
            for subkey_name in key.enum_keys().map(|x| x.unwrap()) {
                let subkey = key.open_subkey_with_flags(&subkey_name, KEY_READ).unwrap();
                match subkey.get_value::<String, _>("DisplayName") {
                    Ok(name) => apps.push(name),
                    Err(_) => (),
                }
            }
        }
    }

    let output = Command::new("powershell")
        .arg("-Command")
        .arg("Get-StartApps | Select-Object -ExpandProperty Name")
        .output()
        .expect("Failed to execute PowerShell command");

    if output.status.success() {
        let uwp_apps = String::from_utf8_lossy(&output.stdout);
        for app in uwp_apps.lines() {
            apps.push(app.to_string());
        }
    } else {
        eprintln!(
            "Failed to list UWP applications: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    for name in sort(apps) {
        println!("{:?}", name);
    }
    Ok(())
}
