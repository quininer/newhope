//! ref https://cryptojedi.org/crypto/#newhope
//! ref https://cryptojedi.org/crypto/data/newhope-20160815.tar.bz2

#![feature(repr_align)]
#![feature(attr_literals)]

pub const N: usize = 1024;
pub const Q: usize = 12289;
pub const POLY_BYTES: usize = 1792;
pub const SEEDBYTES: usize = 32;
pub const RECBYTES: usize = 256;
pub const SENDABYTES: usize = POLY_BYTES + SEEDBYTES;
pub const SENDBBYTES: usize = POLY_BYTES + RECBYTES;


#[repr(C)]
#[repr(align(32))]
pub struct Poly {
    pub coeffs: [u16; N]
}

impl Default for Poly {
    fn default() -> Poly {
        Poly { coeffs: [0; N] }
    }
}

extern "C" {
    pub fn newhope_keygen(send: *mut u8, sk: *mut Poly);
    pub fn newhope_sharedb(sharedkey: *mut u8, send: *mut u8, received: *const u8);
    pub fn newhope_shareda(sharedkey: *mut u8, ska: *const Poly, received: *const u8);
}


#[test]
fn test_newhope() {
    extern crate libc;

    let (mut senda, mut sendb) = ([0; SENDABYTES], [0; SENDBBYTES]);
    let (mut keya, mut keyb) = ([0; 32], [0; 32]);
    let mut ska = Poly::default();

    unsafe {
        newhope_keygen(senda.as_mut_ptr(), &mut ska);
        newhope_sharedb(keyb.as_mut_ptr(), sendb.as_mut_ptr(), senda.as_ptr());
        newhope_shareda(keya.as_mut_ptr(), &ska, sendb.as_ptr());
    }

    assert!(keya != [0; 32]);
    assert_eq!(keya, keyb);
}
