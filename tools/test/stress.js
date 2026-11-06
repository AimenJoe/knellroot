// P5 측정(0013): 시나리오 6(잡몹 40)·7(80)을 띄워 HUD 측정 줄(cpu ms·draws·verts)을 스크린숏으로 남긴다.
//   node tools/test/stress.js dist shots
// 헤드리스 Chromium은 소프트웨어 렌더라 fps·frame ms는 판정 근거가 아니다 — cpu ms(우리 코드)와 draws(그리기 호출 수)를 본다.
// 실기 판정은 사람이 폰·PC 브라우저로(개발판 → 6/7, 개발 줄 F3). 상자/glTF 스위치(render.model)는 0070에서 걷었다.
const { chromium } = require('playwright');
const http = require('http'), fs = require('fs'), path = require('path');
const DIST = path.resolve(process.argv[2] || 'dist'); const OUT = path.resolve(process.argv[3] || 'shots');
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm' };
const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' }); fs.createReadStream(f).pipe(res);
});
(async () => {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(8151, r));
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  const errors = [];
  page.on('console', (m) => { if (m.type() === 'error' && !/register_plugin/.test(m.text())) errors.push(m.text()); });
  page.on('pageerror', (e) => { if (!/register_plugin/.test(String(e))) errors.push(String(e)); });
  await page.goto('http://localhost:8151/'); await page.waitForTimeout(2500);
  const key = async (k) => { await page.keyboard.press(k); await page.waitForTimeout(60); };
  for (const [digit, name] of [['Digit6', 's40'], ['Digit7', 's80']]) {
    await key(digit); await page.waitForTimeout(2500);
    await page.screenshot({ path: path.join(OUT, `${name}.png`) });
  }
  await browser.close(); server.close();
  for (const n of fs.readdirSync(OUT)) console.log(n);
  if (errors.length) { console.error('console errors:\n' + errors.join('\n')); process.exit(1); }
  console.log('ok');
})().catch((e) => { console.error(e); process.exit(1); });
