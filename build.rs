use vergen::ConstantsFlags;

fn main() {
    vergen::gen(ConstantsFlags::all()).expect("Unable to generate the cargo keys!");
}
