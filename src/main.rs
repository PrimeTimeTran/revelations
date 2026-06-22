use revelation::config::Config;
use revelation::evaluator::Evaluator;

fn main() {
    let config = Config::load();

    let mut evaluator = Evaluator::new(config);
    evaluator.evaluate_fs();
}
