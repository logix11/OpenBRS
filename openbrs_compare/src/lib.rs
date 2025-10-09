use openbrs_main_structs::{Change, ChangeType, FilePath, Tree};
use serde_json;
use std::{collections::HashMap, fs};

pub fn compare_trees(old_tree: &Tree, new_tree: &Tree, paths: &FilePath) -> Vec<Change> {
    let mut all_changes: Vec<Change> = Vec::new();

    // First, compare the current level
    let level_changes = current_level_diff(&old_tree, &new_tree);

    // there are any changes
    match level_changes {
        None => {
            // There are no new changes, don't push the directory, rather, return an empty vector.
            all_changes
        }
        Some(level_changes) => {
            // We now want to cover the changes that occured in the lower level; if any. So, if it is an added directory, a
            // modified/added file, or a removal, I will simply save the change in all_changes.
            for change in level_changes {
                // If it's a modification (not an addition/removal) to a directory:
                match (&change.change_type, &change.path.is_dir()) {
                    (ChangeType::Modified, true) => {
                        // A directory was changed
                        if let (Some(old_tree_id), Some(new_tree_id)) =
                            (&change.old_id, &change.new_id)
                        {
                            // Read old tree, then prase it
                            let old_tree = fs::read_to_string(
                                paths.trees.join(format!("{}.json", old_tree_id)),
                            )
                            .unwrap();
                            let old_tree: Tree = serde_json::from_str(&old_tree).ok().unwrap();

                            // Read new tree and parse it
                            let new_tree = fs::read_to_string(
                                paths.trees.join(format!("{}.json", new_tree_id)),
                            )
                            .unwrap();
                            let new_tree: Tree = serde_json::from_str(&new_tree).ok().unwrap();

                            // Recurse
                            let sub_changes = compare_trees(&old_tree, &new_tree, &paths);
                            all_changes.extend(sub_changes);
                        }
                    }
                    _ => {}
                }
                all_changes.push(change);
            }
            all_changes
        }
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
            .map(|f| (f.name.clone(), (f.id.clone(), f.path.clone())))
            .collect();
        let new_map: HashMap<_, _> = new_tree
            .entries
            .iter()
            .map(|f| (f.name.clone(), (f.id.clone(), f.path.clone())))
            .collect();

        // We store changes in this variable
        let mut changes = Vec::new();

        // iterate
        for (name, (id, path)) in &new_map {
            match old_map.get(name) {
                // If you cannot find it:
                None => changes.push(Change {
                    change_type: ChangeType::Added,
                    name: name.clone(),
                    path: path.clone(),
                    old_id: None,
                    new_id: Some(id.clone()),
                }),

                // If you can, but the ID has changed:
                Some((old_id, _)) if old_id != id => {
                    //                    let path = path;
                    changes.push(Change {
                        change_type: ChangeType::Modified,
                        name: name.clone(),
                        path: path.clone(),
                        old_id: Some(old_id.clone()),
                        new_id: Some(id.clone()),
                    })
                }

                // Otherwise, there's no change in here
                _ => {}
            }
        }

        // We also need to detect removed entries:
        for (name, (old_id, path)) in old_map {
            if !new_map.contains_key(&name) {
                changes.push(Change {
                    change_type: ChangeType::Removed,
                    name: name.clone(),
                    path,
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
