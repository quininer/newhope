#![feature(test)]

extern crate test;
extern crate newhope;

use test::Bencher;

#[bench]
fn bench_newhope_keygen(b: &mut Bencher) {
    b.iter(|| newhope::NewHope::new());
}

#[bench]
fn bench_newhope_sharedb(b: &mut Bencher) {
    use newhope::NewHope;

    let mut output = [0; 32];
    let alice = NewHope::new().unwrap();
    let pka = alice.export_public();

    b.iter(|| NewHope::exchange(&pka, &mut output));
}

#[bench]
fn bench_newhope_shareda(b: &mut Bencher) {
    use newhope::NewHope;

    let mut output = [0; 32];
    let alice = NewHope::new().unwrap();
    let pkb = NewHope::exchange(&alice.export_public(), &mut output).unwrap();

    b.iter(|| alice.exchange_from(&pkb, &mut output));
}
