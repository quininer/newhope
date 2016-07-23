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


pub fn uniform(a: &mut [u16], nonce: &[u8]) {
    let (mut nblocks, mut pos, mut ctr) = (16, 0, 0);
    let mut buf = [0; SHAKE128_RATE * 16];
    let mut shake128 = Keccak::new_shake128();

    shake128.update(nonce);
    shake128.pad();
    shake128.keccakf();
    shake128.squeeze(&mut buf);

    while ctr < N {
        let val = LittleEndian::read_u16(&buf[pos..]) & 0x3fff;
        if val < Q as u16 {
            a[ctr] = val;
            ctr += 1;
        }
        pos += 2;

        if pos > SHAKE128_RATE * nblocks - 2 {
            nblocks = 1;
            shake128.squeeze(&mut buf[..SHAKE128_RATE]);
            pos = 0;
        }
    }
}

pub fn noise<R: Rng>(r: &mut [u16], rng: &mut R) {
    let mut buf = [0; 4 * N];
    rng.fill_bytes(&mut buf);

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

#[test]
fn test_uniform() {
    let output = [1, 5456, 1218, 1909, 2147, 1802, 657, 3095, 8641, 2819, 10333, 470, 11422, 11417, 6295, 5712, 10371, 3900, 8857, 5170, 196, 10139, 9515, 1674, 2080, 7009, 3724, 6765, 1206, 226, 7023, 2201, 11702, 9569, 7117, 3573, 5221, 905, 8004, 2059, 6683, 11340, 10140, 7559, 7914, 846, 9780, 9909, 3189, 7050, 742, 1148, 1093, 676, 9782, 4670, 5268, 1100, 8219, 5589, 1715, 252, 10351, 3672, 10994, 513, 4605, 3888, 4490, 5086, 7976, 10677, 3559, 6260, 8275, 11125, 5661, 10063, 8867, 2068, 1200, 5387, 11217, 330, 224, 1849, 6660, 10686, 478, 5948, 1002, 4099, 346, 4272, 3909, 4495, 5414, 4959, 1999, 4570, 8399, 9714, 515, 10933, 6528, 4532, 2109, 2221, 9095, 9760, 30, 180, 11695, 6568, 4684, 6012, 6384, 4827, 7278, 4604, 5206, 9086, 5001, 9020, 2948, 10843, 387, 6389, 5272, 948, 11430, 11697, 1568, 178, 10631, 6868, 6006, 12134, 10257, 3206, 1975, 1502, 4478, 5696, 4926, 6297, 3080, 2657, 7886, 4183, 4258, 4920, 7834, 10043, 2838, 2554, 5421, 8223, 4610, 1967, 848, 3748, 7586, 5025, 2638, 3296, 11570, 9967, 4603, 1768, 4349, 2319, 5136, 1701, 5308, 10206, 3825, 5591, 10785, 3619, 6605, 412, 7647, 5332, 7562, 3261, 11778, 3765, 9097, 431, 2034, 7681, 4817, 7157, 12073, 4974, 11785, 12226, 512, 12228, 4430, 7754, 3734, 2956, 3807, 5799, 2630, 3393, 6896, 4370, 4584, 854, 9636, 11589, 1284, 7612, 11686, 10738, 906, 6262, 1201, 11111, 11207, 1744, 3367, 354, 8921, 9067, 9743, 1121, 3310, 9841, 9750, 4168, 11294, 3134, 7483, 9201, 1390, 1613, 11995, 417, 34, 11094, 11402, 11854, 2926, 10822, 2517, 8644, 11426, 1484, 11538, 8939, 9910, 5208, 4601, 3056, 11654, 552, 10572, 3412, 6652, 7107, 9098, 8533, 8662, 6160, 8052, 8506, 11061, 121, 2450, 2643, 3980, 10984, 10946, 4275, 7266, 10031, 6144, 8186, 5027, 802, 2675, 9706, 1780, 3665, 4040, 2287, 1369, 7564, 95, 5516, 9253, 10566, 6945, 3876, 2443, 11096, 533, 9529, 7848, 9217, 11644, 8255, 3474, 12152, 4339, 3543, 2069, 5658, 2928, 10729, 1806, 2812, 6915, 8155, 738, 7955, 8516, 2304, 2279, 2820, 3672, 848, 2417, 9582, 10418, 8675, 1350, 4200, 5810, 11037, 4276, 1678, 5577, 378, 1582, 10817, 7159, 3728, 149, 11609, 5000, 3837, 5680, 1957, 6454, 1644, 9467, 8631, 10655, 1631, 11532, 5833, 1267, 3214, 4717, 11231, 1291, 5003, 1674, 4805, 2340, 6180, 3713, 7290, 2048, 10540, 9471, 8013, 9941, 4596, 537, 6662, 8821, 5873, 10164, 1733, 5160, 7323, 9564, 4378, 8841, 5862, 11201, 10076, 1307, 185, 2759, 1367, 7117, 11426, 10076, 6189, 11488, 3643, 10850, 6637, 911, 2049, 10462, 2364, 521, 4625, 11160, 4098, 4708, 7301, 3669, 1884, 6253, 5890, 7514, 5618, 10818, 929, 2586, 2898, 5176, 6877, 7640, 9410, 510, 7012, 1461, 7198, 8334, 11951, 1048, 11168, 2943, 4080, 4698, 6979, 6149, 1001, 10322, 12102, 10438, 9530, 7032, 9336, 4292, 978, 4027, 1724, 4572, 5256, 8375, 7396, 6602, 11880, 6207, 11720, 8683, 4628, 7733, 5021, 9786, 5368, 6897, 7957, 9700, 3532, 5835, 10996, 2128, 3627, 2272, 11267, 133, 10663, 6508, 6664, 2154, 5615, 128, 2215, 6644, 6053, 11719, 2796, 4349, 11616, 1093, 5333, 8172, 3132, 11094, 6986, 7179, 10395, 6488, 6252, 9043, 12038, 6102, 6500, 5353, 494, 5479, 8062, 11887, 181, 3159, 3358, 6759, 5065, 1018, 2211, 6018, 1902, 7052, 8324, 3385, 8815, 9287, 1036, 11241, 5998, 2494, 5783, 1941, 10585, 444, 7300, 10338, 9790, 11011, 3962, 5705, 7836, 860, 7498, 7905, 10059, 6409, 9327, 11721, 3896, 10603, 388, 9815, 8869, 4165, 7217, 1654, 11375, 10884, 6710, 7961, 1495, 2164, 5855, 4297, 40, 1055, 3247, 7807, 12208, 7806, 473, 11188, 8106, 6365, 7775, 6395, 3898, 2714, 7537, 9232, 3988, 11128, 9744, 2255, 7201, 11133, 9540, 12201, 11554, 11839, 9379, 3839, 10282, 984, 3526, 9237, 2824, 10660, 11932, 3834, 10867, 7987, 6910, 9186, 3778, 6122, 4367, 840, 11140, 1364, 7186, 5582, 7124, 12039, 9611, 5543, 516, 8198, 10076, 7102, 8783, 4633, 7598, 11138, 4831, 8028, 8896, 6305, 9893, 4365, 11215, 3324, 8991, 7661, 989, 174, 9978, 843, 9638, 11593, 10540, 8756, 11009, 11553, 7251, 1200, 12044, 2748, 5068, 12152, 7543, 11572, 6498, 2272, 9255, 8076, 11078, 5863, 1238, 6455, 5922, 3043, 9227, 9947, 11280, 12091, 7396, 4475, 1886, 11564, 9564, 7603, 6983, 7246, 607, 1514, 2038, 2073, 2806, 2338, 8541, 6550, 3299, 7084, 3480, 10548, 8787, 10934, 11462, 11886, 4431, 7881, 1837, 2236, 2685, 3651, 2864, 442, 3120, 1881, 5237, 11681, 11025, 9576, 7769, 2131, 832, 185, 2826, 7778, 10140, 9640, 404, 10917, 8245, 6177, 10036, 4148, 10742, 10819, 2969, 6404, 5700, 5704, 997, 3332, 11251, 12048, 8690, 3177, 1048, 9257, 7601, 7482, 2990, 526, 61, 1180, 8171, 2006, 6232, 8381, 12090, 4410, 1629, 1036, 12019, 7870, 2351, 2335, 2305, 1012, 7955, 9821, 1196, 6421, 2687, 4002, 8107, 3882, 7291, 2840, 461, 8026, 6465, 1044, 1014, 11560, 6026, 9238, 9498, 3378, 6634, 11121, 2561, 1960, 9922, 11348, 10109, 3449, 2976, 115, 5447, 9968, 4735, 11979, 5175, 3755, 7150, 6817, 7522, 4993, 7730, 5682, 11658, 11984, 8868, 5540, 42, 6149, 1581, 7209, 10621, 8701, 627, 10648, 1090, 8796, 4423, 3694, 4167, 10681, 3885, 5151, 10555, 7909, 11368, 8340, 8925, 2775, 225, 1287, 6765, 3927, 6385, 8441, 8140, 3654, 1105, 6508, 7901, 1431, 9943, 203, 8923, 4090, 10017, 7305, 7644, 11082, 9973, 2445, 8893, 1962, 10752, 377, 5190, 2719, 4803, 4021, 396, 9931, 115, 8829, 3267, 6105, 7639, 10286, 8453, 4360, 1461, 9778, 4459, 8465, 10636, 6425, 9203, 12134, 5872, 6727, 11606, 10281, 762, 2175, 7060, 3624, 9755, 5048, 11385, 6878, 5066, 3664, 8670, 1607, 3696, 8839, 3929, 3347, 5369, 11358, 10529, 1446, 4184, 11850, 4399, 9973, 9408, 11453, 5888, 4049, 7435, 11002, 10380, 3755, 11035, 9481, 1133, 8747, 6292, 759, 8231, 10121, 10363, 10142, 2938, 808, 1167, 11252, 780, 11504, 10103, 4882, 2519, 9556, 3738, 3691, 8338, 2501, 5773, 6350, 3829, 8116, 7979, 10288, 8468, 9814, 256, 4829, 4390, 6605, 7484, 6302, 10288, 1188, 9234, 11029, 11599, 11242, 6212, 11364, 7257, 10076, 11522, 8165, 7791, 1724, 3467, 8145, 11939, 9493, 8631, 1727, 4269, 1165, 1447, 1626, 7427, 8967, 2002, 7649, 507, 349, 7631, 10156, 5958, 11801, 3564, 7523, 1354, 4414, 1516, 5546, 8152, 6761, 8734, 3038, 4826, 8585, 1534, 2688, 5795, 847, 4115, 2140, 5912, 911, 145, 8775, 10718, 9288, 4307, 4531, 8543, 12023, 6348, 2046, 11677, 8264, 2857, 5341, 10363, 11187, 7863, 3292, 11734, 1509, 4054, 3355, 11397, 4867, 621, 5881, 466, 10326, 1119, 10901, 3608, 4180, 1889, 2896, 9540];
    let nonce = [5; 32];
    let mut a = [0; N];

    uniform(&mut a, &nonce);

    for i in 0..N {
        assert_eq!(a[i], output[i]);
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
