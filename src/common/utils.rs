use std::fs;

pub fn is_raspberry_pi_4b() -> bool {
    let model_path = "/proc/device-tree/model";
    let cpuinfo_path = "/proc/cpuinfo";

    // Try reading from /proc/device-tree/model
    if let Ok(contents) = fs::read_to_string(model_path) {
        return contents.contains("Raspberry Pi 4 Model B");
    }

    // Fallback to /proc/cpuinfo
    if let Ok(contents) = fs::read_to_string(cpuinfo_path) {
        return contents.contains("Raspberry Pi 4 Model B");
    }

    false
}
