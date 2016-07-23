#![feature(test)]

extern crate test;
extern crate rand;
extern crate newhope;

use test::Bencher;
use rand::{ Rng, OsRng, ChaChaRng };
use newhope::{
    N,
    keygen, sharedb, shareda
};


#[bench]
fn bench_keygen(b: &mut Bencher) {
    let (mut sk, mut pk) = ([0; N], [0; N]);
    let (mut nonce, mut rng) = ([0; 32], OsRng::new().unwrap().gen::<ChaChaRng>());
    rng.fill_bytes(&mut nonce);

    b.iter(|| keygen(&mut sk, &mut pk, &nonce, rng));
}

#[bench]
fn bench_kex_b(b: &mut Bencher) {
    let (mut output, mut pkb, mut rec) = ([0; N], [0; N], [0; N]);
    let (mut sk, mut pka) = ([0; N], [0; N]);
    let (mut nonce, mut rng) = ([0; 32], OsRng::new().unwrap().gen::<ChaChaRng>());
    rng.fill_bytes(&mut nonce);
    keygen(&mut sk, &mut pka, &nonce, rng);

    b.iter(|| sharedb(&mut output, &mut pkb, &mut rec, &pka, &nonce, rng));
}

#[bench]
fn bench_kex_a(b: &mut Bencher) {
    let (mut output, mut pkb, mut rec) = ([0; N], [0; N], [0; N]);
    let (mut sk, mut pka) = ([0; N], [0; N]);
    let (mut nonce, mut rng) = ([0; 32], OsRng::new().unwrap().gen::<ChaChaRng>());
    rng.fill_bytes(&mut nonce);
    keygen(&mut sk, &mut pka, &nonce, rng);
    sharedb(&mut output, &mut pkb, &mut rec, &pka, &nonce, rng);

    b.iter(|| shareda(&mut output, &sk, &pkb, &rec));
}
