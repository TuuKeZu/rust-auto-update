use std::env;

pub enum ProcessType {
    Windows,
    Unix,
    Unsupported
}

pub fn get_os() -> ProcessType {
    match env::consts::OS {
        "linux" => ProcessType::Unix,
        "macos" => ProcessType::Unix,
        "windows" => ProcessType::Windows,
        _ => ProcessType::Unsupported
    }
}