use rand::{ Rng, ChaChaRng };
use ::params::Q;


fn f(v0: &mut i32, v1: &mut i32, x: i32) -> i32 {
    let mut b = x * 2730;
    let mut t = b >> 25;
    b = x - t * 12289;
    b = 12288 - b;
    b >>= 31;
    t -= b;

    let mut r = t & 1;
    *v0 = (t >> 1) + r;

    t -= 1;
    r = t & 1;
    *v1 = (t >> 1) + r;

    (x - (*v0 * 2 * Q as i32)).abs()
}

fn g(x: i32) -> i32 {
    let mut b = x * 2730;
    let mut t = b >> 27;
    b = x - t * 49156;
    b = 49155 - b;
    b >>= 31;
    t -= b;

    let c = t & 1;
    t = (t >> 1) + c;

    t *= 8 * Q as i32;

    (t - x).abs()
}

fn ld_decode(xi0: i32, xi1: i32, xi2: i32, xi3: i32) -> i16 {
    let mut t = g(xi0) + g(xi1) + g(xi2) + g(xi3);
    t -= 8 * Q as i32;
    t >>= 31;

    t as i16 & 1
}


pub fn helprec(c: &mut [u16], v: &[u16], rng: &mut ChaChaRng) {
    let (mut v0, mut v1, mut v_tmp) = ([0; 4], [0; 4], [0; 4]);
    let mut r = [0; 32];
    rng.fill_bytes(&mut r);

    for i in 0..256 {
        let rbit = ((r[i >> 3] >> (i & 7)) & 1) as i32;
        let mut k = f(&mut v0[0], &mut v1[0], 8 * v[0 + i] as i32 + 4 * rbit)
            + f(&mut v0[1], &mut v1[1], 8 * v[256 + i] as i32 + 4 * rbit)
            + f(&mut v0[2], &mut v1[2], 8 * v[512 + i] as i32 + 4 * rbit)
            + f(&mut v0[3], &mut v1[3], 8 * v[768 + i] as i32 + 4 * rbit);
        k = (2 * Q as i32 - 1 - k) >> 31;

        v_tmp[0] = (!k & v0[0]) ^ (k & v1[0]);
        v_tmp[1] = (!k & v0[1]) ^ (k & v1[1]);
        v_tmp[2] = (!k & v0[2]) ^ (k & v1[2]);
        v_tmp[3] = (!k & v0[3]) ^ (k & v1[3]);

        c[0 + i] = ((v_tmp[0] - v_tmp[3]) & 3) as u16;
        c[256 + i] = ((v_tmp[1] - v_tmp[3]) & 3) as u16;
        c[512 + i] = ((v_tmp[2] - v_tmp[3]) & 3) as u16;
        c[768 + i] = ((-k + 2 * v_tmp[3]) & 3) as u16;
    }
}

pub fn rec(key: &mut [u8], v: &[u16], c: &[u16]) {
    let mut tmp = [0; 4];
    for i in 0..256 {
        tmp[0] = 16 * Q as i32 + 8 * v[0 + i] as i32
            - Q as i32 * (2 * c[0 + i] as i32 + c[768 + i] as i32);
        tmp[1] = 16 * Q as i32 + 8 * v[256 + i] as i32
            - Q as i32 * (2 * c[256 + i] as i32 + c[768 + i] as i32);
        tmp[2] = 16 * Q as i32 + 8 * v[512 + i] as i32
            - Q as i32 * (2 * c[512 + i] as i32 + c[768 + i] as i32);
        tmp[3] = 16 * Q as i32 + 8 * v[0 + i] as i32
            - Q as i32 * c[768 + i] as i32;

        key[i >> 3] |= (ld_decode(tmp[0], tmp[1], tmp[2], tmp[3]) << (i & 7)) as u8;
    }
}
