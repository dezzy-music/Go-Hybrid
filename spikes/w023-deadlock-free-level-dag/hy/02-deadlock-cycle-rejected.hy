// Demonstrates static compile-time rejection of circular channel wait cycles.
//
// REJECT CASE: Inverted acquisition order between concurrent tasks.
// Task 1 holds c1 (level 1) and waits on c2 (level 2). (Legal: 1 < 2)
// Task 2 holds c2 (level 2) and attempts to wait on c1 (level 1). (REJECTED: 2 >= 1)
package main

fn task_inverted_hazard(c1 chan<1> int, c2 chan<2> int) {
    // Task holds c2 (level 2)
    v := <-c2

    // ATTEMPT TO WAIT ON c1 (level 1) WHILE HOLDING c2 (level 2):
    // hy-check intercepts this non-monotonic acquisition and raises:
    // HYB-E3801: inverted channel level acquisition: circular wait hazard detected:
    //            attempted to acquire level 1 while holding level 2
    c1 <- v // <-- COMPILE-TIME REJECTION
}
