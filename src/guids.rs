use anyhow::Context;
use atomicwrites::{AllowOverwrite, AtomicFile};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::Level;

#[instrument]
pub fn read_guids_file<P: AsRef<Path> + std::fmt::Debug>(
    path: P,
) -> anyhow::Result<HashSet<String>> {
    debug!("reading GUIDs");
    let file = File::open(&path).context("Could not open GUIDs file")?;
    let reader = BufReader::new(file);
    let guids: HashSet<String> =
        serde_json::from_reader(reader).context("Could not read GUIDs from JSON")?;

    event!(
        Level::DEBUG,
        num_guids = guids.len(),
        "reading GUIDs successful"
    );

    Ok(guids)
}

#[instrument(skip(guids), fields(num_guids = guids.len()))]
pub fn write_guids_file<P: AsRef<Path> + std::fmt::Debug>(
    path: P,
    guids: &HashSet<String>,
) -> anyhow::Result<()> {
    debug!("writing GUIDs");
    AtomicFile::new(&path, AllowOverwrite)
        .write(|file| serde_json::to_writer_pretty(file, guids))
        .context("Could not write GUIDs to file")?;

    debug!("writing GUIDs successful");
    Ok(())
}
