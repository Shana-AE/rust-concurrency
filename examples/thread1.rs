use std::{sync::mpsc, thread, time};

use anyhow::{anyhow, Result};

const PRODUCERS_NUM: usize = 4;

#[allow(dead_code)]
#[derive(Debug)]
struct Msg {
    idx: usize,
    value: usize,
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    for i in 0..PRODUCERS_NUM {
        let tx = tx.clone();
        thread::spawn(move || producer(i, tx));
    }

    drop(tx); // 释放多余的tx（因为上边clone多了一份）， 否则rx无法结束

    /*
    在这段代码中，闭包内部使用了 `rx`，这是一个接收器（receiver）。Rust 的闭包会自动捕获并借用它们所需的环境。在这种情况下，`rx` 被闭包借用，因为它没有被移动（move）或者复制（copy）。

    然而，如果 `rx` 是一个不可复制的类型，并且在闭包之后还需要使用，那么你就需要使用 `move` 关键字来强制闭包获取 `rx` 的所有权。这样，`rx` 就会被移动到闭包中，而不是被借用。

    在代码中，`rx` 在闭包之后没有被再次使用，所以不需要 `move`。如果你尝试在闭包之后再次使用 `rx`，Rust 编译器会给出错误，因为 `rx` 已经被借用给了闭包，不能再次使用。在这种情况下，你需要使用 `move` 来改变所有权。 */
    let consumer = thread::spawn(|| {
        for msg in rx {
            println!("consumer {:?}", msg);
        }
        42 // 可以返回一个值
    });

    let secret = consumer
        .join()
        .map_err(|e| anyhow!("thread join error: {:?}", e))?;

    println!("secret {}", secret);

    Ok(())
}

fn producer(idx: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    loop {
        let value = rand::random::<usize>();
        tx.send(Msg::new(idx, value))?;
        let sleep_time = rand::random::<u8>() as u64 * 10;
        thread::sleep(time::Duration::from_millis(sleep_time));
        if rand::random::<u8>() % 5 == 0 {
            println!("producer {} exit", idx);
            break;
        }
    }

    Ok(())
}

impl Msg {
    fn new(idx: usize, value: usize) -> Self {
        Self { idx, value }
    }
}
