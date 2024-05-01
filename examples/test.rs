fn main() {
    // let mut a: u8 = 0x80;
    // let mut b: i8 = -2;
    // a = a.wrapping_add(b as u8);
    // print!("{}", b as u8);
    // let a = vec!(1,2,3,4);
    // let b: u8 = 1;
    // let c = &a[b];
    // let c: u16 = 0b1111_1111_1101_1101;
    // print!("{:b}", c as u8)

    let mut a: u8 = 10;
    let mut b: i8 = -10;
    print!("{}", a as i8 > b);
    let a = abc(move || 1);
    a.a();
    // let x = || a+1;
    efg(B{})
}

fn abc<F>(mut f: F) -> impl A
where F: FnMut() -> i32,
{
    f();
    let a: B = B{};
    // let b: Box<B> = Box::new(a);
    a
}

fn cde(mut f: i32) {
    f = 2;
}

struct B {

}
trait A {
    fn a(&self) -> i32;
}


impl A for B {
    fn a(&self) -> i32  {
        1
    }
}

fn efg<G: A>(a: G) {

}

fn ghj<R>() -> R 
where R: A
{
    B{}
}