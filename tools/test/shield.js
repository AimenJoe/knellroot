// 방패 시험(0223): 방패를 가진 기록으로 — 장비 탭의 방패 칸(인형이 방패를 든다 · 후보 목록) · 싸움 화면에서 패링 버튼을 쥔 막는 자세 ·
//   화로의 가게에 방패 줄(없는 기록). 화면마다 스크린숏, 개발판 F4 넘침 검사(`[ui-check]` 줄이 있으면 실패).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/shield.js dist shots
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8141;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const KEY = 'knell.save.0';
const BASE = 'v=1\nembers=300\nclears=1\nrun.rest=0\nrun.gate=1\nweapon=sword\nmigrated.v04=1\n';
const OWNED = BASE + 'shields=pilgrim\nshield.worn=pilgrim\nweapon.sub=hammer\n';

const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});

async function open(browser, seed, tag) {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 }, deviceScaleFactor: 2 });
  const page = await ctx.newPage();
  const bad = [];
  page.on('console', (m) => {
    const t = m.text();
    if (/\[ui-check\]/.test(t) && !/\[ui-check\] (on|off)$/.test(t)) bad.push(t);
    if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(t))) bad.push(t);
  });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) bad.push(String(e)); });
  const shot = (name, clip) => page.screenshot({ path: path.join(OUT, `shield-${tag}-${name}.png`), clip });
  const press = async (key, n = 1) => { for (let i = 0; i < n; i++) { await page.keyboard.press(key); await page.waitForTimeout(90); } };
  const ready = () => page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 60000 });
  await page.goto(`http://localhost:${PORT}/`);
  await ready();
  await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, seed]);
  await page.reload();
  await ready();
  await page.waitForTimeout(600);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  await press('F4');
  await press('Enter'); await page.waitForTimeout(300);
  await press('Enter'); await page.waitForTimeout(1500);
  return { page, shot, press, bad, record: () => page.evaluate((k) => localStorage.getItem(k), KEY) };
}

(async () => {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(PORT, r));
  const browser = await chromium.launch();
  const errors = [];
  const body = { x: 440, y: 160, width: 400, height: 400 };
  {
    // 방패를 든 기록: 싸움 화면(선 채) → 패링 버튼(L-Shift)을 쥔 막는 자세 → 장비 탭(Tab)의 인형·방패 칸·후보.
    const g = await open(browser, OWNED, 'own');
    // 카메라 쪽(아래)으로 돌아서서 — 방패의 앞면이 화면에 보이게.
    await g.page.keyboard.down('KeyS'); await g.page.waitForTimeout(350); await g.page.keyboard.up('KeyS');
    await g.page.waitForTimeout(400);
    await g.shot('1-stand', body);
    await g.shot('1b-hud');
    await g.page.keyboard.down('ShiftLeft');
    await g.page.waitForTimeout(700);
    await g.shot('2-guard', body);
    await g.page.keyboard.down('KeyW');
    await g.page.waitForTimeout(500);
    await g.shot('3-guard-back', body);
    await g.page.keyboard.up('KeyW');
    await g.page.keyboard.up('ShiftLeft');
    await g.page.waitForTimeout(300);
    await g.press('Tab'); await g.page.waitForTimeout(500);
    await g.shot('4-equip');
    await g.press('ArrowDown', 2); await g.page.waitForTimeout(200);
    await g.shot('5-equip-shield-slot');
    await g.press('Enter'); await g.page.waitForTimeout(300);
    await g.shot('6-equip-shield-pick');
    await g.press('ArrowUp'); await g.press('Enter'); await g.page.waitForTimeout(300);
    const r = await g.record();
    if (/shield\.worn=/.test(r)) errors.push(`the shield was not taken off: ${JSON.stringify(r)}`);
    if (!/shields=pilgrim\n/.test(r)) errors.push(`the owned shield was lost: ${JSON.stringify(r)}`);
    await g.shot('7-equip-no-shield');
    errors.push(...g.bad);
  }
  {
    // 방패가 없는 기록: 쉬기(F) → 가게 탭(상태에서 E 넷 — 장비·기술·도감·여정… 가게) → 방패 줄 → 산다.
    const g = await open(browser, BASE, 'buy');
    await g.press('KeyF'); await g.page.waitForTimeout(700);
    await g.press('KeyE', 5); await g.page.waitForTimeout(300);
    await g.shot('1-shop');
    await g.press('ArrowDown', 5); await g.page.waitForTimeout(200);
    await g.shot('2-shop-shield');
    await g.press('Enter'); await g.page.waitForTimeout(300);
    await g.shot('3-shop-ask');
    await g.press('Enter'); await g.page.waitForTimeout(400);
    await g.shot('4-shop-bought');
    const r = await g.record();
    if (!/shields=pilgrim\n/.test(r) || !/shield\.worn=pilgrim\n/.test(r)) errors.push(`buying did not give and wear the shield: ${JSON.stringify(r)}`);
    errors.push(...g.bad);
  }
  await browser.close();
  server.close();
  if (errors.length) { console.error(errors.join('\n')); process.exit(1); }
  console.log('ok');
})();
