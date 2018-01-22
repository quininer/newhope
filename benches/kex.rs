#![feature(test)]

extern crate test;
extern crate rand;
extern crate newhope;

use test::Bencher;
use rand::{ Rng, OsRng, ChaChaRng };
use newhope::params;
use newhope::*;

#[bench]
fn bench_newhope_keygen(b: &mut Bencher) {
    let mut rng = OsRng::new().unwrap().gen::<ChaChaRng>();

    b.iter(|| {
        let (mut ska, mut pka) = ([0; params::POLY_BYTES], [0; params::SENDABYTES]);
        keygen(&mut rng, &mut ska, &mut pka);
    });
}

#[bench]
fn bench_newhope_sharedb(b: &mut Bencher) {
    let mut rng = OsRng::new().unwrap().gen::<ChaChaRng>();
    let (mut ska, mut pka) = ([0; params::POLY_BYTES], [0; params::SENDABYTES]);
    let mut key = [0; 32];
    let mut sendb = [0; params::SENDBBYTES];
    keygen(&mut rng, &mut ska, &mut pka);

    b.iter(|| {
        sharedb(&mut rng, &mut key, &pka, &mut sendb);
    });
}

#[bench]
fn bench_newhope_shareda(b: &mut Bencher) {
    let mut rng = OsRng::new().unwrap().gen::<ChaChaRng>();
    let (mut ska, mut pka) = ([0; params::POLY_BYTES], [0; params::SENDABYTES]);
    let mut key = [0; 32];
    let mut sendb = [0; params::SENDBBYTES];
    keygen(&mut rng, &mut ska, &mut pka);
    sharedb(&mut rng, &mut key, &pka, &mut sendb);

    b.iter(|| {
        shareda(&mut key, &ska, &sendb);
    });
}
