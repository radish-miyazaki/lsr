fn main() {
    if let Err(e) = lsr::run() {
        eprint!("{}", e);
        std::process::exit(1);
    }
}
