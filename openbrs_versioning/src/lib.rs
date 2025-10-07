use openbrs_main_structs::{Change, ChangeType, Commit, FilePath, Tree};
use serde_json;
use std::{
    collections::HashMap,
    fs,
    path::{self, Path},
};

pub fn compare_trees(old_tree: &Tree, new_tree: &Tree, paths: FilePath) {
    /*
    // Read Latest commit ID
    let latest_commit_id = fs::read_to_string(paths.main.join("HEAD")).unwrap();

    // Look for the commit, parse it, and extract the main tree's ID.
    let latest_commit =
        fs::read_to_string(paths.commits.join(format!("{}.json", latest_commit_id))).unwrap();

    // Make it a Commit instance
    let latest_commit: Commit = serde_json::from_str(&latest_commit).ok().unwrap();

    // Get Tree ID
    let old_tree_id = latest_commit.tree_id;

    // Read tree (string)
    let old_tree = fs::read_to_string(paths.trees.join(format!("{}.json", old_tree_id))).unwrap();

    // Read tree (Tree)
    let old_tree: Tree = serde_json::from_str(&old_tree).ok().unwrap();*/

    let mut all_changes: Vec<Change> = Vec::new();

    // First, compare the current level
    let level_changes = current_level_diff(&old_tree, &new_tree);

    // Iterate
    for change in level_changes {
        // If it's a modification (not an addition/removal) to a directory:
        if change.change_type == ChangeType::Modified && change.kind == EntryKind::Dir {
            // A directory was changed
            if let (Some(old_tree_id), Some(new_tree_id)) = (&change.old_id, &change.new_id) {
                // Read old tree, then prase it
                let old_tree =
                    fs::read_to_string(paths.trees.join(format!("{}.json", old_tree_id))).unwrap();
                let old_tree: Tree = serde_json::from_str(&old_tree).ok().unwrap();

                // Read new tree and parse it
                let new_tree =
                    fs::read_to_string(paths.trees.join(format!("{}.json", new_tree_id))).unwrap();
                let new_tree: Tree = serde_json::from_str(&new_tree).ok().unwrap();

                // Recurse
                compare_trees(&old_tree, &new_tree, &paths);
            }
        } else {
            //
        }
        all_changes.push(change);
    }
}

fn current_level_diff(old_tree: &Tree, new_tree: &Tree) -> Option<Vec<Change>> {
    // Did anything change in the tree?
    if old_tree.id != new_tree.id {
        // Content has changed, continue looking for what has changed
        // Convert the two trees to maps
        let old_map: HashMap<_, _> = old_tree
            .entries
            .iter()
            .map(|f| (f.name.clone(), (f.id.clone(), f.kind.clone())))
            .collect();
        let new_map: HashMap<_, _> = new_tree
            .entries
            .iter()
            .map(|f| (f.name.clone(), (f.id.clone(), f.kind.clone())))
            .collect();

        // We store changes in this variable
        let mut changes = Vec::new();

        // iterate
        for (name, (id, kind)) in &new_map {
            match old_map.get(name) {
                // If you cannot find it:
                None => changes.push(Change {
                    change_type: ChangeType::Added,
                    name: name.clone(),
                    kind,
                    old_id: None,
                    new_id: Some(id.clone()),
                }),

                // If you can, but the ID has changed:
                Some(old_id) if old_id != id => changes.push(Change {
                    change_type: ChangeType::Modified,
                    name: name.clone(),
                    kind,
                    old_id: Some(old_id.clone()),
                    new_id: Some(id.clone()),
                }),

                // Otherwise, there's no change in here
                _ => {}
            }
        }

        // We also need to detect removed entries:
        for (name, (old_id, kind)) in old_map {
            if !new_map.contains_key(&name) {
                changes.push(Change {
                    change_type: ChangeType::Removed,
                    name: name.clone(),
                    kind,
                    old_id: Some(old_id.clone()),
                    new_id: None,
                });
            }
        }

        // Return changes
        Some(changes)
    } else {
        // Nothing has changed, stop here.
        None
    }
}
