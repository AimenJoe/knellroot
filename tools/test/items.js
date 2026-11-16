// 아이템 시험(0223 V4): 등급·옵션이 든 아이템이 있는 기록으로 — 장비 탭의 주무기 후보에 여벌이 서고(바탕의 줄 아래), 여벌을 고르면
//   옵션 줄과 바뀌는 능력치가 보이고, 끼면 기록의 낀 자리(gear.main)가 그 아이템을 가리킨다. 옛 기록(아이템 줄 없음)은 보통 것으로 옮겨진다. 화로의 가게에서 여벌을 잔불로 바꾼다.
//   화면마다 스크린숏, 개발판 F4 넘침 검사(`[ui-check]` 줄이 있으면 실패).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/items.js dist shots
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8143;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const KEY = 'knell.save.0';
const BASE = 'v=1\nembers=300\nclears=1\nrun.rest=0\nrun.gate=1\nweapon=sword\nmigrated.v04=1\nweapons=sword,hammer\ncharm.owned=ear\ncharm.worn=ear,,\n';
const ITEMS = BASE + 'item.0=w.sword|p||0\nitem.1=w.sword|r|dmg.1+swift.1|0\nitem.2=w.hammer|s|heavy.1|0\nitem.3=c.ear|s|hp.1|0\nitem.next=4\ngear.main=0\n';

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
  const shot = (name, clip) => page.screenshot({ path: path.join(OUT, `items-${tag}-${name}.png`), clip });
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
  {
    // 아이템이 든 기록: 장비 탭(Tab — 주무기 칸) → 후보(확인) → 여벌(드문 검, 바탕의 줄 바로 아래) → 낀다.
    const g = await open(browser, ITEMS, 'bag');
    await g.press('Tab'); await g.page.waitForTimeout(500);
    await g.shot('1-equip');
    await g.press('Enter'); await g.page.waitForTimeout(300);
    await g.shot('2-main-pick');
    await g.press('ArrowDown'); await g.page.waitForTimeout(250);
    await g.shot('3-spare-focus');
    await g.press('Enter'); await g.page.waitForTimeout(400);
    await g.shot('4-spare-worn');
    const r = await g.record();
    if (!/gear\.main=1\n/.test(r)) errors.push(`the rare sword was not worn: ${JSON.stringify(r)}`);
    if (!/item\.1=w\.sword\|r\|dmg\.1\+swift\.1\|0\n/.test(r)) errors.push(`the item line changed: ${JSON.stringify(r)}`);
    // 장신구 칸(→)과 그 후보: 단단한 장신구의 옵션 줄.
    await g.press('ArrowRight'); await g.page.waitForTimeout(250);
    await g.shot('5-charm-slot');
    errors.push(...g.bad);
    await g.page.context().close(); // 장면마다 닫는다 — 2배 화면 셋이 함께 살아 있으면 셋째가 뜨다 죽는다
  }
  {
    // 옛 기록(아이템 줄 없음): 장비 탭을 한 번 열고 닫으면(저장) 가진 바탕마다 보통 아이템이 생긴다.
    const g = await open(browser, BASE, 'old');
    await g.press('Tab'); await g.page.waitForTimeout(500);
    await g.press('Enter'); await g.page.waitForTimeout(300);
    await g.shot('1-main-pick');
    await g.press('ArrowDown'); await g.press('Enter'); await g.page.waitForTimeout(400); // 망치를 든다 → 저장
    const r = await g.record();
    for (const want of ['item.0=w.sword|p||0', 'w.hammer|p||0', 'c.ear|p||0', 'gear.main=']) {
      if (!r.includes(want)) errors.push(`old record: missing ${want}: ${JSON.stringify(r)}`);
    }
    errors.push(...g.bad);
    await g.page.context().close(); // 장면마다 닫는다 — 2배 화면 셋이 함께 살아 있으면 셋째가 뜨다 죽는다
  }
  {
    // 화로의 가게(쉬기 F → 가게 탭): 줄의 끝에 여벌을 잔불로 바꾸는 줄 — 묻는 창(커서는 취소)에서 바꾸면 가방에서 빠지고 잔불이 는다.
    const g = await open(browser, ITEMS, 'salvage');
    await g.press('KeyF'); await g.page.waitForTimeout(700);
    await g.press('KeyE', 5); await g.page.waitForTimeout(300);
    await g.press('ArrowUp'); await g.page.waitForTimeout(250); // 목록은 끝에서 돈다 — 위로 한 번 = 마지막 줄(거두기)
    await g.shot('1-shop-salvage-row');
    await g.press('Enter'); await g.page.waitForTimeout(300);
    await g.shot('2-ask');
    await g.press('ArrowLeft'); await g.press('Enter'); await g.page.waitForTimeout(400);
    await g.shot('3-salvaged');
    const r = await g.record();
    if (/item\.1=/.test(r)) errors.push(`the spare was not salvaged: ${JSON.stringify(r)}`);
    if (!/embers=328\n/.test(r)) errors.push(`embers after salvage: ${JSON.stringify(r)}`);
    if (!r.includes('item.0=w.sword|p||0')) errors.push(`the worn sword must stay: ${JSON.stringify(r)}`);
    errors.push(...g.bad);
    await g.page.context().close(); // 장면마다 닫는다 — 2배 화면 셋이 함께 살아 있으면 셋째가 뜨다 죽는다
  }
  await browser.close();
  server.close();
  for (const n of fs.readdirSync(OUT).filter((n) => /^items-/.test(n))) console.log(`${n}\t${fs.statSync(path.join(OUT, n)).size} bytes`);
  if (errors.length) { console.error('errors:\n' + errors.join('\n')); process.exit(1); }
  console.log('통과');
})().catch((e) => { console.error(e); process.exit(1); });
