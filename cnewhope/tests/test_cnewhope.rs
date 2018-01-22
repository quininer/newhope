extern crate libc;
extern crate rand;
extern crate newhope;
extern crate cnewhope;

use rand::{ Rng, OsRng, ChaChaRng };
use newhope::params;

#[test]
fn test_kex_rs() {
    let (mut ska, mut pka) = ([0; params::POLY_BYTES], [0; params::SENDABYTES]);
    let (mut keya, mut keyb) = ([0; 32], [0; 32]);
    let mut sendb = [0; cnewhope::SENDBBYTES];

    let mut rng = OsRng::new().unwrap();
    newhope::keygen(rng.gen::<ChaChaRng>(), &mut ska, &mut pka);

    unsafe { cnewhope::newhope_sharedb(
        keyb.as_mut_ptr(),
        sendb.as_mut_ptr(),
        pka.as_ptr()
    ) };

    newhope::shareda(&mut keya, &ska, &sendb);

    assert!(keya != [0; 32]);
    assert_eq!(keya, keyb);
}

#[test]
fn test_kex_c() {
    let (mut keya, mut keyb) = ([0; 32], [0; 32]);
    let mut sendb = [0; params::SENDBBYTES];
    let mut senda = [0; cnewhope::SENDABYTES];
    let mut ska = cnewhope::Poly::default();

    unsafe { cnewhope::newhope_keygen(senda.as_mut_ptr(), &mut ska) };

    newhope::sharedb(
        OsRng::new().unwrap().gen::<ChaChaRng>(),
        &mut keyb, &senda, &mut sendb
    );

    unsafe { cnewhope::newhope_shareda(keya.as_mut_ptr(), &ska, sendb.as_ptr()) };

    assert!(keya != [0; 32]);
    assert_eq!(keya, keyb);
}
