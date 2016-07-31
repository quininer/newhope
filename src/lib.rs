#![feature(step_by, question_mark)]

extern crate rand;
extern crate byteorder;
extern crate tiny_keccak;

mod params;
mod reduce;
mod ntt;
mod error_correction;
mod poly;
mod newhope;

#[cfg(feature = "api")] use std::io;
#[cfg(feature = "api")] use rand::{ Rng, OsRng, ChaChaRng };
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


/// ```
/// use newhope::NewHope;
///
/// let (mut keya, mut keyb) = ([0; 32], [0; 32]);
/// let alice = NewHope::new().unwrap();
/// let pkb = NewHope::exchange(&alice.export_public(), &mut keyb).unwrap();
/// alice.exchange_from(&pkb, &mut keya);
///
/// assert!(keya != [0; 32]);
/// assert_eq!(keya, keyb);
/// ```
#[cfg(feature = "api")]
pub struct NewHope {
    sk: [u16; N],
    pk: [u16; N],
    nonce: [u8; 32]
}

#[cfg(feature = "api")]
impl NewHope {
    pub fn new() -> Result<NewHope, io::Error> {
        let mut rng = OsRng::new()?.gen::<ChaChaRng>();
        let mut newhope = NewHope {
            sk: [0; N],
            pk: [0; N],
            nonce: [0; 32]
        };

        rng.fill_bytes(&mut newhope.nonce);
        keygen(&mut newhope.sk, &mut newhope.pk, &newhope.nonce, rng);

        Ok(newhope)
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

    /// ```
    /// # use newhope::NewHope;
    /// # let (mut keya, mut keyb) = ([0; 32], [0; 32]);
    /// # let alice = NewHope::new().unwrap();
    /// let (alice_sk, alice_pk) = (alice.export_private(), alice.export_public());
    /// let alice = NewHope::import(&alice_sk, &alice_pk);
    /// # let pkb = NewHope::exchange(&alice.export_public(), &mut keyb).unwrap();
    /// # alice.exchange_from(&pkb, &mut keya);
    /// # assert!(keya != [0; 32]);
    /// # assert_eq!(keya, keyb);
    /// ```
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

    pub fn exchange(pka: &[u8], sharedkey: &mut [u8]) -> Result<[u8; SENDBBYTES], io::Error> {
        let (mut key, mut pkb, mut c) = ([0; 32], [0; N], [0; N]);
        let (pk, nonce) = pka.split_at(POLY_BYTES);

        sharedb(
            &mut key, &mut pkb, &mut c,
            &poly_frombytes(pk), nonce, OsRng::new()?.gen::<ChaChaRng>()
        );

        sha3_256(sharedkey, &key);

        let mut output = [0; SENDBBYTES];
        output[..POLY_BYTES].clone_from_slice(&poly_tobytes(&pkb));
        output[POLY_BYTES..].clone_from_slice(&rec_tobytes(&c));
        Ok(output)
    }

    pub fn exchange_from(&self, pkb: &[u8], sharedkey: &mut [u8]) {
        let mut key = [0; 32];
        let (pk, rec) = pkb.split_at(POLY_BYTES);
        shareda(&mut key, &self.sk, &poly_frombytes(pk), &rec_frombytes(rec));

        sha3_256(sharedkey, &key);
    }
}

pub fn sha3_256(output: &mut [u8], input: &[u8]) {
    let mut sha3 = Keccak::new_sha3_256();
    sha3.update(input);
    sha3.finalize(output)
}
