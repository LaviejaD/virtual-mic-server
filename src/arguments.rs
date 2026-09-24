pub fn get(txt: String) -> bool {
    let args: Vec<String> = std::env::args().collect();

    return match args.iter().position(|arg| arg == &txt) {
        Some(_e) => true,
        None => false,
    };
}
