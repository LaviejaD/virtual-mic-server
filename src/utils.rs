use rand::RngExt;
pub fn generate_pin(digits: usize) -> String {
    let mut rng = rand::rng();
    (0..digits)
        .map(|_| rng.random_range(0..digits).to_string())
        .collect()
}
