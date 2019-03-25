use tablam_core::types::*;
use tablam_core::types::DataType::*;
use tablam_core::dsl::*;

mod common;
use crate::common::*;

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

#[test]
fn test_project() {
    let table1= table_1();

    table1;
}

#[test]
fn test_where() {
    let table1= table_1();

    let empty = Table::empty(table1.schema.clone());

    let r1 = table_rows(table1.schema.clone(), nd_array(&table1.row(0), 1, 3));

    check_compare(table1.clone(),Query::eq(0, 1i64.into()), r1);
    check_compare(table1,  Query::eq(0, 101i64.into()), empty);

    let col = array(&vec![1, 2, 3]);
    let empty = array_empty(DataType::I32);

    check_compare(col.clone(),Query::eq(0, 101.into()), empty.clone());
    check_compare(col.clone(),Query::not(0, 1.into()), array(&vec![2, 3]));
    check_compare(col.clone(),Query::less(0, 3.into()), array(&vec![1, 2]));
    check_compare(col.clone(),Query::less_eq(0, 3.into()), array(&vec![1, 2, 3]));
    check_compare(col.clone(),Query::greater(0, 1.into()), array(&vec![2, 3]));
    check_compare(col.clone(),Query::greater_eq(0, 1.into()), array(&vec![1, 2, 3]));
}

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
