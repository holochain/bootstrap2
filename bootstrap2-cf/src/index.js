import wasm from './rust/bootstrap2_cf_bg.wasm'
import * as bg from './rust/bootstrap2_cf_bg.js'

let KV = null
const wasmInst = await WebAssembly.instantiate(wasm, {
  './bootstrap2_cf_bg.js': bg,
  './cloudflare.js': {
    kv_put: async (k, v) => {
      KV.put(k, v)
    },
    kv_get: async (k) => {
      KV.get(k)
    }
  }
})
bg.__wbg_set_wasm(wasmInst.exports)

export default {
  async fetch(request, env, ctx) {
    await env.BOOTSTRAP.put('bob', 'yo')
    console.log(await env.BOOTSTRAP.get('bob'))
    try {
      let path = '/'
      try {
        path = (new URL(request.url)).pathname
      } catch (_e) { /* pass */ }

      let body = ''
      try {
        body = await request.text()
      } catch (_e) { /* pass */ }

      KV = env.BOOTSTRAP
      const [status, respBody] = await bg.bootstrap2(
        KV,
        request.method,
        path,
        body
      )
      KV = null

      return new Response(respBody, {
        status,
        headers: {
          'content-type': 'application/json'
        }
      })
    } catch (err) {
      return new Response('Error: ' + err, {
        status: 500,
        headers: {
          'content-type': 'text/plain'
        }
      })
    } finally {
      KV = null
    }
  },
};
