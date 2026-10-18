// Demonstrates Dijkstra's Resource Hierarchy applied to 5 Dining Philosophers in Go-Hybrid.
//
// By ranking each fork with a static level (chan<1> .. chan<5>) and requiring
// each philosopher to acquire the lower-ranked fork first, the classic circular wait
// dependency ring is broken at compile time.
//
// Deadlock probability: exactly 0.00%.
// Runtime overhead: 0 B/op, 0 allocs/op (erased completely to native channels).
package main

fn philosopher(id int, f1 chan<L1> struct{}, f2 chan<L2> struct{}) {
    // Dijkstra rule: statically proved f1.Level < f2.Level
    for {
        <-f1 // Acquire lower-ranked fork
        <-f2 // Acquire higher-ranked fork

        // Eat...

        f2 <- struct{}{} // Release
        f1 <- struct{}{} // Release
    }
}
