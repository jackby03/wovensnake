use std::fmt::Display;

pub fn print_success<T: Display>(msg: T) {
    println!("\x1b[1m\x1b[32m✓\x1b[0m {msg}");
}

pub fn print_error<T: Display>(msg: T) {
    eprintln!("\x1b[1m\x1b[31m✗ Error:\x1b[0m {msg}");
    if std::env::var("RUST_LOG").unwrap_or_default() == "debug" {
        eprintln!("\x1b[90m[DEBUG] Context: RUST_BACKTRACE=1 enabled in environment.\x1b[0m");
    }
}

pub fn print_warning<T: Display>(msg: T) {
    println!("\x1b[1m\x1b[33m! Warning:\x1b[0m {msg}");
}

pub fn print_info<T: Display>(msg: T) {
    println!("\x1b[1m\x1b[36m•\x1b[0m {msg}");
}

pub fn print_header(msg: &str) {
    println!("\n\x1b[1m\x1b[36m🐍 WovenSnake\x1b[0m \x1b[90m| {msg}\x1b[0m\n");
}
