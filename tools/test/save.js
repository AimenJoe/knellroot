// 기록 시험(0064 M2): 헤드리스 Chromium으로
//   1) 기록이 없으면 타이틀 = 시작·설정(새로 하기 없음), 2) 기록을 심어 두면 = 이어하기·설정·새로 하기,
//   3) 이어하기 → 기록의 쉼터(문 앞)에서 선다(스크린숏), 4) 멈춤 → 처음부터 → 기록에서 판 안의 자리가 지워지고 남는 것은 그대로,
//   3a) 쉼터에서 쉬면 화로(0104: 레벨 · 가게 · 떠나기) — 생명을 한 단 올리고(lv.life), 가게에서 장신구를 사면 기록에(0064 M4).
//   5) 기록 슬롯(0066): 이어하기·시작하기 → 슬롯 창, 지우기·덮어쓰기는 묻는 창(0192 — 커서는 취소에서), 다른 슬롯에서 시작해도 슬롯 1은 그대로, 마지막 슬롯 칸.
//   처음부터·타이틀로·사기는 묻는 창. 그 사이 장비 창에서 장신구를 끼면 기록에 남는지(0064 M3).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/save.js dist shots
const sceneKit = require('./scene');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8135;
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const KEY = 'knell.save.0';

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  const server = await sceneKit.serve(DIST, { port: PORT });
  const browser = await sceneKit.launch();
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const page = await ctx.newPage();
  await sceneKit.throttle(ctx, page);
  const errors = [];
  page.on('console', (m) => { if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(m.text()))) errors.push(m.text()); });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(String(e)); });
  const fail = (msg) => { console.error('FAIL ' + msg); process.exitCode = 1; };
  const shot = (name) => page.screenshot({ path: path.join(OUT, name + '.png') });
  const { press, until, openTab } = sceneKit(page);
  await sceneKit.cheap(page);
  const ready = () => page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 20000 });
  const record = () => page.evaluate((k) => localStorage.getItem(k), KEY);
  // 시간 대신 장면을 기다린다(2026-11-19, tools/test/scene.js): 5c의 덮어쓰기가 느린 CI에서 네 번 빠졌다.
  // 기록이 바뀌기를 기다린다(쓰기는 장면이 바뀐 뒤 몇 프레임 안에 — 바뀌지 않으면 그대로 돌려주고 검사가 말한다).
  const recordUntil = async (ok, ms = 10000) => {
    try {
      await page.waitForFunction(([k, src]) => new Function('r', 'return ' + src)(localStorage.getItem(k)), [KEY, ok], { timeout: ms, polling: 100 });
    } catch (e) { /* 아래 검사가 실패를 말한다 */ }
    return record();
  };
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
  await press('Enter'); await until('Slots', '3 이어하기 → 슬롯 창');
  await shot('save-1a-slots-load');
  await press('Enter'); await until('Play', '3 슬롯 1 이어하기');
  await page.waitForTimeout(1000);
  await shot('save-2-continued-at-gate');

  // 3a) 쉼터 화로(0104 → 0164 큰 창 → 0261): 곁에서 점프 버튼(F) = 쉬기 → 창이 성장(상태) 탭에서 열린다(화로 탭 성장·장비·가방·기술·가게·설정, 0263).
  await press('KeyF'); await until('Status { host: Hearth', '3a 화로에서 쉬기');
  await page.waitForTimeout(300);
  await shot('save-2a-hearth');
  // 상태 탭의 첫 줄 = 생명(레벨 0의 값 50 — 0123) → 묻는 창(커서 "올리기") → 올림. 잔불 115 → 65.
  await press('Enter'); await until('Ask(Level', '3a 생명 올리기 묻는 창');
  await shot('save-2a-level-ask');
  await press('Enter'); await until('Status { host: Hearth', '3a 올린 뒤 성장 탭');
  await shot('save-2a-level-raised');
  r = await recordUntil("/lv\\.life=1\\n/.test(r || '')");
  console.log('레벨을 올린 뒤 기록:', JSON.stringify(r));
  if (!/lv\.life=1\n/.test(r) || !/embers=65\n/.test(r)) fail('raising Life must spend 50 embers and keep lv.life=1');
  // 가게 탭: 성장에서 Q 둘(설정 → 가게 — 탭 줄은 끝에서 돈다, 0263 화로 = 성장·장비·가방·기술·가게·설정) → 첫 줄(잔불 주머니 60, 0117) 사기.
  await openTab('Shop', '3a 가게 탭', 'KeyQ');
  await shot('save-2a-shop');
  // 사기 전에 묻는 창(nail 가게 — 커서는 "사기"에서).
  await press('Enter'); await until('Ask(Buy', '3a 사기 묻는 창');
  await shot('save-2a-shop-ask');
  await press('Enter'); await until('Shop', '3a 산 뒤 가게 탭');
  await shot('save-2a-shop-bought');
  r = await recordUntil("/pouch/.test(r || '')");
  console.log('가게에서 산 뒤 기록:', JSON.stringify(r));
  if (!/charm\.owned=[^\n]*pouch/.test(r) || !/embers=5\n/.test(r)) fail('buying must spend embers and keep the charm');
  // 창을 닫으면 떠난다(어느 탭에서든 한 번에 — 0164) → 싸움.
  await press('Escape'); await until('Play', '3a 창 닫기');

  // 3b) 장비(0064 M3 → 0164 탭 → 0261): 멈춤(새 판은 장비 탭에서 열린다 — 멈춤에 상태 탭은 없다) → 장신구 칸은 종류마다 하나(0224 — 목걸이 · 반지 · 팔찌): 가진 껍질은 팔찌(셋째 칸),
  // 산 주머니는 목걸이(첫 칸)에 들어갔다. 첫 칸에 종지기의 귀(목걸이)를 끼면 주머니와 바뀐다 → 기록.
  // 십자는 칸 자리대로 커서만(→ = 무기와 같은 높이의 첫 장신구 칸, 0176), 확인 = 그 칸의 후보 목록(빈 칸 · 종지기의 귀 · 잔불 주머니 —
  // 커서는 지금 것 "잔불 주머니"에서: 위로 하나 = 종지기의 귀). 키보드+마우스 배치에는 빨리 바꾸기가 없다(공격 = 클릭).
  await press('Escape'); await until('Equip { host: Pause', '3b 멈춤(장비 탭)');
  await press('ArrowRight'); await press('Enter'); await until('EquipPick', '3b 장신구 후보');
  await shot('save-2b-equip-pick');
  await press('ArrowUp'); await press('Enter'); await until('Equip {', '3b 낀 뒤 장비 탭');
  await shot('save-2b-equip');
  r = await recordUntil("/charm\\.worn=ear,,shell/.test(r || '')");
  console.log('장신구를 낀 뒤 기록:', JSON.stringify(r));
  if (!/charm\.worn=ear,,shell/.test(r)) fail('equipping a charm must be saved in its kind\'s slot (0224): ' + JSON.stringify(r));
  await press('Escape'); await until('Play', '3b 창 닫기');

  // 4) 멈춤 → 처음부터: 창은 마지막에 보던 장비 탭에서 열린다 → Q 하나(장비는 멈춤의 첫 탭 — 끝의 설정으로 돈다, 0261) → 설정 탭의 끝에서 둘째 줄(… 처음부터 · 타이틀로).
  await press('Escape'); await until('Equip { host: Pause', '4 멈춤(마지막에 보던 장비 탭)');
  await press('KeyQ'); await until('Settings { host: Pause', '4 설정 탭');
  await press('ArrowUp', 2);
  await press('Enter'); await until('Ask(Restart', '4 처음부터 묻는 창');
  // 처음부터는 묻는 창(커서는 "취소" — 그냥 확인하면 아무 일 없다), 위로 한 번 = "처음부터".
  await shot('save-3a-restart-ask');
  await press('ArrowUp'); await press('Enter'); await until('Play', '4 처음부터');
  r = await recordUntil("!/run\\.rest=/.test(r || 'run.rest=')");
  console.log('처음부터 뒤 기록:', JSON.stringify(r));
  if (!r || /run\.rest=/.test(r)) fail('restart should drop the run position');
  if (!/embers=5\n/.test(r) || !/clears=1/.test(r)) fail('restart must keep what stays');
  if (!/future\.key=keep/.test(r)) fail('unknown keys must survive a write');

  // 5) 기록 슬롯(0066, KB roguelite-save-slots-and-continue). 타이틀로(멈춤 — 새 판이라 장비 탭(0261) → Q = 설정 탭 → 위로 한 번 = 맨 끝 줄
  // → 묻는 창 위로 한 번).
  const slotCell = () => page.evaluate(() => localStorage.getItem('knell.slot'));
  const toTitle = async () => {
    await press('Escape'); await until('Equip { host: Pause', '타이틀로: 멈춤(새 판 = 장비 탭)');
    await press('KeyQ'); await until('Settings { host: Pause', '타이틀로: 설정 탭');
    await press('ArrowUp'); await press('Enter'); await until('Ask(ToTitle', '타이틀로: 묻는 창');
    await press('ArrowUp'); await press('Enter'); await until('Title', '타이틀로');
  };
  await toTitle();
  // 5a) 이어하기 → 지우기(좌우) → 묻는 창(0192)만 뜬다 — 기록은 그대로. 묻는 창·슬롯 창을 닫아도 그대로.
  await press('Enter'); await until('Slots { load: true', '5a 이어하기 → 슬롯 창');
  await press('ArrowRight'); await press('Enter'); await until('Ask(Erase(0))', '5a 슬롯 1 지우기 묻는 창');
  await shot('save-3-erase-ask');
  if ((await record()) === null) fail('one press must not erase');
  await press('Escape'); await until('Slots', '5a 묻는 창 닫기');
  if ((await record()) === null) fail('closing the ask must not erase');
  await press('Escape'); await until('Title', '5a 슬롯 창 닫기');
  if ((await record()) === null) fail('closing must not erase');
  // 5b) 시작하기(둘째 줄 — 닫으면 커서는 이어하기에 돌아와 있다) → 새로 시작 창(커서는 첫 빈 슬롯 = 슬롯 2) → 시작.
  await press('ArrowDown'); await press('Enter'); await until('Slots { load: false', '5b 시작하기 → 슬롯 창');
  await shot('save-4-slots-new');
  await press('Enter'); await until('Play', '5b 슬롯 2에서 시작');
  try { await page.waitForFunction(() => localStorage.getItem('knell.slot') === '1', null, { timeout: 10000, polling: 100 }); } catch (e) { /* 아래 검사 */ }
  if ((await slotCell()) !== '1') fail('starting in slot 2 must remember it as the last slot');
  if ((await record()) === null) fail('starting another slot must keep slot 1');
  // 빈 슬롯의 첫 판은 깨어나는 장면(0171)으로 열린다 — 그동안 멈춤 키는 건너뛰기다. 한 번 건너뛰고 나서야 멈춤이 열린다.
  {
    const slot2 = () => page.evaluate(() => localStorage.getItem('knell.save.1'));
    if (/woke=1\n/.test((await slot2()) || '')) fail('a new slot must open with the waking scene still ahead: ' + JSON.stringify(await slot2()));
    // 건너뛴 뒤 조금 걷는다(플레이 시간은 장면 동안 쌓이지 않는다 — 5c는 슬롯 2에 시간이 쌓여 "기록 있음"인 것에 기댄다).
    await press('Escape');
    try { await page.waitForFunction(() => /woke=1\n/.test(localStorage.getItem('knell.save.1') || ''), null, { timeout: 10000, polling: 100 }); } catch (e) { /* 아래 검사 */ }
    if (!/woke=1\n/.test((await slot2()) || '')) fail('skipping the waking scene must be recorded in slot 2: ' + JSON.stringify(await slot2()));
    // 5c는 슬롯 2가 "기록 있음"이어야 ↑↑가 슬롯 1에 닿는다 — 슬롯 창은 플레이 시간 1초 이상(play > 0, save::has_any)만 기록으로 본다.
    // 예전엔 벽시계로 2~3초 기다렸는데, 느린 CI에서는 그동안 게임 시간이 1초를 못 채워 슬롯 2가 빈 슬롯이 되고, 커서가 슬롯 2에서 시작해
    // ↑↑가 빈 슬롯 3에 닿아 덮어쓰기 대신 새 판이 열렸다(run 568·582 — 슬롯 1은 그대로·마지막 슬롯 ≠ 0). 기록에 시간이 적힐 때까지 기다린다.
    try {
      await page.waitForFunction(() => /(^|\n)play=[1-9]/.test(localStorage.getItem('knell.save.1') || ''), null, { timeout: 60000, polling: 200 });
    } catch (e) {
      throw new Error('5b 슬롯 2에 플레이 시간이 쌓이지 않았다: ' + JSON.stringify(await slot2()));
    }
  }
  // 5c) 다시 타이틀 → 시작하기 → 슬롯 2는 잠깐 했어도 플레이 시간이 쌓여 기록 있음(0081) → 커서는 첫 빈 슬롯(3) → 위로 둘 = 슬롯 1 "덮어쓰기"(두 번) → 슬롯 1이 비워진 채 시작.
  await toTitle();
  await press('ArrowDown'); await press('Enter'); await until('Slots { load: false', '5c 시작하기 → 슬롯 창');
  await press('ArrowUp', 2); await press('Enter'); await until('Ask(Overwrite(0))', '5c 슬롯 1 덮어쓰기 묻는 창');
  await shot('save-5-overwrite-ask');
  if ((await record()) === null) fail('one press must not overwrite');
  // 묻는 창의 커서는 "취소"에서 — 그대로 확인하면 아무것도 지우지 않고 슬롯 창으로.
  await press('Enter'); await until('Slots', '5c 취소 → 슬롯 창');
  if ((await record()) === null || !/embers=5\n/.test(await record())) fail('cancel must keep slot 1');
  // 다시 묻고 왼쪽(덮어쓰기) → 확인 → 새 판.
  await press('Enter'); await until('Ask(Overwrite(0))', '5c 다시 묻는 창');
  await press('ArrowLeft'); await press('Enter'); await until('Play', '5c 덮어쓰고 시작');
  r = await recordUntil("r === null || !/embers=5\\n/.test(r)");
  try { await page.waitForFunction(() => localStorage.getItem('knell.slot') === '0', null, { timeout: 10000, polling: 100 }); } catch (e) { /* 아래 검사 */ }
  console.log('덮어쓰기 뒤 슬롯 1:', JSON.stringify(r));
  if (r !== null && /embers=5\n/.test(r)) fail('overwrite must clear slot 1');
  if (r !== null && !/woke=0\n/.test(r)) fail('an overwritten slot is a new record — the waking scene is ahead again');
  if ((await slotCell()) !== '0') fail('overwriting slot 1 must make it the last slot');

  if (errors.length) { console.error('콘솔 에러:\n' + errors.join('\n')); process.exitCode = 1; }
  await browser.close();
  server.close();
  console.log(process.exitCode ? '실패' : '통과');
}
main().catch((e) => { console.error(e); process.exit(1); });
