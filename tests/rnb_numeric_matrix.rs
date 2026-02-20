#[test]
fn numeric_matrix_roundtrip() {
    let values = vec![
        1.0f32, 2.0, 3.0,
        4.0, 5.0, 6.0,
    ];
    let m = rnb_format::NumericMatrix::new(2, 3, values.clone()).unwrap();

    let mut bytes = Vec::new();
    m.write_to(&mut bytes).unwrap();

    let m2 = rnb_format::NumericMatrix::read_from(&bytes[..]).unwrap();

    assert_eq!(m.rows, m2.rows);
    assert_eq!(m.cols, m2.cols);
    assert_eq!(m.elem_type as u32, m2.elem_type as u32);
    assert_eq!(m2.values, values);
}

