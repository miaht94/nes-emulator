use std::cmp::PartialOrd;

fn main() {
    let i = vec![A{}, A{}];
    let f = vec![9.2, 3.4, 1.0, -5.678, 0.0];
    let c = vec!['*', 'h', '!', '~', 'Q'];

    println!("{}", largest_bound(&i));
    println!("{}", largest_where(&f));
    println!("{}", largest_impl(&c));
}
#[derive(PartialEq, PartialOrd, Clone, Copy)]
struct A {

}
fn largest_bound<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut largest = list[0];
    for &item in list {
        if item > largest {
            largest = item;
        }
    }

    largest
}

fn largest_where<T:>(list: &[T]) -> T
    where T: PartialOrd + Copy
{
    // same body
}

fn largest_impl(list: &[(impl PartialOrd + Copy)]) -> impl PartialOrd + Copy {
    // same body
}