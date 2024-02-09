// This is a lie at the moment as wasm-bindgen an js-sys
// bring in the std library. But if they ever get no_std
// working, we'll be able to benefit.
#![no_std]

use wasm_bindgen::prelude::*;

extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub async fn bootstrap2(
    method: &str,
    path: &str,
    body: &str,
) -> Result<JsValue, JsValue> {
    let resp = js_sys::Array::new();
    resp.push(&JsValue::from(200_u32));
    resp.push(&JsValue::from_str(
        &alloc::format!("{}\n", serde_json::to_string_pretty(&serde_json::json!({
            "js-now": js_sys::Date::now(),
            "method": method,
            "path": path,
            "body": body,
            "ai": bootstrap2_core::AgentInfo {
                space: [1; 32],
                agent: [2; 32],
                url: None,
                signed_at_micros: 42,
                expires_at_micros: 64,
                extra: serde_json::json!({}),
            }
            .encode()
            .map_err(|e| JsValue::from(alloc::string::ToString::to_string(&e)))?,
        })).unwrap()),
    ));
    Ok(resp.into())
}
