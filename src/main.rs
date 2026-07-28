mod graphics;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let test_mode = args.iter().any(|arg| arg == "--test-mode" || arg == "--model=test");
    graphics::run(test_mode);
}
