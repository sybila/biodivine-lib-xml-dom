//! Book chapter "Thread safety": one document, many threads.
//!
//! Run with `cargo run --example book_threads`.

use biodivine_lib_xml_dom::{Document, QualifiedName, write_string};

fn main() {
    let document = Document::empty();
    let root = document.create_element(QualifiedName::without_namespace("root").unwrap());
    document.set_root(root.clone());

    // Handles are `Send + Sync`: pass the document, or a node, to another thread.
    let workers: Vec<_> = (0..4)
        .map(|index| {
            let document = document.clone();
            let root = root.clone();
            std::thread::spawn(move || {
                for item in 0..25 {
                    let node = document.create_element(
                        QualifiedName::without_namespace(format!("item{index}_{item}")).unwrap(),
                    );
                    node.append_child(document.create_text("payload").unwrap());
                    root.append_child(node);
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }

    // Readers can run concurrently with each other; a read never sees a half-finished edit,
    // because every operation is one critical section on the document's single lock.
    let readers: Vec<_> = (0..4)
        .map(|_| {
            let document = document.clone();
            std::thread::spawn(move || {
                let mut total = 0usize;
                for _ in 0..50 {
                    total += document.root().unwrap().children().len();
                    let _ = write_string(&document).unwrap();
                }
                total
            })
        })
        .collect();
    for reader in readers {
        assert!(reader.join().unwrap() > 0);
    }

    assert_eq!(root.children().len(), 4 * 25);
    assert!(document.is_valid());
    println!("{} children, document valid", root.children().len());
}
