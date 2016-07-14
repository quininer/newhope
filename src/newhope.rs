use ::params::N;
use ::error_correction::{ helprec, rec };
use ::poly::{
    uniform, noise, pointwise, add,
    ntt, invntt
};


fn offer_computation(pk: &mut [u16], s: &[u16], e: &[u16], a: &[u16]) {
    let mut r = [0; N];
    pointwise(&mut r, s, a);
    add(pk, e, &r);
}

fn accept_computation(
    key: &mut [u8], bp: &mut [u16], c: &mut [u16],
    sp: &[u16], ep: &[u16], epp: &[u16], pk: &[u16], a: &[u16]
) {
    let (mut v, mut t) = ([0; N], [0; N]);

    pointwise(&mut t, a, sp);
    add(bp, &t, ep);

    pointwise(&mut t, pk, sp);
    invntt(&mut t);
    add(&mut v, &t, epp);
    helprec(c, &v);
    rec(key, &v, c);
}

fn finish_computation(key: &mut [u8], sk: &[u16], bp: &[u16], c: &[u16]) {
    let mut v = [0; N];
    pointwise(&mut v, sk, bp);
    invntt(&mut v);
    rec(key, &v, c);
}

pub fn keygen(seed: &[u8]) -> ([u16; N], [u16; N]) {
    let (mut sk, mut pk) = ([0; N], [0; N]);
    let (mut a, mut e) = ([0; N], [0; N]);

    uniform(&mut a, seed);

    noise(&mut sk);
    ntt(&mut sk);

    noise(&mut e);
    ntt(&mut e);

    offer_computation(&mut pk, &sk, &e, &a);

    (sk, pk)
}

pub fn sharedb(pka: &[u16], seed: &[u8]) -> ([u8; N], [u16; N], [u16; N]) {
    let (mut sharedkey, mut pk, mut c) =
        ([0; N], [0; N], [0; N]);
    let (mut a, mut sp, mut ep, mut epp) =
        ([0; N], [0; N], [0; N], [0; N]);

    uniform(&mut a, seed);

    noise(&mut sp);
    ntt(&mut sp);
    noise(&mut ep);
    ntt(&mut ep);
    noise(&mut epp);

    accept_computation(
        &mut sharedkey, &mut pk, &mut c,
        &sp, &ep, &epp, pka, &a
    );

    (sharedkey, pk, c)
}

pub fn shareda(ska: &[u16], pkb: &[u16], c: &[u16]) -> [u8; N] {
    let mut sharedkey = [0; N];

    finish_computation(&mut sharedkey, ska, pkb, c);

    sharedkey
}


#[test]
fn test_kex() {
    use rand::{ Rng, thread_rng };

    let mut seed = [0; 32];
    thread_rng().fill_bytes(&mut seed);

    let (ask, apk) = keygen(&seed);
    let (bsharedkey, bpk, c) = sharedb(&apk, &seed);
    let asharedkey = shareda(&ask, &bpk, &c);

    for i in 0..asharedkey.len() {
        assert_eq!(asharedkey[i], bsharedkey[i]);
    }
}
