use vergen::EmitBuilder;

fn main() {
    EmitBuilder::builder()
        .git_sha(true)
        .emit()
        .expect("Unable to generate the cargo keys!");
}
