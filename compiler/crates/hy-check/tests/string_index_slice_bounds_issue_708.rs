//! string_index_slice_bounds_issue_708.rs — regression for issue #708:
//! `hyb check` accepted out-of-bounds constant string index and slice bounds,
//! which then failed `go build` with `invalid argument: index N out of bounds [0:L]`.
//!
//! Scope: constant strings only (string literals, file-level `const` identifiers,
//! local `const` identifiers, and const-folded concatenations).
//! Checked against Go's byte-length semantics.

use hy_check::{check_file, Verdict};
use hy_diag::Severity;
use hy_syntax::parse_file;

fn check(src: &str) -> hy_check::CheckResult {
    let path = "test_issue_708.hy";
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

fn assert_rejects_e1002(src: &str, what: &str) {
    let r = check(src);
    let codes = error_codes(&r);
    assert!(
        codes.iter().any(|c| c == "HYB-E1002"),
        "expected HYB-E1002 for {}, got {:?}",
        what,
        codes
    );
    assert_eq!(r.verdict, Verdict::Reject, "expected Reject for {}", what);
}

fn assert_accepts(src: &str, what: &str) {
    let r = check(src);
    let codes = error_codes(&r);
    assert!(
        codes.is_empty(),
        "expected zero errors for {}, got {:?}",
        what,
        codes
    );
    assert_eq!(r.verdict, Verdict::Accept, "expected Accept for {}", what);
}

// ---------------------------------------------------------------------------
// 1. Exact Repros from Issue #708
// ---------------------------------------------------------------------------

#[test]
fn repro1_file_const_string_index_out_of_bounds_rejected() {
    assert_rejects_e1002(
        "const s = \"ab\"\nfn main() {\n    _ = s[5]\n}\n",
        "const s = \"ab\" / s[5]",
    );
}

#[test]
fn repro2_literal_string_index_out_of_bounds_rejected() {
    assert_rejects_e1002(
        "fn main() {\n    _ = \"hello\"[7]\n}\n",
        "\"hello\"[7]",
    );
}

#[test]
fn repro3_file_const_string_slice_out_of_bounds_rejected() {
    assert_rejects_e1002(
        "const s = \"ab\"\nfn main() {\n    _ = s[0:5]\n}\n",
        "const s = \"ab\" / s[0:5]",
    );
}

#[test]
fn repro4_literal_string_slice_out_of_bounds_rejected() {
    assert_rejects_e1002(
        "fn main() {\n    _ = \"hello\"[1:7]\n}\n",
        "\"hello\"[1:7]",
    );
}

// ---------------------------------------------------------------------------
// 2. Exact Controls from Issue #708 (Must Stay Accepted)
// ---------------------------------------------------------------------------

#[test]
fn control1_literal_string_index_in_bounds_accepted() {
    assert_accepts(
        "fn main() {\n    _ = \"hello\"[4]\n}\n",
        "\"hello\"[4] (4 < 5)",
    );
}

#[test]
fn control2_file_const_string_index_in_bounds_accepted() {
    assert_accepts(
        "const s = \"ab\"\nfn main() {\n    _ = s[1]\n}\n",
        "const s = \"ab\" / s[1] (1 < 2)",
    );
}

#[test]
fn control3_literal_string_slice_hi_equals_len_accepted() {
    assert_accepts(
        "fn main() {\n    _ = \"hello\"[0:5]\n}\n",
        "\"hello\"[0:5] (hi == len)",
    );
}

#[test]
fn control4_literal_string_slice_lo_equals_hi_accepted() {
    assert_accepts(
        "fn main() {\n    _ = \"hello\"[2:2]\n}\n",
        "\"hello\"[2:2] (lo == hi)",
    );
}

// ---------------------------------------------------------------------------
// 3. Local Const Declarations
// ---------------------------------------------------------------------------

#[test]
fn local_const_string_index_out_of_bounds_rejected() {
    assert_rejects_e1002(
        "fn main() {\n    const s = \"ab\"\n    _ = s[5]\n}\n",
        "local const s = \"ab\" / s[5]",
    );
    assert_rejects_e1002(
        "fn main() {\n    const s = \"ab\"\n    _ = s[2]\n}\n",
        "local const s = \"ab\" / s[2]",
    );
}

#[test]
fn local_const_string_slice_out_of_bounds_rejected() {
    assert_rejects_e1002(
        "fn main() {\n    const s = \"ab\"\n    _ = s[0:5]\n}\n",
        "local const s = \"ab\" / s[0:5]",
    );
    assert_rejects_e1002(
        "fn main() {\n    const s = \"ab\"\n    _ = s[3:3]\n}\n",
        "local const s = \"ab\" / s[3:3]",
    );
}

#[test]
fn local_const_string_valid_index_and_slice_accepted() {
    assert_accepts(
        "fn main() {\n    const s = \"ab\"\n    _ = s[0]\n    _ = s[1]\n    _ = s[0:2]\n    _ = s[1:2]\n    _ = s[2:2]\n}\n",
        "local const s = \"ab\" valid bounds",
    );
}

// ---------------------------------------------------------------------------
// 4. File-level Const Initializers
// ---------------------------------------------------------------------------

#[test]
fn file_const_initializer_index_out_of_bounds_rejected() {
    assert_rejects_e1002(
        "const s = \"ab\"\nconst x = s[5]\n",
        "const x = s[5]",
    );
    assert_rejects_e1002(
        "const x = \"hello\"[7]\n",
        "const x = \"hello\"[7]",
    );
}

#[test]
fn file_const_initializer_slice_out_of_bounds_rejected() {
    assert_rejects_e1002(
        "const s = \"ab\"\nconst x = s[0:5]\n",
        "const x = s[0:5]",
    );
    assert_rejects_e1002(
        "const x = \"hello\"[1:7]\n",
        "const x = \"hello\"[1:7]",
    );
}

#[test]
fn file_const_string_valid_bounds_in_function_accepted() {
    assert_accepts(
        "const s = \"ab\"\nfn main() {\n    _ = s[1]\n    _ = \"hello\"[4]\n    _ = s[0:2]\n}\n",
        "file const string read with valid bounds inside function",
    );
}

// ---------------------------------------------------------------------------
// 5. Const-folded String Concatenation (+)
// ---------------------------------------------------------------------------

#[test]
fn const_folded_concatenation_index_out_of_bounds_rejected() {
    assert_rejects_e1002(
        "const s = \"a\" + \"b\"\nfn main() {\n    _ = s[2]\n}\n",
        "const s = \"a\" + \"b\" / s[2]",
    );
    assert_rejects_e1002(
        "fn main() {\n    _ = (\"hello\" + \"world\")[10]\n}\n",
        "(\"hello\" + \"world\")[10]",
    );
}

#[test]
fn const_folded_concatenation_slice_out_of_bounds_rejected() {
    assert_rejects_e1002(
        "const s = \"a\" + \"b\"\nfn main() {\n    _ = s[0:3]\n}\n",
        "const s = \"a\" + \"b\" / s[0:3]",
    );
}

#[test]
fn const_folded_concatenation_valid_bounds_accepted() {
    assert_accepts(
        "const s = \"a\" + \"b\"\nfn main() {\n    _ = s[1]\n    _ = s[0:2]\n}\n",
        "const s = \"a\" + \"b\" valid bounds",
    );
    assert_accepts(
        "fn main() {\n    _ = (\"hello\" + \"world\")[9]\n    _ = (\"hello\" + \"world\")[0:10]\n}\n",
        "(\"hello\" + \"world\") valid bounds",
    );
}

// ---------------------------------------------------------------------------
// 6. Multi-byte UTF-8 String Byte-length Semantics
// ---------------------------------------------------------------------------

#[test]
fn utf8_string_byte_length_semantics() {
    // "héllo" has 6 UTF-8 bytes ('h' + 2 bytes for 'é' + 'l' + 'l' + 'o')
    assert_accepts(
        "const s = \"héllo\"\nfn main() {\n    _ = s[5]\n    _ = s[0:6]\n}\n",
        "\"héllo\" byte 5 and slice [0:6] accepted",
    );
    assert_rejects_e1002(
        "const s = \"héllo\"\nfn main() {\n    _ = s[6]\n}\n",
        "\"héllo\"[6] rejected (len is 6 bytes)",
    );
    assert_rejects_e1002(
        "const s = \"héllo\"\nfn main() {\n    _ = s[0:7]\n}\n",
        "\"héllo\"[0:7] rejected (hi 7 > 6)",
    );
}

// ---------------------------------------------------------------------------
// 7. Negative Index and Slice Bounds
// ---------------------------------------------------------------------------

#[test]
fn negative_constant_string_index_rejected() {
    assert_rejects_e1002(
        "fn main() {\n    _ = \"hello\"[-1]\n}\n",
        "\"hello\"[-1]",
    );
    assert_rejects_e1002(
        "const s = \"ab\"\nfn main() {\n    _ = s[-1]\n}\n",
        "s[-1]",
    );
}

#[test]
fn negative_constant_string_slice_bounds_rejected() {
    assert_rejects_e1002(
        "fn main() {\n    _ = \"hello\"[-1:2]\n}\n",
        "\"hello\"[-1:2]",
    );
    assert_rejects_e1002(
        "fn main() {\n    _ = \"hello\"[1:-2]\n}\n",
        "\"hello\"[1:-2]",
    );
}

// ---------------------------------------------------------------------------
// 8. Inverted Slice Bounds (lo > hi)
// ---------------------------------------------------------------------------

#[test]
fn inverted_string_slice_bounds_rejected() {
    assert_rejects_e1002(
        "fn main() {\n    _ = \"hello\"[4:2]\n}\n",
        "\"hello\"[4:2]",
    );
    assert_rejects_e1002(
        "const s = \"ab\"\nfn main() {\n    _ = s[2:1]\n}\n",
        "const s = \"ab\" / s[2:1]",
    );
}

// ---------------------------------------------------------------------------
// 9. Empty String Bounds
// ---------------------------------------------------------------------------

#[test]
fn empty_string_bounds() {
    assert_rejects_e1002(
        "fn main() {\n    _ = \"\"[0]\n}\n",
        "\"\"[0] out of bounds",
    );
    assert_rejects_e1002(
        "fn main() {\n    _ = \"\"[0:1]\n}\n",
        "\"\"[0:1] out of bounds",
    );
    assert_accepts(
        "fn main() {\n    _ = \"\"[0:0]\n    _ = \"\"[:]\n}\n",
        "\"\"[0:0] accepted",
    );
}

// ---------------------------------------------------------------------------
// 10. Omitted Slice Bounds
// ---------------------------------------------------------------------------

#[test]
fn omitted_slice_bounds() {
    assert_accepts(
        "fn main() {\n    _ = \"hello\"[:5]\n    _ = \"hello\"[2:]\n    _ = \"hello\"[:]\n}\n",
        "valid omitted bounds",
    );
    assert_rejects_e1002(
        "fn main() {\n    _ = \"hello\"[:6]\n}\n",
        "\"hello\"[:6]",
    );
    assert_rejects_e1002(
        "fn main() {\n    _ = \"hello\"[6:]\n}\n",
        "\"hello\"[6:]",
    );
}

// ---------------------------------------------------------------------------
// 11. Dynamic Strings Stay Unchecked
// ---------------------------------------------------------------------------

#[test]
fn dynamic_strings_stay_unchecked() {
    assert_accepts(
        "fn main() {\n    s := \"ab\"\n    _ = s[5]\n    _ = s[0:5]\n}\n",
        "dynamic local string s := \"ab\" unchecked at compile time",
    );
    assert_accepts(
        "fn f(s string) {\n    _ = s[5]\n    _ = s[0:5]\n}\n",
        "param string s unchecked at compile time",
    );
}

// ---------------------------------------------------------------------------
// 12. Raw Strings and Const Folding Operations
// ---------------------------------------------------------------------------

#[test]
fn raw_strings_and_const_folded_indices() {
    assert_rejects_e1002(
        "const s = `hello`\nfn main() {\n    _ = s[5]\n}\n",
        "raw string literal index out of bounds",
    );
    assert_accepts(
        "const s = `hello`\nfn main() {\n    _ = s[4]\n}\n",
        "raw string literal index in bounds",
    );
    assert_rejects_e1002(
        "const idx = 2 + 3\nfn main() {\n    _ = \"ab\"[idx]\n}\n",
        "const folded index expression out of bounds",
    );
    assert_accepts(
        "const idx = 1\nfn main() {\n    _ = \"ab\"[idx]\n}\n",
        "const folded index expression in bounds",
    );
}
