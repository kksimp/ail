//! `ail api --expanded` shows the code `@derive(ToJson, FromJson)` generates
//! for maps, optionals, lists and enum payloads (json.derive.1-3, ADR 0343).

use std::path::Path;
use std::process::Command;

#[test]
fn expanded_api_shows_generated_json_code() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/derive_json_enum_payloads.ail");
    let out = Command::new(env!("CARGO_BIN_EXE_ail")).args(["api", "--expanded"]).arg(&path).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let text = String::from_utf8(out.stdout).unwrap();
    for want in [
        "\"signature\":\"fn from_json(j Json) Self ! JsonError\"",
        "name := j.variant_name()?",
        "return Shape.Poly(points: v_points)",
        "return e.under(\\\"points[{v_points_i}]\\\")",
        "JsonMember(\\\"kind\\\", Json.Str(\\\"Circle\\\"))",
        "\"generator\":\"fn:derive_from_json\"",
    ] {
        assert!(text.contains(want), "missing {want} in {text}");
    }
}
