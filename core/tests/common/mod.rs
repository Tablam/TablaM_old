use tablam_core::dsl::*;
use tablam_core::types::*;

pub fn nums_1() -> Vec<i64> {
    vec![1, 2, 3]
}
pub fn nums_2() -> Vec<i64> {
    vec![4, 5, 6]
}
pub fn nums_3() -> Vec<i64> {
    vec![2, 3, 4]
}
pub fn bools_1() -> Vec<bool> {
    vec![true, false, true]
}
pub fn bools_2() -> Vec<bool> {
    vec![false, true, false]
}

pub fn rel_nums1() -> Vector {
    array(nums_1().as_slice())
}

pub fn rel_nums3() -> Vector {
    array(nums_3().as_slice())
}

pub fn rel_empty() -> Vector {
    array_empty(DataType::I32)
}

pub fn check_schema<T: Relation>(of: &T, cols: usize, rows: usize) {
    assert_eq!(of.shape().size(), (cols, rows));
}

pub fn check_compare<T, R>(table: T, result: R)
where
    Rel: From<T>,
    Rel: From<R>,
{
    let rel: Rel = table.into();
    let result = result.into();

    assert_eq!(rel, result);
}

pub fn check_query<T, R>(table: T, query: Query, result: R)
where
    Rel: From<T>,
    Rel: From<R>,
{
    dbg!(&query);
    let rel: Rel = table.into();
    let result = result.into();

    let rel = rel.query(&[query]);

    if let Rel::Seq(mut x) = rel {
        assert_eq!(x.materialize(), result);
    } else {
        assert_eq!(rel, result);
    }
}
