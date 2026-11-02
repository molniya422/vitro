use dock_tauri_lib::state::apply_order;

fn current() -> Vec<String> {
    ["a", "b", "c"].iter().map(|id| id.to_string()).collect()
}

#[test]
fn a_permutation_is_accepted() {
    let requested = ["c", "a", "b"].map(String::from).to_vec();
    assert_eq!(
        apply_order(&current(), &requested).unwrap(),
        ["c", "a", "b"]
    );
}

#[test]
fn the_same_order_is_accepted() {
    assert_eq!(apply_order(&current(), &current()).unwrap(), current());
}

#[test]
fn a_missing_id_is_rejected() {
    let requested = ["a", "b"].map(String::from).to_vec();
    assert!(apply_order(&current(), &requested).is_err());
}

#[test]
fn an_unknown_id_is_rejected() {
    let requested = ["a", "b", "d"].map(String::from).to_vec();
    assert!(apply_order(&current(), &requested).is_err());
}

#[test]
fn a_duplicate_id_is_rejected() {
    let requested = ["a", "a", "c"].map(String::from).to_vec();
    assert!(apply_order(&current(), &requested).is_err());
}

#[test]
fn an_empty_dock_is_valid() {
    assert!(apply_order(&[], &[]).unwrap().is_empty());
    assert!(apply_order(&[], &["a".to_string()]).is_err());
}
