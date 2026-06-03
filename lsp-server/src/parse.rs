//! Parsing helpers for the ZSBC CraftTweaker log dumper format.
//!
//! The dumper script that ships with `zenscript-bracket-completion` writes a
//! block of `<entry> = <info>` lines into `crafttweaker.log` between two
//! sentinel markers. We find the *last* such block and extract the entries.

use std::collections::BTreeMap;

/// Sentinel lines emitted by the upstream dumper scripts.
const MARKER_START: &str = "[ZSBC DUMPER START]";
const MARKER_END: &str = "[ZSBC DUMPER END]";

/// Parse a CraftTweaker log file's contents and return the most recent dump
/// block as a map from `<entry>` to its `info` string.
///
/// Returns `None` if the log does not contain a complete dump block.
pub fn parse_ct_log(contents: &str) -> Option<BTreeMap<String, String>> {
    let lines: Vec<&str> = contents.lines().collect();

    // Search backwards so we get the last (most recent) dump, matching the
    // behaviour of the upstream VSCode extension.
    let end = lines.iter().rposition(|l| l.contains(MARKER_END))?;
    let start = (0..end).rev().find(|&i| lines[i].contains(MARKER_START))?;

    let mut map = BTreeMap::new();
    for raw in &lines[start + 1..end] {
        if let Some((key, value)) = raw.split_once(" = ") {
            let key = key.trim();
            if key.is_empty() {
                continue;
            }
            map.insert(key.to_string(), value.trim().to_string());
        }
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

/// Merge entries from an "additional list" file into an existing map.
///
/// Each line that starts with `<` and contains ` = ` is treated as an entry.
/// Entries are only added if the key is not already present in `into`.
pub fn merge_additional(into: &mut BTreeMap<String, String>, contents: &str) {
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('<') {
            continue;
        }
        if let Some((key, value)) = line.split_once(" = ") {
            let key = key.trim();
            if key.is_empty() || into.contains_key(key) {
                continue;
            }
            into.insert(key.to_string(), value.trim().to_string());
        }
    }
}

