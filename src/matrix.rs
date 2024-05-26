use anyhow::Result;
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Mul};
use std::sync::mpsc;
use std::thread;

use crate::{dot_product, Vector};

const THREAD_NUM: usize = 4;

pub struct Matrix<T> {
    data: Vec<T>, // for better performance, did not use nest Vec,
    row: usize,
    col: usize,
}

pub struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

pub struct MsgOutput<T> {
    value: T,
    idx: usize,
}

pub struct Msg<T> {
    input: MsgInput<T>,
    // sender to send result back
    sender: oneshot::Sender<MsgOutput<T>>,
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Display + Mul<Output = T> + Add<Output = T> + AddAssign + Default + Copy + Send + 'static,
{
    if a.col != b.row {
        anyhow::bail!("Matrix multiply error: a.col != b.row");
    }

    let senders = (0..THREAD_NUM)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<Msg<T>>();
            thread::spawn(|| {
                for msg in rx {
                    let value = dot_product(msg.input.row, msg.input.col)?;
                    if let Err(e) = msg.sender.send(MsgOutput {
                        value,
                        idx: msg.input.idx,
                    }) {
                        eprintln!("Send error: {}", e);
                    }
                }
                Ok::<_, anyhow::Error>(())
            });
            tx
        })
        .collect::<Vec<_>>();

    let matrix_len = a.row * b.col;

    let mut data = vec![T::default(); matrix_len];
    let mut receivers = Vec::with_capacity(matrix_len);

    // map/reduce: map phase
    for i in 0..a.row {
        for j in 0..b.col {
            let row = Vector::new(&a.data[i * a.col..(i + 1) * a.col]);
            let col_data = b.data[j..]
                .iter()
                .step_by(b.col)
                .copied()
                .collect::<Vec<_>>();
            let col = Vector::new(col_data);
            let idx = i * b.col + j;

            let input = MsgInput::new(idx, row, col);
            let (tx, rx) = oneshot::channel();
            let msg = Msg::new(input, tx);
            if let Err(e) = senders[idx % THREAD_NUM].send(msg) {
                eprintln!("Result send error: {}", e);
            }
            receivers.push(rx);
        }
    }

    // map/reduce: reduce phase
    for rx in receivers {
        let rst = rx.recv()?;
        data[rst.idx] = rst.value;
    }

    Ok(Matrix {
        data,
        row: a.row,
        col: b.col,
    })
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

impl<T> MsgInput<T> {
    pub fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}

impl<T> Msg<T> {
    pub fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
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
