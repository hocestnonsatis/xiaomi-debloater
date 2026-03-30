use anyhow::{anyhow, Context, Result};
use std::collections::HashSet;
use std::process::Command;

fn adb_base(serial: Option<&str>) -> Command {
    let mut cmd = Command::new("adb");
    if let Some(s) = serial {
        cmd.arg("-s").arg(s);
    }
    cmd
}

pub fn adb_version() -> Result<String> {
    let out = Command::new("adb")
        .arg("version")
        .output()
        .context("failed to run `adb` — is Android Platform Tools installed and in PATH?")?;
    if !out.status.success() {
        return Err(anyhow!(
            "adb version failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Returns the serial of the device to use (single device, or ANDROID_SERIAL, or first `device`).
pub fn resolve_device_serial() -> Result<Option<String>> {
    let out = adb_base(None)
        .args(["devices", "-l"])
        .output()
        .context("adb devices failed")?;
    if !out.status.success() {
        return Err(anyhow!("adb devices failed"));
    }
    let out_text = String::from_utf8_lossy(&out.stdout).into_owned();
    let lines: Vec<&str> = out_text
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .collect();

    let mut devices: Vec<&str> = Vec::new();
    for line in lines {
        let mut parts = line.split_whitespace();
        let serial = parts.next().unwrap_or("");
        let state = parts.next().unwrap_or("");
        if state == "device" {
            devices.push(serial);
        }
    }

    if devices.is_empty() {
        return Err(anyhow!(
            "No device in `adb devices` state `device`. Enable USB debugging and authorize this computer."
        ));
    }

    if let Ok(pref) = std::env::var("ANDROID_SERIAL") {
        if devices.contains(&pref.as_str()) {
            return Ok(Some(pref));
        }
    }

    if devices.len() == 1 {
        return Ok(Some(devices[0].to_string()));
    }

    Err(anyhow!(
        "Multiple devices attached ({}). Set ANDROID_SERIAL to one of: {}",
        devices.len(),
        devices.join(", ")
    ))
}

pub fn get_prop(serial: Option<&str>, key: &str) -> Result<String> {
    let out = adb_base(serial)
        .args(["shell", "getprop", key])
        .output()
        .context("adb shell getprop failed")?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub fn list_installed_packages(serial: Option<&str>) -> Result<HashSet<String>> {
    let out = adb_base(serial)
        .args(["shell", "pm", "list", "packages"])
        .output()
        .context("pm list packages failed")?;
    if !out.status.success() {
        return Err(anyhow!(
            "pm list packages failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let mut set = HashSet::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("package:") {
            set.insert(rest.trim().to_string());
        }
    }
    Ok(set)
}

pub fn uninstall_user_zero(serial: Option<&str>, package: &str) -> Result<String> {
    let out = adb_base(serial)
        .args(["shell", "pm", "uninstall", "--user", "0", package])
        .output()
        .context("pm uninstall failed")?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    let combined = if stderr.is_empty() {
        stdout
    } else if stdout.is_empty() {
        stderr
    } else {
        format!("{stdout}\n{stderr}")
    };
    if !out.status.success() && !combined.to_lowercase().contains("success") {
        return Err(anyhow!("{package}: {combined}"));
    }
    Ok(combined)
}
