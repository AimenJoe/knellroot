// 터치 연기 시험(0006): 가로 폰(844×390, DPR 3, 터치)으로 웹 빌드를 띄워 가상 스틱·버튼·랩 터치판을 눌러 보고 스크린숏을 남긴다.
//   node tools/test/touch.js dist shots
// 볼 것: 첫 화면부터 버튼이 보이는지(pointer: coarse), 스틱으로 걷는지, 락온 버튼이 노랗게 차는지, 랩 줄이 폰에 들어가는지.
// 손가락 끌기는 CDP Input.dispatchTouchEvent(Playwright touchscreen은 탭만 한다). 콘솔 에러가 있으면 종료 코드 1.
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8132;
const W = 844, H = 390;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});

// src/touch.rs layout()과 같은 식(논리 px). 바꾸면 함께.
function layout(lab) {
  const u = Math.min(W, H), r = u * 0.09, atkR = r * 1.5, dodgeR = r * 1.1, lockR = r * 0.85;
  const atk = [W - atkR * 1.3, H - atkR * 1.3];
  const b = {
    attack: atk,
    dodge: [atk[0] - (atkR + dodgeR + r * 0.5), H - dodgeR * 1.2],
    lock: [atk[0], atk[1] - (atkR + lockR + r * 0.45)],
    lab: [W - r * 0.7 * 1.4, r * 0.7 * 1.4],
    scenario: [W - r * 0.7 * 4.0, r * 0.7 * 1.4],
  };
  if (lab) {
    const k = r * 0.62, gap = k * 2.5, names = ['up', 'down', 'minus', 'plus', 'reset', 'store', 'swap'];
    const x0 = Math.max(W * 0.5 - gap * (names.length - 1) * 0.5, k * 1.3), y = u * 0.30;
    names.forEach((n, i) => { b['lab_' + n] = [x0 + gap * i, y]; });
  }
  return b;
}

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(PORT, r));
  const browser = await chromium.launch();
  const ctx = await browser.newContext({ viewport: { width: W, height: H }, deviceScaleFactor: 3, hasTouch: true, isMobile: true });
  const page = await ctx.newPage();
  const errors = [];
  page.on('console', (m) => { if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(m.text()))) errors.push(m.text()); });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(String(e)); });
  const cdp = await ctx.newCDPSession(page);
  const touch = (type, pts) => cdp.send('Input.dispatchTouchEvent', { type, touchPoints: pts.map(([x, y], i) => ({ x, y, id: i + 1 })) });
  const tap = async ([x, y]) => { await touch('touchStart', [[x, y]]); await page.waitForTimeout(60); await touch('touchEnd', []); await page.waitForTimeout(60); };
  const shot = (n) => page.screenshot({ path: path.join(OUT, n + '.png') });

  await page.goto(`http://localhost:${PORT}/`);
  await page.waitForTimeout(2500);
  let b = layout(false);
  await shot('t01-first-screen');
  // 스틱: 왼쪽 아래에 대고 위로 끌어 0.8초 걷기.
  const s0 = [W * 0.2, H * 0.7];
  await touch('touchStart', [s0]);
  for (let i = 1; i <= 6; i++) { await touch('touchMove', [[s0[0], s0[1] - i * 10]]); await page.waitForTimeout(30); }
  await page.waitForTimeout(800);
  await shot('t02-stick-walk');
  // 스틱을 쥔 채 공격(두 손가락).
  await touch('touchMove', [[s0[0], s0[1] - 60], b.attack]); await page.waitForTimeout(80);
  await touch('touchMove', [[s0[0], s0[1] - 60]]); await page.waitForTimeout(50);
  await touch('touchEnd', []); await page.waitForTimeout(100);
  await tap(b.lock); await page.waitForTimeout(700);
  await shot('t03-lock-on');
  await tap(b.dodge); await page.waitForTimeout(100);
  await shot('t04-dodge');
  // 랩: 열기 → ▼ 넷 → + 다섯 → 담기 → − 열 → 바꾸기.
  await tap(b.lab); await page.waitForTimeout(200);
  b = layout(true);
  for (let i = 0; i < 4; i++) await tap(b.lab_down);
  for (let i = 0; i < 5; i++) await tap(b.lab_plus);
  await tap(b.lab_store);
  for (let i = 0; i < 10; i++) await tap(b.lab_minus);
  await shot('t05-lab');
  await tap(b.lab_swap); await page.waitForTimeout(200);
  await shot('t06-lab-swapped');
  await tap(b.lab); await page.waitForTimeout(100);
  await tap(b.scenario); await page.waitForTimeout(800);
  await shot('t07-next-scenario');
  await browser.close();
  server.close();
  for (const n of fs.readdirSync(OUT).filter((n) => n.startsWith('t0'))) console.log(`${n}\t${fs.statSync(path.join(OUT, n)).size} bytes`);
  if (errors.length) { console.error('console errors:\n' + errors.join('\n')); process.exit(1); }
  console.log('ok');
}
main().catch((e) => { console.error(e); process.exit(1); });
