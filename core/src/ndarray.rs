use std;
use std::cmp;
use std::mem;
use std::iter::{ExactSizeIterator, FromIterator};
use std::ptr;
use std::slice;
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

use super::values::{Scalar, Layout};

type Phantom<'a> = PhantomData<&'a Scalar>;
type PhantomMut<'a> = PhantomData<&'a mut Scalar>;

pub fn in_place_vec_bin_op<F>(u: &mut [Scalar], v: &[Scalar], mut f: F)
    where F: FnMut(&mut Scalar, &Scalar)
{
    debug_assert_eq!(u.len(), v.len());
    let len = cmp::min(u.len(), v.len());

    let ys = &v[..len];
    let xs = &mut u[..len];

    for i in 0..len {
        f(&mut xs[i], &ys[i])
    }
}

pub fn vec_bin_op<F>(u: &[Scalar], v: &[Scalar], f: F) -> Vec<Scalar>
    where F: Fn(Scalar, Scalar) -> Scalar,
{
    debug_assert_eq!(u.len(), v.len());
    let len = cmp::min(u.len(), v.len());

    let xs = &u[..len];
    let ys = &v[..len];

    let mut out_vec = Vec::with_capacity(len);
    unsafe {
        out_vec.set_len(len);
    }

    {
        let out_slice = &mut out_vec[..len];

        for i in 0..len {
            out_slice[i] = f(xs[i].clone(), ys[i].clone());
        }
    }

    out_vec
}

/// The `NDArray` struct, for storing relational/table data (2d)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct NDArray {
    rows: usize,
    cols: usize,
    data: Vec<Scalar>,
}

/// A `NDArraySlice`
/// The struct contains the upper left point of the slice
/// and the width and height of the slice.
#[derive(Debug, Clone, Copy)]
pub struct NDArraySlice<'a> {
    ptr: *const Scalar,
    rows: usize,
    cols: usize,
    row_stride: usize,
    marker: Phantom<'a>,
}

/// A mutable `NDArraySliceMut`
/// The struct contains the upper left point of the slice
/// and the width and height of the slice.
#[derive(Debug)]
pub struct NDArraySliceMut<'a> {
    ptr: *mut Scalar,
    rows: usize,
    cols: usize,
    row_stride: usize,
    marker: PhantomMut<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct Row<'a> {
    row: NDArraySlice<'a>,
}

#[derive(Debug)]
pub struct RowMut<'a> {
    row: NDArraySliceMut<'a>,
}

/// Row iterator.
#[derive(Debug)]
pub struct Rows<'a> {
    slice_start: *const Scalar,
    row_pos: usize,
    slice_rows: usize,
    slice_cols: usize,
    row_stride: isize,
    _marker: Phantom<'a>,
}

/// Mutable row iterator.
#[derive(Debug)]
pub struct RowsMut<'a> {
    slice_start: *mut Scalar,
    row_pos: usize,
    slice_rows: usize,
    slice_cols: usize,
    row_stride: isize,
    _marker: PhantomMut<'a>,
}

impl<'a> Row<'a> {
    pub fn raw_slice(&self) -> &'a [Scalar] {
        unsafe { std::slice::from_raw_parts(self.row.as_ptr(), self.row.cols()) }
    }
}

impl<'a> RowMut<'a> {
    pub fn raw_slice(&self) -> &'a [Scalar] {
        unsafe { std::slice::from_raw_parts(self.row.as_ptr(), self.row.cols()) }
    }

    pub fn raw_slice_mut(&mut self) -> &'a mut [Scalar] {
        unsafe { std::slice::from_raw_parts_mut(self.row.as_mut_ptr(), self.row.cols()) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Column<'a> {
    col: NDArraySlice<'a>,
}

#[derive(Debug)]
pub struct ColumnMut<'a> {
    col: NDArraySliceMut<'a>,
}

/// Column iterator.
#[derive(Debug)]
pub struct Cols<'a> {
    _marker: Phantom<'a>,
    col_pos: usize,
    row_stride: isize,
    slice_cols: usize,
    slice_rows: usize,
    slice_start: *const Scalar,
}

/// Mutable column iterator.
#[derive(Debug)]
pub struct ColsMut<'a> {
    _marker: PhantomMut<'a>,
    col_pos: usize,
    row_stride: isize,
    slice_cols: usize,
    slice_rows: usize,
    slice_start: *mut Scalar,
}

/// Iterator for NDArray.
///
/// Iterates over the underlying slice data
/// in row-major order.
#[derive(Debug)]
pub struct SliceIter<'a> {
    slice_start: *const Scalar,
    row_pos: usize,
    col_pos: usize,
    slice_rows: usize,
    slice_cols: usize,
    row_stride: usize,
    _marker: Phantom<'a>,
}

/// Iterator for mutable NDArray.
///
/// Iterates over the underlying slice data
/// in row-major order.
#[derive(Debug)]
pub struct SliceIterMut<'a> {
    slice_start: *mut Scalar,
    row_pos: usize,
    col_pos: usize,
    slice_rows: usize,
    slice_cols: usize,
    row_stride: usize,
    _marker: PhantomMut<'a>,
}

impl<'a> NDArraySlice<'a> {
    pub fn from_data(mat: &'a NDArray,
                     start: [usize; 2],
                     rows: usize,
                     cols: usize)
                     -> NDArraySlice {
        assert!(start[0] + rows <= mat.rows,
                "View dimensions exceed NDArray dimensions.");
        assert!(start[1] + cols <= mat.cols,
                "View dimensions exceed NDArray dimensions.");
        unsafe {
            NDArraySlice {
                ptr: mat.data().get_unchecked(start[0] * mat.cols + start[1]) as *const Scalar,
                rows,
                cols,
                row_stride: mat.cols,
                marker: PhantomData::<&'a Scalar>,
            }
        }
    }

    /// Creates a `NDArraySlice` from raw parts.
    ///
    /// # Safety
    ///
    /// The pointer must be followed by a contiguous slice of data larger than `row_stride * rows`.
    /// If not then other operations will produce undefined behaviour.
    ///
    /// Additionally `cols` should be less than the `row_stride`. It is possible to use this
    /// function safely whilst violating this condition. So long as
    /// `max(cols, row_stride) * rows` is less than the data size.
    pub unsafe fn from_raw_parts(ptr: *const Scalar,
                                 rows: usize,
                                 cols: usize,
                                 row_stride: usize)
                                 -> NDArraySlice<'a> {
        NDArraySlice {
            ptr,
            rows,
            cols,
            row_stride,
            marker: PhantomData::<&'a Scalar>,
        }
    }
}

impl<'a> NDArraySliceMut<'a> {
    pub fn from_data(mat: &'a mut NDArray,
                     start: [usize; 2],
                     rows: usize,
                     cols: usize)
                     -> NDArraySliceMut {
        assert!(start[0] + rows <= mat.rows,
                "View dimensions exceed NDArray dimensions.");
        assert!(start[1] + cols <= mat.cols,
                "View dimensions exceed NDArray dimensions.");

        let row_stride = mat.cols;

        unsafe {
            NDArraySliceMut {
                ptr: mat.mut_data().get_unchecked_mut(start[0] * row_stride + start[1]) as *mut Scalar,
                rows,
                cols,
                row_stride,
                marker: PhantomData::<&'a mut Scalar>,
            }
        }
    }

    /// Creates a `NDArraySliceMut` from raw parts.
    ///
    /// # Safety
    ///
    /// The pointer must be followed by a contiguous slice of data larger than `row_stride * rows`.
    /// If not then other operations will produce undefined behaviour.
    ///
    /// Additionally `cols` should be less than the `row_stride`. It is possible to use this
    /// function safely whilst violating this condition. So long as
    /// `max(cols, row_stride) * rows` is less than the data size.
    pub unsafe fn from_raw_parts(ptr: *mut Scalar,
                                 rows: usize,
                                 cols: usize,
                                 row_stride: usize)
                                 -> NDArraySliceMut<'a> {
        NDArraySliceMut {
            ptr,
            rows,
            cols,
            row_stride,
            marker: PhantomData::<&'a mut Scalar>,
        }
    }
}

macro_rules! impl_slice_iter (
    ($slice_iter:ident, $data_type:ty) => (
/// Iterates over the NDArray slice data in row-major order.
impl<'a> Iterator for $slice_iter<'a> {
    type Item = $data_type;

    fn next(&mut self) -> Option<$data_type> {
        let offset = self.row_pos * self.row_stride + self.col_pos;
        let end = self.slice_rows * self.row_stride;
        // Set the position of the next element
        if offset < end {
            unsafe {
                let iter_ptr = self.slice_start.offset(offset as isize);

                // If end of row, set to start of next row
                if self.col_pos + 1 == self.slice_cols {
                    self.row_pos += 1usize;
                    self.col_pos = 0usize;
                } else {
                    self.col_pos += 1usize;
                }

                Some(mem::transmute(iter_ptr))
            }
        } else {
            None
        }
    }
}
    );
);

impl_slice_iter!(SliceIter, &'a Scalar);
impl_slice_iter!(SliceIterMut, &'a mut Scalar);

macro_rules! impl_col_iter (
    ($cols:ident, $col_type:ty, $col_base:ident, $slice_base:ident) => (

/// Iterates over the columns in the NDArray.
impl<'a> Iterator for $cols<'a> {
    type Item = $col_type;

    fn next(&mut self) -> Option<Self::Item> {
        if self.col_pos >= self.slice_cols {
            return None;
        }

        let column: $col_type;
        unsafe {
            let ptr = self.slice_start.offset(self.col_pos as isize);
            column  = $col_base {
                col: $slice_base::from_raw_parts(ptr, self.slice_rows, 1, self.row_stride as usize)
            };
        }
        self.col_pos += 1;
        Some(column)
    }

    fn last(self) -> Option<Self::Item> {
        if self.col_pos >= self.slice_cols {
            return None;
        }

        unsafe {
            let ptr = self.slice_start.offset((self.slice_cols - 1) as isize);
            Some($col_base {
                col: $slice_base::from_raw_parts(ptr, self.slice_rows, 1, self.row_stride as usize)
            })
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.col_pos + n >= self.slice_cols {
            return None;
        }

        let column: $col_type;
        unsafe {
            let ptr = self.slice_start.offset((self.col_pos + n) as isize);
            column = $col_base {
                col: $slice_base::from_raw_parts(ptr, self.slice_rows, 1, self.row_stride as usize)
            }
        }
        self.col_pos += n + 1;
        Some(column)
    }

    fn count(self) -> usize {
        self.slice_cols - self.col_pos
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.slice_cols - self.col_pos, Some(self.slice_cols - self.col_pos))
    }
}
    );
);

impl_col_iter!(Cols, Column<'a>, Column, NDArraySlice);
impl_col_iter!(ColsMut, ColumnMut<'a>, ColumnMut, NDArraySliceMut);

impl<'a> ExactSizeIterator for Cols<'a> {}
impl<'a> ExactSizeIterator for ColsMut<'a> {}

macro_rules! impl_row_iter (
    ($rows:ident, $row_type:ty, $row_base:ident, $slice_base:ident) => (

/// Iterates over the rows in the NDArray.
impl<'a> Iterator for $rows<'a> {
    type Item = $row_type;

    fn next(&mut self) -> Option<Self::Item> {
// Check if we have reached the end
        if self.row_pos < self.slice_rows {
            let row: $row_type;
            unsafe {
// Get pointer and create a slice from raw parts
                let ptr = self.slice_start.offset(self.row_pos as isize * self.row_stride);
                row = $row_base {
                    row: $slice_base::from_raw_parts(ptr, 1, self.slice_cols, self.row_stride as usize)
                };
            }

            self.row_pos += 1;
            Some(row)
        } else {
            None
        }
    }

    fn last(self) -> Option<Self::Item> {
// Check if already at the end
        if self.row_pos < self.slice_rows {
            unsafe {
// Get pointer to last row and create a slice from raw parts
                let ptr = self.slice_start.offset((self.slice_rows - 1) as isize * self.row_stride);
                Some($row_base {
                    row: $slice_base::from_raw_parts(ptr, 1, self.slice_cols, self.row_stride as usize)
                })
            }
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.row_pos + n < self.slice_rows {
            let row: $row_type;
            unsafe {
                let ptr = self.slice_start.offset((self.row_pos + n) as isize * self.row_stride);
                row = $row_base {
                    row: $slice_base::from_raw_parts(ptr, 1, self.slice_cols, self.row_stride as usize)
                }
            }

            self.row_pos += n + 1;
            Some(row)
        } else {
            None
        }
    }

    fn count(self) -> usize {
        self.slice_rows - self.row_pos
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.slice_rows - self.row_pos, Some(self.slice_rows - self.row_pos))
    }
}
    );
);

impl_row_iter!(Rows, Row<'a>, Row, NDArraySlice);
impl_row_iter!(RowsMut, RowMut<'a>, RowMut, NDArraySliceMut);

impl<'a> ExactSizeIterator for Rows<'a> {}
impl<'a> ExactSizeIterator for RowsMut<'a> {}

/// Creates a `NDArray` from an iterator over slices.
///
/// Each of the slices produced by the iterator will become a row in the NDArray.
///
/// # Panics
///
/// Will panic if the iterators items do not have constant length.
///
impl<'a> FromIterator<&'a [Scalar]> for NDArray {
    fn from_iter<I: IntoIterator<Item = &'a [Scalar]>>(iterable: I) -> Self {
        let mut data: Vec<Scalar>;
        let cols: usize;
        let mut rows = 0;

        let mut iterator = iterable.into_iter();

        match iterator.next() {
            None => {
                return NDArray {
                    data: Vec::new(),
                    rows: 0,
                    cols: 0,
                }
            }
            Some(row) => {
                rows += 1;
                // Here we set the capacity - get iterator size and the cols
                let (lower_rows, _) = iterator.size_hint();
                cols = row.len();

                data = Vec::with_capacity(lower_rows.saturating_add(1).saturating_mul(cols));
                data.extend_from_slice(row);
            }
        }

        for row in iterator {
            assert!(row.len() == cols, "Iterator slice length must be constant.");
            data.extend_from_slice(row);
            rows += 1;
        }

        data.shrink_to_fit();

        NDArray {
            data,
            rows,
            cols,
        }
    }
}

macro_rules! impl_from_iter_row(
    ($row_type:ty) => (
impl<'a> FromIterator<$row_type> for NDArray {
    fn from_iter<I: IntoIterator<Item = $row_type>>(iterable: I) -> Self {
        let mut mat_data: Vec<Scalar>;
        let cols: usize;
        let mut rows = 0;

        let mut iterator = iterable.into_iter();

        match iterator.next() {
            None => {
                return NDArray {
                    data: Vec::new(),
                    rows: 0,
                    cols: 0,
                }
            }
            Some(row) => {
                rows += 1;
                // Here we set the capacity - get iterator size and the cols
                let (lower_rows, _) = iterator.size_hint();
                cols = row.row.cols();

                mat_data = Vec::with_capacity(lower_rows.saturating_add(1).saturating_mul(cols));
                mat_data.extend_from_slice(row.raw_slice());
            }
        }

        for row in iterator {
            assert!(row.row.cols() == cols, "Iterator row size must be constant.");
            mat_data.extend_from_slice(row.raw_slice());
            rows += 1;
        }

        mat_data.shrink_to_fit();

        NDArray {
            data: mat_data,
            rows,
            cols,
        }
    }
}
    );
);

impl_from_iter_row!(Row<'a>);
impl_from_iter_row!(RowMut<'a>);

impl<'a> IntoIterator for NDArraySlice<'a> {
    type Item = &'a Scalar;
    type IntoIter = SliceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a NDArraySlice<'a> {
    type Item = &'a Scalar;
    type IntoIter = SliceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut NDArraySlice<'a> {
    type Item = &'a Scalar;
    type IntoIter = SliceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for NDArraySliceMut<'a> {
    type Item = &'a mut Scalar;
    type IntoIter = SliceIterMut<'a>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a> IntoIterator for &'a NDArraySliceMut<'a> {
    type Item = &'a Scalar;
    type IntoIter = SliceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut NDArraySliceMut<'a> {
    type Item = &'a mut Scalar;
    type IntoIter = SliceIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a> ColumnMut<'a>  {
    /// Clones the elements of the given slice of compatible size
    /// into this column.
    ///
    /// # Panics
    /// - The slice does not have the same length as
    ///   the number of rows in the column.
    pub fn clone_from_slice(&mut self, slice: &[Scalar]) {
        assert_eq!(slice.len() , self.rows());
        let slice_iter = slice.iter().cloned();
        for (c, s) in self.iter_mut().zip(slice_iter) {
            *c = s;
        }
    }

    /// Clones the elements of this column into a
    /// slice of compatible size.
    ///
    /// # Panics
    /// - The slice does not have the same length as
    ///   the number of rows in the column.
    pub fn clone_into_slice(&self, slice: &mut [Scalar]) {
        assert_eq!(slice.len(), self.rows());
        let col_iter = self.iter().cloned();
        for (s, c) in slice.iter_mut().zip(col_iter) {
            *s = c;
        }
    }
}

impl<'a> Column<'a>  {
    /// Clones the elements of this column into a
    /// slice of compatible size.
    ///
    /// # Panics
    /// - The slice does not have the same length as
    ///   the number of rows in the column.
    pub fn clone_into_slice(&self, slice: &mut [Scalar]) {
        assert_eq!(slice.len(), self.rows());
        let col_iter = self.iter().cloned();
        for (s, c) in slice.iter_mut().zip(col_iter) {
            *s = c;
        }
    }
}

impl BaseNDArray for NDArray {
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }
    fn row_stride(&self) -> usize {
        self.cols
    }
    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    fn as_ptr(&self) -> *const Scalar {
        self.data.as_ptr()
    }

    fn into_array(self) -> NDArray
    {
        self // for NDArray, this is a no-op
    }

    fn vcat<S>(&self, m: &S) -> NDArray
        where S: BaseNDArray
    {
        assert_eq!(self.cols(), m.cols(),
                   "NDArray column counts are not equal.");

        let mut new_data = self.data.clone();
        new_data.reserve(m.rows() * m.cols());

        for row in m.row_iter() {
            new_data.extend_from_slice(row.raw_slice());
        }

        NDArray {
            cols: self.cols(),
            rows: (self.rows() + m.rows()),
            data: new_data,
        }
    }
}

impl<'a> BaseNDArray for NDArraySlice<'a> {
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }
    fn row_stride(&self) -> usize {
        self.row_stride
    }
    fn as_ptr(&self) -> *const Scalar {
        self.ptr
    }
}

impl<'a> BaseNDArray for NDArraySliceMut<'a> {
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }
    fn row_stride(&self) -> usize {
        self.row_stride
    }
    fn as_ptr(&self) -> *const Scalar {
        self.ptr as *const Scalar
    }
}

impl BaseNDArrayMut for NDArray {
    /// Top left index of the slice.
    fn as_mut_ptr(&mut self) -> *mut Scalar {
        self.data.as_mut_ptr()
    }
}

impl<'a> BaseNDArrayMut for NDArraySliceMut<'a> {
    /// Top left index of the slice.
    fn as_mut_ptr(&mut self) -> *mut Scalar {
        self.ptr
    }
}

impl<'a> BaseNDArray for Row<'a> {
    fn rows(&self) -> usize {
        1
    }
    fn cols(&self) -> usize {
        self.row.cols()
    }
    fn row_stride(&self) -> usize {
        self.row.row_stride()
    }

    fn as_ptr(&self) -> *const Scalar {
        self.row.as_ptr()
    }
}

impl<'a> BaseNDArray for RowMut<'a> {
    fn rows(&self) -> usize {
        1
    }
    fn cols(&self) -> usize {
        self.row.cols()
    }
    fn row_stride(&self) -> usize {
        self.row.row_stride()
    }

    fn as_ptr(&self) -> *const Scalar {
        self.row.as_ptr()
    }
}

impl<'a> BaseNDArrayMut for RowMut<'a> {
    /// Top left index of the slice.
    fn as_mut_ptr(&mut self) -> *mut Scalar {
        self.row.as_mut_ptr()
    }
}

impl<'a> BaseNDArray for Column<'a> {
    fn rows(&self) -> usize {
        self.col.rows()
    }
    fn cols(&self) -> usize {
        1
    }
    fn row_stride(&self) -> usize {
        self.col.row_stride()
    }

    fn as_ptr(&self) -> *const Scalar {
        self.col.as_ptr()
    }
}

impl<'a> BaseNDArray for ColumnMut<'a> {
    fn rows(&self) -> usize {
        self.col.rows()
    }
    fn cols(&self) -> usize {
        1
    }
    fn row_stride(&self) -> usize {
        self.col.row_stride()
    }

    fn as_ptr(&self) -> *const Scalar {
        self.col.as_ptr()
    }
}

impl<'a> BaseNDArrayMut for ColumnMut<'a> {
    /// Top left index of the slice.
    fn as_mut_ptr(&mut self) -> *mut Scalar {
        self.col.as_mut_ptr()
    }
}

impl NDArray {
    /// Constructor for NDArray struct.
    ///
    /// Requires both the row and column dimensions.
    ///
    /// # Panics
    ///
    /// - The input data does not match the given dimensions.
    pub fn new<U: Into<Vec<Scalar>>>(rows: usize, cols: usize, data: U) -> NDArray {
        let data = data.into();

        assert_eq!(cols * rows, data.len(),
                   "NDArray does not match given dimensions.");
        NDArray {
            cols,
            rows,
            data,
        }
    }

    /// Constructor for NDArray struct that takes a function `f`
    /// and constructs a new NDArray such that `A_ij = f(j, i)`,
    /// where `i` is the row index and `j` the column index.
    ///
    /// Requires both the row and column dimensions
    /// as well as a generating function.
    pub fn from_fn<F>(rows: usize, cols: usize, mut f: F) -> NDArray
        where F: FnMut(usize, usize) -> Scalar
    {
        let mut data = Vec::with_capacity(rows * cols);
        for row in 0..rows {
            for col in 0..cols {
                data.push(f(col, row));
            }
        }
        NDArray::new(rows, cols, data)
    }

    /// Returns a non-mutable reference to the underlying data.
    pub fn data(&self) -> &Vec<Scalar> {
        &self.data
    }

    /// Returns a mutable slice of the underlying data.
    pub fn mut_data(&mut self) -> &mut [Scalar] {
        &mut self.data
    }

    /// Consumes the NDArray and returns the Vec of data.
    pub fn into_vec(self) -> Vec<Scalar> {
        self.data
    }
}

/// Trait for immutable NDArray structs.
pub trait BaseNDArray: Sized {
    /// Rows in the NDArray.
    fn rows(&self) -> usize;

    /// Columns in the NDArray.
    fn cols(&self) -> usize;

    /// Row stride in the NDArray.
    fn row_stride(&self) -> usize;

    /// Returns true if the NDArray contais no elements
    fn is_empty(&self) -> bool {
        self.rows() == 0 || self.cols() == 0
    }

    /// Top left index of the NDArray.
    fn as_ptr(&self) -> *const Scalar;

    fn as_slice(&self) -> NDArraySlice {
        unsafe {
            NDArraySlice::from_raw_parts(self.as_ptr(), self.rows(), self.cols(), self.row_stride())
        }
    }

    /// Get a reference to an element in the NDArray without bounds checking.
    unsafe fn get_unchecked(&self, index: [usize; 2]) -> &Scalar {
        &*(self.as_ptr().offset((index[0] * self.row_stride() + index[1]) as isize))
    }

    /// Get a reference to an element in the NDArray
    fn get(&self, index: [usize; 2]) -> Option<&Scalar> {
        let row_ind = index[0];
        let col_ind = index[1];

        if row_ind >= self.rows() || col_ind >= self.cols() {
            None
        } else {
            unsafe { Some(self.get_unchecked(index)) }
        }
    }

    fn col(&self, index: usize) -> Column {
        if index < self.cols() {
            unsafe { self.col_unchecked(index) }
        } else {
            panic!("Column index out of bounds.")
        }
    }

    /// Returns the column of a NDArray at the given
    /// index without doing a bounds check.
    unsafe fn col_unchecked(&self, index: usize) -> Column {
        let ptr = self.as_ptr().offset(index as isize);
        Column { col: NDArraySlice::from_raw_parts(ptr, self.rows(), 1, self.row_stride()) }
    }

    /// Returns the row of a NDArray at the given index.
    ///
    /// # Panics
    ///
    /// Will panic if the row index is out of bounds.
    fn row(&self, index: usize) -> Row {
        if index < self.rows() {
            unsafe { self.row_unchecked(index) }
        } else {
            panic!("Row index out of bounds.")
        }
    }

    /// Returns the row of a NDArray at the given index without doing bounds checking
    unsafe fn row_unchecked(&self, index: usize) -> Row {
        let ptr = self.as_ptr().offset((self.row_stride() * index) as isize);
        Row { row: NDArraySlice::from_raw_parts(ptr, 1, self.cols(), self.row_stride()) }
    }

    fn iter<'a>(&self) -> SliceIter<'a>
    {
        SliceIter {
            slice_start: self.as_ptr(),
            row_pos: 0,
            col_pos: 0,
            slice_rows: self.rows(),
            slice_cols: self.cols(),
            row_stride: self.row_stride(),
            _marker: PhantomData::<&Scalar>,
        }
    }

    fn col_iter(&self) -> Cols {
        Cols {
            _marker: PhantomData::<&Scalar>,
            col_pos: 0,
            row_stride: self.row_stride() as isize,
            slice_cols: self.cols(),
            slice_rows: self.rows(),
            slice_start: self.as_ptr(),
        }
    }

    fn row_iter(&self) -> Rows {
        Rows {
            slice_start: self.as_ptr(),
            row_pos: 0,
            slice_rows: self.rows(),
            slice_cols: self.cols(),
            row_stride: self.row_stride() as isize,
            _marker: PhantomData::<&Scalar>,
        }
    }

    /// Convert the NDArray struct into a owned NDArray.
    fn into_array(self) -> NDArray
    {
        self.row_iter().collect()
    }

    /// Select rows from NDArray
    ///
    /// # Panics
    ///
    /// - Panics if row indices exceed the NDArray dimensions.
    fn select_rows<'a, I>(&self, rows: I) -> NDArray
        where
            I: IntoIterator<Item = &'a usize>,
            I::IntoIter: ExactSizeIterator + Clone
    {
        let row_iter = rows.into_iter();
        let mut mat_vec = Vec::with_capacity(row_iter.len() * self.cols());

        for row in row_iter.clone() {
            assert!(*row < self.rows(),
                    "Row index is greater than number of rows.");
        }

        for row_idx in row_iter.clone() {
            unsafe {
                let row = self.row_unchecked(*row_idx);
                mat_vec.extend_from_slice(row.raw_slice());
            }
        }

        NDArray {
            cols: self.cols(),
            rows: row_iter.len(),
            data: mat_vec,
        }
    }

    /// Select columns from NDArray
    ///
    /// # Panics
    ///
    /// - Panics if column indices exceed the NDArray dimensions.
    fn select_cols<'a, I>(&self, cols: I) -> NDArray
        where
            I: IntoIterator<Item = &'a usize>,
            I::IntoIter: ExactSizeIterator + Clone
    {
        let col_iter = cols.into_iter();
        let mut mat_vec = Vec::with_capacity(col_iter.len() * self.rows());

        for col in col_iter.clone() {
            assert!(*col < self.cols(),
                    "Column index is greater than number of columns.");
        }

        unsafe {
            for i in 0..self.rows() {
                for col in col_iter.clone() {
                    let data = self.get_unchecked( [i, *col]).clone();
                    mat_vec.push(data);
                }
            }
        }

        NDArray {
            cols: col_iter.len(),
            rows: self.rows(),
            data: mat_vec,
        }
    }

    /// Select block NDArray from NDArray
    ///
    /// # Panics
    ///
    /// - Panics if row or column indices exceed the NDArray dimensions.
    fn select(&self, rows: &[usize], cols: &[usize]) -> NDArray
    {

        let mut mat_vec = Vec::with_capacity(cols.len() * rows.len());

        for col in cols {
            assert!(*col < self.cols(),
                    "Column index is greater than number of columns.");
        }

        for row in rows {
            assert!(*row < self.rows(),
                    "Row index is greater than number of columns.");
        }

        unsafe {
            for row in rows {
                for col in cols {
                    let data = self.get_unchecked([*row, *col]).clone();
                    mat_vec.push(data);
                }
            }
        }

        NDArray {
            cols: cols.len(),
            rows: rows.len(),
            data: mat_vec,
        }
    }

    /// Horizontally concatenates two matrices. With self on the left.
    ///
    /// # Panics
    ///
    /// - Self and m have different row counts.
    fn hcat<S>(&self, m: &S) -> NDArray
        where S: BaseNDArray
    {
        assert_eq!(self.rows(), m.rows(), "NDArray row counts are not equal.");

        let mut new_data = Vec::with_capacity((self.cols() + m.cols()) * self.rows());

        for (self_row, m_row) in self.row_iter().zip(m.row_iter()) {
            new_data.extend_from_slice(self_row.raw_slice());
            new_data.extend_from_slice(m_row.raw_slice());
        }

        NDArray {
            cols: (self.cols() + m.cols()),
            rows: self.rows(),
            data: new_data,
        }
    }

    /// Vertically concatenates two matrices. With self on top.
    ///
    /// # Panics
    ///
    /// - Self and m have different column counts.
    fn vcat<S>(&self, m: &S) -> NDArray
        where S: BaseNDArray
    {
        assert_eq!(self.cols(), m.cols(),
                   "NDArray column counts are not equal.");

        let mut new_data = Vec::with_capacity((self.rows() + m.rows()) * self.cols());

        for row in self.row_iter().chain(m.row_iter()) {
            new_data.extend_from_slice(row.raw_slice());
        }

        NDArray {
            cols: self.cols(),
            rows: (self.rows() + m.rows()),
            data: new_data,
        }
    }

    fn pivot(&self) -> NDArray
    {
        let mut new_data = Vec::with_capacity(self.rows() * self.cols());
        //let mut new_data = vec![Scalar::None; self.rows() * self.cols()];
        unsafe {
            new_data.set_len(self.rows() * self.cols());
            for i in 0..self.cols() {
                for j in 0..self.rows() {
                    let data =  self.get_unchecked([j, i]).clone();
                    std::ptr::write(new_data.get_unchecked_mut(i * self.rows() + j) , data);
                }
            }
        }

        NDArray {
            cols: self.rows(),
            rows: self.cols(),
            data: new_data,
        }
    }

    /// Split the NDArray at the specified layout returning two `NDArraySlice`s.
    fn split_at(&self, mid: usize, layout: Layout) -> (NDArraySlice, NDArraySlice) {
        let slice_1: NDArraySlice;
        let slice_2: NDArraySlice;

        match layout {
            Layout::Row => {
                assert!(mid < self.rows());
                unsafe {
                    slice_1 = NDArraySlice::from_raw_parts(self.as_ptr(),
                                                         mid,
                                                         self.cols(),
                                                         self.row_stride());
                    slice_2 = NDArraySlice::from_raw_parts(self.as_ptr()
                                                             .offset((mid * self.row_stride()) as
                                                                 isize),
                                                         self.rows() - mid,
                                                         self.cols(),
                                                         self.row_stride());
                }
            }
            Layout::Col => {
                assert!(mid < self.cols());
                unsafe {
                    slice_1 = NDArraySlice::from_raw_parts(self.as_ptr(),
                                                         self.rows(),
                                                         mid,
                                                         self.row_stride());
                    slice_2 = NDArraySlice::from_raw_parts(self.as_ptr().offset(mid as isize),
                                                         self.rows(),
                                                         self.cols() - mid,
                                                         self.row_stride());
                }
            }
        }

        (slice_1, slice_2)
    }

    /// Produce a `NDArraySlice` from an existing NDArray.
    fn sub_slice<'a>(&self, start: [usize; 2], rows: usize, cols: usize) -> NDArraySlice<'a>
    {
        assert!(start[0] + rows <= self.rows(),
                "View dimensions exceed NDArray dimensions.");
        assert!(start[1] + cols <= self.cols(),
                "View dimensions exceed NDArray dimensions.");

        unsafe {
            NDArraySlice::from_raw_parts(self.as_ptr()
                                           .offset((start[0] * self.row_stride() + start[1]) as
                                               isize),
                                       rows,
                                       cols,
                                       self.row_stride())
        }
    }
}

/// Trait for mutable matrices.
pub trait BaseNDArrayMut: BaseNDArray {
    /// Top left index of the slice.
    fn as_mut_ptr(&mut self) -> *mut Scalar;

    fn as_mut_slice(&mut self) -> NDArraySliceMut {
        unsafe {
            NDArraySliceMut::from_raw_parts(self.as_mut_ptr(),
                                          self.rows(),
                                          self.cols(),
                                          self.row_stride())
        }
    }

    /// Get a mutable reference to an element in the NDArray without bounds checks.
    unsafe fn get_unchecked_mut(&mut self, index: [usize; 2]) -> &mut Scalar {
        &mut *(self.as_mut_ptr().offset((index[0] * self.row_stride() + index[1]) as isize))
    }

    fn get_mut(&mut self, index: [usize; 2]) -> Option<&mut Scalar> {
        let row_ind = index[0];
        let col_ind = index[1];

        if row_ind >= self.rows() || col_ind >= self.cols() {
            None
        } else {
            unsafe { Some(self.get_unchecked_mut(index)) }
        }
    }

    fn iter_mut<'a>(&mut self) -> SliceIterMut<'a>
    {
        SliceIterMut {
            slice_start: self.as_mut_ptr(),
            row_pos: 0,
            col_pos: 0,
            slice_rows: self.rows(),
            slice_cols: self.cols(),
            row_stride: self.row_stride(),
            _marker: PhantomData::<&mut Scalar>,
        }
    }

    fn col_mut(&mut self, index: usize) -> ColumnMut {
        if index < self.cols() {
            unsafe { self.col_unchecked_mut(index) }
        } else {
            panic!("Column index out of bounds.")
        }
    }

    unsafe fn col_unchecked_mut(&mut self, index: usize) -> ColumnMut {
        let ptr = self.as_mut_ptr().offset(index as isize);
        ColumnMut { col: NDArraySliceMut::from_raw_parts(ptr, self.rows(), 1, self.row_stride()) }
    }

    fn row_mut(&mut self, index: usize) -> RowMut {
        if index < self.rows() {
            unsafe { self.row_unchecked_mut(index) }
        } else {
            panic!("Row index out of bounds.")
        }
    }
    unsafe fn row_unchecked_mut(&mut self, index: usize) -> RowMut {
        let ptr = self.as_mut_ptr().offset((self.row_stride() * index) as isize);
        RowMut { row: NDArraySliceMut::from_raw_parts(ptr, 1, self.cols(), self.row_stride()) }
    }

    fn swap_rows(&mut self, a: usize, b: usize) {
        assert!(a < self.rows(),
                format!("Row index {0} larger than row count {1}", a, self.rows()));
        assert!(b < self.rows(),
                format!("Row index {0} larger than row count {1}", b, self.rows()));

        if a != b {
            unsafe {
                let row_a = slice::from_raw_parts_mut(self.as_mut_ptr()
                                                          .offset((self.row_stride() * a) as
                                                              isize),
                                                      self.cols());
                let row_b = slice::from_raw_parts_mut(self.as_mut_ptr()
                                                          .offset((self.row_stride() * b) as
                                                              isize),
                                                      self.cols());

                for (x, y) in row_a.into_iter().zip(row_b.into_iter()) {
                    mem::swap(x, y);
                }
            }
        }

    }

    fn swap_cols(&mut self, a: usize, b: usize) {
        assert!(a < self.cols(),
                format!("Row index {0} larger than row count {1}", a, self.rows()));
        assert!(b < self.cols(),
                format!("Row index {0} larger than row count {1}", b, self.rows()));

        if a != b {
            unsafe {
                for i in 0..self.rows() {
                    let a_ptr: *mut Scalar = self.get_unchecked_mut([i, a]);
                    let b_ptr: *mut Scalar = self.get_unchecked_mut([i, b]);
                    ptr::swap(a_ptr, b_ptr);
                }
            }
        }

    }

    fn col_iter_mut(&mut self) -> ColsMut {
        ColsMut {
            _marker: PhantomData::<&mut Scalar>,
            col_pos: 0,
            row_stride: self.row_stride() as isize,
            slice_cols: self.cols(),
            slice_rows: self.rows(),
            slice_start: self.as_mut_ptr(),
        }
    }

    fn row_iter_mut(&mut self) -> RowsMut {
        RowsMut {
            slice_start: self.as_mut_ptr(),
            row_pos: 0,
            slice_rows: self.rows(),
            slice_cols: self.cols(),
            row_stride: self.row_stride() as isize,
            _marker: PhantomData::<&mut Scalar>,
        }
    }

    fn set_to<M: BaseNDArray>(mut self, target: M)
    {
        assert_eq!(self.rows(), target.rows(),
                   "Target has different row count to self.");
        assert_eq!(self.cols(), target.cols(),
                   "Target has different column count to self.");
        for (mut s, t) in self.row_iter_mut().zip(target.row_iter()) {
            // Vectorized assignment per row.
            in_place_vec_bin_op(s.raw_slice_mut(), t.raw_slice(), |x, y| *x = y.clone());
        }
    }

    /// Applies a function to each element in the NDArray.
    fn apply(mut self, f: &Fn(Scalar) -> Scalar) -> Self
    {
        for val in self.iter_mut() {
            *val = f(val.clone());
        }
        self
    }

    /// Split the NDArray at the specified layout returning two `NDArraySliceMut`s.
    fn split_at_mut(&mut self, mid: usize, layout: Layout) -> (NDArraySliceMut, NDArraySliceMut) {

        let slice_1: NDArraySliceMut;
        let slice_2: NDArraySliceMut;

        match layout {
            Layout::Row => {
                assert!(mid < self.rows());
                unsafe {
                    slice_1 = NDArraySliceMut::from_raw_parts(self.as_mut_ptr(),
                                                            mid,
                                                            self.cols(),
                                                            self.row_stride());
                    slice_2 = NDArraySliceMut::from_raw_parts(self.as_mut_ptr()
                                                                .offset((mid *
                                                                    self.row_stride()) as
                                                                    isize),
                                                            self.rows() - mid,
                                                            self.cols(),
                                                            self.row_stride());
                }
            }
            Layout::Col => {
                assert!(mid < self.cols());
                unsafe {
                    slice_1 = NDArraySliceMut::from_raw_parts(self.as_mut_ptr(),
                                                            self.rows(),
                                                            mid,
                                                            self.row_stride());
                    slice_2 = NDArraySliceMut::from_raw_parts(self.as_mut_ptr()
                                                                .offset(mid as isize),
                                                            self.rows(),
                                                            self.cols() - mid,
                                                            self.row_stride());
                }
            }
        }

        (slice_1, slice_2)
    }

    /// Produce a `NDArraySliceMut` from an existing NDArray.
    fn sub_slice_mut<'a>(&mut self,
                         start: [usize; 2],
                         rows: usize,
                         cols: usize)
                         -> NDArraySliceMut<'a>
    {
        assert!(start[0] + rows <= self.rows(),
                "View dimensions exceed NDArray dimensions.");
        assert!(start[1] + cols <= self.cols(),
                "View dimensions exceed NDArray dimensions.");

        unsafe {
            NDArraySliceMut::from_raw_parts(self.as_mut_ptr()
                                              .offset((start[0] * self.row_stride() + start[1]) as
                                                  isize),
                                          rows,
                                          cols,
                                          self.row_stride())
        }
    }
}

impl<'a> Deref for Row<'a> {
    type Target = NDArraySlice<'a>;

    fn deref(&self) -> &NDArraySlice<'a> {
        &self.row
    }
}

impl<'a> Deref for RowMut<'a> {
    type Target = NDArraySliceMut<'a>;

    fn deref(&self) -> &NDArraySliceMut<'a> {
        &self.row
    }
}

impl<'a> DerefMut for RowMut<'a> {
    fn deref_mut(&mut self) -> &mut NDArraySliceMut<'a> {
        &mut self.row
    }
}

impl<'a> Deref for Column<'a> {
    type Target = NDArraySlice<'a>;

    fn deref(&self) -> &NDArraySlice<'a> {
        &self.col
    }
}

impl<'a> Deref for ColumnMut<'a> {
    type Target = NDArraySliceMut<'a>;

    fn deref(&self) -> &NDArraySliceMut<'a> {
        &self.col
    }
}

impl<'a> DerefMut for ColumnMut<'a> {
    fn deref_mut(&mut self) -> &mut NDArraySliceMut<'a> {
        &mut self.col
    }
}