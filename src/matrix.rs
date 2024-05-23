use anyhow::Result;
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Mul};

use crate::Vector;

pub struct Matrix<T> {
    data: Vec<T>, // for better performance, did not use nest Vec,
    row: usize,
    col: usize,
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Display + Mul<Output = T> + Add<Output = T> + AddAssign + Default + Copy,
{
    if a.col != b.row {
        anyhow::bail!("Matrix multiply error: a.col != b.row");
    }
    let mut data = vec![T::default(); a.row * b.col];
    for i in 0..a.row {
        for j in 0..b.col {
            for k in 0..b.row {
                data[i * b.col + j] += a.data[i * a.col + k] * b.data[k * b.col + j];
            }
        }
    }
    Ok(Matrix {
        data,
        row: a.row,
        col: b.col,
    })
}

pub fn dot_product<T>(a: Vector<T>, b: Vector<T>) -> Result<T>
where
    T: Copy + Default + Add<Output = T> + AddAssign + Mul<Output = T>,
{
    if a.len() != b.len() {
        anyhow::bail!("Dot product error: a.len != b.len");
    }

    let mut sum = T::default();
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }

    Ok(sum)
}

impl<T> Display for Matrix<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for i in 0..self.row {
            for j in 0..self.col {
                write!(f, "{}", self.data[i * self.col + j])?;
                if j != self.col - 1 {
                    write!(f, " ")?;
                }
            }
            if i != self.row - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<T> Debug for Matrix<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Matrix(row={}, col={}, {})", self.row, self.col, self)
    }
}

impl<T> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, row: usize, col: usize) -> Self {
        Self {
            data: data.into(),
            row,
            col,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_display_and_debug() {
        let a = Matrix::new([1, 2, 3, 4], 2, 2);
        assert_eq!(format!("{}", a), "{1 2, 3 4}");
        assert_eq!(format!("{:?}", a), "Matrix(row=2, col=2, {1 2, 3 4})");
    }

    #[test]
    fn test_matrix_multiply() {
        let a = Matrix::new([1, 2, 3, 4], 2, 2);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = multiply(&a, &b).unwrap();
        assert_eq!(c.row, 2);
        assert_eq!(c.col, 2);
        assert_eq!(c.data, [7, 10, 15, 22]);
        assert_eq!(format!("{c:?}"), "Matrix(row=2, col=2, {7 10, 15 22})");

        let a = Matrix::new([1, 2, 3, 4], 2, 2);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let c = multiply(&a, &b).unwrap();
        assert_eq!(c.row, 2);
        assert_eq!(c.col, 3);
        assert_eq!(c.data, [9, 12, 15, 19, 26, 33]);
        assert_eq!(format!("{c}"), "{9 12 15, 19 26 33}");
    }
}
