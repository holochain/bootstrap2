// This is a lie at the moment as wasm-bindgen and js-sys
// bring in the std library. But if they ever get no_std
// working, we'll be able to benefit.
#![no_std]

use wasm_bindgen::prelude::*;

extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: JsValue);
}

#[wasm_bindgen(inline_js = "export async function kv_put(KV, k, v) { return await KV.put(k, v) }")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn kv_put(kv: &JsValue, k: &JsValue, v: &JsValue) -> Result<(), JsValue>;
}

#[wasm_bindgen(inline_js = "export async function kv_get(KV, k) { return await KV.get(k) }")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn kv_get(kv: &JsValue, k: &JsValue) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
pub async fn bootstrap2(
    kv: JsValue,
    method: &str,
    path: &str,
    body: &str,
) -> Result<JsValue, JsValue> {
    kv_put(&kv, &"test".into(), &"val".into()).await?;
    log("kv_get:".into());
    log(kv_get(&kv, &"test".into()).await?);
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
