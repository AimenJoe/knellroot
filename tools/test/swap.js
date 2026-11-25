// 주무기·보조무기 시험(0272 1b): ① 보조무기를 멘 기록으로 장비 탭 — 주무기 슬롯(주특성 먼저) · 보조무기 슬롯(보조특성 먼저) · 후보 목록의
//   견줌(멘 것만으로 먹는 능력치 줄), 한국어·영어 + 개발판 F4 넘침 검사(`[ui-check]` 줄이 있으면 실패). ② 개발 아레나에서 교체 공격을
//   틱마다(F6 — 한 프레임 = 한 틱): 개발 창 기술 묶음의 "든 무기 바꾸기"·"보조무기 바꾸기"로 무기 둘을 정하고, 평타가 닿은 직후
//   바꾸기(X)를 눌러 들어오는 무기의 한 타를 찍는다(검 → 망치 = 대시 공격 · 검 → 활 = 한 발).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/swap.js dist shots [equip,strike]
// 눈으로 볼 것: 보조특성 줄이 밝게·주특성 줄이 흐리게(보조무기 슬롯), 교체 공격의 조여드는 금빛 고리·별·빛줄, HUD 보조무기 그림의 금빛 테.
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');
const sceneKit = require('./scene');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const ONLY = process.argv[4] ? process.argv[4].split(',') : null;
const want = (n) => !ONLY || ONLY.includes(n);
const PORT = 8141;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/, /register_plugin/];
const KEY = 'knell.save.0';
// 검을 들고 망치를 멘 기록(활·지팡이·쇠뇌도 가졌다 — 후보 목록의 견줌).
const SEED = 'v=1\nembers=300\nclears=1\nrun.rest=0\nrun.gate=1\nweapons=sword,hammer,staff,crossbow,bow\nweapon=sword\nweapon.sub=hammer\nmigrated.v04=1\n';

const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});

const watch = (page, bad) => {
  page.on('console', (m) => {
    const t = m.text();
    if (/\[ui-check\]/.test(t) && !/\[ui-check\] (on|off)$/.test(t)) bad.push(t);
    if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(t))) bad.push(t);
  });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) bad.push(String(e)); });
};
const ready = (page) => page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 60000 });

// ① 장비 탭: 주무기 슬롯 → 보조무기 슬롯 → 후보 목록(망치 → 다른 무기들).
async function equip(browser, lang, errors) {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 }, deviceScaleFactor: 2 });
  const page = await ctx.newPage();
  const bad = [];
  watch(page, bad);
  const shot = (name) => page.screenshot({ path: path.join(OUT, `swap-equip-${lang}-${name}.png`) });
  const { press, until } = sceneKit(page, 90);
  await sceneKit.cheap(page);
  await page.goto(`http://localhost:${PORT}/`);
  await ready(page);
  await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, SEED]);
  await page.reload();
  await ready(page);
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
  await shot('0-hud');
  await press('Tab'); await until('Equip {', 'Tab = 장비 탭'); await page.waitForTimeout(400);
  await shot('1-main-slot');
  await press('ArrowDown'); await page.waitForTimeout(300);
  await shot('2-sub-slot');
  // 후보 목록: 커서는 지금 멘 망치 — 위·아래로 다른 무기(멘 것만으로 먹는 줄이 "값 → 값"으로 견줘진다).
  await press('Enter'); await until('EquipPick', '보조무기 슬롯 → 후보 목록'); await page.waitForTimeout(300);
  await shot('3-sub-pick');
  for (let i = 0; i < 6; i++) { await press('ArrowDown'); await page.waitForTimeout(200); await shot(`4-sub-pick-${i}`); }
  await press('Escape'); await until('Equip {', '후보 → 장비 탭');
  // 주무기 슬롯의 후보: 멘 망치를 고르면 둘이 맞바뀐다(보조특성도 검의 것으로).
  await press('ArrowUp'); await press('Enter'); await until('EquipPick', '주무기 슬롯 → 후보 목록'); await page.waitForTimeout(300);
  for (let i = 0; i < 3; i++) { await shot(`5-main-pick-${i}`); await press('ArrowDown'); await page.waitForTimeout(200); }
  await ctx.close();
  for (const b of bad) errors.push(`[equip ${lang}] ${b}`);
  console.log(`[equip ${lang}] errors/ui-check ${bad.length}`);
}

// ② 교체 공격을 틱마다. arm = "든 무기 바꾸기"를 누를 수(맨손 → 활 → 검 = 둘), sub = "보조무기 바꾸기"를 누를 수(없음 → 검 → 망치 → 지팡이 → 쇠뇌 → 활).
// 판마다 새 페이지, 컨텍스트는 하나(처음에 고른 "키보드만 배치"가 설정에 남는다).
async function strike(ctx, name, arm, sub, errors, first) {
  const page = await ctx.newPage();
  const bad = [];
  watch(page, bad);
  const { press, until } = sceneKit(page, 70);
  await sceneKit.unhurt(page); await sceneKit.still(page);
  await page.goto(`http://localhost:${PORT}/`);
  await until('Title', '타이틀', 60000); await page.waitForTimeout(300);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  if (first) {
    // "키보드만 배치로"(J 공격 · K 패링 · L 락온 · X 무기 바꾸기) — 타이틀의 설정 → 조작 설정(맨 끝 줄, fx.js와 같은 길).
    await press('ArrowDown'); await press('Enter'); await until('Settings { host: Title', '타이틀 → 설정');
    await press('ArrowUp'); await press('Enter'); await until('Controls', '설정 → 조작');
    await press('ArrowUp', 2); await press('Enter'); await page.waitForTimeout(200);
    await press('Escape'); await until('Settings { host: Title', '조작 → 설정');
    await press('Escape'); await until('Title', '설정 → 타이틀'); await press('ArrowUp');
  }
  await press('ArrowUp'); await press('Enter'); await until('Dev { tab: 0', '타이틀 → 개발 창');
  await press('ArrowRight', 2); await until('Dev { tab: 2', '개발 묶음 기술');
  // 기술 묶음의 끝 두 줄: 든 무기 바꾸기(여섯째) · 보조무기 바꾸기(일곱째).
  await press('ArrowDown', 5);
  for (let i = 0; i < arm; i++) { await page.keyboard.press('Enter'); await page.waitForTimeout(150); }
  await press('ArrowDown');
  for (let i = 0; i < sub; i++) { await page.keyboard.press('Enter'); await page.waitForTimeout(150); }
  await page.screenshot({ path: path.join(OUT, `swap-${name}-arm.png`) });
  // 전투 묶음의 넷째 줄 = 엘리트 아레나(앞의 굳은 잔재 — 평타 둘과 교체 공격을 받아도 서 있다). 쿼터뷰 그대로(락온하지 않는다 —
  // 숄더뷰에서는 적이 방랑자를 가린다) 다가선다. 적은 서 있고(`still`) 방랑자는 맞지 않는다(`unhurt`) — 동작만 본다.
  await press('ArrowRight', 2); await until('Dev { tab: 4', '개발 묶음 전투');
  await press('ArrowDown', 3); await press('Enter'); await until('Sandbox', '엘리트 아레나');
  // 락온해서 곧장 다가가 몸이 닿게 서고(적은 서 있다), 락온을 풀어 쿼터뷰로 돌아온다 — 어느 판이든 같은 자리에서 친다.
  await page.waitForTimeout(700);
  await page.keyboard.press('KeyL'); await page.waitForTimeout(500);
  await page.keyboard.down('KeyW'); await page.waitForTimeout(1500); await page.keyboard.up('KeyW');
  await page.keyboard.press('KeyL'); await page.waitForTimeout(900);
  await page.keyboard.press('F6'); await page.waitForTimeout(300);
  // 한 장에 서너 틱이 지난다(찍는 동안에도 프레임이 돈다 — fx.js와 같다): 교체 공격의 창(0.6초 = 36틱)은 여덟 장 남짓이다.
  const burst = async (tag, n) => { for (let i = 0; i < n; i++) await page.screenshot({ path: path.join(OUT, `swap-${name}-${tag}-${String(i).padStart(2, '0')}.jpg`), type: 'jpeg', quality: 92 }); };
  // 평타가 닿은 직후(금빛 테가 켜진 동안) 바꾸기 → 들어오는 한 타 → 그 뒤.
  await page.keyboard.press('KeyJ'); await burst('b-hit', 3);
  await page.keyboard.press('KeyX'); await burst('c-swap', 22);
  // 잠금 동안의 바꾸기 = 그냥 바꾼다(돌아온다).
  await page.keyboard.press('KeyX'); await burst('d-plain', 6);
  await page.close();
  for (const b of bad) errors.push(`[strike ${name}] ${b}`);
  console.log(`[strike ${name}] errors ${bad.length}`);
}

(async () => {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(PORT, r));
  const browser = await chromium.launch();
  const errors = [];
  if (want('equip')) for (const lang of (process.env.SWAP_LANGS || 'ko,en').split(',')) await equip(browser, lang, errors);
  if (want('strike')) {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    let first = true;
    for (const [name, arm, sub] of [['sword-hammer', 2, 2], ['sword-bow', 2, 5]]) {
      await strike(ctx, name, arm, sub, errors, first); first = false;
    }
    await ctx.close();
  }
  await browser.close();
  server.close();
  if (errors.length) { console.error(errors.join('\n')); process.exit(1); }
  console.log('ok');
})();
