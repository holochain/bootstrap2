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
    try {
      let path = '/'
      try {
        path = (new URL(request.url)).pathname
      } catch (_e) { /* pass */ }

      let body = ''
      if (request.method === 'POST' || request.method == 'PUT') {
        if (!request.headers || request.headers.get('content-type') !== 'application/json') {
          throw new Error('content-type must be application/json')
        }
        try {
          body = await request.text()
        } catch (_e) { /* pass */ }
      }

      KV = env.BOOTSTRAP
      const respBody = await bg.bootstrap2(
        KV,
        request.method,
        path,
        body
      )
      KV = null

      return new Response(respBody, {
        status: 200,
        headers: {
          'content-type': 'application/json'
        }
      })
    } catch (err) {
      return new Response(JSON.stringify({
        error: err.toString(),
      }, null, 2) + '\n', {
        status: 500,
        headers: {
          'content-type': 'application/json'
        }
      })
    } finally {
      KV = null
    }
  },
};
