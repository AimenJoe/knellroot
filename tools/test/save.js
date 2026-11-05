// 기록 시험(0064 M2): 헤드리스 Chromium으로
//   1) 기록이 없으면 타이틀 = 시작·설정(새로 하기 없음), 2) 기록을 심어 두면 = 이어하기·설정·새로 하기,
//   3) 이어하기 → 기록의 쉼터(문 앞)에서 선다(스크린숏), 4) 멈춤 → 처음부터 → 기록에서 판 안의 자리가 지워지고 남는 것은 그대로,
//   3a) 쉼터에서 쉬면 화로 가게 — 잔불로 장신구를 사면 기록에(0064 M4).
//   5) 새로 하기(타이틀 맨 끝 — 이 시험은 DEV 줄이 있는 개발판)는 묻는 창(커서는 취소, KB)에서 "기록 지우기"를 골라야 지운다. 처음부터·타이틀로·사기도 묻는다. 그 사이 장비 창에서 장신구를 끼면 기록에 남는지(0064 M3).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/save.js dist shots
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8135;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const KEY = 'knell.save.0';

const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
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
  const shot = (name) => page.screenshot({ path: path.join(OUT, name + '.png') });
  const press = async (key, n = 1) => { for (let i = 0; i < n; i++) { await page.keyboard.press(key); await page.waitForTimeout(80); } };
  const ready = () => page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 20000 });
  const record = () => page.evaluate((k) => localStorage.getItem(k), KEY);
  let r;

  // 1) 기록 없음.
  await page.goto(`http://localhost:${PORT}/`);
  await ready();
  await page.waitForTimeout(600);
  await shot('save-0-title-fresh');
  if ((await record()) !== null) fail('a fresh browser already has a record');

  // 2) 문 앞 쉼터(번호 0 — 지도 위에서부터 첫 R)에서 쉰 기록 + 잔불·끝낸 판 + 나중 칸(모르는 키).
  await page.evaluate((k) => localStorage.setItem(k, 'v=1\nembers=30\nclears=1\nrun.rest=0\nrun.gate=1\nfuture.key=keep\ncharm.owned=shell,ear\ncharm.worn=shell,,\n'), KEY);
  await page.reload();
  await ready();
  await page.waitForTimeout(600);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  await shot('save-1-title-continue');

  // 3) 이어하기(첫 줄).
  await press('Enter');
  await page.waitForTimeout(1500);
  await shot('save-2-continued-at-gate');

  // 3a) 쉼터 화로(0064 M4): 곁에서 점프 버튼(F) = 쉬기 → 가게 → 첫 줄(잔불 주머니 25) 사기 → 떠나기.
  await press('KeyF'); await page.waitForTimeout(600);
  await shot('save-2a-shop');
  // 사기 전에 묻는 창(nail 가게 — 커서는 "사기"에서).
  await press('Enter'); await page.waitForTimeout(300);
  await shot('save-2a-shop-ask');
  await press('Enter'); await page.waitForTimeout(400);
  await shot('save-2a-shop-bought');
  r = await record();
  console.log('가게에서 산 뒤 기록:', JSON.stringify(r));
  if (!/charm\.owned=[^\n]*pouch/.test(r) || !/embers=5\n/.test(r)) fail('buying must spend embers and keep the charm');
  await press('Escape'); await page.waitForTimeout(300);

  // 3b) 장비(0064 M3): 멈춤 → 둘째 줄(장비) → 장신구 3칸(산 주머니가 2칸에 들어갔다)에 종지기의 귀 → 기록.
  await press('Escape'); await page.waitForTimeout(200);
  await press('ArrowDown', 1); await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowDown', 3); await press('ArrowRight'); await page.waitForTimeout(300);
  await shot('save-2b-equip');
  r = await record();
  console.log('장신구를 낀 뒤 기록:', JSON.stringify(r));
  if (!/charm\.worn=shell,pouch,ear/.test(r)) fail('equipping a charm must be saved');
  await press('Escape'); await page.waitForTimeout(200);
  await press('Escape'); await page.waitForTimeout(300);

  // 4) 멈춤 → 처음부터(넷째 줄: 계속·무기·설정·처음부터).
  await press('Escape');
  await page.waitForTimeout(200);
  await press('ArrowDown', 3);
  await press('Enter');
  await page.waitForTimeout(300);
  // 처음부터는 묻는 창(커서는 "취소" — 그냥 확인하면 아무 일 없다), 위로 한 번 = "처음부터".
  await shot('save-3a-restart-ask');
  await press('ArrowUp'); await press('Enter');
  await page.waitForTimeout(800);
  r = await record();
  console.log('처음부터 뒤 기록:', JSON.stringify(r));
  if (!r || /run\.rest=/.test(r)) fail('restart should drop the run position');
  if (!/embers=5\n/.test(r) || !/clears=1/.test(r)) fail('restart must keep what stays');
  if (!/future\.key=keep/.test(r)) fail('unknown keys must survive a write');

  // 5) 타이틀로 → 새로 하기(셋째 줄) 한 번 = 확인 글만, 두 번 = 지움.
  await press('Escape');
  await page.waitForTimeout(200);
  await press('ArrowDown', 4);
  await press('Enter');
  await page.waitForTimeout(300);
  await press('ArrowUp'); await press('Enter');
  await page.waitForTimeout(600);
  // 타이틀: 이어하기·설정·DEV(개발판)·새로 하기(맨 끝).
  await press('ArrowDown', 3);
  await press('Enter');
  await page.waitForTimeout(300);
  await shot('save-3-new-game-ask');
  if ((await record()) === null) fail('opening the question must not erase');
  // 커서는 "취소"에서 — 그대로 확인하면 지우지 않고 타이틀로.
  await press('Enter'); await page.waitForTimeout(400);
  if ((await record()) === null) fail('cancel must not erase');
  // 다시: 새로 하기(맨 끝) → 위로 한 번 = "기록 지우기".
  await press('ArrowDown', 3); await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowUp'); await press('Enter');
  await page.waitForTimeout(800);
  r = await record();
  console.log('새로 하기 뒤 기록:', JSON.stringify(r));
  if (r !== null && /embers=5\n/.test(r)) fail('new game must erase the record');

  if (errors.length) { console.error('콘솔 에러:\n' + errors.join('\n')); process.exitCode = 1; }
  await browser.close();
  server.close();
  console.log(process.exitCode ? '실패' : '통과');
}
main().catch((e) => { console.error(e); process.exit(1); });
