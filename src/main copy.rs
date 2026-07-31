use revelation::config::Config;
use revelation::evaluator::Evaluator;
use std::path::PathBuf;
// fn main() {
//     let config = Config::load();

//     let mut evaluator = Evaluator::new(config);
//     evaluator.evaluate_fs();
// }
fn main() {
    let mut config = Config::default();
    config.analysis_root = PathBuf::from("./src")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("./src"));
    config.output_name = String::from("eval-debug.md");
    let mut evaluator = Evaluator::new(config);
    evaluator.evaluate_fs();
}

// pub fn main() {
//     let foo = "1";

//     let bar = foo;

//     let spam = bar;

//     let ham = spam;

//     if foo >= 100 {
//         println!("Hi foo")
//     }

//     if bar <= 100 {
//         println!("Hi bar")
//     }

//     if bar >= 100 && foo == 1000 {
//         println!("Hi bar")
//     }
// }
