// 도감 그림: 크게 보기·끌어 돌리기 확인(임시).
const { chromium } = require('playwright');
const http = require('http'), fs = require('fs'), path = require('path');
const sceneKit = require('./scene');
const [DIST, OUT] = [path.resolve(process.argv[2]), path.resolve(process.argv[3])];
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm', '.webmanifest': 'application/manifest+json', '.png': 'image/png' };
const SEED = 'v=1\nembers=700\nclears=1\nrun.rest=0\nrun.gate=1\nlv.life=1\ncore.warden=1\nmigrated.v04=1\nwoke=1\nhearth.lit=1\ncodex=husk,thorn,shell\ncharm.owned=shell\n';
const server = http.createServer((req, res) => { const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0])); if (!fs.existsSync(f)) { res.writeHead(404); return res.end(); } res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' }); fs.createReadStream(f).pipe(res); });
(async () => {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(8193, r));
  const browser = await chromium.launch();
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const page = await ctx.newPage();
  await page.addInitScript((s) => { if (!localStorage.getItem('knell.save.0')) { localStorage.setItem('knell.save.0', s); localStorage.setItem('knell.slot', '0'); } }, SEED);
  await sceneKit.unhurt(page);
  const { until, press } = sceneKit(page);
  const shot = (n) => page.screenshot({ path: path.join(OUT, n + '.png') });
  await page.goto('http://localhost:8193/');
  await until('Title', '타이틀', 90000);
  await press('Enter'); await until('Slots', '슬롯'); await press('Enter'); await until('Play', '판');
  await page.waitForTimeout(1500);
  await press('Escape'); await until(/host: Pause/, '멈춤');
  for (let i = 0; i < 8; i++) { const sc = await page.evaluate(() => window.knellScene); if (/^Codex/.test(sc)) break; await press('KeyE'); await page.waitForTimeout(250); }
  await until('Codex', '도감');
  await page.waitForTimeout(600);
  await shot('c1-codex');
  await press('Enter'); await page.waitForTimeout(800);
  await shot('c2-zoom');
  // 끌기: 화면 가운데에서 가로로.
  await page.mouse.move(640, 360); await page.mouse.down();
  for (let i = 1; i <= 10; i++) { await page.mouse.move(640 + i * 25, 360); await page.waitForTimeout(40); }
  await page.mouse.up(); await page.waitForTimeout(600);
  await shot('c3-zoom-turned');
  await press('Escape'); await page.waitForTimeout(600);
  await shot('c4-back');
  console.log('scene', await page.evaluate(() => window.knellScene));
  await browser.close(); server.close();
})().catch((e) => { console.error(e); process.exit(1); });
