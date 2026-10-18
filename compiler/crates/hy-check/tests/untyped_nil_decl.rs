//! untyped_nil_decl.rs — regressions for issue #672:
//! Untyped nil local declarations (var x = nil / x := nil / let x = nil)
//! passed `hyb check` with 0 diagnostics and emitted Go that failed `go build`
//! with "use of untyped nil in variable declaration" (BUILDABLE / N1).
//!
//! Rejects with HYB-E4003. Controls (ascribed error, zero-value declarations,
//! comparisons/calls, ascribed non-nilable E4001) stay intact.

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
// Rejections (HYB-E4003): Untyped nil declarations
// ---------------------------------------------------------------------------

#[test]
fn reject_var_nil() {
    rejects_with(
        r#"fn main() {
    var ch = nil
    _ = ch
}
"#,
        "HYB-E4003",
    );
}

#[test]
fn reject_define_nil() {
    rejects_with(
        r#"fn main() {
    ch := nil
    _ = ch
}
"#,
        "HYB-E4003",
    );
}

#[test]
fn reject_let_nil() {
    rejects_with(
        r#"fn main() {
    let x = nil
    _ = x
}
"#,
        "HYB-E4003",
    );
}

#[test]
fn reject_parallel_define_nil() {
    rejects_with(
        r#"fn main() {
    a, b := nil, 1
    _ = a
    _ = b
}
"#,
        "HYB-E4003",
    );
}

#[test]
fn reject_blank_ident_nil() {
    rejects_with(
        r#"fn main() {
    var _ = nil
}
"#,
        "HYB-E4003",
    );
}

#[test]
fn reject_closure_body_nil() {
    rejects_with(
        r#"fn main() {
    g := fn() {
        var x = nil
        _ = x
    }
    g()
}
"#,
        "HYB-E4003",
    );
}

#[test]
fn reject_loop_body_nil() {
    rejects_with(
        r#"fn main() {
    for i in 0..1 {
        var x = nil
        _ = x
    }
}
"#,
        "HYB-E4003",
    );
}

#[test]
fn reject_if_body_nil() {
    rejects_with(
        r#"fn main() {
    if true {
        var x = nil
        _ = x
    }
}
"#,
        "HYB-E4003",
    );
}

#[test]
fn reject_if_init_nil() {
    rejects_with(
        r#"fn main() {
    if x := nil; x != nil {
        _ = x
    }
}
"#,
        "HYB-E4003",
    );
}

#[test]
fn reject_match_arm_nil() {
    rejects_with(
        r#"fn main() {
    match 1 {
        1 => {
            var x = nil
            _ = x
        }
        _ => {}
    }
}
"#,
        "HYB-E4003",
    );
}

// ---------------------------------------------------------------------------
// Controls: Shapes that must KEEP accepting or keep E4001
// ---------------------------------------------------------------------------

#[test]
fn accept_ascribed_error_nil() {
    accepts(
        r#"fn main() {
    var e error = nil
    _ = e
}
"#,
    );
}

#[test]
fn accept_zero_value_decl_without_initializer() {
    accepts(
        r#"fn main() {
    var ch chan int
    var s []int
    _ = ch
    _ = s
}
"#,
    );
}

#[test]
fn accept_call_arg_and_comparison_nil() {
    accepts(
        r#"fn check_err(e error) bool {
    return e == nil
}

fn main() {
    _ = check_err(nil)
}
"#,
    );
}

#[test]
fn ascribed_non_nilable_keeps_e4001() {
    rejects_with(
        r#"fn main() {
    var s []int = nil
    _ = s
}
"#,
        "HYB-E4001",
    );
}
