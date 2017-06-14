extern crate flame;

#[test]
fn implicit_guarded_event() {
    flame::clear();
    flame::start_guard("foo");
}

#[test]
fn named_guarded_event() {
    flame::clear();
    let _name = flame::start_guard("foo");
}

#[test]
fn dropped_guarded_event() {
    flame::clear();
    let name = flame::start_guard("foo");
    name.end();
}

#[test]
#[allow(unreachable_code)]
fn multiple_guard_early_return() {
    flame::clear();
    let _first = flame::start_guard("foo");
    let _second = flame::start_guard("bar");
}

#[test]
fn single_event() {
    flame::clear();
    flame::start("event1");
    flame::end("event1");

    let spans = flame::spans();
    assert!(spans.len() == 1);
    assert!(spans[0].name == "event1");
}

#[test]
fn single_nested() {
    flame::clear();
    flame::start("event1");
        flame::start("event2");
        flame::end("event2");
    flame::end("event1");

    let spans = flame::spans();
    assert!(spans.len() == 1);
    assert!(spans[0].name == "event1");
    assert!(spans[0].depth == 0);

    let first = &spans[0];
    assert!(first.children.len() == 1);
    assert!(first.children[0].name == "event2");
    assert!(first.children[0].depth == 1);
}

#[test]
fn double_nested() {
    flame::clear();
    flame::start("event1");
        flame::start("event2");
        flame::end("event2");
        flame::start("event3");
        flame::end("event3");
    flame::end("event1");

    let spans = flame::spans();
    assert!(spans.len() == 1);
    assert!(spans[0].name == "event1");
    assert!(spans[0].depth == 0);

    let first = &spans[0];
    assert!(first.children.len() == 2);
    assert!(first.children[0].name == "event2");
    assert!(first.children[1].name == "event3");
    assert!(first.children[0].depth == 1);
    assert!(first.children[1].depth == 1);
}

#[test]
fn threads() {
    use std::thread::spawn;
    flame::clear();
    flame::start("main thread");
    let mut handles = vec![];

    for i in 0 .. 10 {
        handles.push(spawn(move || {
            if i % 2 == 0 {
                let s = format!("thread {}", i);
                flame::start(s.clone());
                flame::end(s);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    flame::end("main thread");

    let threads = flame::threads().into_iter().filter(|&flame::Thread { ref spans, .. }| {
        spans.len() == 1 && (spans[0].name.starts_with("thread ") ||
                             spans[0].name.starts_with("main thread"))
    });

    assert_eq!(threads.count(), 6);
}

#[test]
#[should_panic]
fn wrong_name() {
    flame::clear();
    flame::start("a");
    flame::end("b");
}

#[test]
#[should_panic]
fn cant_note() {
    flame::clear();
    flame::note("hi", None);
}

#[test]
fn end_with() {
    fn _inner() -> u32 {
        flame::clear();
        flame::start("w");
        flame::end_with("w", 1)
    }
    assert_eq!(1, _inner());
}
