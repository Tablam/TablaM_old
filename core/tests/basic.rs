use tablam_core::types::*;

mod common;

#[test]
fn test_mem_size() {
    assert_eq!(std::mem::size_of::<Scalar>(), 32);
}
