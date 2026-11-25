// 이펙트를 틱마다 찍는다(0141): 헤드리스 Chromium으로 개발 시나리오를 띄우고, 개발 키 F6(한 프레임 = 한 틱)을 켠 뒤
// 동작마다 연달아 스크린숏 — 0.1초짜리 번쩍임·먼지를 실제 시간으로는 못 잡는다(헤드리스는 초당 8프레임 남짓).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/fx.js dist shots-fx [duel,horde,boss,stage,skills] [스킬 이름들]
// skills(0158 G2): 개발 창 기술 탭 "기술 다 올리기"(끝 단 · 고르기 ㄱ, 기록에 안 쓴다) → "기술 끼우기"에서 칸·큰 기술을 고르고 →
//   전투 탭 "skills" 판에서 Q(칸 1)·E(칸 2)·Q+E(큰 기술)를 틱마다 찍는다 — 스킬·큰 기술마다 새 판(sk-<이름>-<n>.png).
// 눈으로 볼 것: 맞힌 자리의 별·불똥, 휘두름의 초승달, 쓰러진 적의 재, 뛰기·회피·적 내리침의 먼지, 마법 탄의 마름모, 화로 불티.
// 메뉴 길은 shot.js와 같다(처음에 "키보드만 배치로" — J 공격·K 패링·L 락온·Space 회피·F 점프·U 마법). 콘솔 에러가 있으면 종료 코드 1.
const { chromium } = require('playwright'); const http = require('http'); const fs = require('fs'); const path = require('path');
const sceneKit = require('./scene');
const DIST = path.resolve(process.argv[2] || 'dist'); const OUT = path.resolve(process.argv[3] || 'shots-fx'); const PORT = 8141;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm' };
const server = http.createServer((req, res) => { const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' }); fs.createReadStream(f).pipe(res); });
(async () => {
  fs.mkdirSync(OUT, { recursive: true }); await new Promise(r => server.listen(PORT, r));
  const b = await chromium.launch(); const ctx = await b.newContext({ viewport: { width: 1280, height: 720 } });
  const errors = []; let first = true;
  const open = async (tab, row) => {
    const p = await ctx.newPage();
    p.on('console', m => { if (m.type() === 'error' && !/register_plugin/.test(m.text())) errors.push(m.text()); });
    p.on('pageerror', e => { if (!/register_plugin/.test(String(e))) errors.push(String(e)); });
    // 장면이 바뀌는 누름 뒤에는 장면을 기다린다(tools/test/scene.js).
    const kit = sceneKit(p, 70); const { press, until } = kit; p.kit = kit;
    await p.goto(`http://localhost:${PORT}/`); await until('Title', '타이틀', 60000); await p.waitForTimeout(300); await p.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
    if (first) { first = false;
      await press('ArrowDown'); await press('Enter'); await until('Settings { host: Title', '타이틀 → 설정'); await press('ArrowUp'); await press('Enter'); await until('Controls', '설정 → 조작'); // 타이틀의 설정: 조작 설정은 맨 끝 줄(0164)
      await press('ArrowUp', 2); await press('Enter'); await p.waitForTimeout(200); await press('Escape'); await until('Settings { host: Title', '조작 → 설정'); await press('Escape'); await until('Title', '설정 → 타이틀'); await press('ArrowUp'); }
    await press('ArrowUp'); await press('Enter'); await until('Dev { tab: 0', '타이틀 → 개발 창'); await press('ArrowRight', tab); await until(`Dev { tab: ${tab}`, `개발 묶음 ${tab}`); await press('ArrowDown', row); await press('Enter');
    // 기술 묶음의 "기술 다 올리기"(켬)는 창에 머문다 — 그 밖은 판이 선다.
    if (!(tab === 2 && row === 1)) await until(['Play', 'Sandbox'], `개발 줄 ${tab}·${row} → 판`);
    return p;
  };
  const burst = async (p, name, n) => { for (let i = 0; i < n; i++) await p.screenshot({ path: path.join(OUT, `${name}-${i}.png`) }); };
  const hold = async (p, k, ms) => { await p.keyboard.down(k); await p.waitForTimeout(ms); await p.keyboard.up(k); };
  const only = process.argv[4] ? process.argv[4].split(',') : null;
  const want = (n) => !only || only.includes(n);
  let p;
  if (want('duel')) {
    // 결투(잔재 + 가시짐승): 락온 → 한 프레임 = 한 틱(F6) → 평타·마법·회피·내려찍기·강타
    p = await open(4, 2); await p.waitForTimeout(900); await p.keyboard.press('KeyL'); await p.waitForTimeout(600);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 3; k++) { await p.keyboard.press('KeyJ'); await burst(p, `hit${k}`, 12); }
    await p.keyboard.press('KeyU'); await burst(p, 'cast', 6);
    await p.keyboard.press('Space'); await burst(p, 'dodge', 8);
    await p.keyboard.press('KeyF'); await burst(p, 'jump', 5); await p.keyboard.press('KeyJ'); await burst(p, 'plunge', 16);
    await p.keyboard.down('KeyJ'); await burst(p, 'charge', 14); await p.keyboard.up('KeyJ'); await burst(p, 'heavy', 14);
    await p.close();
  }
  if (want('horde')) {
    p = await open(4, 8); await p.waitForTimeout(1500); await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 5; k++) { await p.keyboard.press('KeyJ'); await burst(p, `horde${k}`, 10); }
    await p.close();
  }
  if (want('boss')) {
    p = await open(4, 0); await p.waitForTimeout(800); await hold(p, 'KeyW', 900); await p.keyboard.press('KeyL'); await p.waitForTimeout(700);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 10; k++) { await p.keyboard.press('KeyK'); await burst(p, `parry${k}`, 6); }
    await p.close();
  }
  if (want('skills')) {
    // 판마다 새 페이지(떼가 몸을 덮기 전에 낸다): [이름, 끼울 스킬의 자리(줄, 칸) — 없으면 그대로, 끼울 큰 기술의 칸 — 없으면 그대로, 기다림 ms, 누름, 찍을 수].
    // 기술 화면(0161)의 자리: 스킬은 보이는 줄 0(휘돌이·가로지르기·버팀·끌어모으기)·줄 2(땅울림·되울림·던지기·잔불 흩기), 큰 기술은 줄 4의
    // 칸 0·2·4(몰아치기·한 줄기·쟁반 뒤집기). 칸에서 확인 = 목록의 첫 줄 "칸 1에 끼우기"(큰 기술은 "끼우기"). 처음 끼워진 것은 휘돌이(Q)·
    // 가로지르기(E)·쟁반 뒤집기라 그 셋은 건드리지 않는다. 큰 기술은 Q+E.
    // 일곱째 = 개발 창 "든 무기 바꾸기"를 누를 횟수(0223 V3 — 땅울림은 양손 무기의 스킬: 끼우기 전에 망치를 든다).
    // 0225: 스킬마다 쥔 꼴이 있다 — 땅울림 = 망치(셋), 되울림·잔불 흩기 = 지팡이(넷: 맨손 → 활 → 검 → 망치 → 지팡이 — 활(0231)이
    // 돌림에 끼어 하나씩 밀렸다. 그 전엔 둘·셋이라 망치 대신 검을 들고 "못 쓴다"로 끼우지 못했다).
    // 끼울 칸은 이름(게임이 기술 탭의 고른 칸을 장면 이름 끝에 붙인다 — `… @Skill(Embers)`, 개발판)으로 찾는다: 자리 숫자는 판이 넓어질
    // 때마다 낡았다(0214·0215 뒤 잔불 흩기를 못 찾았다).
    // 0272: 맨손은 무기 판의 스킬을 쓰지 못한다(새 판은 맨손) — 한손검의 스킬도 검을 든다(둘). 쇠뇌 판·활 판의 새 스킬 둘·큰 기술 하나.
    const RUNS = [['whirl', null, null, 1500, 'Q', 22, 2], ['cut', null, null, 1100, 'E', 26, 2], ['gather', 'Skill(Gather)', null, 900, 'Q', 26, 2],
      ['quake', 'Skill(Quake)', null, 1000, 'Q', 26, 3], ['brace', 'Skill(Brace)', null, 1300, 'Q', 40, 2], ['echo', 'Skill(Echo)', null, 1300, 'Q', 16, 4],
      ['throw', 'Skill(Throw)', null, 900, 'Q', 30, 2], ['embers', 'Skill(Embers)', null, 1300, 'Q', 50, 4],
      ['upend', null, null, 1500, 'ult', 30, 2], ['flurry', null, 'Ult(Flurry)', 1300, 'ult', 14, 2], ['line', null, 'Ult(Line)', 900, 'ult', 30, 2],
      // 0270 C3: 무기 판의 새 스킬 여덟·큰 기술 둘. "든 무기 바꾸기" 수: 1 활 · 2 검 · 3 망치 · 4 지팡이 · 5 쇠뇌 · 6 검과 방패.
      ['stitch', 'Skill(Stitch)', null, 900, 'Q', 30, 2], ['ram', 'Skill(Ram)', null, 900, 'Q', 34, 6], ['toll', 'Skill(Toll)', null, 900, 'Q', 80, 3],
      ['swat', 'Skill(Swat)', null, 1000, 'Q', 30, 3], ['ripple', 'Skill(Ripple)', null, 1000, 'Q', 44, 3],
      ['snow', 'Skill(Snow)', null, 1100, 'Q', 90, 4], ['flare', 'Skill(Flare)', null, 1000, 'Q', 34, 4], ['snare', 'Skill(Snare)', null, 500, 'Q', 120, 1],
      ['bell', null, 'Ult(Bell)', 1300, 'ult', 44, 6], ['blizzard', null, 'Ult(Blizzard)', 1300, 'ult', 90, 6],
      // 0272: 쇠뇌 판의 충격 갈래 둘(쇠뇌 = 다섯) · 활 판의 큰 기술(활 = 하나).
      ['fuse', 'Skill(Fuse)', null, 900, 'Q', 90, 5], ['mine', 'Skill(Mine)', null, 500, 'Q', 120, 5], ['rain', null, 'Ult(Rain)', 1300, 'ult', 120, 1],
      // 0272 2단계: 창 판(창 = 일곱 — 검과 방패 다음). 질풍 찌르기는 찌름 여섯이 1.4초 남짓.
      ['skewer', 'Skill(Skewer)', null, 1000, 'Q', 40, 7], ['pole', 'Skill(Pole)', null, 1400, 'Q', 34, 7], ['pin', 'Skill(Pin)', null, 1000, 'Q', 30, 7],
      ['gale', null, 'Ult(Gale)', 1000, 'ult', 70, 7]];
    const pick = async (p, n, key = 'ArrowRight') => { for (let i = 0; i < n; i++) { await p.keyboard.press(key); await p.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))); await p.waitForTimeout(90); } };
    // 이름으로 칸 찾기(0270 C2 — 판 하나씩): 판 줄로 올라가(위를 넉넉히 — `@Strip`) ↓ = 보이는 판(든 무기의 판)의 맨 윗줄 → 줄마다 왼쪽 끝 →
    // 오른쪽으로 훑으며 장면 이름 끝의 `@칸`을 본다. 없으면 무기 판을 한손검부터 차례로(판 줄에서 → — 공통 판에는 스킬이 없다).
    // 찾으면 확인(목록) → 확인(첫 줄 "칸 1에 끼우기", 큰 기술은 "끼우기"). 못 찾으면 실패(조용히 지나가 끼우지 않은 채 찍지 않게).
    const at = (p) => p.evaluate(() => window.knellScene || '');
    const wear = async (p, target) => {
      const hit = async () => (await at(p)).endsWith(`@${target}`);
      const scan = async () => {
        await pick(p, 1, 'ArrowDown');
        for (let row = 0; row < 6; row++) {
          if (row > 0) await pick(p, 1, 'ArrowDown');
          await pick(p, 9, 'ArrowLeft');
          for (let col = 0; col < 9; col++) { if (await hit()) return true; await pick(p, 1); }
        }
        return false;
      };
      await pick(p, 12, 'ArrowUp');
      let found = await scan();
      for (let plate = 1; plate < 8 && !found; plate++) {
        await pick(p, 12, 'ArrowUp'); await pick(p, 8, 'ArrowLeft'); await pick(p, plate);
        found = await scan();
      }
      if (!found) throw new Error(`기술 판에서 ${target} 칸을 못 찾았다 — 지금 ${await at(p)}`);
      await p.keyboard.press('Enter');
      try { await p.kit.until('SkillAct', `${target} → 할 일 목록`, 4000); } catch (e) {
        await p.screenshot({ path: path.join(OUT, `sk-wear-fail-${target}.png`) }); throw e;
      }
      await p.keyboard.press('Enter'); await p.kit.until('Skills', '끼우기 → 기술 탭');
    };
    // 다섯째 인자로 스킬 이름을 고르면 그것만(예: … skills cut,embers).
    const names = process.argv[5] ? process.argv[5].split(',') : null;
    for (const [name, slot, ult, wait, key, n, arm] of RUNS) {
      if (names && !names.includes(name)) continue;
      p = await open(2, 1); await p.waitForTimeout(300); // 기술 탭(셋째 묶음) 둘째 줄 = 기술 다 올리기(켬)
      if (arm) { // 끝 줄 "든 무기 바꾸기"를 `arm`번(새 판은 맨손 → 활 → 검 → 망치 → 지팡이) — 그 무기로 끼우고 아레나에 선다
        await pick(p, 4, 'ArrowDown'); for (let i = 0; i < arm; i++) { await p.keyboard.press('Enter'); await p.waitForTimeout(150); }
        await p.screenshot({ path: path.join(OUT, `sk-arm-${name}.png`) }); await pick(p, 4, 'ArrowUp');
      }
      await p.keyboard.press('ArrowUp'); await p.waitForTimeout(90); await p.keyboard.press('Enter'); await p.kit.until('Skills', '기술 끼우기 → 기술 탭'); // 기술 끼우기
      if (slot) await wear(p, slot);
      if (arm) await p.screenshot({ path: path.join(OUT, `sk-arm-${name}-worn.png`) });
      if (ult !== null) await wear(p, ult);
      if (name === 'whirl' || name === 'line') await p.screenshot({ path: path.join(OUT, `sk-window-${name}.png`) });
      await p.keyboard.press('Escape'); await p.kit.until('Dev', '기술 탭 → 개발 창');
      await p.waitForTimeout(200); await p.kit.press('ArrowDown', 3); // 같은 기술 탭의 넷째 줄 = 기술 아레나
      await p.keyboard.press('Enter'); await p.kit.until(['Play', 'Sandbox'], '기술 아레나'); await p.waitForTimeout(wait); // 떼가 다가오게
      await p.keyboard.press('F6'); await p.waitForTimeout(300);
      if (key === 'ult') {
        await p.keyboard.down('KeyQ'); await p.keyboard.down('KeyE'); await p.waitForTimeout(40);
        await p.keyboard.up('KeyQ'); await p.keyboard.up('KeyE');
      } else {
        await p.keyboard.press(`Key${key}`);
      }
      await burst(p, `sk-${name}`, n);
      // 되울림은 받아쳐야 울린다(맞는 틱은 운 — 몇 번), 몰아치기는 평타를 이어 본다.
      if (name === 'echo') for (let k = 0; k < 8; k++) { await p.keyboard.press('KeyK'); await burst(p, `sk-echo-parry${k}`, 5); }
      if (name === 'flurry') for (let k = 0; k < 5; k++) { await p.keyboard.press('KeyJ'); await burst(p, `sk-flurry-hit${k}`, 5); }
      await p.close();
    }
  }
  if (want('spear')) {
    // 0272 2단계 창: 개발 창 기술 묶음 "든 무기 바꾸기"를 일곱 번(맨손 → … → 검과 방패 → 창) → 전투 묶음 끝 줄 "한 줄로 선 잔재"(적은 서 있고 방랑자는 맞지 않는다) — 쿼터뷰 ·
    // 락온 한 장씩, 그다음 틱마다(F6): 찌르기 셋 · 모아 내달려 꿰뚫기 · 달리며 찌르기(회피 뒤 공격).
    p = await ctx.newPage(); const kit = sceneKit(p, 70); p.kit = kit;
    await sceneKit.unhurt(p); await sceneKit.still(p);
    p.on('pageerror', e => { if (!/register_plugin/.test(String(e))) errors.push(String(e)); });
    await p.goto(`http://localhost:${PORT}/`); await kit.until('Title', '타이틀', 60000); await p.waitForTimeout(300); await p.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
    if (first) { first = false;
      await kit.press('ArrowDown'); await kit.press('Enter'); await kit.until('Settings { host: Title', '타이틀 → 설정'); await kit.press('ArrowUp'); await kit.press('Enter'); await kit.until('Controls', '설정 → 조작');
      await kit.press('ArrowUp', 2); await kit.press('Enter'); await p.waitForTimeout(200); await kit.press('Escape'); await kit.until('Settings { host: Title', '조작 → 설정'); await kit.press('Escape'); await kit.until('Title', '설정 → 타이틀'); await kit.press('ArrowUp'); }
    await kit.press('ArrowUp'); await kit.press('Enter'); await kit.until('Dev { tab: 0', '타이틀 → 개발 창');
    await kit.press('ArrowRight', 2); await kit.until('Dev { tab: 2', '기술 묶음'); await kit.press('ArrowDown', 5);
    for (let i = 0; i < 7; i++) { await p.keyboard.press('Enter'); await p.waitForTimeout(150); }
    await p.screenshot({ path: path.join(OUT, 'spear-arm.png') });
    await kit.press('ArrowRight', 2); await kit.until('Dev { tab: 4', '전투 묶음'); await kit.press('ArrowDown', 15); await kit.press('Enter');
    await kit.until(['Play', 'Sandbox'], '한 줄로 선 잔재'); await p.waitForTimeout(1200);
    await p.keyboard.press('ArrowUp'); await p.waitForTimeout(700); // 한 단계 가까운 쿼터뷰
    await p.screenshot({ path: path.join(OUT, 'spear-quarter.png') });
    await p.keyboard.press('KeyL'); await p.waitForTimeout(900); await p.screenshot({ path: path.join(OUT, 'spear-lockon.png') });
    await p.keyboard.press('KeyL'); await p.waitForTimeout(700);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 3; k++) { await p.keyboard.press('KeyJ'); await burst(p, `spear-hit${k}`, 10); }
    await p.keyboard.down('KeyJ'); await burst(p, 'spear-charge', 12); await p.keyboard.up('KeyJ'); await burst(p, 'spear-lunge', 16);
    await p.keyboard.press('Space'); await burst(p, 'spear-dodge', 6); await p.keyboard.press('KeyJ'); await burst(p, 'spear-dash', 14);
    await p.close();
  }
  if (want('stage')) {
    // 입구: 화로 불티 + 회피·달리기 먼지(진짜 바닥)
    p = await open(3, 0); await p.waitForTimeout(6500); await hold(p, 'KeyW', 2200); await hold(p, 'KeyD', 2500); await p.waitForTimeout(500);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    await burst(p, 'brazier', 3); await p.keyboard.down('KeyD'); await p.keyboard.press('Space'); await burst(p, 'stagedodge', 10); await p.keyboard.up('KeyD');
    await p.close();
  }
  await b.close(); server.close(); if (errors.length) { console.error('console errors:\n' + errors.slice(0, 5).join('\n')); process.exit(1); } console.log('ok');
})();
