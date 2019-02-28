use tablam_core::types::*;
use tablam_core::types::DataType::*;
use tablam_core::dsl::*;

mod common;
use crate::common::*;

#[test]
fn test_mem_size() {
    assert_eq!(std::mem::size_of::<Scalar>(), 32);
}

#[test]
fn test_create() {
    let num1 = nums_1();
    let empty_schema = &Schema::scalar_field(I32);

    let fnull = rel_empty();
    assert_eq!(fnull.schema(), empty_schema);

    check_schema(&fnull, 1, 0);
    let fcol1 = rel_nums1();
    check_schema(&fcol1, 1, 3);

    let frow1 = row_infer(num1.as_slice());
    check_schema(&frow1, 3, 1);

    let table1 = table_1();
    check_schema(&table1, 3, 3);
}
