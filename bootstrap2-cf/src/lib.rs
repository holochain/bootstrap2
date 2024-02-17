// This is a lie at the moment as wasm-bindgen and js-sys
// bring in the std library. But if they ever get no_std
// working, we'll be able to benefit.
#![no_std]

use wasm_bindgen::prelude::*;
use bootstrap2_core::*;

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

fn js_val_to_string(v: JsValue) -> alloc::string::String {
    v.as_string().unwrap_or(alloc::string::ToString::to_string("null"))
}

fn js_val_to_err(v: JsValue) -> BootstrapError {
    BootstrapError::from_str(js_val_to_string(v))
}

struct CfSys(JsValue);

impl Sys for CfSys {
    fn date_now(&mut self) -> f64 {
        js_sys::Date::now()
    }

    /// Put a value into the KV store.
    fn kv_put(&mut self, key: &str, val: &str) -> BoxFut<'_, Result<(), BootstrapError>> {
        let key = JsValue::from(key);
        let val = JsValue::from(val);
        alloc::boxed::Box::pin(async move {
            crate::kv_put(&self.0, &key, &val)
                .await
                .map_err(js_val_to_err)
        })
    }

    /// Get a value from the KV store.
    fn kv_get(&mut self, key: &str) -> BoxFut<'_, Result<alloc::string::String, BootstrapError>> {
        let key = JsValue::from(key);
        alloc::boxed::Box::pin(async move {
            crate::kv_get(&self.0, &key)
                .await
                .map(js_val_to_string)
                .map_err(js_val_to_err)
        })
    }
}

#[wasm_bindgen]
pub async fn bootstrap2(
    kv: JsValue,
    method: &str,
    path: &str,
    body: &str,
) -> Result<JsValue, JsValue> {
    bootstrap2_core::bootstrap2(
        CfSys(kv),
        method,
        path,
        body,
    )
    .await
    .map(JsValue::from)
    .map_err(|e| JsValue::from(alloc::string::ToString::to_string(&e)))
}
