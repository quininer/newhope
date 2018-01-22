#![feature(iterator_step_by)]

extern crate rand;
extern crate byteorder;
extern crate tiny_keccak;

#[cfg(feature = "tor")] pub mod batcher;
mod reduce;
mod ntt;
mod error_correction;
pub mod poly;
pub mod params;
pub mod newhope;

use rand::Rng;
use tiny_keccak::Keccak;
pub use params::{
    N, Q,
    POLY_BYTES,
    SEEDBYTES, RECBYTES,
    SENDABYTES, SENDBBYTES
};


pub fn keygen<R: Rng>(mut r: R, sk: &mut [u8; POLY_BYTES], pk: &mut [u8; SENDABYTES]) {
    let (mut ska, mut pka) = ([0; N], [0; N]);
    let (pk, nonce) = pk.split_at_mut(POLY_BYTES);
    r.fill_bytes(nonce);

    newhope::keygen(&mut ska, &mut pka, nonce, r);

    poly::poly_tobytes(&ska, sk);
    poly::poly_tobytes(&pka, pk);
}

pub fn sharedb<R: Rng>(r: R, sharedkey: &mut [u8; 32], pk: &[u8; SENDABYTES], msg: &mut [u8; SENDBBYTES]) {
    let (pk, nonce) = pk.split_at(POLY_BYTES);
    let (pkb_bytes, rec_bytes) = msg.split_at_mut(POLY_BYTES);
    let mut pka = [0; N];
    let mut pkb = [0; N];
    let mut rec = [0; N];
    poly::poly_frombytes(pk, &mut pka);

    newhope::sharedb(
        sharedkey, &mut pkb, &mut rec,
        &pka, nonce, r
    );

    let mut sha3 = Keccak::new_sha3_256();
    sha3.update(sharedkey);
    sha3.finalize(sharedkey);

    poly::poly_tobytes(&pkb, pkb_bytes);
    newhope::rec_tobytes(&rec, rec_bytes);
}

pub fn shareda(sharedkey: &mut [u8; 32], sk: &[u8; POLY_BYTES], msg: &[u8; SENDBBYTES]) {
    let mut ska = [0; N];
    let (mut pkb, mut rec) = ([0; N], [0; N]);
    let (pkb_bytes, rec_bytes) = msg.split_at(POLY_BYTES);

    poly::poly_frombytes(sk, &mut ska);
    poly::poly_frombytes(pkb_bytes, &mut pkb);
    newhope::rec_frombytes(rec_bytes, &mut rec);

    newhope::shareda(sharedkey, &ska, &pkb, &rec);

    let mut sha3 = Keccak::new_sha3_256();
    sha3.update(sharedkey);
    sha3.finalize(sharedkey);
}
