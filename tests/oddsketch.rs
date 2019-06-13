use mempool_sync::oddsketch::Oddsketch;

#[test]
fn is_empty() {
    let mut oddsketch = Oddsketch::default();
    assert!(oddsketch.is_empty());

    oddsketch.insert(0);

    assert!(!oddsketch.is_empty());
}

#[test]
fn involution() {
    let mut oddsketch = Oddsketch::default();
    oddsketch.insert(0);
    oddsketch.insert(0);
    assert!(oddsketch.is_empty());
}

#[test]
fn size() {
    let mut oddsketch = Oddsketch::default();
    oddsketch.insert(0);
    assert_eq!(oddsketch.size(), 1);
}
