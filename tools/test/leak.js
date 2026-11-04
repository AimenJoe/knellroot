// 오래 켜 두면 자라는 것을 잰다(2026-11-03: 핸드헬드에서 오래 두면 화면 일부가 하얗게 되고 꺼짐).
//   node tools/test/leak.js dist <초> [stage|stress]
// WebGL 함수를 감싸 살아 있는 버퍼·텍스처 수와 바이트, 프레임당 그리기 호출, wasm 선형 메모리를 10초마다 찍는다.
// 셋 중 하나라도 시간에 따라 계속 오르면 샌다. 처음 몇십 초 오르다 평평해지는 것은 "가장 많았던 순간"을 붙잡는 것(macroquad 호출 칸).
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const SECS = Number(process.argv[3] || 120);
const MODE = process.argv[4] || 'stage';
const PORT = 8137;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm' };
const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});

const PROBE = () => {
  const s = { buffers: 0, bufBytes: 0, textures: 0, texBytes: 0, draws: 0, frames: 0, peakDraws: 0, cur: 0 };
  const size = new WeakMap();
  const bound = { buf: new Map(), tex: null };
  window.__gl = s;
  for (const C of [window.WebGLRenderingContext, window.WebGL2RenderingContext]) {
    if (!C) continue;
    const P = C.prototype;
    const wrap = (name, fn) => { const o = P[name]; P[name] = function (...a) { return fn.call(this, o, a); }; };
    wrap('createBuffer', function (o, a) { s.buffers++; return o.apply(this, a); });
    wrap('deleteBuffer', function (o, a) { if (a[0]) { s.buffers--; s.bufBytes -= size.get(a[0]) || 0; size.delete(a[0]); } return o.apply(this, a); });
    wrap('bindBuffer', function (o, a) { bound.buf.set(a[0], a[1]); return o.apply(this, a); });
    wrap('bufferData', function (o, a) {
      const b = bound.buf.get(a[0]);
      const n = typeof a[1] === 'number' ? a[1] : (a[1] ? a[1].byteLength : 0);
      if (b) { s.bufBytes += n - (size.get(b) || 0); size.set(b, n); }
      return o.apply(this, a);
    });
    wrap('createTexture', function (o, a) { s.textures++; if (s.frames > 300 && (s.stacks = s.stacks || []).length < 3) s.stacks.push(new Error().stack); return o.apply(this, a); });
    wrap('deleteTexture', function (o, a) { if (a[0]) { s.textures--; s.texBytes -= size.get(a[0]) || 0; size.delete(a[0]); } return o.apply(this, a); });
    wrap('bindTexture', function (o, a) { bound.tex = a[1]; return o.apply(this, a); });
    wrap('texImage2D', function (o, a) {
      if (bound.tex && a.length >= 9) { const n = a[3] * a[4] * 4; s.texBytes += n - (size.get(bound.tex) || 0); size.set(bound.tex, n); }
      return o.apply(this, a);
    });
    wrap('drawElements', function (o, a) { s.cur++; return o.apply(this, a); });
    wrap('drawElementsInstanced', function (o, a) { s.cur++; return o.apply(this, a); });
  }
  // WebGL1에서 miniquad는 ANGLE 확장의 인스턴스 그리기를 쓴다 — 확장 객체를 받을 때 감싼다.
  const ge = WebGLRenderingContext.prototype.getExtension;
  WebGLRenderingContext.prototype.getExtension = function (name) {
    const e = ge.call(this, name);
    if (e && name === 'ANGLE_instanced_arrays' && !e.__wrapped) {
      const o = e.drawElementsInstancedANGLE.bind(e);
      e.drawElementsInstancedANGLE = (...a) => { s.cur++; return o(...a); };
      e.__wrapped = true;
    }
    return e;
  };
  const tick = () => { s.frames++; s.draws = s.cur; s.peakDraws = Math.max(s.peakDraws, s.cur); s.cur = 0; requestAnimationFrame(tick); };
  requestAnimationFrame(tick);
};

async function main() {
  await new Promise((r) => server.listen(PORT, r));
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  await page.addInitScript(PROBE);
  const errors = [];
  page.on('pageerror', (e) => errors.push(String(e)));
  page.on('console', (m) => { if (m.type() === 'error' && !/register_plugin/.test(m.text())) errors.push(m.text()); });
  await page.goto(`http://localhost:${PORT}/`);
  await page.waitForTimeout(2500);
  await page.locator('#glcanvas').click({ position: { x: 640, y: 360 } });
  const press = async (k, n = 1) => { for (let i = 0; i < n; i++) { await page.keyboard.press(k); await page.waitForTimeout(60); } };
  if (MODE === 'stress') { await press('ArrowDown', 2); await press('Enter'); await page.waitForTimeout(300); await press('ArrowDown', 2); await press('Enter'); await page.waitForTimeout(300); await press('Digit7'); }
  else { await press('Enter'); }
  const t0 = Date.now();
  let next = 0, i = 0;
  const keys = ['KeyW', 'KeyD', 'KeyS', 'KeyA'];
  console.log('sec\twasmMB\tbuffers\tbufMB\ttextures\ttexMB\tdraws\tpeak\tjsMB');
  while ((Date.now() - t0) / 1000 < SECS) {
    // 걷고 치고 돌리기(땅 돌리기는 대기가 있어 가끔).
    const k = keys[i++ % 4];
    await page.keyboard.down(k); await page.waitForTimeout(700); await page.keyboard.up(k);
    await press('KeyJ', 3);
    if (i % 5 === 0) await press('KeyE');
    if (i % 7 === 0) await press('KeyL');
    const sec = (Date.now() - t0) / 1000;
    if (sec >= next) {
      next += 10;
      const r = await page.evaluate(() => ({ g: window.__gl, w: (typeof wasm_memory !== 'undefined' && wasm_memory) ? wasm_memory.buffer.byteLength : 0, js: performance.memory ? performance.memory.usedJSHeapSize : 0 }));
      const mb = (x) => (x / 1048576).toFixed(1);
      console.log([sec.toFixed(0), mb(r.w), r.g.buffers, mb(r.g.bufBytes), r.g.textures, mb(r.g.texBytes), r.g.draws, r.g.peakDraws, mb(r.js)].join('\t'));
    }
  }
  const st = await page.evaluate(() => window.__gl.stacks || []);
  if (st.length) console.log('createTexture stacks:\n' + st.join('\n---\n'));
  if (errors.length) console.log('errors:', errors.slice(0, 5));
  await browser.close(); server.close();
}
main().catch((e) => { console.error(e); process.exit(1); });
