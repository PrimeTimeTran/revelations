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



// pub fn main() {
//     let foo = 1000;
//     let bar = 1000;

//     if 250 < foo + bar {
//         println!("Which");
//     } else if (1000 < 500) {
//         println!("Block");
//     } else {
//         println!("Prints");
//     }
// }

// fn foo() {}
// fn bar() {}
// fn spam() {}


// pub fn main() {
//     let foo = 1000;
//     let bar = 1000;

//     if 250 < foo + bar {
//         println!("Which");
//         foo();
//     } else if (1000 < 500) {
//         println!("Block");
//         bar();
//     } else {
//         println!("Prints");
//         spam();
//     }
// }


// fn main() {
//     let number = 1000;

//     if number >= 100 {
//         println!("If I changed 1000, doesn't that effect me?")
//     }
// }