use rand::Rng;
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

fn accept_computation<R: Rng>(
    key: &mut [u8], bp: &mut [u16], c: &mut [u16],
    sp: &[u16], ep: &[u16], epp: &[u16], pk: &[u16], a: &[u16],
    rng: &mut R
) {
    let (mut v, mut t) = ([0; N], [0; N]);

    pointwise(&mut t, a, sp);
    add(bp, &t, ep);

    pointwise(&mut t, pk, sp);
    invntt(&mut t);
    add(&mut v, &t, epp);
    helprec(c, &v, rng);
    rec(key, &v, c);
}

fn finish_computation(key: &mut [u8], sk: &[u16], bp: &[u16], c: &[u16]) {
    let mut v = [0; N];
    pointwise(&mut v, sk, bp);
    invntt(&mut v);
    rec(key, &v, c);
}

pub fn rec_frombytes(r: &[u8], c: &mut [u16; N]) {
    for i in 0..(N / 4) {
        c[4 * i + 0] = r[i] as u16 & 0x03;
        c[4 * i + 1] = (r[i] >> 2) as u16 & 0x03;
        c[4 * i + 2] = (r[i] >> 4) as u16 & 0x03;
        c[4 * i + 3] = (r[i] >> 6) as u16;
    }
}

pub fn rec_tobytes(c: &[u16; N], r: &mut [u8]) {
    for i in (0..N).step_by(4) {
        r[i / 4] = c[i] as u8
            | (c[i + 1] << 2) as u8
            | (c[i + 2] << 4) as u8
            | (c[i + 3] << 6) as u8;
    }
}


/// ```
/// # extern crate rand;
/// # extern crate newhope;
/// # use newhope::{ N, newhope as nh };
/// # fn main() {
/// use rand::{ Rng, OsRng, ChaChaRng };
///
/// let mut rng = OsRng::new().unwrap();
/// let mut nonce = [0; 32];
/// rng.fill_bytes(&mut nonce);
///
/// let (mut ask, mut apk, mut asharedkey) = ([0; N], [0; N], [0; 32]);
/// let (mut bpk, mut c, mut bsharedkey) = ([0; N], [0; N], [0; 32]);
///
/// nh::keygen(&mut ask, &mut apk, &nonce, rng.gen::<ChaChaRng>());
/// nh::sharedb(&mut bsharedkey, &mut bpk, &mut c, &apk, &nonce, rng.gen::<ChaChaRng>());
/// nh::shareda(&mut asharedkey, &ask, &bpk, &c);
///
/// assert_eq!(asharedkey, bsharedkey);
/// # }
/// ```
#[inline]
pub fn keygen<R: Rng>(sk: &mut [u16], pk: &mut [u16], nonce: &[u8], mut rng: R) {
    let (mut a, mut e) = ([0; N], [0; N]);

    uniform(&mut a, nonce);

    noise(sk, &mut rng);
    ntt(sk);

    noise(&mut e, &mut rng);
    ntt(&mut e);

    offer_computation(pk, sk, &e, &a);
}

#[inline]
pub fn sharedb<R: Rng>(
    sharedkey: &mut [u8], pk: &mut [u16], c: &mut [u16],
    pka: &[u16], nonce: &[u8], mut rng: R
) {
    let (mut a, mut sp, mut ep, mut epp) =
        ([0; N], [0; N], [0; N], [0; N]);

    uniform(&mut a, nonce);

    noise(&mut sp, &mut rng);
    ntt(&mut sp);
    noise(&mut ep, &mut rng);
    ntt(&mut ep);
    noise(&mut epp, &mut rng);

    accept_computation(
        sharedkey, pk, c,
        &sp, &ep, &epp, pka, &a, &mut rng
    );
}

#[inline]
pub fn shareda(sharedkey: &mut [u8], ska: &[u16], pkb: &[u16], c: &[u16]) {
    finish_computation(sharedkey, ska, pkb, c);
}
