// 큰 창의 탭 줄 굴리기(0232 → 0233): 헤드리스 Chromium으로 한국어·영어 × 넓은 화면(1280×720)·폰 가로(844×390) —
//   쉼터 기록을 심고 이어하기 → 쉬기(화로 = 탭 여섯, 0263) → E로 탭을 끝까지, Q로 처음까지 넘기며 화면마다 스크린숏 → 닫고 멈춤(탭 다섯)도 같이.
//   (0261 전에는 화로가 탭 여덟이라 넘쳤다 — 지금은 넘치지 않을 수 있다: 그래도 탭 줄·◀▶·글이 넘치지 않는지 F4로 본다.)
//   탭이 넘치면 글을 줄이지 않고 탭 줄을 굴린다(고른 탭이 보이게 따라오고, 양끝 ◀▶는 "더 있다"). 그리고 영어 폰(터치)에서
//   탭 줄(화로 창 — 탭이 가장 많다, 0263)을 손가락으로 왼쪽으로 끌어(CDP 터치) 가려진 탭을 드러내고, 드러난 끝 탭을 눌러 고른다 — 몸의 목록은 굴리지 않는다.
//   F4 넘침 검사의 `[ui-check]` 줄이 있으면 실패.
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/tabs.js dist shots
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');
const sceneKit = require('./scene');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8141;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const KEY = 'knell.save.0';
const SEED = 'v=1\nembers=700\nclears=1\nrun.rest=0\nrun.gate=1\nlv.life=1\nlv.stamina=4\nlv.might=4\ncore.warden=1\nmigrated.v04=1\n';
const HEARTH_TABS = 6; // 화로(0263): 성장 · 장비 · 가방 · 기술 · 가게 · 설정
const PAUSE_TABS = 5; // 멈춤(0261): 장비 · 가방 · 기술 · 기록 · 설정

const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});

async function run(browser, lang, size) {
  const [w, h] = size;
  const ctx = await browser.newContext({ viewport: { width: w, height: h } });
  const page = await ctx.newPage();
  const errors = [];
  const checks = [];
  page.on('console', (m) => {
    const t = m.text();
    if (/\[ui-check\]/.test(t) && !/\[ui-check\] (on|off)$/.test(t)) checks.push(t);
    if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(t))) errors.push(t);
  });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(String(e)); });
  const shot = (name) => page.screenshot({ path: path.join(OUT, `tabs-${lang}-${w}-${name}.png`) });
  // 장면이 바뀌는 누름 뒤에는 장면을 기다린다(tools/test/scene.js).
  const { press, until, flip } = sceneKit(page, 90);
  await sceneKit.cheap(page);
  const ready = () => page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 20000 });
  await page.goto(`http://localhost:${PORT}/`);
  await ready();
  await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, SEED]);
  await page.reload();
  await ready();
  await page.waitForTimeout(600);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  await until('Title', '타이틀');
  await press('F4');
  if (lang === 'en') {
    await press('ArrowDown', 2); await press('Enter'); await until('Settings { host: Title', '타이틀 → 설정');
    await press('ArrowRight'); await page.waitForTimeout(200);
    await press('Escape'); await until('Title', '설정 → 타이틀');
    await press('ArrowUp', 2);
  }
  await press('Enter'); await until('Slots', '이어하기 → 슬롯 창');
  await press('Enter'); await until('Play', '슬롯 → 판'); await page.waitForTimeout(1500);
  // 탭 줄 한 바퀴: E로 끝까지, Q로 처음까지.
  const cycle = async (n, tag) => {
    for (let i = 0; i < n; i++) {
      await shot(`${tag}e${i}`);
      if (i + 1 < n) { await flip('KeyE', `${tag} E — 탭 ${i + 1}`); await page.waitForTimeout(150); }
    }
    for (let i = n - 1; i > 0; i--) {
      await flip('KeyQ', `${tag} Q — 탭 ${i - 1}`); await page.waitForTimeout(150);
      await shot(`${tag}q${i - 1}`);
    }
  };
  await press('KeyF'); await until('Status { host: Hearth', '화로에서 쉬기 → 큰 창(성장 탭)'); await page.waitForTimeout(300);
  await cycle(HEARTH_TABS, 'hearth-');
  await press('Escape'); await until('Play', '화로 창 닫기');
  await press('Escape'); await until('Equip { host: Pause', '멈춤 = 장비 탭'); await page.waitForTimeout(300);
  await cycle(PAUSE_TABS, 'pause-');
  await ctx.close();
  return { errors, checks };
}

// 영어 폰(터치): 화로를 열고(키보드로) 탭 줄을 손가락으로 끌어 굴린 뒤 오른쪽 끝 탭을 눌러 고른다.
async function touchRun(browser) {
  const [w, h] = [844, 390];
  const ctx = await browser.newContext({ viewport: { width: w, height: h }, deviceScaleFactor: 2, hasTouch: true, isMobile: true });
  const page = await ctx.newPage();
  const errors = [];
  const checks = [];
  page.on('console', (m) => {
    const t = m.text();
    if (/\[ui-check\]/.test(t) && !/\[ui-check\] (on|off)$/.test(t)) checks.push(t);
    if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(t))) errors.push(t);
  });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(String(e)); });
  const cdp = await ctx.newCDPSession(page);
  const touch = (type, pts) => cdp.send('Input.dispatchTouchEvent', { type, touchPoints: pts.map(([x, y], i) => ({ x, y, id: i + 1 })) });
  const shot = (name) => page.screenshot({ path: path.join(OUT, `tabs-touch-${name}.png`) });
  // 장면이 바뀌는 누름 뒤에는 장면을 기다린다(tools/test/scene.js).
  const { press, until, flip } = sceneKit(page, 90);
  await sceneKit.cheap(page);
  const ready = () => page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 20000 });
  await page.goto(`http://localhost:${PORT}/`);
  await ready();
  await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, SEED]);
  await page.reload();
  await ready();
  await page.waitForTimeout(600);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  await until('Title', '타이틀');
  await press('F4');
  await press('ArrowDown', 2); await press('Enter'); await until('Settings { host: Title', '타이틀 → 설정');
  await press('ArrowRight'); await page.waitForTimeout(200);
  await press('Escape'); await until('Title', '설정 → 타이틀');
  await press('ArrowUp', 2);
  await press('Enter'); await until('Slots', '이어하기 → 슬롯 창');
  await press('Enter'); await until('Play', '슬롯 → 판'); await page.waitForTimeout(1500);
  await press('KeyF'); await until('Status { host: Hearth', '화로에서 쉬기 → 큰 창(성장 탭)'); await page.waitForTimeout(300);
  await shot('0-open');
  // 탭 줄(창 머리, 화면 위쪽)을 오른쪽 → 왼쪽으로 천천히 끈다.
  const y = 26;
  await touch('touchStart', [[420, y]]);
  for (let x = 420; x >= 140; x -= 20) { await touch('touchMove', [[x, y]]); await page.waitForTimeout(30); }
  await touch('touchEnd', []);
  await page.waitForTimeout(300);
  await shot('1-dragged');
  // 드러난 오른쪽 끝 탭(설정)을 누른다 — 화살(▶) 바로 왼쪽(넘치지 않으면 그 자리가 끝 탭이다).
  await touch('touchStart', [[455, y]]);
  await page.waitForTimeout(60);
  await touch('touchEnd', []);
  await until('Settings { host: Hearth', '끌어 드러난 끝 탭(설정)을 누른다');
  await page.waitForTimeout(200);
  await shot('2-tapped');
  await ctx.close();
  return { errors, checks };
}

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(PORT, r));
  const browser = await chromium.launch();
  let bad = false;
  for (const lang of ['ko', 'en']) {
    for (const size of [[1280, 720], [844, 390]]) {
      const { errors, checks } = await run(browser, lang, size);
      for (const e of errors) { console.error(`[${lang} ${size[0]}] ERROR ${e}`); bad = true; }
      for (const c of checks) { console.error(`[${lang} ${size[0]}] ${c}`); bad = true; }
      console.log(`[${lang} ${size[0]}] errors ${errors.length}, ui-check ${checks.length}`);
    }
  }
  {
    const { errors, checks } = await touchRun(browser);
    for (const e of errors) { console.error(`[touch] ERROR ${e}`); bad = true; }
    for (const c of checks) { console.error(`[touch] ${c}`); bad = true; }
    console.log(`[touch] errors ${errors.length}, ui-check ${checks.length}`);
  }
  await browser.close();
  server.close();
  process.exit(bad ? 1 : 0);
}
main().catch((e) => { console.error(e); process.exit(1); });
