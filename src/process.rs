use std::env;

pub enum ProcessType {
    Windows,
    Linux,
    Unsupported
}

pub fn get_os() -> ProcessType {
    match env::consts::OS {
        "linux" => ProcessType::Linux,
        "windows" => ProcessType::Windows,
        _ => ProcessType::Unsupported
    }
}