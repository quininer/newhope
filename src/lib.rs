#![feature(step_by)]

extern crate rand;
extern crate byteorder;
extern crate tiny_keccak;

mod params;
mod reduce;
mod ntt;
mod error_correction;
mod poly;
mod newhope;

use rand::{ Rng, OsRng };
use tiny_keccak::Keccak;
use poly::{ poly_frombytes, poly_tobytes };
use newhope::{ rec_frombytes, rec_tobytes };
pub use params::{
    N, Q,
    POLY_BYTES,
    SEEDBYTES, RECBYTES,
    SENDABYTES, SENDBBYTES
};
pub use newhope::{ keygen, sharedb, shareda };


/// ```
/// use newhope::NewHope;
///
/// let (mut keya, mut keyb) = ([0; 32], [0; 32]);
/// let alice = NewHope::new();
/// let pkb = NewHope::exchange(&alice.export_public(), &mut keyb);
/// alice.exchange_from(&pkb, &mut keya);
///
/// assert_eq!(keya, keyb);
/// ```
pub struct NewHope {
    sk: [u16; N],
    pk: [u16; N],
    nonce: [u8; 32]
}

impl NewHope {
    pub fn new() -> NewHope {
        let mut rng = OsRng::new().unwrap();

        let mut nonce = [0; 32];
        rng.fill_bytes(&mut nonce);

        let (sk, pk) = keygen(&nonce, rng.gen());

        NewHope {
            sk: sk,
            pk: pk,
            nonce: nonce
        }
    }

    pub fn export_public(&self) -> [u8; SENDABYTES] {
        let mut output = [0; SENDABYTES];
        output[..POLY_BYTES].clone_from_slice(&poly_tobytes(&self.pk));
        output[POLY_BYTES..].clone_from_slice(&self.nonce);
        output
    }

    pub fn export_private(&self) -> [u8; POLY_BYTES] {
        poly_tobytes(&self.sk)
    }

    pub fn import(sk: &[u8], pk: &[u8]) -> NewHope {
        let (pk, nonce) = pk.split_at(POLY_BYTES);

        let mut newhope = NewHope {
            sk: [0; N],
            pk: [0; N],
            nonce: [0; 32]
        };
        newhope.sk.clone_from_slice(&poly_frombytes(sk));
        newhope.pk.clone_from_slice(&poly_frombytes(pk));
        newhope.nonce.clone_from_slice(nonce);
        newhope
    }

    pub fn exchange(pka: &[u8], sharedkey: &mut [u8]) -> [u8; SENDBBYTES] {
        let (pk, nonce) = pka.split_at(POLY_BYTES);
        let (key, pkb, c) = sharedb(&poly_frombytes(pk), nonce, OsRng::new().unwrap().gen());

        let mut sha3 = Keccak::new_sha3_256();
        sha3.update(&key);
        sha3.finalize(sharedkey);

        let mut output = [0; SENDBBYTES];
        output[..POLY_BYTES].clone_from_slice(&poly_tobytes(&pkb));
        output[POLY_BYTES..].clone_from_slice(&rec_tobytes(&c));
        output
    }

    pub fn exchange_from(&self, pkb: &[u8], sharedkey: &mut [u8]) {
        let (pk, rec) = pkb.split_at(POLY_BYTES);
        let key = shareda(&self.sk, &poly_frombytes(pk), &rec_frombytes(rec));

        let mut sha3 = Keccak::new_sha3_256();
        sha3.update(&key);
        sha3.finalize(sharedkey);
    }
}
