//! Anti-cheat audit (Phase 2 rule: do not fake emergence).
//!
//! 1. The mind crate may depend only on alife-core and alife-interface, so it
//!    cannot read world truth or body truth.
//! 2. The mind's source must not mention kinds of things or world-truth
//!    properties. It may talk about sensations, its own concepts, actions and
//!    body signals only.

use std::path::Path;

const FORBIDDEN: &[&str] = &[
    "fire", "flame", "food", "water", "rock", "stone", "flower", "metal", "berry", "berries", "edible",
    "nutrition", "kcal", "celsius", "temp_c", "objprops", "props.", "alife_world", "alife_biology", "poison",
    "fruit", "burn",
];

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().map_or(false, |x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn mind_depends_only_on_core_and_interface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../simulation/mind/Cargo.toml");
    let toml = std::fs::read_to_string(root).unwrap();
    let deps = toml.split("[dependencies]").nth(1).unwrap_or("");
    for line in deps.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let name = line.split('=').next().unwrap().trim();
        assert!(name == "alife-core" || name == "alife-interface", "mind depends on forbidden crate `{name}`");
    }
}

#[test]
fn mind_source_never_names_kinds_of_things() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../simulation/mind/src");
    let mut files = Vec::new();
    walk(&dir, &mut files);
    assert!(!files.is_empty());
    let mut violations = Vec::new();
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap().to_lowercase();
        for (ln, line) in text.lines().enumerate() {
            for w in FORBIDDEN {
                if line.contains(w) {
                    violations.push(format!("{}:{}: `{}` in: {}", f.display(), ln + 1, w, line.trim()));
                }
            }
        }
    }
    assert!(violations.is_empty(), "anti-cheat violations:\n{}", violations.join("\n"));
}
