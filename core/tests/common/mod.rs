use tablam_core::types::*;
use tablam_core::types::DataType::*;
use tablam_core::utils::*;
use tablam_core::dsl::*;

pub fn nums_1() -> Vec<i64> {vec![1, 2, 3]}
pub fn nums_2() -> Vec<i64> {vec![4, 5, 6]}
pub fn nums_3() -> Vec<i64> {vec![2, 3, 4]}
pub fn bools_1() -> Vec<bool> {vec![true, false, true]}
pub fn bools_2() -> Vec<bool> {vec![false, true, false]}

pub fn columns3_1() -> Vec<Col> {
    let c1 = col(&nums_1());
    let c2 = col(&nums_2());
    let c3 =col(&bools_1());

    [c1, c2, c3].to_vec()
}

pub fn columns3_2() -> Vec<Col> {
    let c1 = col(&nums_1());
    let c2 = col(&nums_3());
    let c3 =col(&bools_1());

    [c1, c2, c3].to_vec()
}

pub fn reverse(of:Col) -> Col {
    let mut col = of.clone();
    col.reverse();
    col
}

pub fn columns3_3() -> Vec<Col> {
    let c1 = reverse(col(&nums_1()));
    let c2 = reverse(col(&nums_2()));
    let c3 = col(&bools_2());

    [c1, c2, c3].to_vec()
}

pub fn rel_empty() -> Table { array_empty(DataType::I32) }
pub fn rel_nums1() -> Table {
    array(nums_1().as_slice())
}
pub fn rel_nums3() -> Table {
    array(nums_3().as_slice())
}

pub fn make_both(left:Vec<i64>, right:Vec<i64>) -> (Table, Table, usize, usize) {
    let f1 = array(left.as_slice());
    let f2 = array(right.as_slice());
    let (pick1, pick2) = (0,  0);

    (f1, f2, pick1, pick2)
}

pub fn schema1() ->  Schema {
    Schema::new([field("one", I64), field("two", I64), field("three", Bool)].to_vec())
}

pub fn schema2() ->  Schema {
    Schema::new([field("four", I64), field("five", I64), field("six", Bool)].to_vec())
}

pub fn vector_1() -> Table {
    let schema = schema1();
    let data = col(&nums_1());
    table_rows(schema, vec![data])
}

pub fn table_1() -> Table {
    let schema = schema1();
    let data= columns3_1();
    table_cols(schema, &data)
}

pub fn btree_1() -> BTree {
    let schema = schema1().deselect(&[2]);
    let data= columns3_1();
    table_btree(schema.clone(), &table_rows(schema, data))
}

pub fn btree_2() -> BTree {
    let schema = schema1().deselect(&[2]);
    let data= columns3_3();
    table_btree(schema.clone(),&table_rows(schema, data))
}

pub fn table_2() -> Table {
    let schema = schema2();
    let data= columns3_2();

    table_cols(schema, &data)
}

pub fn table_3() -> Table {
    let schema = schema1();
    let data:Col = [4i64.into(), 6i64.into(), true.into()].to_vec();
    let col = &vec![data];

    table_cols(schema, &col)
}

pub fn table_4() -> Table {
    let schema = schema2();
    let data:Col = [5i64.into(), 1i64.into(), false.into()].to_vec();
    let col = &vec![data];

    table_cols(schema, &col)
}

pub fn open_test_file(path:&str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let paths = [root,  "test_data", path].to_vec();

    let paths =  path_combine(paths.as_slice());
    println!("{:?}, {:?}",paths, root);

    let name =paths.to_str().expect("Wrong path?");
    read_all(name).expect("File not exist")
}

pub fn compare_lines(a:String, b:String) {
    let x:Vec<&str> = a.lines().collect();
    let y:Vec<&str> = b.lines().collect();
    let total_x = x.len();
    let total_y = x.len();

    for (left, right) in x.into_iter().zip(y.into_iter()) {
        assert_eq!(left, right, "Line not equal");
    }

    assert_eq!(total_x, total_y, "Lines not equal");
}

pub fn check_schema<T:Relation>(of:&T, cols:usize, rows:usize) {
    assert_eq!(of.col_count(), cols);
    assert_eq!(of.row_count(), rows);
}

pub fn check_filter<T:Relation>(of:&T, cols:usize, rows:usize) {
    assert_eq!(of.col_count(), cols);
    assert_eq!(of.row_count(), rows);
}

pub fn check_compare<T:Relation>(table:T, query:Query, result:T) {
    dbg!(&query);
    let rel = table.query(&[query]);

    assert_eq!(rel, result);
}
