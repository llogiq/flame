# FLAME
#### A cool flamegraph library for rust

Flamegraphs are a great way to view profiling information.
At a glance, they give you information about how much time your
program spends in critical sections of your code giving you some
much-needed insight into where optimizations may be needed.

Unlike tools like `perf` which have the OS interrupt your running
program repeatadly and reports on every function in your callstack,
Flame lets you choose what you want to see in the graph by adding
performance instrumentation to your own code.

Simply use any of `flame`s APIs to annotate the start and end of a
block code that you want timing information from, and `flame` will
organize these timings hierarchically.

Here's an example of how to use flame:

```rust
extern crate flame;

fn main() {
    // Manual `start` and `end`
    flame::start("read file");
    let x = read_a_file();
    flame::end("read file");

    // Time the execution of a closure
    let y = flame::span_of("database query", || query_database());

    // Time the execution of a block by placing a guard
    let z = {
        let _ = flame::start_guard("cpu-heavy calculation");
        cpu_heavy_operations_1();
        cpu_heavy_operations_2();
    }

    // Dump the report to disk
    flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
    println!("{} {} {}", x, y, z);
}
```

And here's a screenshot of the flamegraph produced by one of my projects:

![flamegraph](./resources/screenshot.png "Flamegraph example")
