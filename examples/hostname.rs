use nix::unistd;

fn main() {
    let hostname = unistd::gethostname().unwrap();
    println!("{hostname:?}");
}
