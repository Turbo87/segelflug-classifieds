use anyhow::Context;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::path::Path;

pub fn read_guids_file<P: AsRef<Path> + std::fmt::Debug>(
    path: P,
) -> anyhow::Result<HashSet<String>> {
    debug!("reading GUIDs from {:?}", path);
    let file = File::open(&path).context("Could not open GUIDs file")?;
    let reader = BufReader::new(file);
    let guids: HashSet<String> =
        serde_json::from_reader(reader).context("Could not read GUIDs from JSON")?;
    debug!("read {} GUIDs from {:?}", guids.len(), path);
    Ok(guids)
}

pub fn write_guids_file<P: AsRef<Path> + std::fmt::Debug>(
    path: P,
    guids: &HashSet<String>,
) -> anyhow::Result<()> {
    debug!("writing {} GUIDs to {:?}", guids.len(), path);
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&path)
        .context("Could not open GUIDs file")?;

    serde_json::to_writer_pretty(&file, guids).context("Could not serialize GUIDs to JSON")?;
    debug!("wrote {} GUIDs to {:?}", guids.len(), path);
    Ok(())
}
