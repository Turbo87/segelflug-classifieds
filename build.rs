use vergen_gitcl::{Emitter, GitclBuilder};

pub fn main() -> anyhow::Result<()> {
    let gitcl = GitclBuilder::default().sha(true).build()?;
    Emitter::default().add_instructions(&gitcl)?.emit()?;
    Ok(())
}
