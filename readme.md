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
