pub fn square(num: f32) -> f32 {
    return num + num;
}

pub fn main() {
    let pi = 3.14;

    let hi = pi.clone();

    let number = square(pi);

    println!("Hi {}", hi);
}
