fn main() {
    if let Err(error) = mdir4::run() {
        eprintln!("mdir4: {error}");
        std::process::exit(1);
    }
}
