use vergen::Config;

fn main() {
    vergen::vergen(Config::default()).expect("Unable to generate the cargo keys!");
}
