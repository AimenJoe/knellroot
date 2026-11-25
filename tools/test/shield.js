// 방패 시험(0223): 방패를 가진 기록으로 — 장비 탭의 방패 칸(인형이 방패를 든다 · 후보 목록) · 싸움 화면에서 패링 버튼을 쥔 막는 자세 ·
//   화로의 가게에 방패 줄(없는 기록). 화면마다 스크린숏, 개발판 F4 넘침 검사(`[ui-check]` 줄이 있으면 실패).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/shield.js dist shots
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
const BASE = 'v=1\nembers=300\nclears=1\nrun.rest=0\nrun.gate=1\nweapon=sword\nmigrated.v04=1\n';
// 방패는 보조무기 칸에 든다(0224) — 그 칸에 무기가 있으면 방패는 내려진다(둘 가운데 하나).
const OWNED = BASE + 'shields=pilgrim\nshield.worn=pilgrim\n';

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
  // 장면이 바뀌는 누름 뒤에는 장면을 기다린다(tools/test/scene.js).
  const { press, until, flip } = sceneKit(page, 90);
  await sceneKit.cheap(page);
  const ready = () => page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 60000 });
  await page.goto(`http://localhost:${PORT}/`);
  await ready();
  await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, seed]);
  await page.reload();
  await ready();
  await page.waitForTimeout(600);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  await until('Title', '타이틀');
  await press('F4');
  await press('Enter'); await until('Slots', '이어하기 → 슬롯 창');
  await press('Enter'); await until('Play', '슬롯 → 판'); await page.waitForTimeout(1500);
  return { page, shot, press, until, flip, bad, record: () => page.evaluate((k) => localStorage.getItem(k), KEY) };
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
    await g.press('Tab'); await g.until('Equip {', 'Tab = 장비 탭'); await g.page.waitForTimeout(300);
    await g.shot('4-equip');
    // 보조무기 칸(주무기 아래 — 0224: 방패 칸은 따로 없다) → 목록(비우기 · 맨손 · 가진 무기 · 방패 — 커서는 낀 방패 = 끝 줄) →
    // 아래로 한 번 = 끝에서 돌아 첫 줄 "비우기".
    await g.press('ArrowDown'); await g.page.waitForTimeout(200);
    await g.shot('5-equip-shield-slot');
    await g.press('Enter'); await g.until('EquipPick', '보조무기 칸 → 후보 목록');
    await g.shot('6-equip-shield-pick');
    await g.press('ArrowDown'); await g.press('Enter'); await g.until('Equip {', '비우기 → 장비 탭'); await g.page.waitForTimeout(200);
    const r = await g.record();
    if (/shield\.worn=/.test(r)) errors.push(`the shield was not taken off: ${JSON.stringify(r)}`);
    if (!/shields=pilgrim\n/.test(r)) errors.push(`the owned shield was lost: ${JSON.stringify(r)}`);
    await g.shot('7-equip-no-shield');
    errors.push(...g.bad);
  }
  {
    // 0274 ②: 한손 망치 + 방패(성직자의 메 — 망치도 방패와 든다): 선 채 · 막는 자세 · 3연타(J 셋 — 타마다 한 장) · 장비 탭.
    const HAMMER = BASE.replace('weapon=sword\n', 'weapons=sword,hammer\nweapon=hammer\n') + 'shields=pilgrim\nshield.worn=pilgrim\n';
    const g = await open(browser, HAMMER, 'hammer');
    // 화톳불에서 비켜 서고(왼쪽 아래로), 쿼터뷰를 한 단계 가까이(↑ 하나 — 둘이면 숄더뷰) — 몸과 메가 크게.
    await g.page.keyboard.down('KeyA'); await g.page.keyboard.down('KeyS'); await g.page.waitForTimeout(900);
    await g.page.keyboard.up('KeyA'); await g.page.keyboard.up('KeyS');
    await g.press('ArrowUp');
    await g.page.keyboard.down('KeyS'); await g.page.waitForTimeout(250); await g.page.keyboard.up('KeyS');
    await g.page.waitForTimeout(600);
    await g.shot('1-stand', body);
    await g.shot('1b-quarter');
    await g.page.keyboard.down('ShiftLeft');
    await g.page.waitForTimeout(700);
    await g.shot('2-guard', body);
    await g.page.keyboard.up('ShiftLeft');
    await g.page.waitForTimeout(400);
    // 공격 = 왼쪽 클릭(새 설치의 기본 배치는 키보드 + 마우스, 0058). 화면 가운데 아래(몸 앞)를 누른다.
    for (let i = 0; i < 3; i++) {
      await g.page.mouse.click(640, 420);
      await g.page.waitForTimeout(i === 2 ? 260 : 200);
      await g.shot(`3-swing-${i + 1}`, body);
      await g.page.waitForTimeout(250);
    }
    await g.page.waitForTimeout(600);
    await g.press('Tab'); await g.until('Equip {', 'Tab = 장비 탭'); await g.page.waitForTimeout(300);
    await g.shot('4-equip');
    const r = await g.record();
    if (!/shield\.worn=pilgrim\n/.test(r) || !/weapon=hammer\n/.test(r)) errors.push(`the hammer and the shield were not both held: ${JSON.stringify(r)}`);
    errors.push(...g.bad);
  }
  {
    // 방패가 없는 기록: 쉬기(F) → 가게 탭(성장에서 Q 둘 — 끝에서 돈다, 0263 화로 = 성장 · 장비 · 가방 · 기술 · 가게 · 설정) → 방패 줄 → 산다.
    const g = await open(browser, BASE, 'buy');
    await g.press('KeyF'); await g.until('Status { host: Hearth', '화로에서 쉬기 → 큰 창(성장 탭)');
    for (let i = 0; i < 2; i++) await g.flip('KeyQ', `Q ${i + 1}`);
    await g.until('Shop', '가게 탭'); await g.page.waitForTimeout(200);
    await g.shot('1-shop');
    await g.press('ArrowDown', 5); await g.page.waitForTimeout(200);
    await g.shot('2-shop-shield');
    await g.press('Enter'); await g.until('Ask(Buy', '방패 줄 → 사기 묻는 창');
    await g.shot('3-shop-ask');
    await g.press('Enter'); await g.until('Shop', '산다 → 가게 탭'); await g.page.waitForTimeout(300);
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
