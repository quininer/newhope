#![feature(test)]

extern crate test;
extern crate rand;
extern crate newhope;

use test::Bencher;
use rand::{ Rng, OsRng, ChaChaRng };
use newhope::*;

#[bench]
fn bench_newhope_keygen(b: &mut Bencher) {
    let (mut sk, mut pk) = ([0; N], [0; N]);
    let mut nonce = [0; 32];
    let mut rng = OsRng::new().unwrap().gen::<ChaChaRng>();

    b.iter(|| {
        rng.fill_bytes(&mut nonce);
        keygen(&mut sk, &mut pk, &nonce, rng.gen::<ChaChaRng>());
    });
}

#[bench]
fn bench_newhope_sharedb(b: &mut Bencher) {
    let (mut ska, mut pka) = ([0; N], [0; N]);
    let (mut pkb, mut rec) = ([0; N], [0; N]);
    let mut nonce = [0; 32];
    let mut keyb = [0; 32];
    let mut output = [0; 32];
    let mut rng = OsRng::new().unwrap().gen::<ChaChaRng>();
    rng.fill_bytes(&mut nonce);
    keygen(&mut ska, &mut pka, &nonce, rng.gen::<ChaChaRng>());

    b.iter(|| {
        sharedb(
            &mut keyb, &mut pkb, &mut rec,
            &pka, &nonce, rng.gen::<ChaChaRng>()
        );
        sha3_256(&mut output, &keyb)
    });
}

#[bench]
fn bench_newhope_shareda(b: &mut Bencher) {
    let (mut ska, mut pka) = ([0; N], [0; N]);
    let (mut pkb, mut rec) = ([0; N], [0; N]);
    let (mut keya, mut keyb) = ([0; 32], [0; 32]);
    let mut nonce = [0; 32];
    let mut output = [0; 32];

    let mut rng = OsRng::new().unwrap().gen::<ChaChaRng>();
    rng.fill_bytes(&mut nonce);
    keygen(&mut ska, &mut pka, &nonce, rng.gen::<ChaChaRng>());
    sharedb(
        &mut keyb, &mut pkb, &mut rec,
        &pka, &nonce, rng.gen::<ChaChaRng>()
    );

    b.iter(|| {
        shareda(&mut keya, &ska, &pkb, &rec);
        sha3_256(&mut output, &keya);
    })
}
