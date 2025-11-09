pub fn log(print_log: String) {
    let now_debug = chrono::Local::now();
    let date_debug = now_debug.format("%Y-%m-%d %H:%M:%S").to_string();
    println!("[ {} ] {}", date_debug, print_log);
}
