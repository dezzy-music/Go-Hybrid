// Row 16 comparison: Today's "No claim" on deadlock-freedom vs W-023 Level-Ordered Channels.
//
// 1. TODAY (GUARANTEES.md Row 16 — NO SOUNDNESS CLAIM):
// Today, Go-Hybrid explicitly disclaims deadlock-freedom:
// "Deadlock-freedom is an emergent whole-program liveness property... outside Tier-0's static scope."
// Two tasks exchanging messages on unranked channels can form a circular wait
// and hang the entire program at runtime with zero compiler diagnostics.
package main

fn worker_a_today(chA chan int, chB chan int) {
    // Task A waits for Task B on chA, then signals chB
    v := <-chA
    chB <- v + 1
}

fn worker_b_today(chA chan int, chB chan int) {
    // Task B waits for Task A on chB, then signals chA
    // Classic circular wait deadlock: completely undetected by compiler today!
    v := <-chB
    chA <- v + 1
}

// 2. W-023 LEVEL-ORDERED CHANNEL SURFACE (CERTIFIED DEADLOCK-FREEDOM):
// Channels are ascribed a static partial order level: `ch1: chan<1> int`, `ch2: chan<2> int`.
// hy-check enforces Dijkstra's Resource Hierarchy Theorem:
// A task holding or blocked on a channel of level L may ONLY acquire channels of level L' > L.
//
// - Inverted acquisition (acquiring level 1 after level 2) is statically rejected (HYB-E3801).
// - Multi-channel select statements verify DAG acyclicity (HYB-E3802).
// - In hy-emit, `chan<L> T` is erased completely to native `chan T` (0 B/op, 0 allocs/op).
