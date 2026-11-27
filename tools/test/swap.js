// 주무기·보조무기 시험(0272 1b): ① 보조무기를 멘 기록으로 장비 탭 — 주무기 슬롯(주특성 먼저) · 보조무기 슬롯(보조특성 먼저) · 후보 목록의
//   견줌(멘 것만으로 먹는 능력치 줄), 한국어·영어 + 개발판 F4 넘침 검사(`[ui-check]` 줄이 있으면 실패). ② 개발 아레나에서 교체 공격을
//   틱마다(F6 — 한 프레임 = 한 틱): 개발 명령 `Arm(…)`·`ArmSub(…)`(개발 창 "주무기 바꾸기"·"보조무기 바꾸기"와 같은 일)로 무기 둘을 정하고, 평타가 닿은 직후
//   바꾸기(X)를 눌러 들어오는 무기의 한 타를 찍는다(검 → 망치 = 대시 공격 · 검 → 활 = 한 발).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/swap.js dist shots [equip,strike,icon,touch]
// 눈으로 볼 것: 보조특성 줄이 밝게·주특성 줄이 흐리게(보조무기 슬롯), 교체 공격의 조여드는 금빛 고리·별·빛줄, HUD 보조무기 그림의 금빛 테.
const fs = require('fs');
const path = require('path');
const sceneKit = require('./scene');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const ONLY = process.argv[4] ? process.argv[4].split(',') : null;
const want = (n) => !ONLY || ONLY.includes(n);
const PORT = 8141;
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/, /register_plugin/];
const KEY = 'knell.save.0';
// 검을 들고 망치를 멘 기록(활·지팡이·쇠뇌도 가졌다 — 후보 목록의 견줌).
const SEED = 'v=1\nembers=300\nclears=1\nrun.rest=0\nrun.gate=1\nweapons=sword,hammer,staff,crossbow,bow\nweapon=sword\nweapon.sub=hammer\nmigrated.v04=1\n';

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
async function equip(browser, lang, errors, seed = SEED, tag = '') {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 }, deviceScaleFactor: 2 });
  const page = await ctx.newPage();
  const bad = [];
  watch(page, bad);
  const shot = (name) => page.screenshot({ path: path.join(OUT, `swap-equip${tag}-${lang}-${name}.png`) });
  const { press, until } = sceneKit(page, 90);
  await sceneKit.cheap(page);
  await page.goto(`http://localhost:${PORT}/`);
  await ready(page);
  await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, seed]);
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

// ② 교체 공격을 틱마다. arm = 들 무기 · sub = 멜 무기(개발 명령 `Arm(…)` · `ArmSub(…)`의 이름 — `weapons::Id`의 Debug).
// 판마다 새 페이지, 컨텍스트는 하나(처음에 고른 "키보드만 배치"가 설정에 남는다).
async function strike(ctx, name, arm, sub, errors, first, touchy = false) {
  const page = await ctx.newPage();
  const bad = [];
  watch(page, bad);
  const { until, dev, keyboardOnly } = sceneKit(page, 70);
  await sceneKit.unhurt(page); await sceneKit.still(page);
  await page.goto(`http://localhost:${PORT}/`);
  await until('Title', '타이틀', 60000); await page.waitForTimeout(300);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  // "키보드만 배치로"(J 공격 · K 패링 · L 락온 · X 무기 바꾸기 — 처음 한 번, 설정은 컨텍스트에 남는다).
  if (first) await keyboardOnly();
  // 무기 둘은 개발 명령의 이름으로(0278 R13 — `Arm(<무기>)` · `ArmSub(<무기>)`), 개발 창을 "보조무기 바꾸기" 줄에 열어 한 장.
  await dev(`Arm(${arm})`, `ArmSub(${sub})`, 'Dev(ArmSub)'); await until(/^Dev \{.* @ArmSub$/, '개발 창 보조무기 바꾸기 줄');
  await page.screenshot({ path: path.join(OUT, `swap-${name}-arm.png`) });
  // 엘리트 아레나(앞의 굳은 잔재 — 평타 둘과 교체 공격을 받아도 서 있다). 쿼터뷰 그대로(락온하지 않는다 —
  // 숄더뷰에서는 적이 방랑자를 가린다) 다가선다. 적은 서 있고(`still`) 방랑자는 맞지 않는다(`unhurt`) — 동작만 본다.
  await dev('Scenario(Elites)'); await until('Sandbox', '엘리트 아레나');
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
  if (touchy) {
    // 0277: 화면의 빈 자리(오른쪽 위)를 톡 — 장치가 터치가 되어 터치 버튼이 뜬다. 교체 공격의 창 동안 바꾸기 버튼이 금빛, 창이 닫히면 흰 테.
    const cdp = await page.context().newCDPSession(page);
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [{ x: 900, y: 150, id: 1 }] });
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
    await burst('t-gold', 6); await burst('t-after', 14);
    await page.close();
    for (const b of bad) errors.push(`[strike ${name}] ${b}`);
    console.log(`[strike ${name}] errors ${bad.length}`);
    return;
  }
  await page.keyboard.press('KeyX'); await burst('c-swap', 22);
  // 잠금 동안의 바꾸기 = 그냥 바꾼다(돌아온다).
  await page.keyboard.press('KeyX'); await burst('d-plain', 6);
  await page.close();
  for (const b of bad) errors.push(`[strike ${name}] ${b}`);
  console.log(`[strike ${name}] errors ${bad.length}`);
}

(async () => {
  fs.mkdirSync(OUT, { recursive: true });
  const server = await sceneKit.serve(DIST, { port: PORT });
  const browser = await sceneKit.launch();
  const errors = [];
  if (want('equip')) for (const lang of (process.env.SWAP_LANGS || 'ko,en').split(',')) await equip(browser, lang, errors);
  // 0277: 쌍단검을 멘 기록의 장비 탭 — 보조무기 슬롯의 아이템 그림이 엇갈린 두 날인가.
  if (want('icon')) await equip(browser, 'ko', errors, SEED.replace('weapons=sword,hammer,staff,crossbow,bow', 'weapons=sword,hammer,staff,crossbow,bow,daggers').replace('weapon.sub=hammer', 'weapon.sub=daggers'), '-daggers');
  // 0277: 터치에서 교체 공격의 창 = 바꾸기 버튼이 금빛(키보드로 치고 화면을 톡 — 장치를 터치로).
  if (want('touch')) {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 }, hasTouch: true });
    await strike(ctx, 'touch', 'Sword', 'Hammer', errors, true, true);
    await ctx.close();
  }
  if (want('strike')) {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    let first = true;
    for (const [name, arm, sub] of [['sword-hammer', 'Sword', 'Hammer'], ['sword-bow', 'Sword', 'Bow']]) {
      await strike(ctx, name, arm, sub, errors, first); first = false;
    }
    await ctx.close();
  }
  await browser.close();
  server.close();
  if (errors.length) { console.error(errors.join('\n')); process.exit(1); }
  console.log('ok');
})();
