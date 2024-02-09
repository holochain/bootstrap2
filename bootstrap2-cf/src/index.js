/**
 * Welcome to Cloudflare Workers! This is your first worker.
 *
 * - Run `npm run dev` in your terminal to start a development server
 * - Open a browser tab at http://localhost:8787/ to see your worker in action
 * - Run `npm run deploy` to publish your worker
 *
 * Learn more at https://developers.cloudflare.com/workers/
 */

import wasm from './rust/bootstrap2_cf_bg.wasm'
import * as bg from './rust/bootstrap2_cf_bg.js'

const wasmInst = await WebAssembly.instantiate(wasm, {
  './bootstrap2_cf_bg.js': bg,
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

      const [status, respBody] = await bg.bootstrap2(
        request.method,
        path,
        body
      )
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
    }
  },
};
