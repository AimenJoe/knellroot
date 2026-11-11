// 기록 시험(0064 M2): 헤드리스 Chromium으로
//   1) 기록이 없으면 타이틀 = 시작·설정(새로 하기 없음), 2) 기록을 심어 두면 = 이어하기·설정·새로 하기,
//   3) 이어하기 → 기록의 쉼터(문 앞)에서 선다(스크린숏), 4) 멈춤 → 처음부터 → 기록에서 판 안의 자리가 지워지고 남는 것은 그대로,
//   3a) 쉼터에서 쉬면 화로(0104: 레벨 · 가게 · 떠나기) — 생명을 한 단 올리고(lv.life), 가게에서 장신구를 사면 기록에(0064 M4).
//   5) 기록 슬롯(0066): 이어하기·시작하기 → 슬롯 창, 지우기·덮어쓰기는 두 번 눌러야, 다른 슬롯에서 시작해도 슬롯 1은 그대로, 마지막 슬롯 칸.
//   처음부터·타이틀로·사기는 묻는 창. 그 사이 장비 창에서 장신구를 끼면 기록에 남는지(0064 M3).
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
  await page.evaluate((k) => localStorage.setItem(k, 'v=1\nembers=115\nclears=1\nrun.rest=0\nrun.gate=1\nfuture.key=keep\ncharm.owned=shell,ear\ncharm.worn=shell,,\n'), KEY);
  await page.reload();
  await ready();
  await page.waitForTimeout(600);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  await shot('save-1-title-continue');

  // 3) 이어하기(첫 줄) → 슬롯 창(이어하기, 0066 — 커서는 마지막 슬롯 = 칸이 없으니 슬롯 1) → 이어하기.
  await press('Enter');
  await page.waitForTimeout(300);
  await shot('save-1a-slots-load');
  await press('Enter');
  await page.waitForTimeout(1500);
  await shot('save-2-continued-at-gate');

  // 3a) 쉼터 화로(0104): 곁에서 점프 버튼(F) = 쉬기 → 화로(레벨 · 가게 · 떠나기).
  await press('KeyF'); await page.waitForTimeout(600);
  await shot('save-2a-hearth');
  // 레벨(첫 줄) → 생명(첫 줄, 레벨 0의 값 50 — 0123) → 묻는 창(커서 "올리기") → 올림. 잔불 115 → 65.
  await press('Enter'); await page.waitForTimeout(300);
  await shot('save-2a-level');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('save-2a-level-ask');
  await press('Enter'); await page.waitForTimeout(400);
  await shot('save-2a-level-raised');
  r = await record();
  console.log('레벨을 올린 뒤 기록:', JSON.stringify(r));
  if (!/lv\.life=1\n/.test(r) || !/embers=65\n/.test(r)) fail('raising Life must spend 50 embers and keep lv.life=1');
  // 레벨 창을 닫으면 화로(연 줄) → 셋째 줄 가게(레벨·나무·가게, 0106) → 첫 줄(잔불 주머니 60, 0117) 사기.
  await press('Escape'); await page.waitForTimeout(300);
  await press('ArrowDown', 2); await press('Enter'); await page.waitForTimeout(300);
  await shot('save-2a-shop');
  // 사기 전에 묻는 창(nail 가게 — 커서는 "사기"에서).
  await press('Enter'); await page.waitForTimeout(300);
  await shot('save-2a-shop-ask');
  await press('Enter'); await page.waitForTimeout(400);
  await shot('save-2a-shop-bought');
  r = await record();
  console.log('가게에서 산 뒤 기록:', JSON.stringify(r));
  if (!/charm\.owned=[^\n]*pouch/.test(r) || !/embers=5\n/.test(r)) fail('buying must spend embers and keep the charm');
  // 가게 → 화로 → 싸움.
  await press('Escape'); await page.waitForTimeout(300);
  await press('Escape'); await page.waitForTimeout(300);

  // 3b) 장비(0064 M3): 멈춤 → 둘째 줄(장비) → 장신구 3칸(산 주머니가 2칸에 들어갔다)에 종지기의 귀 → 기록.
  // 십자는 칸 자리대로 커서만(→ = 무기 옆 가운데 장신구 칸, ↓ = 셋째 칸), 빨리 바꾸기는 땅 돌리기 E(2026-11-06).
  await press('Escape'); await page.waitForTimeout(200);
  await press('ArrowDown', 1); await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowRight'); await press('ArrowDown', 1); await press('KeyE'); await page.waitForTimeout(300);
  await shot('save-2b-equip');
  r = await record();
  console.log('장신구를 낀 뒤 기록:', JSON.stringify(r));
  if (!/charm\.worn=shell,pouch,ear/.test(r)) fail('equipping a charm must be saved');
  await press('Escape'); await page.waitForTimeout(200);
  await press('Escape'); await page.waitForTimeout(300);

  // 4) 멈춤 → 처음부터(여덟째 줄: 계속·장비·나무(0106)·도감(0130)·기술(0151)·설정·개발(개발판, 0090)·처음부터).
  await press('Escape');
  await page.waitForTimeout(200);
  await press('ArrowDown', 7);
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

  // 5) 기록 슬롯(0066, KB roguelite-save-slots-and-continue). 타이틀로(멈춤 → 아홉째 — 나무(0106)·도감(0130)·기술(0151)·개발 줄(0090)이 끼어 → 묻는 창 위로 한 번).
  const slotCell = () => page.evaluate(() => localStorage.getItem('knell.slot'));
  const toTitle = async () => {
    await press('Escape'); await page.waitForTimeout(200);
    await press('ArrowDown', 8); await press('Enter'); await page.waitForTimeout(300);
    await press('ArrowUp'); await press('Enter'); await page.waitForTimeout(600);
  };
  await toTitle();
  // 5a) 이어하기 → 지우기(좌우) → 한 번 누르면 "정말 지울까요?"만 — 기록은 그대로. 닫아도 그대로.
  await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowRight'); await press('Enter'); await page.waitForTimeout(300);
  await shot('save-3-erase-armed');
  if ((await record()) === null) fail('one press must not erase');
  await press('Escape'); await page.waitForTimeout(300);
  if ((await record()) === null) fail('closing must not erase');
  // 5b) 시작하기(둘째 줄 — 닫으면 커서는 이어하기에 돌아와 있다) → 새로 시작 창(커서는 첫 빈 슬롯 = 슬롯 2) → 시작.
  await press('ArrowDown'); await press('Enter'); await page.waitForTimeout(300);
  await shot('save-4-slots-new');
  await press('Enter'); await page.waitForTimeout(800);
  if ((await slotCell()) !== '1') fail('starting in slot 2 must remember it as the last slot');
  if ((await record()) === null) fail('starting another slot must keep slot 1');
  // 5c) 다시 타이틀 → 시작하기 → 슬롯 2는 잠깐 했어도 플레이 시간이 쌓여 기록 있음(0081) → 커서는 첫 빈 슬롯(3) → 위로 둘 = 슬롯 1 "덮어쓰기"(두 번) → 슬롯 1이 비워진 채 시작.
  await toTitle();
  await press('ArrowDown'); await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowUp', 2); await press('Enter'); await page.waitForTimeout(300);
  await shot('save-5-overwrite-armed');
  if ((await record()) === null) fail('one press must not overwrite');
  await press('Enter'); await page.waitForTimeout(800);
  r = await record();
  console.log('덮어쓰기 뒤 슬롯 1:', JSON.stringify(r));
  if (r !== null && /embers=5\n/.test(r)) fail('overwrite must clear slot 1');
  if ((await slotCell()) !== '0') fail('overwriting slot 1 must make it the last slot');

  if (errors.length) { console.error('콘솔 에러:\n' + errors.join('\n')); process.exitCode = 1; }
  await browser.close();
  server.close();
  console.log(process.exitCode ? '실패' : '통과');
}
main().catch((e) => { console.error(e); process.exit(1); });
