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

pub fn columns3_1() -> Vec<Col> {
    let c1 = col(&nums_1());
    let c2 = col(&nums_2());
    let c3 = col(&bools_1());

    [c1, c2, c3].to_vec()
}

pub fn columns3_2() -> Vec<Col> {
    let c1 = col(&nums_1());
    let c2 = col(&nums_3());
    let c3 = col(&bools_1());

    [c1, c2, c3].to_vec()
}

pub fn columns3_3() -> Vec<Col> {
    let c1 = reverse(col(&nums_1()));
    let c2 = reverse(col(&nums_2()));
    let c3 = col(&bools_2());

    [c1, c2, c3].to_vec()
}

pub fn schema1() -> Schema {
    Schema::new(
        [
            field("one", DataType::I64),
            field("two", DataType::I64),
            field("three", DataType::Bool),
        ]
        .to_vec(),
    )
}

pub fn schema2() -> Schema {
    Schema::new(
        [
            field("four", DataType::I64),
            field("five", DataType::I64),
            field("six", DataType::Bool),
        ]
        .to_vec(),
    )
}

pub fn table_1() -> Table {
    let schema = schema1();
    let data = columns3_1();
    table_cols(schema, &data)
}

pub fn table_2() -> Table {
    let schema = schema2();
    let data = columns3_2();

    table_cols(schema, &data)
}

pub fn table_3() -> Table {
    let schema = schema1();
    let data: Col = [4i64.into(), 6i64.into(), true.into()].to_vec();
    let col = &vec![data];

    table_cols(schema, &col)
}

pub fn table_4() -> Table {
    let schema = schema2();
    let data: Col = [5i64.into(), 1i64.into(), false.into()].to_vec();
    let col = &vec![data];

    table_cols(schema, &col)
}

pub fn check_schema<T: Relation>(of: &T, cols: usize, rows: usize) {
    assert_eq!(of.shape().size(), (cols, rows));
}

pub fn find_1() -> Query {
    Query::eq(0, int64(1))
}

pub fn find_1000() -> Query {
    Query::eq(0, int64(1000))
}

pub fn empty_I64() -> Vector {
    array_empty(DataType::I64)
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

    println!("Result {}", rel.printer());

    if let Rel::Seq(mut x) = rel {
        assert_eq!(x.materialize(), result);
    } else {
        assert_eq!(rel, result);
    }
}
