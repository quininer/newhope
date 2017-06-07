extern crate libc;
extern crate rand;
extern crate newhope;
extern crate cnewhope;

use rand::{ Rng, OsRng, ChaChaRng };

#[test]
fn test_kex_rs() {
    let (mut ska, mut pka) = ([0; newhope::N], [0; newhope::N]);
    let (mut keya, mut keyb) = ([0; 32], [0; 32]);
    let mut nonce = [0; 32];
    let mut bob = [0; cnewhope::SENDBBYTES];

    let mut rng = OsRng::new().unwrap().gen::<ChaChaRng>();
    rng.fill_bytes(&mut nonce);
    newhope::keygen(&mut ska, &mut pka, &nonce, rng.gen::<ChaChaRng>());
    let pka_bytes = newhope::poly_tobytes(&pka);
    let alice = [&pka_bytes[..], &nonce[..]].concat();

    unsafe { cnewhope::newhope_sharedb(
        keyb.as_mut_ptr(),
        bob.as_mut_ptr(),
        alice.as_ptr()
    ) };

    let (pkb_bytes, rec_bytes) = bob.split_at(newhope::POLY_BYTES);
    let pkb = newhope::poly_frombytes(pkb_bytes);
    let rec = newhope::rec_frombytes(rec_bytes);
    let mut tmp = [0; 32];
    newhope::shareda(&mut tmp, &ska, &pkb, &rec);
    newhope::sha3_256(&mut keya, &tmp);

    assert!(keya != [0; 32]);
    assert_eq!(keya, keyb);
}

#[test]
fn test_kex_c() {
    let (mut keya, mut keyb) = ([0; 32], [0; 32]);
    let (mut pkb, mut rec) = ([0; newhope::N], [0; newhope::N]);
    let mut senda = [0; cnewhope::SENDABYTES];
    let mut ska = cnewhope::Poly::default();

    unsafe { cnewhope::newhope_keygen(senda.as_mut_ptr(), &mut ska) };

    let (pka_bytes, nonce) = senda.split_at(newhope::POLY_BYTES);
    let pka = newhope::poly_frombytes(pka_bytes);
        let mut tmp = [0; 32];
        newhope::sharedb(
            &mut tmp, &mut pkb, &mut rec,
            &pka, nonce, OsRng::new().unwrap().gen::<ChaChaRng>()
        );
        newhope::sha3_256(&mut keyb, &tmp);
    let bob = [
        &newhope::poly_tobytes(&pkb)[..],
        &newhope::rec_tobytes(&rec)[..]
    ].concat();

    unsafe { cnewhope::newhope_shareda(keya.as_mut_ptr(), &ska, bob.as_ptr()) };

    assert!(keya != [0; 32]);
    assert_eq!(keya, keyb);
}
