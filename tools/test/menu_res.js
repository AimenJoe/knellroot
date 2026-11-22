// 메뉴 그림의 해상도(0250): 폰(844×390, DPR 3)·PC(1920×1080)에서 성장(화로)·장비·가방·기록(도감) 탭을 찍는다 — 인형·아이템·도감 생물이 도트로
// 뭉개지지 않는지 눈으로 본다. 연구소 `render.menu fine`을 끈 판(옛 꼴)과 견주려면 옛 빌드 dist로 돌린다.
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/menu_res.js dist shots [태그]
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');
const sceneKit = require('./scene');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const TAG = process.argv[4] || 'now';
const PORT = 8151;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm', '.webmanifest': 'application/manifest+json' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const KEY = 'knell.save.0';
const SEED = 'v=1\nembers=300\nclears=1\nrun.rest=0\nrun.gate=1\nweapon=sword\nmigrated.v04=1\nweapons=sword,hammer\ncharm.owned=ear\ncharm.worn=ear,,\n'
  + 'item.0=w.sword|p||0\nitem.1=w.sword|r|dmg.1+swift.1|0\nitem.2=w.hammer|s|heavy.1|0\nitem.3=c.ear|s|hp.1|0\nitem.next=4\ngear.main=0\n'
  + 'codex=husk,thorn\n';

const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});

async function run(browser, name, opts) {
  const ctx = await browser.newContext(opts);
  const page = await ctx.newPage();
  const errors = [];
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(String(e)); });
  const { press, until, flip } = sceneKit(page, 90);
  await sceneKit.unhurt(page);
  await page.goto(`http://localhost:${PORT}/manifest.webmanifest`);
  await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, SEED]);
  await page.goto(`http://localhost:${PORT}/`);
  await until('Title', '타이틀', 60000);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  await press('Enter'); await until('Slots', '이어하기 → 슬롯 창');
  await press('Enter'); await until('Play', '슬롯 → 판'); await page.waitForTimeout(1500);
  const shot = (n) => page.screenshot({ path: path.join(OUT, `menu-${TAG}-${name}-${n}.png`) });
  // 상태(성장) 탭은 0261부터 화로에만 — 쉬어서 찍고(인형), 닫은 뒤 멈춤(장비 탭에서 열린다)의 장비 · 가방 · 기록.
  await press('KeyF'); await until('Status { host: Hearth', '화로 = 성장 탭'); await page.waitForTimeout(400);
  await shot('1-status');
  await press('Escape'); await until('Play', '화로 창 닫기');
  await press('Escape'); await until('Equip { host: Pause', '멈춤 = 장비 탭'); await page.waitForTimeout(400);
  await shot('2-equip');
  await flip('KeyE', '가방'); await page.waitForTimeout(400);
  await shot('3-bag');
  await flip('KeyE', '기술'); await flip('KeyE', '기록'); await until('Codex', '기록 탭');
  // 기록 탭은 여정 묶음에서 열린다(0265) — → 한 번이 적(도감 생물 그림).
  await press('ArrowRight'); await until('Codex { host: Pause, tab: 1', '기록 탭의 적 묶음'); await page.waitForTimeout(400);
  await shot('4-codex');
  await ctx.close();
  return errors;
}

(async () => {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(PORT, r));
  const browser = await chromium.launch();
  const errors = [
    ...await run(browser, 'phone', { viewport: { width: 844, height: 390 }, deviceScaleFactor: 3 }),
    ...await run(browser, 'pc', { viewport: { width: 1920, height: 1080 } }),
  ];
  await browser.close();
  server.close();
  if (errors.length) { console.error(errors.join('\n')); process.exit(1); }
  console.log('ok');
})();
