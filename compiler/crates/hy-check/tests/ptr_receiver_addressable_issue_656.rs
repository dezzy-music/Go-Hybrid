//! ptr_receiver_addressable_issue_656.rs � regressions for issue #656:
//! Calling a `*T`-receiver method on a non-addressable receiver
//! (map element, call result, struct literal, chan receive) passes
//! `hyb check` with 0 diagnostics and emits Go that fails `go build`
//! (`cannot call pointer method on T`).
//!
//! Rejects with HYB-E2011 (BUILDABLE / T11). Controls (variables, slices,
//! pointers, value-receiver methods, slot loans) stay accepted.

use hy_check::{check_file, Verdict};
use hy_diag::Severity;
use hy_syntax::parse_file;

fn check(src: &str) -> hy_check::CheckResult {
    let path = "inline.hy";
    let ast = parse_file(path, src)
        .unwrap_or_else(|diags| panic!("must parse (parser contract): {:?}", diags));
    check_file(&ast)
}

fn error_codes(r: &hy_check::CheckResult) -> Vec<String> {
    r.diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

fn rejects_with(src: &str, code: &str) {
    let r = check(src);
    let cs = error_codes(&r);
    assert!(
        cs.iter().any(|c| c == code),
        "expected {}, got {:?}\nsrc:\n{}",
        code,
        cs,
        src
    );
    assert_eq!(r.verdict, Verdict::Reject, "src:\n{}", src);
}

fn accepts(src: &str) {
    let r = check(src);
    let cs = error_codes(&r);
    assert!(
        cs.is_empty(),
        "expected 0 errors, got {:?}\nsrc:\n{}",
        cs,
        src
    );
    assert_eq!(r.verdict, Verdict::Accept, "src:\n{}", src);
}

// ---------------------------------------------------------------------------
// Rejections (HYB-E2011): Non-addressable receivers on `*T` methods
// ---------------------------------------------------------------------------

#[test]
fn reject_map_element_ptr_method() {
    // Smallest reproducer from issue #656:
    rejects_with(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    by_name := map[string]counter{"a": counter{n: 0}}
    by_name["a"].bump()
}
"#,
        "HYB-E1002",
    );
}

#[test]
fn reject_call_result_ptr_method() {
    rejects_with(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn fresh() counter {
    return counter{n: 0}
}

fn main() {
    fresh().bump()
}
"#,
        "HYB-E1002",
    );
}

#[test]
fn reject_struct_literal_ptr_method() {
    rejects_with(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    counter{n: 5}.bump()
}
"#,
        "HYB-E1002",
    );
}

#[test]
fn reject_channel_receive_ptr_method() {
    rejects_with(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    ch := make(chan counter, 1)
    ch <- counter{n: 1}
    (<-ch).bump()
}
"#,
        "HYB-E1002",
    );
}

#[test]
fn reject_map_array_element_ptr_method() {
    // Map of arrays: array elements are not addressable in Go
    rejects_with(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    m := map[string][2]counter{}
    m["a"][0].bump()
}
"#,
        "HYB-E1002",
    );
}

#[test]
fn reject_nested_field_of_map_element() {
    rejects_with(
        r#"struct inner {
    n int
}

fn (i *inner) bump() {
    i.n += 1
}

struct outer {
    in inner
}

fn main() {
    by_name := map[string]outer{"a": outer{in: inner{n: 0}}}
    by_name["a"].in.bump()
}
"#,
        "HYB-E1002",
    );
}

#[test]
fn reject_task_spawn_on_non_addressable_receiver() {
    rejects_with(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    by_name := map[string]counter{"a": counter{n: 0}}
    task by_name["a"].bump()
}
"#,
        "HYB-E1002",
    );
}

#[test]
fn reject_defer_on_non_addressable_receiver() {
    rejects_with(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    by_name := map[string]counter{"a": counter{n: 0}}
    defer by_name["a"].bump()
}
"#,
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Controls (Accept): Addressable or value-receiver shapes must keep passing
// ---------------------------------------------------------------------------

#[test]
fn accept_local_var_ptr_method() {
    accepts(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    c := counter{n: 0}
    c.bump()
}
"#,
    );
}

#[test]
fn accept_pointer_var_ptr_method() {
    accepts(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    p := &counter{n: 0}
    p.bump()
}
"#,
    );
}

#[test]
fn accept_explicit_addr_of_ptr_method() {
    accepts(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    (&counter{n: 5}).bump()
}
"#,
    );
}

#[test]
fn accept_slice_element_ptr_method() {
    accepts(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    xs := []counter{counter{n: 0}}
    xs[0].bump()
}
"#,
    );
}

#[test]
fn accept_array_var_element_ptr_method() {
    accepts(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    var arr [2]counter
    arr[0].bump()
}
"#,
    );
}

#[test]
fn accept_map_of_pointers_ptr_method() {
    accepts(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    m_ptr := map[string]*counter{"a": &counter{n: 0}}
    m_ptr["a"].bump()
}
"#,
    );
}

#[test]
fn accept_value_receiver_method_on_map_element() {
    accepts(
        r#"struct counter {
    n int
}

fn (c counter) get() int {
    return c.n
}

fn main() {
    by_name := map[string]counter{"a": counter{n: 42}}
    _ = by_name["a"].get()
}
"#,
    );
}

#[test]
fn accept_value_receiver_method_on_call_result_and_lit() {
    accepts(
        r#"struct counter {
    n int
}

fn (c counter) get() int {
    return c.n
}

fn fresh() counter {
    return counter{n: 10}
}

fn main() {
    _ = fresh().get()
    _ = counter{n: 5}.get()
}
"#,
    );
}

#[test]
fn accept_field_of_pointer_struct() {
    accepts(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

struct parent {
    c counter
}

fn main() {
    p := &parent{c: counter{n: 0}}
    p.c.bump()
}
"#,
    );
}

#[test]
fn accept_field_of_addressable_var_struct() {
    accepts(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

struct parent {
    c counter
}

fn main() {
    v := parent{c: counter{n: 0}}
    v.c.bump()
}
"#,
    );
}

#[test]
fn accept_slot_loan_ptr_method() {
    accepts(
        r#"struct counter {
    n int
}

fn (c *counter) bump() {
    c.n += 1
}

fn main() {
    by_name := map[string]counter{"a": counter{n: 0}}
    slot := &by_name["a"]
    slot.bump()
}
"#,
    );
}
