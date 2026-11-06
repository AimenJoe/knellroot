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
// 둘째: CDP로 오래 누른 touchStart에 Chromium이 다는 개입 경고(우리 코드가 아니라 브라우저의 스크롤 판정, 무해).
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/, /Ignored attempt to cancel a touchstart event with cancelable=false/];
const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});

// src/touch.rs layout()과 같은 식(논리 px, 0053). 바꾸면 함께. 락온·등잔은 상황 버튼(뜰 때만) — 자리는 늘 같다.
function layout() {
  const u = Math.min(W, H, 520), r = u * 0.09;
  const atk = [W - u * 0.238, H - u * 0.21];
  const at = (deg, dist) => { const a = deg * Math.PI / 180; return [atk[0] + Math.cos(a) * u * dist, atk[1] - Math.sin(a) * u * dist]; };
  const turn = at(118, 0.44);
  const b = {
    attack: atk,
    dodge: at(188, 0.26),
    parry: at(146, 0.26),
    cast: at(104, 0.26),
    jump: at(62, 0.26),
    turn_left: [turn[0] - u * 0.08, turn[1]],
    turn_right: [turn[0] + u * 0.08, turn[1]],
    lock: at(80, 0.44),
    flask: at(150, 0.44),
    pause: [W - r * 0.7 * 1.4, r * 0.7 * 1.4],
    lab: [W - r * 0.7 * 4.0, r * 0.7 * 1.4],
    scenario: [W - r * 0.7 * 6.6, r * 0.7 * 1.4],
  };
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
  let b = layout();
  await shot('t00-title');
  // 타이틀(0019): 첫 줄 "시작"을 탭(뗄 때 누른다). 자리는 `app::draw_title`·`ui::menu`와 같은 셈.
  {
    // 화면에 맞춘 내부 해상도(0046, render::fit_internal): 배율 = 높이/360 반올림, 줄 수 = 높이/배율, UI 배율 = 그린 높이/270.
    const dpi = 3, s = Math.max(1, Math.min(Math.round(H * dpi / 360), Math.floor(W * dpi / 480)));
    const w = Math.floor(W * dpi / s), rows = Math.min(Math.floor(H * dpi / s), Math.floor(w * 3 / 4));
    const dh = rows * s / dpi, oy = Math.floor((H - dh) / 2), px = 13 * dh / 270;  // ui::GLYPH × Frame::ui
    await tap([W / 2, oy + dh * 0.37 + px * 0.5]);  // app::TITLE_MENU_Y
    // 시작하기 → 슬롯 창(0066): 첫 줄(슬롯 1, 비어 있음 = 시작)을 탭. 창은 480×270 기준 높이 106 —
    // 위 6·아래 안내 줄(2.8글) 사이 가운데, 제목 줄 뒤 첫 줄 가운데 ≈ 108/270(`app::menu` `draw_menu`·`ui::list`).
    await page.waitForTimeout(400);
    await tap([W / 2, oy + dh * 108 / 270]);
  }
  await page.waitForTimeout(1200);
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
  // 랩(0070): LAB을 누르면 멈춤 메뉴 안의 랩 창(게임은 멈춘다) — 창 밖을 누르면 닫힌다.
  await tap(b.lab); await page.waitForTimeout(300);
  await shot('t05-lab');
  await tap([4, 4]); await page.waitForTimeout(300);
  await shot('t06-lab-closed');
  await tap(b.scenario); await page.waitForTimeout(800);
  await shot('t07-next-scenario');
  // 땅 돌리기 오른쪽 ↷(0012·0017 — 카메라가 오른쪽으로): 도는 중 · 다 돈 뒤(버튼이 대기로 옅어짐).
  await tap(b.turn_right); await page.waitForTimeout(120);
  await shot('t08-turn-cw');
  await page.waitForTimeout(600);
  await shot('t09-after-turn');
  // 멈춤(2026-11-06): 버튼을 누른 채 창이 열리고 나서 떼도 창이 남아야 한다(뗀 자리가 "창 밖 탭"이 되어 닫히던 것).
  await touch('touchStart', [b.pause]); await page.waitForTimeout(300);
  await touch('touchEnd', []); await page.waitForTimeout(400);
  await shot('t10-pause-stays');
  await tap([4, H - 4]); await page.waitForTimeout(300);
  await shot('t11-pause-closed');
  await browser.close();
  server.close();
  for (const n of fs.readdirSync(OUT).filter((n) => /^t\d/.test(n))) console.log(`${n}\t${fs.statSync(path.join(OUT, n)).size} bytes`);
  if (errors.length) { console.error('console errors:\n' + errors.join('\n')); process.exit(1); }
  console.log('ok');
}
main().catch((e) => { console.error(e); process.exit(1); });
