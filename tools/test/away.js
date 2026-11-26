// 앱이 내려감 시험(0257, nail 0298 `bg:` 꼴): 싸움 중에 탭을 숨기면(visibilityState를 덮고 사건 — 헤드리스는 진짜로 숨지 않는다)
//   1) 소리 컨텍스트가 멈추고(web/index.html — `window.knellAudioContexts`), 2) 게임은 멈춤 창을 연다(src/app/away.rs → `Equip { host: Pause` — 0261부터 멈춤의 새 판 첫 탭은 장비),
//   3) 다시 보이면 소리가 돌아오고 멈춤 창은 그대로(손이 오기 전에 싸움이 흐르지 않는다).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/away.js dist shots
const sceneKit = require('./scene');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8143;
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  const server = await sceneKit.serve(DIST, { port: PORT });
  const browser = await sceneKit.launch(['--autoplay-policy=no-user-gesture-required']);
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  const errors = [];
  page.on('console', (m) => { if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(m.text()))) errors.push(m.text()); });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(String(e)); });
  const fail = (msg) => { console.error('FAIL ' + msg); process.exitCode = 1; };
  const { until, press } = sceneKit(page, 60);
  await sceneKit.cheap(page);
  await page.goto(`http://localhost:${PORT}/`);
  await until('Title', '타이틀', 60000);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  // 새 판(빈 슬롯) → 깨어나는 장면은 멈춤 키로 건너뛴다.
  await press('Enter'); await until('Slots { load: false', '시작하기 → 슬롯 창');
  await press('Enter'); await until('Play', '새 판');
  await page.waitForTimeout(800); await press('Escape'); await page.waitForTimeout(1500);
  const scene = () => page.evaluate(() => window.knellScene);
  if (!/^Play/.test(await scene())) { await press('Escape'); await until('Play', '멈춤 닫기'); }
  const vis = (st) => page.evaluate((v) => {
    Object.defineProperty(document, 'visibilityState', { value: v, configurable: true });
    Object.defineProperty(document, 'hidden', { value: v === 'hidden', configurable: true });
    document.dispatchEvent(new Event('visibilitychange'));
  }, st);
  const states = () => page.evaluate(() => (window.knellAudioContexts || []).map((c) => c.state));
  await page.waitForTimeout(500);
  await vis('hidden'); await page.waitForTimeout(500);
  const whileHidden = await states();
  await vis('visible');
  await until('Equip { host: Pause', '숨었다 돌아오면 멈춤 창');
  await page.waitForTimeout(400);
  const afterBack = await states();
  await page.screenshot({ path: path.join(OUT, 'away-pause.png') });
  if (!(whileHidden.length > 0 && whileHidden.every((x) => x === 'suspended'))) fail(`숨으면 소리 컨텍스트가 멈춘다 ${JSON.stringify(whileHidden)}`);
  if (!(afterBack.length > 0 && afterBack.some((x) => x === 'running'))) fail(`보이면 소리가 돌아온다 ${JSON.stringify(afterBack)}`);
  if (errors.length) fail(`콘솔 에러 ${JSON.stringify(errors)}`);
  console.log(`away: hidden ${JSON.stringify(whileHidden)} back ${JSON.stringify(afterBack)} scene ${await scene()}`);
  await browser.close(); server.close();
}
main().catch((e) => { console.error(e); process.exit(1); });
