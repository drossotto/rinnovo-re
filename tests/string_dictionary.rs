#[test]
fn string_dictionary_roundtrip() {
    let dict = rnb_format::StringDict::new(vec![
        "alpha".to_string(),
        "beta".to_string(),
        "gamma".to_string(),
        "".to_string(),
    ]);

    let bytes = dict.to_bytes().unwrap();
    let dict2 = rnb_format::StringDict::from_bytes(&bytes[..]).unwrap();

    assert_eq!(dict, dict2);
    assert_eq!(dict2.get(0), Some("alpha"));
    assert_eq!(dict2.get(3), Some(""));
}
