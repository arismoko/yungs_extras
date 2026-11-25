//! The data files name each other by id: placed features name features, features name
//! templates, biome modifiers name placed features. These tests resolve every such name.
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/yungsextras")
}

/// Every file under `dir` with the given extension, as (id path without extension, text).
fn files(dir: &Path, ext: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == ext) {
                let id = path.strip_prefix(dir).unwrap().with_extension("");
                let text = if ext == "json" {
                    fs::read_to_string(&path).unwrap()
                } else {
                    String::new()
                };
                out.push((id.to_string_lossy().replace('\\', "/"), text));
            }
        }
    }
    out.sort();
    out
}

/// The string values of `"key": "..."` in `json`, in file order.
fn strings<'a>(json: &'a str, key: &str) -> Vec<&'a str> {
    let needle = format!("\"{key}\": \"");
    json.match_indices(&needle)
        .map(|(at, _)| {
            let rest = &json[at + needle.len()..];
            &rest[..rest.find('"').unwrap()]
        })
        .collect()
}

fn ids(dir: &str, ext: &str) -> BTreeSet<String> {
    files(&data().join(dir), ext)
        .into_iter()
        .map(|(id, _)| format!("yungsextras:{id}"))
        .collect()
}

#[test]
fn every_feature_has_one_placed_feature_and_back() {
    let features = ids("worldgen/feature", "json");
    let placed: BTreeSet<String> = files(&data().join("worldgen/placed_feature"), "json")
        .iter()
        .map(|(_, text)| strings(text, "feature")[0].to_owned())
        .collect();
    assert_eq!(features.len(), 62);
    assert_eq!(placed, features);
}

#[test]
fn every_feature_names_a_template() {
    let templates = ids("structure", "nbt");
    for (id, text) in files(&data().join("worldgen/feature"), "json") {
        let named = strings(&text, "id");
        assert!(templates.contains(named[0]), "{id} names {}", named[0]);
    }
}

#[test]
fn biome_modifiers_add_every_placed_feature() {
    let placed = ids("worldgen/placed_feature", "json");
    let mut added = BTreeSet::new();
    for (id, text) in files(&data().join("neoforge/biome_modifier"), "json") {
        if text.contains("neoforge:add_features") {
            let list = &text[text.find("\"features\"").unwrap()..];
            let list = &list[..list.find(']').unwrap()];
            for name in list.split('"').skip(3).step_by(2) {
                assert!(placed.contains(name), "{id} adds unknown {name}");
                added.insert(name.to_owned());
            }
        }
    }
    assert_eq!(added, placed);
}

#[test]
fn no_file_names_the_original_java_types() {
    for dir in ["worldgen", "neoforge"] {
        for (id, text) in files(&data().join(dir), "json") {
            for key in ["type", "processor_type"] {
                for kind in strings(&text, key) {
                    let registered = ["pick", "count_cap", "yung_pillar"]
                        .iter()
                        .any(|name| kind == format!("yungsextras:{name}"));
                    assert!(
                        !kind.starts_with("yungsextras:")
                            || (key == "processor_type" && registered),
                        "{id}: {kind}"
                    );
                }
            }
        }
    }
}
