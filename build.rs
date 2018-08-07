extern crate lalrpop;

use lalrpop::Configuration;

fn main() {
    let mut config = Configuration::new();
    config.use_cargo_dir_conventions();
    config.set_in_dir("lang/src");
    config.process().unwrap();
}
