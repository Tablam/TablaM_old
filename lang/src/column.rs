use self::types::*;

/// Convert vectors to columns - erases the concrete type
/// implement `std::convert::From` to get `std::convert::Into` for free
impl From<Vec<i64>> for Column {
    fn from(vec: Vec<i64>) -> Self {
        Column::I64(Rc::from(vec))
    }
}

/// Recovers the elements type through iterators and slices
pub trait ColumnIter: Value {
    fn as_slice<'b>(col: &'b Column) -> &'b [Self];
    fn iter<'b>(col: &'b Column) -> Iter<'b, Self> {
        Self::as_slice(col).iter()
    }
}

impl ColumnIter for i64 {
    fn as_slice<'b>(col: &'b Column) -> &'b [i64] {
        if let Column::I64(ref vec) = *col {
            vec
        } else {
            panic!("Improper cast of {:?} to [i64]", col)
        }
    }
}

impl ColumnIter for BytesMut {
    fn as_slice<'b>(col: &'b Column) -> &'b [BytesMut] {
        if let Column::UTF8(ref vec) = *col {
            vec
        } else {
            panic!("Improper cast of {:?} to [Str]", col)
        }
    }
}

/// `ColumnType` is the type of the elements if the columns.
/// It composes all column traits and is used as a type bound
/// to bring all the dependencies at once
pub trait ColumnType: Value {
    fn to_column(Vec<Self>) -> Column;
    fn iter<'b>(&'b Column) -> Iter<'b, Self>;
    fn as_slice<'b>(&'b Column) -> &'b [Self];
}

/// Implement `ColumnType` for each type that implements
/// `ColumnIter<Self>` and `From<Vec<Self>>` for `Column`
impl<T> ColumnType for T
where
    T: ColumnIter + Value,
    Column: From<Vec<T>>,
{
    fn to_column(vec: Vec<T>) -> Column {
        vec.into()
    }

    fn iter<'b>(col: &'b Column) -> Iter<'b, T> {
        T::iter(col)
    }

    fn as_slice<'b>(col: &'b Column) -> &'b [T] {
        T::as_slice(col)
    }
}

impl Column {
    /// Construct a column from a vector
    pub fn from<T: ColumnType>(vec: Vec<T>) -> Column {
        T::to_column(vec)
    }

    /// column.iter()
    pub fn iter<T: ColumnType>(&self) -> Iter<T> {
        T::iter(self)
    }

    /// column.as_slice()
    pub fn as_slice<T: ColumnType>(&self) -> &[T] {
        T::as_slice(self)
    }
}