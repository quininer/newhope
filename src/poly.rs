use rand::{ Rng, ChaChaRng, SeedableRng, thread_rng };
use byteorder::{ ByteOrder, LittleEndian };
use ::params::{
    N, Q,
    PSIS_BITREV_MONTGOMERY, OMEGAS_MONTGOMERY,
    PSIS_INV_MONTGOMERY, OMEGAS_INV_MONTGOMERY
};
use ::reduce::{ montgomery_reduce, barrett_reduce };
use ::ntt::{ bitrev_vector, mul_coefficients, ntt as fft };


#[inline]
fn gen<R: Rng>(rng: &mut R) -> u16 {
    let output = rng.gen::<u16>() & 0x3fff;
    if output < Q as u16 {
        output
    } else {
        gen(rng)
    }
}

pub fn uniform(a: &mut [u16], seed: &[u8]) {
    let mut seed_u32 = [0; 8];
    for i in 0..8 {
        seed_u32[i] = LittleEndian::read_u32(&seed[4 * i..]);
    }
    let mut rng = ChaChaRng::from_seed(&seed_u32);

    for i in 0..N {
        a[i] = gen(&mut rng);
    }
}

pub fn noise(r: &mut [u16]) {
    let mut buf = [0; 4 * N];
    thread_rng().fill_bytes(&mut buf);

    for i in 0..N {
        let t = LittleEndian::read_u32(&buf[4 * i..]);
        let d = (0..8).fold(0, |sum, j| sum + ((t >> j) & 0x01010101));
        let a = ((d >> 8) & 0xff) + (d & 0xff);
        let b = (d >> 24) + ((d >> 16) & 0xff);

        r[i] = a as u16 + Q as u16 - b as u16;
    }
}

pub fn pointwise(r: &mut [u16], a: &[u16], b: &[u16]) {
    for i in 0..N {
        let t = montgomery_reduce(3186 * b[i] as u32);
        r[i] = montgomery_reduce(t as u32 * a[i] as u32);
    }
}

pub fn add(r: &mut [u16], a: &[u16], b: &[u16]) {
    for i in 0..N {
        r[i] = barrett_reduce(a[i].wrapping_add(b[i]));
    }
}

pub fn ntt(r: &mut [u16]) {
    mul_coefficients(r, &PSIS_BITREV_MONTGOMERY);
    fft(r, &OMEGAS_MONTGOMERY);
}

pub fn invntt(r: &mut [u16]) {
    bitrev_vector(r);
    fft(r, &OMEGAS_INV_MONTGOMERY);
    mul_coefficients(r, &PSIS_INV_MONTGOMERY);
}


#[test]
fn test_pointwise() {
    let (a, b, mut r) = ([3; N], [4; N], [0; N]);
    pointwise(&mut r, &a, &b);
    for i in 0..N {
        assert_eq!(r[i], 12);
    }

    let (a, b, mut r) = ([333; N], [444; N], [0; N]);
    pointwise(&mut r, &a, &b);
    for i in 0..N {
        assert_eq!(r[i], 384);
    }

    let (a, b, mut r) = ([33333; N], [44444; N], [0; N]);
    pointwise(&mut r, &a, &b);
    for i in 0..N {
        assert_eq!(r[i], 12902);
    }
}

#[test]
fn test_add() {
    let (a, b, mut r) = ([3; N], [4; N], [0; N]);
    add(&mut r, &a, &b);
    for i in 0..N {
        assert_eq!(r[i], 7);
    }

    let (a, b, mut r) = ([333; N], [444; N], [0; N]);
    add(&mut r, &a, &b);
    for i in 0..N {
        assert_eq!(r[i], 777);
    }

    let (a, b, mut r) = ([33333; N], [44444; N], [0; N]);
    add(&mut r, &a, &b);
    for i in 0..N {
        assert_eq!(r[i], 12241);
    }
}
