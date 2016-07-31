extern crate newhope;
extern crate cnewhope;


#[test]
fn test_kex_rs() {
    let (mut keya, mut keyb) = ([0; 32], [0; 32]);
    let mut bob = [0; cnewhope::SENDBBYTES];

    let alice = newhope::NewHope::new().unwrap();
    unsafe { cnewhope::newhope_sharedb(
        keyb.as_mut_ptr(),
        bob.as_mut_ptr(),
        alice.export_public().as_ptr()
    ) };
    alice.exchange_from(&bob, &mut keya);

    assert!(keya != [0; 32]);
    assert_eq!(keya, keyb);
}

#[test]
fn test_kex_c() {
    let (mut keya, mut keyb) = ([0; 32], [0; 32]);
    let mut ska = cnewhope::Poly::default();
    let mut senda = [0; cnewhope::SENDABYTES];

    unsafe { cnewhope::newhope_keygen(senda.as_mut_ptr(), &mut ska) };
    let bob = newhope::NewHope::exchange(&senda, &mut keyb).unwrap();
    unsafe { cnewhope::newhope_shareda(keya.as_mut_ptr(), &ska, bob.as_ptr()) };

    assert!(keya != [0; 32]);
    assert_eq!(keya, keyb);
}
