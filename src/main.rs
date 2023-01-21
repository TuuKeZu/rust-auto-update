use create_process_w::Command;

fn main() {
    let child = Command::new("version-cache/data-analysis.exe")
    .status()
    .expect("failed to start process");

    if child.success() {
        println!("Success!");
    } else {
        println!("Process exited with status {}", child.code());
    }
}
