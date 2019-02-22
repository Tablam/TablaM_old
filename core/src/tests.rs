//use super::types::*;
//use super::types::DataType::*;
//use super::utils::*;
//use super::dsl::*;
//
//fn nums_1() -> Vec<i64> {vec![1, 2, 3]}
//fn nums_2() -> Vec<i64> {vec![4, 5, 6]}
//fn nums_3() -> Vec<i64> {vec![2, 3, 4]}
//fn bools_1() -> Vec<bool> {vec![true, false, true]}
//fn bools_2() -> Vec<bool> {vec![false, true, false]}
//
//fn columns3_1() -> (usize, Vec<Scalar>) {
//    let c1 = col(&nums_1());
//    let c2 = col(&nums_2());
//    let c3 =col(&bools_1());
//
//    concat([c1, c2, c3].to_vec())
//}
//
//fn columns3_2() -> (usize, Vec<Scalar>) {
//    let c1 = col(&nums_1());
//    let c2 = col(&nums_3());
//    let c3 =col(&bools_1());
//
//    concat([c1, c2, c3].to_vec())
//}
//
//fn reverse(of:Col) -> Col {
//    let mut col = of.clone();
//    col.reverse();
//    col
//}
//
//fn columns3_3() -> (usize, Vec<Scalar>) {
//    let c1 = reverse(col(&nums_1()));
//    let c2 = reverse(col(&nums_2()));
//    let c3 = col(&bools_2());
//
//    concat([c1, c2, c3].to_vec())
//}
//
//fn rel_empty() -> Data { array_empty(DataType::I32) }
//fn rel_nums1() -> Data {
//    array(nums_1().as_slice())
//}
//fn rel_nums3() -> Data {
//    array(nums_3().as_slice())
//}
//
//fn make_both(left:Vec<i64>, right:Vec<i64>) -> (Data, Data, usize, usize) {
//    let f1 = array(left.as_slice());
//    let f2 = array(right.as_slice());
//    let (pick1, pick2) = (0,  0);
//
//    (f1, f2, pick1, pick2)
//}
//
//fn schema1() ->  Schema {
//    Schema::new([field("one", I64), field("two", I64), field("three", Bool)].to_vec())
//}
//
//fn schema2() ->  Schema {
//    Schema::new([field("four", I64), field("five", I64), field("six", Bool)].to_vec())
//}
//
//fn table_1() -> Data {
//    let schema = schema1();
//    let (rows, data) = columns3_1();
//    table_cols(schema, &nd_array(&data, rows, 3))
//}
//
//fn btree_1() -> BTree {
//    let schema = schema1().deselect(&[2]);
//    let (rows, data) = columns3_1();
//    table_btree(schema, &nd_array(&data, rows, 3))
//}
//
//fn btree_2() -> BTree {
//    let schema = schema1().deselect(&[2]);
//    let (rows, data) = columns3_3();
//    table_btree(schema, &nd_array(&data, rows, 3))
//}
//
//fn table_2() -> Data {
//    let schema = schema2();
//    let (rows, data) = columns3_2();
//
//    table_cols(schema, &nd_array(&data, rows, 3))
//}
//
//fn table_3() -> Data {
//    let schema = schema1();
//    let data:Col = [4i64.into(), 6i64.into(), true.into()].to_vec();
//    let col = nd_array(&data, 3, 1);
//
//    table_cols(schema, &col)
//}
//
//fn table_4() -> Data {
//    let schema = schema2();
//    let data:Col = [5i64.into(), 1i64.into(), false.into()].to_vec();
//    let col = nd_array(&data, 3, 1);
//
//    table_cols(schema, &col)
//}
//
//fn open_test_file(path:&str) -> String {
//    let root = env!("CARGO_MANIFEST_DIR");
//    let paths = [root,  "test_data", path].to_vec();
//
//    let paths =  path_combine(paths.as_slice());
//    println!("{:?}, {:?}",paths, root);
//
//    let name =paths.to_str().expect("Wrong path?");
//    read_all(name).expect("File not exist")
//}
//
//fn compare_lines(a:String, b:String) {
//    let x:Vec<&str> = a.lines().collect();
//    let y:Vec<&str> = b.lines().collect();
//    let total_x = x.len();
//    let total_y = x.len();
//
//    for (left, right) in x.into_iter().zip(y.into_iter()) {
//        assert_eq!(left, right, "Line not equal");
//    }
//
//    assert_eq!(total_x, total_y, "Lines not equal");
//}
//
//#[test]
//fn test_mem_size() {
//    assert_eq!(std::mem::size_of::<Scalar>(), 32);
//}
//
//#[test]
//fn test_create() {
//    let num1 = nums_1();
//    let empty_schema = &Schema::scalar_field(I32);
//
//    let fnull = rel_empty();
//    assert_eq!(fnull.schema(), empty_schema);
//    println!("Empty {:?}", fnull);
//
//    assert_eq!(fnull.col_count(), 1);
//    assert_eq!(fnull.row_count(), 0);
//
//    let fcol1 = rel_nums1();
//    println!("NDArray {}", fcol1);
//    assert_eq!(fnull.schema(), empty_schema);
//
//    assert_eq!(fcol1.col_count(), 1);
//    assert_eq!(fcol1.row_count(), 3);
//
//    let frow1 = row_infer(num1.as_slice());
//    println!("Rows {}", frow1);
//    assert_eq!(frow1.col_count(), 3);
//    assert_eq!(frow1.row_count(), 1);
//
//    let table1 = table_1();
//    println!("Table Cols {}", table1);
//    assert_eq!(table1.col_count(), 3);
//    assert_eq!(table1.row_count(), 3);
//}
//
//fn _test_select<T:Relation>(table:&T, total_deselect:usize)
//{
//    let (pick1, pick2) = (colp(0),  coln("two"));
//
//    println!("Table {}", table);
//    let query_empty = select(table, &[]);
//    println!("Select 0 {}", query_empty);
//    assert_eq!(query_empty.schema().len(), 0);
//
//    let query1 = select(table, &[pick1]);
//    println!("Select 0 {}", query1);
//    assert_eq!(query1.schema().len(), 1);
//    let query2 = select(table, &[pick2.clone()]);
//    println!("Select 1 {}", query2);
//    assert_eq!(query2.schema().len(), 1);
//
//    let query3 = deselect(table, &[pick2]);
//    println!("DeSelect 1 {}", query3);
//    assert_eq!(query3.schema().len(), total_deselect);
//    //assert_eq!(query1.schema(), query3.schema());
//}

//#[test]
//fn test_select() {
//    let table1 = table_1();
//    let table2= btree_1();
//
//    _test_select(&table1, 2);
//    _test_select(&table2, 1);
//}
//
//fn _test_rename<T:Relation>(table:&T) {
//    let renamed = rename(table, &[(colp(0), "changed")]);
//
//    assert_eq!(table.col_count(), renamed.col_count());
//    assert_eq!(renamed.schema().columns[0].name, "changed".to_string());
//}
//
//#[test]
//fn test_rename() {
//    let table1 = table_1();
//    let table2= btree_1();
//    _test_rename( &table1);
//    _test_rename( &table2);
//}
//
//fn _test_compare<T:Relation>(table:&T, pos:usize) {
//    println!("Table CMP {}", table);
//    let none = &none();
//    let one = &value(1i64);
//
//    let pos1 = table.find_all(0, 0, one, &PartialEq::eq);
//    assert_eq!(pos1, [pos].to_vec());
//
//    let query1 = where_value_late(table, 0, none, &PartialEq::eq);
//    println!("Where1 = None {}", query1);
//    assert_eq!(query1.row_count(), 0);
//
//    let query2 = where_value_late(table,0, one, &PartialEq::eq);
//    println!("Where2 = 1 {}", query2);
//    assert_eq!(query2.row_count(), 1);
//}
//
//#[test]
//fn test_where() {
//    let table2= btree_1();
//    let s1 = scan_op(table2);
//    let cmp = CmOp::eq(0, value(1i64).into());
//    let w1 = where_op(s1, cmp);
//    println!("Where2 = 1 {:?}", 1);
//}
//
//#[test]
//fn test_compare() {
//    let table1 = table_1();
//    let table2= btree_1();
//    _test_compare(&table1, 0);
//    _test_compare(&table2, 0); //Btree order by key
//}
//
//#[test]
//fn test_hash() {
//    let table1 =  &table_1();
//    let hashed = table1.hash_rows();
//    println!("Hash {:?}", hashed);
//    assert_eq!(hashed.len(), table1.row_count());
//    assert_eq!(hashed.contains_key(&hash_column(&table1.get_row(0).unwrap())), true);
//}
//
//#[test]
//fn test_union() {
//    let table1 = &deselect(&table_1(), &[(colp(2))]);
//    let table2 =  &btree_2();
//    println!("Union {} U {}", table1, table2);
//
//    let table3 = union(table1, table2);
//    println!("Union {}", table3);
//
//    assert_eq!(table1.col_count(), table3.col_count());
//    assert_eq!(table1.row_count() * 2, table3.row_count());
//}
//
//#[test]
//fn test_intersection() {
//    let left = &rel_nums1();
//    let right = &rel_nums3();
//
//    let result = _compare_hash(left, right, true);
//    println!("CMP {:?}", result);
//
//    println!("Left {}", left);
//    println!("Right {}", right);
//
//    let result = intersection(left, right);
//    println!("Intersection {}", result);
//    assert_eq!(result,  array(&[2i64, 3i64]));
//}
//
//#[test]
//fn test_cross_join() {
//    let left = &rel_nums1();
//    let right= &rename(&rel_nums3(), &[(colp(0), "changed")]);
//
//    let both = cross(left, right);
//    println!("Cross {}", both);
//    assert_eq!(both.col_count(), 2);
//
//    let col1 = both.col(0);
//    let col2 = both.col(1);
//    let r1:Vec<i64> =vec![1, 1, 1, 2, 2, 2, 3, 3, 3];
//    let r2:Vec<i64> =vec![2, 3, 4, 2, 3, 4, 2, 3, 4];
//
//    assert_eq!(col1, col(&r1));
//    assert_eq!(col2, col(&r2));
//}
//
//#[test]
//fn test_difference() {
//    let left = &rel_nums1();
//    let right = &rel_nums3();
//
//    let result1 = _compare_hash(left, right, false);
//    let result2 = _compare_hash(right, left, false);
//    println!("CMP {:?}, {:?}", result1, result2);
//
//    println!("Left {}", left);
//    println!("Right {}", right);
//
//    let result = difference(left, right);
//    println!("Difference {}", result);
//    assert_eq!(result,  array(&[4i64, 1i64]));
//}
//
//fn _test_join(join:Join, left:Vec<i64>, right:Vec<i64>, mut join_left:Vec<isize>, mut join_right:Vec<isize>)
//{
//    let null_lefts = join.produce_null(true);
//    let null_rights = join.produce_null(false);
//
//    println!("CMP {:?}: {:?}, {:?}", join, left, right);
//
//    let (ds1, ds2, p1, p2) = make_both(left, right);
//
//    let (left, right) = _join_late(&ds1, &ds2, &[p1], null_lefts, &[p2], null_rights, &PartialEq::eq);
//    println!("BIT {:?}, {:?}", left, right);
//
//    let mut l = _bitvector_to_pos(&left);
//    let mut r =_bitvector_to_pos(&right);
//
//    l.sort();
//    r.sort();
//    join_left.sort();
//    join_right.sort();
//
//    assert_eq!(l, join_left, "Left Side");
//    assert_eq!(r, join_right, "Right Side");
//}
//
//#[test]
//fn test_joins_raw() {
//    _test_join(Join::Inner, vec![1], vec![1], vec![0], vec![0]);
//    _test_join(Join::Full, vec![1], vec![2], vec![-1, 0], vec![-1, 0]);
//    _test_join(Join::Full, vec![1], vec![], vec![0], vec![-1]);
//    _test_join(Join::Inner, vec![1, 2, 3], vec![1, 2, 3], vec![0, 1, 2], vec![0, 1, 2]);
//    _test_join(Join::Full, vec![1, 2, 3], vec![2, 3, 4], vec![0, 1, 2, -1], vec![-1, 0, 1, 2]);
//    _test_join(Join::Full, vec![1, 1, 1], vec![2, 3, 4], vec![0, 1, 2, -1, -1, -1], vec![-1, -1, -1, 0, 1, 2]);
//}
//
//fn _test_joins(left:&Data, right:&Data, using:Join, total_cols:usize, test_file:&str)
//{
//    let result = &join(left, right, using.clone(), &[0], &[0], &PartialEq::eq);
//    println!("Result {:?}: {}", using, result);
//    assert_eq!(result.col_count(), total_cols, "Wrong columns");
//
//    let txt = format!("{}", result);
//    let compare = open_test_file(test_file);
//    compare_lines(compare, txt);
//}
//
//#[test]
//fn test_joins() {
//    let table1 =  &table_1();
//    let table2 =  &table_2();
//    let table3 =  &table_3();
//    let table4 =  &table_4();
//
//    let left = &append(table1, table3);
//    let right = &append(table2, table4);
//
//    println!("Table1 {}", left);
//    println!("Table2 {}", right);
//
//    _test_joins(left, right, Join::Full, 6, "full_join.txt");
//
//    _test_joins(left, right, Join::Inner, 6, "inner_join.txt");
//}
