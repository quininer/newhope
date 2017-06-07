#![feature(test)]

extern crate test;
extern crate libc;
extern crate cnewhope;

use test::Bencher;
use cnewhope::*;

#[bench]
fn bench_cnewhope_keygen(b: &mut Bencher) {
    b.iter(|| unsafe {
        let mut senda = [0; SENDABYTES];
        let mut ska = Poly::default();
        newhope_keygen(senda.as_mut_ptr(), &mut ska);
    });
}

#[bench]
fn bench_cnewhope_sharedb(b: &mut Bencher) {
    let (mut senda, mut sendb) = ([0; SENDABYTES], [0; SENDBBYTES]);
    let mut keyb = [0; 32];
    let mut ska = Poly::default();
    unsafe { newhope_keygen(senda.as_mut_ptr(), &mut ska) };
    drop(ska);
    b.iter(|| unsafe {
        newhope_sharedb(keyb.as_mut_ptr(), sendb.as_mut_ptr(), senda.as_ptr())
    });
}

#[bench]
fn bench_cnewhope_shareda(b: &mut Bencher) {
    let (mut senda, mut sendb) = ([0; SENDABYTES], [0; SENDBBYTES]);
    let (mut keya, mut keyb) = ([0; 32], [0; 32]);
    let mut ska = Poly::default();
    unsafe { newhope_keygen(senda.as_mut_ptr(), &mut ska) };
    unsafe { newhope_sharedb(keyb.as_mut_ptr(), sendb.as_mut_ptr(), senda.as_ptr()) };
    b.iter(|| unsafe {
        newhope_shareda(keya.as_mut_ptr(), &ska, sendb.as_ptr())
    });
}
