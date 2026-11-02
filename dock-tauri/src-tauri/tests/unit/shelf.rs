use dock_tauri_lib::shelf::{display_name, push_unique, remove, SHELF_CAP};

fn list(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

#[test]
fn adding_a_new_path_appends_it() {
    let mut items = list(&[r"C:\one.txt"]);
    assert!(push_unique(&mut items, r"C:\two.txt"));
    assert_eq!(items, list(&[r"C:\one.txt", r"C:\two.txt"]));
}

#[test]
fn duplicates_are_case_insensitive() {
    let mut items = list(&[r"C:\One.TXT"]);
    assert!(!push_unique(&mut items, r"c:\one.txt"));
    assert_eq!(items.len(), 1);
}

#[test]
fn empty_paths_are_ignored() {
    let mut items = Vec::new();
    assert!(!push_unique(&mut items, "   "));
    assert!(items.is_empty());
}

#[test]
fn the_cap_is_enforced() {
    let mut items: Vec<String> = (0..SHELF_CAP).map(|i| format!(r"C:\{i}.txt")).collect();
    assert!(!push_unique(&mut items, r"C:\overflow.txt"));
    assert_eq!(items.len(), SHELF_CAP);
}

#[test]
fn removing_is_case_insensitive() {
    let mut items = list(&[r"C:\One.txt", r"C:\two.txt"]);
    assert!(remove(&mut items, r"c:\one.TXT"));
    assert_eq!(items, list(&[r"C:\two.txt"]));
    assert!(!remove(&mut items, r"c:\one.txt"));
}

#[test]
fn the_display_name_is_the_last_component_or_the_path() {
    assert_eq!(display_name(r"C:\Users\me\report.pdf"), "report.pdf");
    assert_eq!(display_name(r"C:\Users\me\folder"), "folder");
    assert_eq!(display_name(r"C:\"), r"C:\");
}
