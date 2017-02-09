#![feature(test)]

extern crate test;
extern crate libc;
extern crate cnewhope;

use test::Bencher;
use libc::{ free, c_void };
use cnewhope::*;

#[bench]
fn bench_cnewhope_keygen(b: &mut Bencher) {
    b.iter(|| unsafe {
        let mut senda = [0; SENDABYTES];
        let ska = newhope_keygen_poly(senda.as_mut_ptr());
        free(ska as *mut c_void);
    });
}

#[bench]
fn bench_cnewhope_sharedb(b: &mut Bencher) {
    let (mut senda, mut sendb) = ([0; SENDABYTES], [0; SENDBBYTES]);
    let mut keyb = [0; 32];
    let ska = unsafe { newhope_keygen_poly(senda.as_mut_ptr()) };
    unsafe { free(ska as *mut c_void) };
    b.iter(|| unsafe {
        newhope_sharedb(keyb.as_mut_ptr(), sendb.as_mut_ptr(), senda.as_ptr())
    });
}

#[bench]
fn bench_cnewhope_shareda(b: &mut Bencher) {
    let (mut senda, mut sendb) = ([0; SENDABYTES], [0; SENDBBYTES]);
    let (mut keya, mut keyb) = ([0; 32], [0; 32]);
    let ska = unsafe { newhope_keygen_poly(senda.as_mut_ptr()) };
    unsafe { newhope_sharedb(keyb.as_mut_ptr(), sendb.as_mut_ptr(), senda.as_ptr()) };
    b.iter(|| unsafe {
        newhope_shareda(keya.as_mut_ptr(), ska, sendb.as_ptr())
    });
    unsafe { free(ska as *mut c_void) };
}
