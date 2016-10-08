use rand::Rng;
use byteorder::{ ByteOrder, LittleEndian };
use tiny_keccak::Keccak;
use ::params::{
    N, Q, POLY_BYTES,
    PSIS_BITREV_MONTGOMERY, OMEGAS_MONTGOMERY,
    PSIS_INV_MONTGOMERY, OMEGAS_INV_MONTGOMERY
};
use ::reduce::{ montgomery_reduce, barrett_reduce };
use ::ntt::{ bitrev_vector, mul_coefficients, ntt as fft };

#[cfg(not(feature = "tor"))] const MODULI: [usize; 5] = [0 * Q, 1 * Q, 2 * Q, 3 * Q, 4 * Q];
const SHAKE128_RATE: usize = 168;


pub fn poly_frombytes(a: &[u8]) -> [u16; N] {
    let mut output = [0; N];
    for i in 0..(N / 4) {
        output[4 * i + 0] = (a[7 * i + 0] as u16)
            | ((a[7 * i + 1] as u16 & 0x3f) << 8);
        output[4 * i + 1] = ((a[7 * i + 1] as u16) >> 6)
            | ((a[7 * i + 2] as u16) << 2)
            | ((a[7 * i + 3] as u16 & 0x0f) << 10);
        output[4 * i + 2] = ((a[7 * i + 3] as u16) >> 4)
            | ((a[7 * i + 4] as u16) << 4)
            | ((a[7 * i + 5] as u16 & 0x03) << 12);
        output[4 * i + 3] = ((a[7 * i + 5] as u16) >> 2)
            | ((a[7 * i + 6] as u16) << 6);
    }
    output
}

pub fn poly_tobytes(p: &[u16; N]) -> [u8; POLY_BYTES] {
    let mut output = [0; POLY_BYTES];
    for i in 0..(N / 4) {
        let mut t0 = barrett_reduce(p[4 * i + 0]);
        let mut t1 = barrett_reduce(p[4 * i + 1]);
        let mut t2 = barrett_reduce(p[4 * i + 2]);
        let mut t3 = barrett_reduce(p[4 * i + 3]);

        let mut m = t0.wrapping_sub(Q as u16);
        let mut c = m as i16;
        c >>= 15;
        t0 = m ^ ((t0 ^ m) & c as u16);

        m = t1.wrapping_sub(Q as u16);
        c = m as i16;
        c >>= 15;
        t1 = m ^ ((t1 ^ m) & c as u16);

        m = t2.wrapping_sub(Q as u16);
        c = m as i16;
        c >>= 15;
        t2 = m ^ ((t2 ^ m) & c as u16);

        m = t3.wrapping_sub(Q as u16);
        c = m as i16;
        c >>= 15;
        t3 = m ^ ((t3 ^ m) & c as u16);

        output[7 * i + 0] = (t0 & 0xff) as u8;
        output[7 * i + 1] = ((t0 >> 8) | (t1 << 6)) as u8;
        output[7 * i + 2] = (t1 >> 2) as u8;
        output[7 * i + 3] = ((t1 >> 10) | (t2 << 4)) as u8;
        output[7 * i + 4] = (t2 >> 4) as u8;
        output[7 * i + 5] = ((t2 >> 12) | (t3 << 2)) as u8;
        output[7 * i + 6] = (t3 >> 6) as u8;
    }
    output
}

#[cfg(not(feature = "tor"))]
pub fn uniform(a: &mut [u16], nonce: &[u8]) {
    let (mut nblocks, mut pos, mut ctr) = (13, 0, 0);
    let mut buf = [0; SHAKE128_RATE * 13];
    let mut shake128 = Keccak::new_shake128();

    shake128.update(nonce);
    shake128.pad();
    shake128.keccakf();
    shake128.squeeze(&mut buf);

    while ctr < N {
        let val = LittleEndian::read_u16(&buf[pos..]);
        pos += 2;
        let r = (val as usize) / Q;
        if r < 5 {
            a[ctr] = val - (MODULI[r] as u16);
            ctr += 1;
        }

        if pos > SHAKE128_RATE * nblocks - 2 {
            nblocks = 1;
            shake128.squeeze(&mut buf[..SHAKE128_RATE]);
            pos = 0;
        }
    }
}

#[cfg(feature = "tor")]
fn discardtopoly(a: &mut [u16], buf: &[u8]) -> bool {
    use ::batcher::batcher84;

    let mut x = [0; SHAKE128_RATE * 16 / 2];

    for (i, b) in x.iter_mut().enumerate() {
        *b = LittleEndian::read_u16(&buf[2 * i..]);
    }

    for i in 0..16 {
        batcher84(&mut x[i..]);
    }

    let r = (1008..1024)
        .map(|i| 61444u16.wrapping_sub(x[i]))
        .fold(0, |sum, next| sum | next);

    if r.checked_shr(31).is_some() {
        true
    } else {
        for i in 0..N {
            a[i] = x[i];
        }
        false
    }
}

#[cfg(feature = "tor")]
pub fn uniform(a: &mut [u16], nonce: &[u8]) {
    let mut buf = [0; SHAKE128_RATE * 16];
    let mut shake128 = Keccak::new_shake128();
    shake128.absorb(nonce);
    shake128.pad();
    shake128.keccakf();

    loop {
        shake128.squeeze(&mut buf);
        if !discardtopoly(a, &buf) { break }
    }
}

pub fn noise<R: Rng>(r: &mut [u16], rng: &mut R) {
    for i in 0..N {
        let t = rng.gen::<u32>();
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

#[cfg(not(feature = "tor"))]
#[test]
fn test_uniform() {
    let output = [8191, 9551, 1218, 1909, 506, 6242, 1802, 657, 7190, 8637, 2819, 7895, 2139, 8660, 11418, 11417, 6291, 3367, 1613, 10371, 3896, 8853, 1071, 196, 1945, 9511, 5769, 2080, 11104, 11914, 6761, 4048, 5301, 8416, 7019, 2201, 11702, 2079, 9501, 1375, 7113, 7668, 9316, 5000, 12099, 6154, 10778, 3146, 10136, 11654, 3815, 842, 9780, 9909, 6110, 3189, 11145, 11403, 6704, 742, 1148, 5188, 8866, 1588, 571, 5268, 1100, 8215, 9684, 1711, 7862, 503, 8442, 10926, 2157, 3668, 2800, 513, 506, 10162, 12078, 391, 5086, 3877, 10673, 9855, 7654, 2161, 81, 7026, 9424, 5657, 10063, 673, 2064, 1200, 9482, 4031, 11217, 326, 220, 1849, 10755, 8418, 2492, 9933, 7636, 4573, 5948, 9192, 0, 4441, 2014, 8367, 3909, 4491, 1315, 7752, 6452, 9054, 10189, 471, 4300, 9714, 5631, 511, 10933, 6528, 4528, 6204, 2221, 9091, 9760, 4125, 180, 11691, 6568, 5727, 4684, 10107, 2285, 728, 2787, 11373, 4600, 5206, 4987, 4997, 826, 10575, 7043, 10843, 4482, 3775, 6385, 5268, 9138, 11426, 11697, 8812, 1564, 8368, 10631, 6864, 1671, 1907, 10709, 12134, 10253, 11396, 1971, 9692, 4852, 4474, 1597, 9021, 2198, 11270, 2657, 130, 7886, 3572, 9315, 4183, 4258, 4916, 7834, 1849, 10615, 11028, 2550, 5417, 29, 8705, 1967, 9038, 3744, 11681, 5025, 6733, 3292, 3376, 1773, 8698, 1768, 250, 10509, 5132, 2691, 5075, 1697, 1209, 6107, 3821, 9686, 6686, 11809, 6601, 4507, 7643, 1233, 9030, 7562, 7356, 3584, 3042, 3765, 903, 431, 10224, 7677, 8912, 7157, 8011, 12069, 4970, 3591, 3253, 12226, 508, 4034, 6437, 8525, 337, 11849, 449, 11924, 2952, 11997, 5795, 10820, 11583, 7548, 10991, 4841, 3526, 11254, 4366, 8679, 10044, 850, 9636, 11267, 11585, 5379, 3209, 3513, 11686, 8166, 2544, 8534, 5001, 879, 2204, 8130, 6258, 5296, 11111, 11207, 9934, 2051, 11557, 354, 727, 9063, 5644, 9311, 10273, 7405, 9837, 5651, 4164, 3100, 3134, 7738, 7479, 9201, 1390, 9803, 7896, 417, 4129, 3602, 6995, 11402, 9966, 3660, 7021, 6723, 12080, 2513, 8640, 3232, 1480, 11538, 8935, 9910, 9303, 3200, 502, 3052, 10342, 11650, 3987, 548, 7403, 6473, 7507, 2553, 5903, 7107, 9098, 8533, 8662, 6160, 3953, 11331, 8506, 11057, 117, 7600, 10640, 6738, 3976, 10984, 3758, 10946, 176, 7266, 10027, 10239, 2537, 8186, 5027, 798, 10865, 1512, 1780, 5524, 16, 6786, 7760, 12230, 2287, 3957, 1369, 4822, 7564, 4190, 5516, 9249, 2372, 11040, 3872, 6538, 6997, 4628, 11758, 1335, 7848, 5118, 3450, 4156, 11664, 12152, 8434, 7638, 469, 10259, 1559, 11118, 2535, 9996, 11002, 6915, 8155, 8928, 4439, 7955, 8512, 10494, 2275, 2820, 3668, 844, 6512, 700, 9578, 10418, 4576, 6824, 5445, 8295, 9905, 11037, 2720, 177, 1998, 5773, 5577, 8568, 6106, 1578, 2623, 11254, 3728, 8339, 11609, 901, 3837, 5680, 1953, 6450, 5739, 5368, 437, 6130, 2461, 9821, 3338, 7799, 1734, 9457, 3210, 618, 12095, 11227, 9481, 5003, 9864, 9486, 4805, 10530, 10275, 3713, 3191, 8425, 7610, 6143, 10536, 9471, 12108, 11352, 5842, 4592, 8727, 2563, 627, 5869, 1548, 1970, 1733, 5160, 7319, 2750, 1370, 4374, 8841, 5858, 2395, 821, 11197, 6565, 10076, 1307, 4280, 2759, 5462, 11212, 7327, 2809, 10076, 6189, 3294, 3639, 9909, 6510, 6751, 6637, 5006, 2045, 6363, 6459, 8260, 517, 526, 4933, 7061, 4094, 609, 11396, 11859, 1884, 2154, 9985, 3415, 9713, 2624, 9119, 2582, 2898, 1077, 6098, 6873, 7636, 1216, 510, 7008, 5556, 3099, 140, 7852, 5143, 11168, 2943, 4080, 4698, 2880, 2050, 5096, 6547, 1246, 10322, 12102, 6339, 9526, 9017, 2933, 5237, 4288, 978, 8122, 9914, 473, 5252, 8371, 1831, 3297, 6602, 11880, 2108, 3526, 489, 4624, 7733, 5017, 5687, 5364, 10303, 5459, 6893, 7953, 9696, 11722, 9930, 6897, 2128, 7722, 10462, 11267, 4228, 10659, 10603, 6664, 6249, 5615, 1540, 8318, 6310, 10739, 10148, 7620, 6891, 2079, 6683, 2139, 250, 11616, 1093, 5333, 8172, 7227, 6995, 2887, 3080, 10395, 4221, 2389, 2153, 849, 12038, 2003, 6500, 5349, 494, 1380, 12157, 12066, 11883, 4276, 11020, 7254, 7453, 5161, 10804, 2660, 966, 1018, 2207, 1919, 1902, 11147, 4791, 8320, 11575, 4716, 3079, 9846, 9287, 1032, 11241, 1394, 5998, 6589, 9878, 1941, 10585, 8634, 7300, 6239, 9786, 6912, 3962, 5705, 7836, 856, 11593, 6416, 12000, 6077, 10059, 6409, 5228, 4950, 7755, 7622, 7991, 10603, 4483, 9815, 8869, 8213, 4161, 11312, 5749, 3181, 10880, 6706, 3862, 1495, 6259, 7778, 5855, 10255, 4297, 3839, 40, 11469, 5150, 11437, 7803, 12204, 7802, 8663, 2994, 8102, 2266, 9498, 3676, 2296, 7993, 10904, 7745, 7537, 5133, 3984, 3847, 11128, 2428, 1550, 2251, 6529, 11296, 11133, 5886, 9540, 12197, 3360, 3645, 1185, 7934, 10282, 980, 7621, 10820, 9237, 2820, 2466, 3738, 10546, 3830, 10867, 12082, 6906, 5087, 7873, 2023, 4363, 4935, 12271, 11140, 1360, 7186, 643, 1483, 7120, 12039, 9611, 5539, 4611, 8194, 10072, 7102, 1369, 11835, 589, 2000, 4633, 11693, 7039, 4831, 3929, 1482, 8896, 6305, 9889, 266, 11215, 3320, 9714, 797, 7661, 5084, 170, 9978, 839, 1444, 11593, 10540, 1367, 562, 8558, 11009, 11553, 11346, 1196, 12044, 2744, 12154, 3239, 5068, 3958, 11638, 11568, 6494, 10462, 9510, 1027, 9255, 3977, 11074, 5863, 5587, 4215, 1234, 2356, 10017, 3039, 1033, 3194, 9947, 7181, 3897, 3297, 4471, 1886, 11560, 9605, 9564, 7603, 6983, 11341, 607, 9704, 2034, 2619, 10263, 6901, 2334, 4442, 10645, 3295, 7080, 7575, 12108, 5211, 6594, 2354, 8783, 10934, 3268, 2013, 11886, 332, 3782, 8703, 5932, 10426, 2685, 3651, 8770, 2860, 442, 3116, 652, 6096, 9130, 5976, 1138, 6162, 11681, 6926, 5477, 3670, 11878, 2131, 11635, 7647, 832, 181, 6921, 11873, 6041, 6654, 9636, 4499, 10917, 4146, 6173, 5937, 4144, 2548, 10815, 2969, 2305, 9795, 1849, 1605, 997, 3332, 11247, 7949, 496, 11367, 9238, 9257, 580, 7601, 3383, 2990, 8716, 57, 1176, 9446, 12266, 10196, 3664, 6232, 8377, 12086, 8505, 1625, 8735, 1036, 12015, 11965, 2351, 2335, 10495, 1008, 492, 7955, 1627, 8529, 9386, 10516, 6782, 12192, 12202, 12072, 7291, 11030, 8651, 2358, 12121, 6461, 1040, 1014, 1108, 11560, 1927, 5139, 9494, 7473, 10729, 3213, 11121, 6656, 1956, 3694, 8777, 9918, 3154, 6010, 11639, 5086, 10542, 7071, 8305, 1348, 5724, 5869, 1392, 6238, 8830, 11979, 9270, 7850, 3051, 6817, 7518, 96, 894, 3631, 9777, 11654, 11984, 3050, 674, 5540, 42, 4655, 6145, 9771, 7205, 6522, 4602, 8817, 10648, 1086, 602, 4419, 3694, 8262, 4017, 10681, 3881, 1052, 2361, 12004, 3174, 776, 8340, 8925, 6870, 4320, 9477, 6765, 12117, 10480, 10355, 6809, 247, 8136, 3650, 5200, 6508, 11996, 1427, 5844, 203, 4824, 10552, 4090];
    let nonce = [5; 32];
    let mut a = [0; N];

    uniform(&mut a, &nonce);

    for i in 0..N {
        assert_eq!(a[i], output[i]);
    }
}

#[cfg(feature = "tor")]
#[test]
fn test_uniform() {
    let output = [32769, 21840, 1218, 1909, 12795, 18531, 1802, 657, 19479, 57793, 2819, 32473, 26717, 33238, 60574, 11417, 55447, 15656, 38480, 10371, 53052, 58009, 37938, 196, 26523, 58667, 18058, 2080, 23393, 36492, 55917, 16337, 17590, 32994, 59292, 56175, 2201, 11702, 14368, 46368, 25953, 3189, 56269, 19957, 21605, 17289, 24388, 18443, 23067, 27724, 513, 23943, 40682, 49998, 9780, 9909, 30688, 14303, 23434, 48270, 31282, 742, 32330, 1148, 17477, 33444, 26166, 37438, 10063, 47029, 5268, 1100, 57371, 21973, 50867, 32440, 12792, 33020, 47793, 26735, 52824, 27378, 46291, 37373, 16862, 5948, 36656, 37258, 5086, 40744, 59829, 46722, 19943, 38182, 23044, 39028, 24659, 43893, 14980, 54813, 50853, 25251, 51220, 1200, 21771, 16320, 11217, 49482, 49376, 1849, 38100, 45285, 27070, 46800, 28162, 32214, 3765, 12134, 33770, 36867, 16730, 56833, 20656, 3909, 53647, 61225, 21310, 31030, 21343, 34767, 37338, 41167, 9714, 30209, 49667, 10933, 6528, 53684, 18493, 2221, 58247, 9760, 16414, 180, 60847, 6568, 30305, 4684, 22396, 39152, 37595, 15076, 23662, 53756, 5206, 41854, 54153, 25404, 47442, 19332, 10843, 16771, 38774, 16064, 47576, 55541, 54424, 35974, 33716, 60582, 11697, 45679, 53630, 50720, 32946, 10631, 56020, 13960, 24607, 15331, 1967, 33616, 59409, 431, 51127, 34270, 29430, 7157, 32589, 38464, 1768, 54126, 39065, 35848, 2657, 12419, 7886, 15861, 46182, 4183, 4258, 54072, 7834, 26427, 47482, 35606, 51706, 37117, 7562, 54573, 5680, 20994, 5003, 38076, 52900, 23970, 5025, 19022, 52448, 27954, 26351, 20987, 34035, 45897, 35087, 54288, 60383, 29653, 19796, 34442, 42974, 52977, 21975, 43553, 36387, 55757, 16796, 56799, 32188, 18432, 59692, 19645, 24397, 48219, 42709, 53748, 25481, 39430, 34802, 16047, 21201, 13837, 10418, 1733, 5160, 56475, 28169, 15542, 12226, 49664, 28612, 48121, 31015, 20814, 12626, 24138, 12738, 36502, 52108, 36575, 54951, 35398, 36161, 32126, 23280, 29419, 15815, 42511, 53522, 20968, 46911, 50006, 19694, 55414, 9636, 48134, 60741, 17668, 15498, 40380, 11686, 32744, 27122, 45401, 17290, 30481, 13168, 14493, 32708, 8533, 17585, 11111, 11207, 34512, 14340, 36135, 354, 25305, 58219, 27810, 33889, 53132, 58091, 47140, 10946, 37043, 58993, 42518, 53320, 27678, 3134, 32316, 56635, 9201, 1390, 34381, 44763, 417, 16418, 15891, 43862, 11402, 46833, 28238, 47209, 19310, 43590, 48947, 51669, 57796, 32178, 35218, 50636, 11538, 10984, 9910, 21592, 15489, 37369, 52208, 59183, 60806, 16276, 49704, 31981, 43340, 20723, 19927, 39420, 4805, 35108, 7107, 9098, 40058, 8662, 6160, 40820, 48198, 29017, 8506, 60213, 49273, 55606, 51431, 19027, 52824, 25205, 27039, 55025, 58734, 32377, 7266, 31402, 22528, 14826, 8186, 5027, 49954, 35443, 26090, 1780, 30102, 12305, 31364, 20049, 36808, 2287, 16246, 1369, 29400, 7564, 7955, 16479, 5516, 58405, 26950, 23329, 53028, 18827, 18801, 43864, 16917, 48625, 25913, 7848, 41985, 28028, 41023, 36242, 12152, 37044, 14287, 12758, 34837, 38426, 35696, 27113, 34574, 35580, 6915, 8155, 33506, 37768, 9471, 57668, 35072, 44816, 18028, 2820, 25015, 50000, 22564, 12989, 27916, 26548, 41443, 32917, 17734, 20584, 22194, 11037, 15009, 5705, 25074, 18062, 5577, 33146, 30684, 50734, 27201, 23543, 3728, 42095, 11609, 37485, 3837, 14647, 51109, 15953, 50196, 42235, 57533, 30708, 33964, 34399, 19071, 45602, 38601, 55862, 52366, 40729, 48962, 35073, 34059, 55617, 12781, 46353, 16128, 40, 48336, 3713, 36015, 36770, 45292, 56958, 33241, 28475, 33229, 39133, 24410, 60716, 40543, 33305, 20282, 11560, 23630, 607, 34282, 58650, 14908, 38472, 23018, 15039, 23501, 25948, 53530, 37393, 8841, 55014, 14684, 52795, 13110, 60353, 31143, 10076, 1307, 16569, 2759, 17751, 27202, 44194, 15098, 2898, 10076, 6189, 27872, 56792, 46776, 36437, 31088, 43618, 6637, 17295, 51201, 43230, 18748, 45127, 49673, 53780, 29511, 43928, 53250, 37476, 23685, 30037, 1884, 39021, 22274, 39966, 40282, 22002, 58682, 33697, 51738, 2128, 37944, 30676, 56029, 16517, 25794, 54408, 510, 56164, 17845, 5615, 24718, 43206, 44719, 17432, 39800, 11168, 2943, 4080, 4698, 39747, 38917, 17385, 31125, 14120, 13535, 10322, 12102, 36300, 57258, 45884, 43764, 42104, 53444, 978, 20411, 34492, 37340, 23601, 57527, 6664, 40164, 6602, 11880, 38975, 1495, 28104, 25067, 5855, 7733, 54173, 42554, 54520, 47170, 17439, 22892, 56049, 18538, 57109, 58852, 27572, 41583, 22219, 46713, 46365, 20011, 35040, 11267, 35482, 59815, 7537, 42000, 53140, 60036, 16136, 13829, 32896, 18599, 23028, 22437, 44487, 19180, 14368, 31261, 14428, 37117, 11616, 1093, 5333, 8172, 19516, 43862, 39754, 39947, 10395, 28799, 39256, 39020, 25427, 12038, 38870, 6500, 54505, 494, 38247, 24446, 48933, 61039, 16565, 47887, 19543, 19742, 29739, 47671, 39527, 37833, 1018, 51363, 38786, 1902, 23436, 10585, 29369, 57476, 36153, 58942, 15368, 20280, 10603, 9287, 50188, 11241, 13683, 5998, 18878, 22167, 1941, 61360, 33212, 7300, 43106, 3332, 43779, 3962, 7661, 7836, 50012, 23882, 30994, 53317, 24289, 30655, 10059, 6409, 25140, 29528, 32333, 46313, 44489, 32356, 50352, 16772, 9815, 8869, 45080, 32323, 38794, 18038, 27759, 24491, 55650, 11128, 46377, 2335, 18548, 50164, 60230, 47122, 4297, 26205, 45396, 39223, 22805, 52195, 56959, 15483, 36650, 44048, 58792, 16788, 53627, 1886, 55329, 46472, 39163, 13397, 59971, 2969, 39172, 42006, 51190, 19762, 61171, 24254, 14717, 26128, 51407, 31107, 23585, 11133, 30464, 52986, 9540, 61353, 27938, 28223, 25763, 20223, 10282, 50136, 19910, 47687, 9237, 51976, 27044, 28316, 47413, 4831, 10867, 24371, 56062, 41954, 20162, 38890, 37133, 53519, 17224, 49138, 11140, 50516, 7186, 12932, 38350, 56276, 12039, 9611, 54695, 16900, 57350, 59228, 9947, 7102, 13658, 48702, 25167, 14289, 4633, 23982, 43906, 7603, 6983, 40796, 13771, 8896, 6305, 59045, 34841, 19190, 11215, 52476, 46581, 25375, 56236, 17373, 49326, 9978, 49021, 49995, 26022, 11593, 10540, 13656, 11886, 45425, 11009, 40164, 11553, 23635, 35004, 12044, 51900, 28793, 50390, 15528, 5068, 28536, 23927, 60724, 18265, 35040, 30740, 13316, 9255, 40844, 40537, 5863, 30165, 48502, 32225, 832, 22306, 22084, 25611, 24162, 42908, 31232, 35608, 40649, 10917, 41013, 28876, 42804, 9564, 27126, 61242, 20794, 7601, 50781, 14138, 45997, 33294, 7291, 51490, 41309, 22934, 52451, 44902, 19864, 48975, 29789, 31172, 26932, 57939, 10934, 27846, 14302, 46219, 38005, 37199, 11681, 45570, 18221, 31656, 2685, 3651, 45637, 52016, 442, 52272, 12941, 30674, 1036, 14082, 997, 50332, 46038, 43793, 42344, 15397, 48745, 2131, 33816, 9257, 12869, 49337, 40250, 19210, 37167, 42741, 49213, 2351, 46346, 24555, 34774, 53201, 56587, 53300, 57317, 3755, 1724, 42249, 32175, 2990, 8747, 8631, 6292, 20653, 17143, 8231, 60403, 26747, 26526, 35945, 49960, 1167, 31768, 33548, 27888, 59255, 37650, 29929, 28760, 36332, 58708, 52890, 3691, 24722, 51653, 6232, 30201, 22734, 57886, 40884, 57131, 10288, 46247, 29071, 8468, 5795, 33024, 36883, 37158, 22989, 23868, 7955, 26672, 50340, 59870, 25618, 14993, 53683, 11599, 28407, 60394, 6348, 6212, 30253, 23641, 46979, 11677, 1014, 31809, 24648, 19241, 14301, 8145, 28766, 9493, 59515, 1727];
    let nonce = [5; 32];
    let mut a = [0; N];

    uniform(&mut a, &nonce);

    for i in 0..N {
        assert_eq!(a[i], output[i]);
    }
}

#[cfg(feature = "tor")]
#[test]
fn test_discardtopoly() {
    let mut a = [0; N];
    let x = [5; SHAKE128_RATE * 16];

    discardtopoly(&mut a, &x);

    for i in 0..N {
        assert_eq!(a[i], 1285);
    }
}

#[test]
fn test_frombytes_tobytes() {
    let a = [35572; N];

    let b = poly_tobytes(&a);
    let a = poly_frombytes(&b);

    for i in 0..N {
        assert_eq!(a[i], 10994);
    }
}
