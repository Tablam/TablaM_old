mod core;

use self::core::types::*;

fn main() {
    let nums1 = (1..10).into_iter().collect();

    let col1 = Column::from(nums1);

    println!("Column {:?}", col1);
}