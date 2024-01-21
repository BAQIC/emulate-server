use std::process::Command;

fn main() {
    let output = Command::new(
        "/home/lucky/Code/cuda-quantum/docs/sphinx/examples/cpp/providers/out-emulate.x",
    )
    .arg("/home/lucky/Code/cuda-quantum/docs/sphinx/examples/cpp/providers/emulate_message")
    .output()
    .expect("failed to execute process");

    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
}
