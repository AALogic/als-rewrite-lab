use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/desktop-ipc/v1")
        .join(name);
    serde_json::from_slice(&fs::read(path).expect("read desktop wire fixture"))
        .expect("parse desktop wire fixture")
}

fn assert_roundtrip<T>(name: &str)
where
    T: DeserializeOwned + Serialize,
{
    let expected = fixture(name);
    let decoded: T = serde_json::from_value(expected.clone()).expect("decode Rust wire type");
    let actual = serde_json::to_value(decoded).expect("encode Rust wire type");
    assert_eq!(actual, expected, "wire fixture drifted: {name}");
}

#[test]
fn desktop_ipc_wire_contract_is_stable() {
    assert_roundtrip::<rescue_application::DesktopAnalyzeResult>("analysis-result.json");
    assert_roundtrip::<rescue_application::DesktopCopyPreview>("copy-preview.json");
    assert_roundtrip::<rescue_application::DesktopExecuteCopyRequest>("copy-execute-request.json");
    assert_roundtrip::<rescue_application::DesktopCopyResult>("copy-result.json");
    assert_roundtrip::<rescue_application::DesktopCopyResult>("copy-error-result.json");
}
