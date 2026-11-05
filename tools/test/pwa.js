// 웹 로딩 화면·PWA 시험(0062): 헤드리스 Chromium으로
//   1) 로딩 화면이 보이다가 첫 프레임 뒤 사라지는가(막대가 거꾸로 가지 않는가), 2) 매니페스트·아이콘이 읽히는가,
//   3) 서비스 워커가 켜지고 캐시 이름에 빌드 아이디가 들어가는가, 4) 네트워크를 끊고 다시 읽어도 게임이 뜨는가.
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/pwa.js dist shots
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8134;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];

// wasm을 천천히 흘려 로딩 막대가 움직이는 것을 본다(64 KB씩, 조각 사이 15 ms ≈ 2.6 MB에 0.6초).
const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  const body = fs.readFileSync(f);
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream', 'Content-Length': body.length });
  if (!f.endsWith('.wasm')) return res.end(body);
  let at = 0;
  const step = () => { if (at >= body.length) return res.end(); res.write(body.subarray(at, at + 65536)); at += 65536; setTimeout(step, 15); };
  step();
});

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(PORT, r));
  const browser = await chromium.launch();
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const page = await ctx.newPage();
  const errors = [];
  page.on('console', (m) => { if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(m.text()))) errors.push(m.text()); });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(String(e)); });
  const fail = (msg) => { console.error('FAIL ' + msg); process.exitCode = 1; };

  // 1) 로딩 화면: 막대 값을 매 25 ms 적는다.
  await page.addInitScript(() => {
    window.__bar = [];
    const t = setInterval(() => {
      const el = document.getElementById('knell-load');
      if (!el) { window.__bar.push('gone'); clearInterval(t); return; }
      window.__bar.push(Number(el.getAttribute('aria-valuenow')));
    }, 25);
  });
  await page.goto(`http://localhost:${PORT}/`);
  await page.waitForTimeout(250);
  await page.screenshot({ path: path.join(OUT, 'pwa_loading.png') });
  await page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 15000 }).catch(() => fail('로딩 화면이 사라지지 않았다'));
  const bar = await page.evaluate(() => window.__bar);
  const nums = bar.filter((v) => v !== 'gone');
  const mid = nums.filter((v) => v > 0 && v < 90).length;
  const back = nums.some((v, i) => i > 0 && v < nums[i - 1]);
  console.log(`막대: ${nums.length}번 적음, 중간값 ${mid}개, 최대 ${Math.max(...nums)}%, 거꾸로 ${back}`);
  if (mid < 3) fail('막대가 중간을 거의 지나지 않았다(진행이 안 보인다)');
  if (back) fail('막대가 거꾸로 갔다');
  if (await page.evaluate(() => document.getElementById('knell-load') && document.getElementById('knell-load').classList.contains('fail'))) fail('실패 표시');

  // 2) 매니페스트·아이콘.
  const man = await page.evaluate(async () => {
    const href = document.querySelector('link[rel=manifest]').href;
    const m = await (await fetch(href)).json();
    const icons = await Promise.all(m.icons.map(async (i) => {
      const r = await fetch(new URL(i.src, href)); const b = await r.blob();
      const img = await createImageBitmap(b); return `${i.src} ${img.width}x${img.height} ${i.purpose}`;
    }));
    return { name: m.name, display: m.display, orientation: m.orientation, icons };
  });
  console.log('매니페스트:', JSON.stringify(man));
  if (man.display !== 'fullscreen' || man.orientation !== 'landscape') fail('매니페스트 display/orientation');
  for (const i of man.icons) { const [, wh] = i.split(' '); const [w, h] = wh.split('x').map(Number); if (w !== h || !(i.includes(String(w)))) fail('아이콘 크기 ' + i); }

  // 3) 서비스 워커.
  await page.waitForFunction(() => navigator.serviceWorker && navigator.serviceWorker.controller === null && navigator.serviceWorker.ready, null, { timeout: 10000 }).catch(() => {});
  const sw = await page.evaluate(async () => {
    const reg = await navigator.serviceWorker.ready;
    await new Promise((r) => setTimeout(r, 1500)); // 미리 담기가 끝나기를
    const keys = await caches.keys();
    const c = await caches.open(keys[0]);
    const urls = (await c.keys()).map((r) => new URL(r.url).pathname);
    return { state: reg.active && reg.active.state, keys, urls };
  });
  console.log('서비스 워커:', JSON.stringify(sw));
  const build = fs.readFileSync(path.join(DIST, 'build.txt'), 'utf8').trim();
  if (sw.state !== 'activated') fail('서비스 워커가 켜지지 않았다');
  if (sw.keys.length !== 1 || sw.keys[0] !== 'knell-' + build) fail('캐시 이름이 빌드 아이디가 아니다');
  if (!sw.urls.includes('/knellroot.wasm')) fail('wasm이 미리 담기지 않았다');

  // 4) 오프라인으로 다시 읽기.
  await ctx.setOffline(true);
  await page.reload();
  await page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 15000 }).catch(() => fail('오프라인에서 게임이 뜨지 않았다'));
  await page.waitForTimeout(800);
  await page.screenshot({ path: path.join(OUT, 'pwa_offline.png') });
  await ctx.setOffline(false);

  if (errors.length) { console.error('콘솔 에러:\n' + errors.join('\n')); process.exitCode = 1; }
  await browser.close();
  server.close();
  console.log(process.exitCode ? '실패' : '통과');
}
main().catch((e) => { console.error(e); process.exit(1); });
