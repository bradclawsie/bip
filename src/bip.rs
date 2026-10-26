fn main() {
    if std::env::args_os().skip(1).any(|arg| arg == "--help") {
        println!("{}", env!("BIP_COMMIT_HASH"));
        return;
    }
    println!("bip!");
}
