use ::params::Q;

const QINV: u32 = 12287;
const RLOG: u32 = 18;


pub fn montgomery_reduce(a: u32) -> u16 {
    let mut u = a.wrapping_mul(QINV);
    u &= (1 << RLOG) - 1;
    u *= Q as u32;
    ((a + u) >> 18) as u16
}

pub fn barrett_reduce(a: u16) -> u16 {
    let mut u = (5 * a as u32) >> 16;
    u *= Q as u32;
    a - u as u16
}


#[test]
fn test_montgomery_reduce() {
    assert_eq!(montgomery_reduce(3), 1728);
    assert_eq!(montgomery_reduce(34), 7295);
    assert_eq!(montgomery_reduce(345), 2096);
    assert_eq!(montgomery_reduce(3456), 12127);
    assert_eq!(montgomery_reduce(34567), 2412);
    assert_eq!(montgomery_reduce(345678), 4150);
    assert_eq!(montgomery_reduce(3456789), 9817);
    assert_eq!(montgomery_reduce(34567890), 12147);
}

#[test]
fn test_barrett_reduce() {
    assert_eq!(barrett_reduce(3), 3);
    assert_eq!(barrett_reduce(34), 34);
    assert_eq!(barrett_reduce(345), 345);
    assert_eq!(barrett_reduce(3456), 3456);
    assert_eq!(barrett_reduce(34567), 9989);
}
