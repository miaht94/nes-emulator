fn main() {
    // let mut a: u8 = 0x80;
    // let mut b: i8 = -2;
    // a = a.wrapping_add(b as u8);
    // print!("{}", b as u8);
    let a = vec!(1,2,3,4);
    let b: u8 = 1;
    // let c = &a[b];
    let c: u16 = 0b1111_1111_1101_1101;
    print!("{:b}", c as u8)
}