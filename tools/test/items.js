// 아이템 시험(0223 V4): 등급·옵션이 든 아이템이 있는 기록으로 — 장비 탭의 주무기 후보에 여벌이 서고(바탕의 줄 아래), 여벌을 고르면
//   옵션 줄과 바뀌는 능력치가 보이고, 끼면 기록의 낀 자리(gear.main)가 그 아이템을 가리킨다. 옛 기록(아이템 줄 없음)은 보통 것으로 옮겨진다.
//   가방 탭(0226): 지키기·내려놓기·줍기, 가득이면 못 줍는다, 화로에서 잔불로·여벌 정리(0263 — 화로 창의 가방 탭에서 잔불로·여벌 정리가
//   켜진다: 화로 창의 Select(Tab) = 그 창의 장비 탭 → E = 가방, 또는 성장 탭에서 E 둘).
//   화면마다 스크린숏, 개발판 F4 넘침 검사(`[ui-check]` 줄이 있으면 실패).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/items.js dist shots
const fs = require('fs');
const path = require('path');
const sceneKit = require('./scene');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8143;
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const KEY = 'knell.save.0';
const BASE = 'v=1\nembers=300\nclears=1\nrun.rest=0\nrun.gate=1\nweapon=sword\nmigrated.v04=1\nweapons=sword,hammer\ncharm.owned=ear\ncharm.worn=ear,,\n';
const ITEMS = BASE + 'item.0=w.sword|p||0\nitem.1=w.sword|r|dmg.1+swift.1|0\nitem.2=w.hammer|s|heavy.1|0\nitem.3=c.ear|s|hp.1|0\nitem.next=4\ngear.main=0\n';

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
  // 장면이 바뀌는 누름 뒤에는 장면을 기다린다(tools/test/scene.js).
  const { press, until, flip, openTab } = sceneKit(page, 90);
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
  return { page, shot, press, until, flip, openTab, bad, record: () => page.evaluate((k) => localStorage.getItem(k), KEY) };
}

// 껐다 켠다(같은 기록으로 이어하기).
async function again(g) {
  await g.page.reload();
  await g.page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 60000 });
  await g.page.waitForTimeout(600);
  await g.page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  await g.until('Title', '다시 켠 타이틀');
  await g.press('F4');
  await g.press('Enter'); await g.until('Slots', '이어하기 → 슬롯 창');
  await g.press('Enter'); await g.until('Play', '슬롯 → 판'); await g.page.waitForTimeout(1500);
}

(async () => {
  fs.mkdirSync(OUT, { recursive: true });
  const server = await sceneKit.serve(DIST, { port: PORT });
  const browser = await sceneKit.launch();
  const errors = [];
  {
    // 아이템이 든 기록: 장비 탭(Tab — 주무기 칸) → 후보(확인) → 여벌(드문 검, 바탕의 줄 바로 아래) → 낀다.
    const g = await open(browser, ITEMS, 'bag');
    await g.press('Tab'); await g.until('Equip {', 'Tab = 장비 탭');
    await g.shot('1-equip');
    await g.press('Enter'); await g.until('EquipPick', '주무기 칸 → 후보');
    await g.shot('2-main-pick');
    await g.press('ArrowDown'); await g.page.waitForTimeout(250);
    await g.shot('3-spare-focus');
    await g.press('Enter'); await g.until('Equip {', '여벌을 낀다 → 장비 탭'); await g.page.waitForTimeout(200);
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
    await g.press('Tab'); await g.until('Equip {', 'Tab = 장비 탭');
    await g.press('Enter'); await g.until('EquipPick', '주무기 칸 → 후보');
    await g.shot('1-main-pick');
    await g.press('ArrowDown'); await g.press('Enter'); await g.until('Equip {', '망치를 든다 → 장비 탭'); await g.page.waitForTimeout(300); // 망치를 든다 → 저장
    const r = await g.record();
    for (const want of ['item.0=w.sword|p||0', 'w.hammer|p||0', 'c.ear|p||0', 'gear.main=']) {
      if (!r.includes(want)) errors.push(`old record: missing ${want}: ${JSON.stringify(r)}`);
    }
    errors.push(...g.bad);
    await g.page.context().close(); // 장면마다 닫는다 — 2배 화면 셋이 함께 살아 있으면 셋째가 뜨다 죽는다
  }
  {
    // 가방 탭(0226 — 멈춤 Esc(새 판은 장비 탭, 0261) → E 한 번): 칸의 차례는 저절로(무기 → 방패 → 장신구, 값진 것 먼저) — 첫 칸 = 드문 검(여벌).
    // 지킨다 → 기록에 item.lock, 풀고 → 내려놓는다 → 가방에서 빠지고 발치에 떨어진다 → 닫고 주우면(점프 버튼) 다시 든다.
    const g = await open(browser, ITEMS, 'tab');
    await g.press('Escape'); await g.until('Equip { host: Pause', '멈춤 = 장비 탭');
    await g.openTab('Bag { host: Pause', '가방 탭');
    await g.shot('1-bag');
    await g.press('ArrowRight'); await g.page.waitForTimeout(200);
    await g.shot('2-worn-sword');
    await g.press('ArrowRight', 2); await g.page.waitForTimeout(200);
    await g.shot('3-charm');
    await g.press('ArrowDown', 2); await g.page.waitForTimeout(200);
    await g.shot('4-empty-cell');
    await g.press('ArrowUp', 2); await g.press('ArrowLeft', 3); await g.page.waitForTimeout(200);
    await g.press('Enter'); await g.page.waitForTimeout(300);
    await g.shot('5-acts');
    await g.press('ArrowDown'); await g.press('Enter'); await g.page.waitForTimeout(300); // 지킨다
    await g.shot('6-kept');
    let r = await g.record();
    if (!/item\.lock=1\n/.test(r)) errors.push(`the kept mark was not saved: ${JSON.stringify(r)}`);
    await g.press('Enter'); await g.page.waitForTimeout(300);
    await g.shot('7-acts-kept'); // 지키는 것은 내려놓기·잔불로가 없다
    await g.press('ArrowDown'); await g.press('Enter'); await g.page.waitForTimeout(300); // 지키기를 푼다
    await g.press('Enter'); await g.page.waitForTimeout(300);
    await g.press('ArrowDown', 3); await g.page.waitForTimeout(200);
    await g.shot('8-away'); // 멈춤에서는 "잔불로 — 쉼터 화로에서"(흐린 줄)
    await g.press('ArrowUp'); await g.press('Enter'); await g.page.waitForTimeout(300); // 내려놓는다
    await g.shot('9-put-down');
    r = await g.record();
    if (/item\.1=/.test(r)) errors.push(`the spare was not put down: ${JSON.stringify(r)}`);
    if (/item\.lock=/.test(r)) errors.push(`the kept mark outlived the item: ${JSON.stringify(r)}`);
    // 바닥에 둔 것은 기록에 남는다(지역 0 · 자리 · 아이템) — 게임을 껐다 켜도 그 자리에 있다.
    if (!/ground\.0=0\|-?\d+\|-?\d+\|-?\d+\|w\.sword\|r\|dmg\.1\+swift\.1\|0\n/.test(r)) errors.push(`the item on the ground was not saved: ${JSON.stringify(r)}`);
    await again(g);
    await g.shot('10-on-the-ground');
    await g.press('KeyF'); await g.page.waitForTimeout(700); // 줍기 = 상호작용 버튼(쉬기보다 앞선다)
    await g.shot('11-picked-up');
    await g.press('Escape'); await g.until('Equip { host: Pause', '멈춤(다시 켠 판 = 장비 탭)'); await g.page.waitForTimeout(300); // 멈추면 저장
    r = await g.record();
    if (!/item\.4=w\.sword\|r\|dmg\.1\+swift\.1\|0\n/.test(r)) errors.push(`the sword was not picked back up: ${JSON.stringify(r)}`);
    if (/ground\./.test(r)) errors.push(`a picked-up item still lies on the ground: ${JSON.stringify(r)}`);
    await g.openTab('Bag { host: Pause', '가방 탭(다시)');
    await g.shot('12-bag-again'); // 다시 켠 판의 멈춤은 장비 탭에서 열린다 → E = 가방 — 주운 것은 새 것(민트 점)
    errors.push(...g.bad);
    await g.page.context().close();
  }
  {
    // 가득 찬 가방(0226 — 사장 "가득이면 못 주워야지, 바닥에 떨어져 있어야지"): 서른하나에서 하나를 내려놓으면 서른 = 가득 —
    // 곁에 있어도 줍지 못한다(같은 버튼은 그대로 다른 일 — 화로 곁이면 쉬기). 화로에서 여벌 하나를 잔불로 바꿔 자리를 만들면 줍는다.
    // ("가방이 가득 찼다" 글은 화로에서 떨어진 자리에서 뜬다 — 쉬기가 앞선다. 규칙은 단위 시험이 본다.)
    let full = BASE + 'item.0=w.sword|p||0\nitem.1=w.hammer|p||0\nitem.2=c.ear|p||0\ngear.main=0\nitem.next=31\n';
    for (let i = 3; i <= 30; i++) full += `item.${i}=w.sword|s|dmg.1|0\n`;
    const g = await open(browser, full, 'full');
    const count = (r) => (r.match(/\nitem\.\d+=/g) || []).length;
    const down = async () => { // 가방 탭의 첫 칸 = 단단한 검 여벌: 주무기로 · 지킨다 · 내려놓는다
      await g.press('Enter'); await g.page.waitForTimeout(300);
      await g.press('ArrowDown', 2); await g.press('Enter'); await g.page.waitForTimeout(300);
    };
    await g.press('Escape'); await g.until('Equip { host: Pause', '멈춤 = 장비 탭');
    await g.openTab('Bag { host: Pause', '가방 탭');
    await g.shot('1-over-the-cap'); // 한도를 넘은 가방(31): 줄이 하나 늘고 칸이 작아진다
    await down();
    await g.shot('2-full');
    // 떨어지는 순간의 번쩍임(창을 닫자마자).
    await g.press('Escape'); await g.page.waitForTimeout(140);
    await g.shot('2a-drop-flash');
    // 다시 켠다: 내려놓은 것은 그 자리에 있고(기록), 이제 여느 떨어진 것이다 — 그런데 가방이 가득이라 줍기가 아니다.
    // 화로 곁이라 같은 버튼은 쉬기(쉬기가 못 줍는 것보다 앞선다) → 화로의 창이 열린다.
    await again(g);
    await g.shot('3-cannot-pick-up');
    await g.press('KeyF'); await g.until(/host: Hearth/, '가득일 때 같은 버튼 = 쉬기');
    let r = await g.record();
    if (count(r) !== 30) errors.push(`a full bag took an item (${count(r)}): ${JSON.stringify(r)}`);
    if (!/ground\.0=0\|/.test(r)) errors.push(`the item must stay on the ground: ${JSON.stringify(r)}`);
    // 화로의 가방(0263 — 화로 창에서 Select(Tab) = 그 창의 장비 탭 → E = 가방)에서 여벌 하나를 잔불로
    // (주무기로 · 지킨다 · 내려놓는다 · 잔불로) → 자리가 났다 → 닫고(화로 창 → 싸움) 줍는다.
    await g.press('Tab'); await g.until('Equip { host: Hearth', '화로에서 Select = 화로 창의 장비 탭');
    await g.openTab('Bag { host: Hearth', '화로의 가방 탭');
    await g.press('Enter'); await g.until('BagAct', '칸 → 할 일');
    await g.press('ArrowDown', 3); await g.press('Enter'); await g.until('Ask(Salvage', '잔불로 → 묻는 창');
    await g.press('ArrowLeft'); await g.press('Enter'); await g.until('Bag {', '바꾼다 → 가방 탭'); await g.page.waitForTimeout(200);
    await g.press('Escape'); await g.until('Play', '화로 창 닫기'); await g.page.waitForTimeout(600);
    await g.shot('4-room-made');
    await g.press('KeyF'); await g.page.waitForTimeout(700); // 줍기가 쉬기보다 앞선다
    await g.shot('5-picked-up');
    await g.press('Escape'); await g.page.waitForTimeout(500);
    r = await g.record();
    if (count(r) !== 30) errors.push(`room was made but nothing was picked up (${count(r)}): ${JSON.stringify(r)}`);
    if (!/item\.31=/.test(r)) errors.push(`the picked-up item has no new number: ${JSON.stringify(r)}`);
    if (/ground\./.test(r)) errors.push(`a picked-up item still lies on the ground: ${JSON.stringify(r)}`);
    errors.push(...g.bad);
    await g.page.context().close();
  }
  {
    // 화로의 가방(0263 — 쉬기 F → 성장 탭에서 E 둘 = 장비를 지나 가방 탭): 여벌을 잔불로 — 묻는 창(커서는 취소)에서 바꾸면 가방에서 빠지고 잔불이 는다.
    // 그리고 "여벌 정리": 맨 아랫줄 아래의 단추 — 지키지 않는 여벌을 한꺼번에.
    const g = await open(browser, ITEMS + 'item.4=w.hammer|p||0\nitem.5=c.ear|p||0\nitem.next=6\n', 'hearth');
    await g.press('KeyF'); await g.until('Status { host: Hearth', '화로에서 쉬기 = 성장 탭');
    await g.openTab('Bag { host: Hearth', '성장 → 장비 → 가방 탭(화로)'); await g.page.waitForTimeout(200);
    await g.shot('1-bag');
    await g.press('Enter'); await g.until('BagAct', '칸 → 할 일');
    await g.shot('2-acts');
    await g.press('ArrowDown', 3); await g.press('Enter'); await g.until('Ask(Salvage', '잔불로 → 묻는 창');
    await g.shot('3-ask');
    await g.press('ArrowLeft'); await g.press('Enter'); await g.until('Bag {', '바꾼다 → 가방 탭'); await g.page.waitForTimeout(200);
    await g.shot('4-salvaged');
    let r = await g.record();
    if (/item\.1=/.test(r)) errors.push(`the spare was not salvaged: ${JSON.stringify(r)}`);
    if (!/embers=328\n/.test(r)) errors.push(`embers after salvage: ${JSON.stringify(r)}`);
    if (!r.includes('item.0=w.sword|p||0')) errors.push(`the worn sword must stay: ${JSON.stringify(r)}`);
    await g.press('ArrowDown', 5); await g.page.waitForTimeout(250);
    await g.shot('5-tidy');
    await g.press('Enter'); await g.until('Ask(Tidy', '여벌 정리 → 묻는 창');
    await g.shot('6-tidy-ask');
    await g.press('ArrowLeft'); await g.press('Enter'); await g.until('Bag {', '정리한다 → 가방 탭'); await g.page.waitForTimeout(200);
    await g.shot('7-tidied');
    r = await g.record();
    // 보통 망치·보통 귀가 나가고(4 + 4), 단단한 망치·단단한 귀·낀 검은 남는다.
    if (/item\.4=|item\.5=/.test(r)) errors.push(`the plain spares were not cleared: ${JSON.stringify(r)}`);
    if (!r.includes('item.2=w.hammer|s|heavy.1|0') || !r.includes('item.3=c.ear|s|hp.1|0')) errors.push(`the best of each must stay: ${JSON.stringify(r)}`);
    if (!/embers=336\n/.test(r)) errors.push(`embers after clearing: ${JSON.stringify(r)}`);
    errors.push(...g.bad);
    await g.page.context().close();
  }
  await browser.close();
  server.close();
  for (const n of fs.readdirSync(OUT).filter((n) => /^items-/.test(n))) console.log(`${n}\t${fs.statSync(path.join(OUT, n)).size} bytes`);
  if (errors.length) { console.error('errors:\n' + errors.join('\n')); process.exit(1); }
  console.log('통과');
})().catch((e) => { console.error(e); process.exit(1); });
