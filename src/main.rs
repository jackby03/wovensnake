use wovensnake::core::config;

fn main() {
    let config = config::read_config("./../wovenpkg.json").expect("Failed to read config");
    println!("{:?}", config);
}