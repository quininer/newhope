#![feature(iterator_step_by)]

extern crate rand;
extern crate byteorder;
extern crate tiny_keccak;

#[cfg(feature = "tor")] pub mod batcher;
mod params;
mod reduce;
mod ntt;
mod error_correction;
mod poly;
mod newhope;

use tiny_keccak::Keccak;
pub use poly::{ poly_frombytes, poly_tobytes };
pub use params::{
    N, Q,
    POLY_BYTES,
    SEEDBYTES, RECBYTES,
    SENDABYTES, SENDBBYTES
};
pub use newhope::{
    keygen, sharedb, shareda,
    rec_frombytes, rec_tobytes,
};

#[inline]
pub fn sha3_256(output: &mut [u8], input: &[u8]) {
    let mut sha3 = Keccak::new_sha3_256();
    sha3.update(input);
    sha3.finalize(output)
}
