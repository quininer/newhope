extern crate gcc;

use std::path::Path;
use gcc::Config;


fn main() {
    #[cfg(not(feature = "tor"))] let cnewhope_root = Path::new("newhope").join("ref");
    #[cfg(feature = "tor")] let cnewhope_root = Path::new("newhope").join("torref");
    let mut cfg = Config::new();

    for src in &[
        "crypto_stream_chacha20.c",
        "poly.c",
        "ntt.c",
        "precomp.c",
        "error_correction.c",
        "newhope.c",
        "reduce.c",
        "fips202.c",
        "randombytes.c",

        #[cfg(feature = "tor")] "batcher.c",
    ] {
        cfg.file(cnewhope_root.join(src));
    }

    cfg.file("keygen_poly.c");

    cfg.include(cnewhope_root)
        .opt_level(3)
        .debug(true)
        .flag("-march=native")
        .compile("libnewhope.a");
}
