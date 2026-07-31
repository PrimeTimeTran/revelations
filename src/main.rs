pub fn main() {
    let foo = 1;
    let bar: i32 = 10;

    if foo >= 100 {
        println!("Hi foo")
    }

    if bar <= 100 {
        println!("Hi bar")
    }

    if bar >= 100 && foo == 1000 {
        println!("Hi bar")
    }
}
