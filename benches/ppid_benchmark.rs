#![feature(test)]

extern crate test;

#[cfg(test)]

mod tests {
    use super::*;
    use nix::unistd::getppid;
    use test::Bencher;

    #[bench]
    pub fn simple_syscall(b: &mut Bencher) {
        b.iter(|| {
            test::black_box(getppid());
        })
    }
}
