// 이펙트를 틱마다 찍는다(0141): 헤드리스 Chromium으로 개발 시나리오를 띄우고, 개발 키 F6(한 프레임 = 한 틱)을 켠 뒤
// 동작마다 연달아 스크린숏 — 0.1초짜리 번쩍임·먼지를 실제 시간으로는 못 잡는다(헤드리스는 초당 8프레임 남짓).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/fx.js dist shots-fx [duel,horde,boss,stage,skills,spear,daggers,greatsword,scythe,pull,wand,rb,bare,throw] [스킬 이름들 · rb면 axe,light,javelin,stone,magic,bow,crossbow,knives,quake,seed,curse · bare면 sword,hammer,staff,crossbow,bow,spear,daggers,greatsword,scythe,wand,shield]
// skills(0158 G2): 개발 창 기술 탭 "스킬 다 올리기"(끝 단 · 고르기 ㄱ, 기록에 안 쓴다) → "스킬 장착"에서 칸·큰 기술을 고르고 →
//   전투 탭 "skills" 판에서 Q(칸 1)·E(칸 2)·Q+E(큰 기술)를 틱마다 찍는다 — 스킬·큰 기술마다 새 판(sk-<이름>-<n>.png).
// 눈으로 볼 것: 맞힌 자리의 별·불똥, 휘두름의 초승달, 쓰러진 적의 재, 뛰기·회피·적 내리침의 먼지, 마법 탄의 마름모, 화로 불티.
// 처음에 "키보드만 배치로"(J 공격·K 패링·L 락온·Space 회피·F 점프·U 마법), 판은 개발 명령의 이름으로 연다(0278 R13 — `Scenario(Line)` ·
// `Arm(Wand)` · `Dev(Arm)`: 개발 창을 세어 누르지 않는다, tools/test/scene.js `dev`). 콘솔 에러가 있으면 종료 코드 1.
const fs = require('fs'); const path = require('path');
const sceneKit = require('./scene');
const DIST = path.resolve(process.argv[2] || 'dist'); const OUT = path.resolve(process.argv[3] || 'shots-fx');
(async () => {
  fs.mkdirSync(OUT, { recursive: true }); const srv = await sceneKit.serve(DIST, { port: 8141 });
  const b = await sceneKit.launch(); const ctx = await b.newContext({ viewport: { width: 1280, height: 720 } });
  const errors = []; let first = true;
  // 새 페이지 → 타이틀(처음 한 번은 "키보드만 배치로" — 설정은 컨텍스트에 남는다). quiet = 방랑자는 맞지 않고 적은 서 있다.
  const fresh = async ({ quiet = false } = {}) => {
    const p = await ctx.newPage();
    // 장면이 바뀌는 누름 뒤에는 장면을 기다린다(tools/test/scene.js).
    const kit = sceneKit(p, 70); p.kit = kit;
    if (quiet) { await sceneKit.unhurt(p); await sceneKit.still(p); }
    sceneKit.watch(p, errors);
    await p.goto(srv.url); await kit.until('Title', '타이틀', 60000); await p.waitForTimeout(300); await p.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
    if (first) { first = false; await kit.keyboardOnly(); }
    return p;
  };
  // 개발 명령으로 판을 연다(개발 창 줄의 이름 — 마지막 것이 판을 세운다).
  const open = async (...names) => {
    const p = await fresh(); await p.kit.dev(...names);
    await p.kit.until(['Play', 'Sandbox'], `개발 명령 ${names.join(' · ')} → 판`);
    return p;
  };
  // 무기를 들고(`Arm(<무기>)`) 아레나에 선다. shot이면 개발 창을 그 줄(`Dev(Arm)` — 설명에 든 무기)에 열어 한 장 찍고 아레나로.
  const armed = async (arm, { quiet = true, shot = null, row = 'Arm', arena = 'Scenario(Line)' } = {}) => {
    const p = await fresh({ quiet });
    await p.kit.dev(`Arm(${arm})`);
    if (shot) { await p.kit.dev(`Dev(${row})`); await p.kit.until(new RegExp(`^Dev \\{.* @${row}$`), `개발 창 ${row} 줄`); await p.screenshot({ path: path.join(OUT, shot) }); }
    await p.kit.dev(arena); await p.kit.until(['Play', 'Sandbox'], arena);
    return p;
  };
  const burst = async (p, name, n) => { for (let i = 0; i < n; i++) await p.screenshot({ path: path.join(OUT, `${name}-${i}.png`) }); };
  const hold = async (p, k, ms) => { await p.keyboard.down(k); await p.waitForTimeout(ms); await p.keyboard.up(k); };
  const only = process.argv[4] ? process.argv[4].split(',') : null;
  const want = (n) => !only || only.includes(n);
  let p;
  if (want('duel')) {
    // 결투(잔재 + 가시짐승): 락온 → 한 프레임 = 한 틱(F6) → 평타·마법·회피·내려찍기·강타
    p = await open('Scenario(Duel)'); await p.waitForTimeout(900); await p.keyboard.press('KeyL'); await p.waitForTimeout(600);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 3; k++) { await p.keyboard.press('KeyJ'); await burst(p, `hit${k}`, 12); }
    await p.keyboard.press('KeyU'); await burst(p, 'cast', 6);
    await p.keyboard.press('Space'); await burst(p, 'dodge', 8);
    await p.keyboard.press('KeyF'); await burst(p, 'jump', 5); await p.keyboard.press('KeyJ'); await burst(p, 'plunge', 16);
    await p.keyboard.down('KeyJ'); await burst(p, 'charge', 14); await p.keyboard.up('KeyJ'); await burst(p, 'heavy', 14);
    await p.close();
  }
  if (want('horde')) {
    // 떼(0278 R13: 세던 길은 전투 묶음 아홉째 줄 — 줄이 늘어 이끼 덮인 문지기를 찍고 있었다. 이름으로 떼를 연다).
    p = await open('Scenario(Horde)'); await p.waitForTimeout(1500); await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 5; k++) { await p.keyboard.press('KeyJ'); await burst(p, `horde${k}`, 10); }
    await p.close();
  }
  if (want('boss')) {
    p = await open('Scenario(Boss)'); await p.waitForTimeout(800); await hold(p, 'KeyW', 900); await p.keyboard.press('KeyL'); await p.waitForTimeout(700);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 10; k++) { await p.keyboard.press('KeyK'); await burst(p, `parry${k}`, 6); }
    await p.close();
  }
  if (want('skills')) {
    // 판마다 새 페이지(떼가 몸을 덮기 전에 낸다): [이름, 끼울 스킬의 자리(줄, 칸) — 없으면 그대로, 끼울 큰 기술의 칸 — 없으면 그대로, 기다림 ms, 누름, 찍을 수].
    // 기술 화면(0161)의 자리: 스킬은 보이는 줄 0(휘돌이·가로지르기·버팀·끌어모으기)·줄 2(땅울림·되울림·던지기·잔불 흩기), 큰 기술은 줄 4의
    // 칸 0·2·4(몰아치기·한 줄기·쟁반 뒤집기). 칸에서 확인 = 목록의 첫 줄 "칸 1에 끼우기"(큰 기술은 "끼우기"). 처음 끼워진 것은 휘돌이(Q)·
    // 가로지르기(E)·쟁반 뒤집기라 그 셋은 건드리지 않는다. 큰 기술은 Q+E.
    // 일곱째 = 들 무기(개발 명령 `Arm(<무기>)`, 0223 V3 — 스킬마다 쥔 꼴이 있다: 땅울림 = 망치, 되울림·잔불 흩기 = 지팡이. 예전에는
    // 개발 창 "주무기 바꾸기"를 누를 횟수였고, 활(0231)이 돌림에 끼자 하나씩 밀려 망치 대신 검을 들고 "못 쓴다"로 끼우지 못했다 — 0278 R13에서 이름으로).
    // 끼울 칸은 이름(게임이 기술 탭의 고른 칸을 장면 이름 끝에 붙인다 — `… @Skill(Embers)`, 개발판)으로 찾는다: 자리 숫자는 판이 넓어질
    // 때마다 낡았다(0214·0215 뒤 잔불 흩기를 못 찾았다).
    // 0272: 맨손은 무기 판의 스킬을 쓰지 못한다(새 판은 맨손) — 한손검의 스킬도 검을 든다(둘). 쇠뇌 판·활 판의 새 스킬 둘·큰 기술 하나.
    // 0274 ③: 가로지르기 · 던지기 · 바늘땀 · 제비걸음은 쌍단검의 것 — 가로지르기는 처음 끼워진 둘째 칸(E).
    const RUNS = [['whirl', null, null, 1500, 'Q', 22, 'Sword'], ['cut', null, null, 1100, 'E', 26, 'Daggers'], ['gather', 'Skill(Gather)', null, 900, 'Q', 26, 'Sword'],
      ['quake', 'Skill(Quake)', null, 1000, 'Q', 26, 'Hammer'], ['brace', 'Skill(Brace)', null, 1300, 'Q', 40, 'Sword'], ['echo', 'Skill(Echo)', null, 1300, 'Q', 16, 'Staff'],
      ['throw', 'Skill(Throw)', null, 900, 'Q', 30, 'Daggers'], ['embers', 'Skill(Embers)', null, 1300, 'Q', 50, 'Staff'],
      ['upend', null, null, 1500, 'ult', 30, 'Sword'], ['flurry', null, 'Ult(Flurry)', 1300, 'ult', 14, 'Sword'], ['line', null, 'Ult(Line)', 900, 'ult', 30, 'Sword'],
      // 0270 C3: 무기 판의 새 스킬 여덟·큰 기술 둘.
      ['stitch', 'Skill(Stitch)', null, 900, 'Q', 30, 'Daggers'], ['ram', 'Skill(Ram)', null, 900, 'Q', 34, 'Sword+Shield'], ['toll', 'Skill(Toll)', null, 900, 'Q', 80, 'Hammer'],
      ['swat', 'Skill(Swat)', null, 1000, 'Q', 30, 'Hammer'], ['ripple', 'Skill(Ripple)', null, 1000, 'Q', 44, 'Hammer'],
      ['snow', 'Skill(Snow)', null, 1100, 'Q', 90, 'Staff'], ['flare', 'Skill(Flare)', null, 1000, 'Q', 34, 'Staff'], ['snare', 'Skill(Snare)', null, 500, 'Q', 120, 'Bow'],
      ['bell', null, 'Ult(Bell)', 1300, 'ult', 44, 'Sword+Shield'], ['blizzard', null, 'Ult(Blizzard)', 1300, 'ult', 90, 'Sword+Shield'],
      // 0272: 쇠뇌 판의 충격 갈래 둘 · 활 판의 큰 기술.
      ['fuse', 'Skill(Fuse)', null, 900, 'Q', 90, 'Crossbow'], ['mine', 'Skill(Mine)', null, 500, 'Q', 120, 'Crossbow'], ['rain', null, 'Ult(Rain)', 1300, 'ult', 120, 'Bow'],
      // 0272 2단계: 창 판. 질풍 찌르기는 찌름 여섯이 1.4초 남짓.
      ['skewer', 'Skill(Skewer)', null, 1000, 'Q', 40, 'Spear'], ['pole', 'Skill(Pole)', null, 1400, 'Q', 34, 'Spear'], ['pin', 'Skill(Pin)', null, 1000, 'Q', 30, 'Spear'],
      ['gale', null, 'Ult(Gale)', 1000, 'ult', 70, 'Spear'],
      // 0274 ②: 한손검의 전투 기술.
      ['pommel', 'Skill(Pommel)', null, 900, 'Q', 26, 'Sword'], ['shove', 'Skill(Shove)', null, 900, 'Q', 30, 'Sword'], ['rally', 'Skill(Rally)', null, 1000, 'Q', 34, 'Sword'],
      // 0274 ③: 쌍단검 판 — 제비걸음 · 빠지며 베기 · 그림자 습격(숨어들어 찌름 넷이 1.2초 남짓).
      ['swallow', 'Skill(Swallow)', null, 1000, 'Q', 40, 'Daggers'], ['slip', 'Skill(Slip)', null, 900, 'Q', 30, 'Daggers'],
      ['ambush', null, 'Ult(Ambush)', 1000, 'ult', 80, 'Daggers'],
      // 0274 ④: 대검 판 — 땅 가르기 · 불씨 휩쓸기 · 칼바람(휩쓸기 다섯이 0.8초 남짓).
      ['sunder', 'Skill(Sunder)', null, 1000, 'Q', 34, 'Greatsword'], ['sear', 'Skill(Sear)', null, 900, 'Q', 30, 'Greatsword'],
      ['tempest', null, 'Ult(Tempest)', 1000, 'ult', 80, 'Greatsword'],
      // 0274 ⑤: 낫 판 — 걸어 당기기 · 생기 거두기 · 깨어나는 숲(고리 셋이 1초 남짓).
      ['reap', 'Skill(Reap)', null, 2400, 'Q', 30, 'Scythe'], ['harvest', 'Skill(Harvest)', null, 1000, 'Q', 36, 'Scythe'],
      ['grove', null, 'Ult(Grove)', 1300, 'ult', 100, 'Scythe'],
      // 0274 ⑥: 완드 판 — 저주 새기기 · 시드는 그늘(4초 — 앞의 1.6초) · 치른 값(두 박이 1초 남짓).
      ['hex', 'Skill(Hex)', null, 1000, 'Q', 34, 'Wand'], ['blight', 'Skill(Blight)', null, 1000, 'Q', 60, 'Wand'],
      ['price', null, 'Ult(Price)', 1300, 'ult', 100, 'Wand']];
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
      for (let plate = 1; plate < 12 && !found; plate++) {
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
      // 스킬 다 올리기(켬 — 판은 그대로 타이틀) → 무기를 들고 개발 창을 "주무기 바꾸기" 줄에 열어 한 장 → 그 위에 기술 탭(host: Dev).
      p = await fresh(); await p.kit.dev('SkillsMax', `Arm(${arm})`, 'Dev(Arm)'); await p.kit.until(/^Dev \{.* @Arm$/, '개발 창 주무기 바꾸기 줄');
      await p.screenshot({ path: path.join(OUT, `sk-arm-${name}.png`) });
      await p.kit.dev('Skills'); await p.kit.until('Skills', '스킬 장착 → 스킬 탭');
      if (slot) await wear(p, slot);
      await p.screenshot({ path: path.join(OUT, `sk-arm-${name}-worn.png`) });
      if (ult !== null) await wear(p, ult);
      if (name === 'whirl' || name === 'line') await p.screenshot({ path: path.join(OUT, `sk-window-${name}.png`) });
      await p.keyboard.press('Escape'); await p.kit.until('Dev', '기술 탭 → 개발 창');
      await p.kit.dev('Scenario(Skills)'); await p.kit.until(['Play', 'Sandbox'], '기술 아레나'); await p.waitForTimeout(wait); // 떼가 다가오게
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
    // 0272 2단계 창: 창을 들고(`Arm(Spear)`) → "한 줄로 선 잔재"(`Scenario(Line)`)(적은 서 있고 방랑자는 맞지 않는다) — 쿼터뷰 ·
    // 락온 한 장씩, 그다음 틱마다(F6): 찌르기 셋 · 모아 내달려 꿰뚫기 · 달리며 찌르기(회피 뒤 공격).
    p = await armed('Spear', { shot: 'spear-arm.png' }); await p.waitForTimeout(1200);
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
  if (want('daggers')) {
    // 0274 ③ 쌍단검: `Arm(Daggers)` → "한 줄로 선 잔재"(`Scenario(Line)`) — 쿼터뷰 · 락온 한 장씩, 그다음
    // 틱마다(F6): 4연타 · 모아 꿰뚫고 지나가기 · 달려들며 찌르기(회피 뒤 공격).
    p = await armed('Daggers', { shot: 'daggers-arm.png' }); await p.waitForTimeout(1200);
    await p.keyboard.press('ArrowUp'); await p.waitForTimeout(700); // 한 단계 가까운 쿼터뷰
    await p.screenshot({ path: path.join(OUT, 'daggers-quarter.png') });
    await p.keyboard.press('KeyL'); await p.waitForTimeout(900); await p.screenshot({ path: path.join(OUT, 'daggers-lockon.png') });
    await p.keyboard.press('KeyL'); await p.waitForTimeout(700);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 4; k++) { await p.keyboard.press('KeyJ'); await burst(p, `daggers-hit${k}`, 8); }
    await p.keyboard.down('KeyJ'); await burst(p, 'daggers-charge', 12); await p.keyboard.up('KeyJ'); await burst(p, 'daggers-lunge', 16);
    await p.keyboard.press('Space'); await burst(p, 'daggers-dodge', 6); await p.keyboard.press('KeyJ'); await burst(p, 'daggers-dash', 12);
    await p.close();
  }
  if (want('greatsword')) {
    // 0274 ④ 대검: `Arm(Greatsword)` → "한 줄로 선 잔재"(`Scenario(Line)`) — 쿼터뷰 · 락온 한 장씩, 그다음
    // 틱마다(F6): 휩쓸기 다섯(앞의 셋까지) · 모아 두 손 내려베기 · 달려들며 올려 베기(회피 뒤 공격) · RB 땅 내려찍기(락온을 풀고).
    p = await armed('Greatsword', { shot: 'gs-arm.png' }); await p.waitForTimeout(1200);
    await p.keyboard.press('ArrowUp'); await p.waitForTimeout(700); // 한 단계 가까운 쿼터뷰
    await p.screenshot({ path: path.join(OUT, 'gs-quarter.png') });
    await p.keyboard.press('KeyL'); await p.waitForTimeout(900); await p.screenshot({ path: path.join(OUT, 'gs-lockon.png') });
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 5; k++) { await p.keyboard.press('KeyJ'); await burst(p, `gs-lock-hit${k}`, 9); }
    await p.keyboard.press('F6'); await p.keyboard.press('KeyL'); await p.waitForTimeout(700); await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 5; k++) { await p.keyboard.press('KeyJ'); await burst(p, `gs-hit${k}`, 9); }
    await p.keyboard.down('KeyJ'); await burst(p, 'gs-charge', 16); await p.keyboard.up('KeyJ'); await burst(p, 'gs-heavy', 18);
    await p.keyboard.press('Space'); await burst(p, 'gs-dodge', 6); await p.keyboard.press('KeyJ'); await burst(p, 'gs-dash', 12);
    await p.keyboard.press('KeyU'); await burst(p, 'gs-rb', 30);
    await p.close();
  }
  if (want('scythe')) {
    // 0274 ⑤ 낫: `Arm(Scythe)` → "한 줄로 선 잔재"(`Scenario(Line)`) — 쿼터뷰 · 락온 한 장씩, 그다음
    // 틱마다(F6): 걸어 베기 둘 · 걸어 당기며 베기(줄의 앞이 끌려온다) · 모아 크게 걸어 끌어오기 · 달려들며 올려 베기(회피 뒤 공격) · RB 가시 씨앗(락온을 풀고).
    p = await armed('Scythe', { shot: 'sc-arm.png' }); await p.waitForTimeout(1200);
    await p.keyboard.press('ArrowUp'); await p.waitForTimeout(700); // 한 단계 가까운 쿼터뷰
    await p.screenshot({ path: path.join(OUT, 'sc-quarter.png') });
    await p.keyboard.press('KeyL'); await p.waitForTimeout(900); await p.screenshot({ path: path.join(OUT, 'sc-lockon.png') });
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 3; k++) { await p.keyboard.press('KeyJ'); await burst(p, `sc-lock-hit${k}`, 12); }
    await p.keyboard.press('F6'); await p.keyboard.press('KeyL'); await p.waitForTimeout(700); await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 3; k++) { await p.keyboard.press('KeyJ'); await burst(p, `sc-hit${k}`, 12); }
    await p.keyboard.down('KeyJ'); await burst(p, 'sc-charge', 16); await p.keyboard.up('KeyJ'); await burst(p, 'sc-heavy', 24);
    await p.keyboard.press('Space'); await burst(p, 'sc-dodge', 6); await p.keyboard.press('KeyJ'); await burst(p, 'sc-dash', 12);
    // 줄에서 물러나 다시 줄 쪽을 본다(RB의 판과 같은 걸음) — 씨앗이 날아가 떨어지는 것이 보이게.
    await p.keyboard.press('F6'); await hold(p, 'KeyS', 900); await hold(p, 'KeyW', 120); await p.waitForTimeout(400); await p.keyboard.press('F6'); await p.waitForTimeout(300);
    await p.keyboard.press('KeyU'); await burst(p, 'sc-rb', 36);
    await p.close();
  }
  if (want('pull')) {
    // 0277 걸어 당김의 덩굴: 낫으로 엘리트 아레나의 굳은 잔재 앞에서 한 걸음 물러나(앞의 두 타는 허공) 셋째 타 · 강공격이 줄의 앞을 끌어오는 동안 손과 적을 잇는 덩굴을 틱마다.
    // 엘리트 아레나(`Scenario(Elites)` — 앞의 굳은 잔재는 여러 대를 받아도 서 있다: 한 줄의 잔재는 셋째 타 전에 쓰러져 끌 것이 남지 않았다).
    p = await armed('Scythe', { shot: 'pull-arm.png', arena: 'Scenario(Elites)' }); await p.waitForTimeout(700);
    // 락온한 채 다가갔다가 그대로 뒷걸음(굳은 잔재를 본 채 2 m 남짓)으로 물러나 락온을 푼다 — 쿼터뷰에서 그 쪽을 보고 친다.
    await p.keyboard.press('KeyL'); await p.waitForTimeout(500); await hold(p, 'KeyW', 1500); await hold(p, 'KeyS', 320);
    await p.keyboard.press('KeyL'); await p.waitForTimeout(900);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 3; k++) { await p.keyboard.press('KeyJ'); await burst(p, `pull-hit${k}`, 10); }
    await p.keyboard.press('F6'); await p.keyboard.press('KeyL'); await p.waitForTimeout(500); await hold(p, 'KeyS', 320); await p.keyboard.press('KeyL'); await p.waitForTimeout(900); await p.keyboard.press('F6'); await p.waitForTimeout(300);
    await p.keyboard.down('KeyJ'); await burst(p, 'pull-charge', 16); await p.keyboard.up('KeyJ'); await burst(p, 'pull-heavy', 16);
    await p.close();
  }
  if (want('wand')) {
    // 0274 ⑥ 완드: `Arm(Wand)` → "한 줄로 선 잔재"(`Scenario(Line)`) — 쿼터뷰 · 락온 한 장씩, 그다음
    // 틱마다(F6): 짧게 쳐 내기 둘 · 저주를 새기는 찌르기(균열) · 쥐었다 터뜨리는 저주 · 달려들며 찌르기(회피 뒤 공격) · RB 저주탄 셋(금이 간다).
    p = await armed('Wand', { shot: 'wd-arm.png' }); await p.waitForTimeout(1200);
    await p.keyboard.press('ArrowUp'); await p.waitForTimeout(700); // 한 단계 가까운 쿼터뷰
    await p.screenshot({ path: path.join(OUT, 'wd-quarter.png') });
    await p.keyboard.press('KeyL'); await p.waitForTimeout(900); await p.screenshot({ path: path.join(OUT, 'wd-lockon.png') });
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 3; k++) { await p.keyboard.press('KeyJ'); await burst(p, `wd-lock-hit${k}`, 12); }
    await p.keyboard.press('F6'); await p.keyboard.press('KeyL'); await p.waitForTimeout(700); await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 3; k++) { await p.keyboard.press('KeyJ'); await burst(p, `wd-hit${k}`, 12); }
    await p.keyboard.down('KeyJ'); await burst(p, 'wd-charge', 16); await p.keyboard.up('KeyJ'); await burst(p, 'wd-heavy', 24);
    await p.keyboard.press('Space'); await burst(p, 'wd-dodge', 6); await p.keyboard.press('KeyJ'); await burst(p, 'wd-dash', 12);
    // 줄에서 물러나 다시 줄 쪽을 본다(RB의 판과 같은 걸음) — 저주탄이 곧게 날아 줄의 앞에 금을 낸다.
    await p.keyboard.press('F6'); await hold(p, 'KeyS', 900); await hold(p, 'KeyW', 120); await p.waitForTimeout(400); await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 3; k++) { await p.keyboard.press('KeyU'); await burst(p, `wd-rb${k}`, 14); }
    await p.close();
  }
  if (want('rb')) {
    // 0274 ②b RB를 종류마다: 무기마다 새 판 — `Arm(<무기>)`(표의 둘째 칸) → "한 줄로 선 잔재"(`Scenario(Line)`)
    // (서 있는 적 · 방랑자는 맞지 않는다). 쿼터뷰에서 RB(U)를 틱마다(F6) 찍고, 숄더뷰(↑ 넷 — 가장 가까운 단계에서 더)
    // 에서 한 번 더. 활은 당기는 단계(U를 쥔 채 · 뗀 뒤), 쇠뇌는 셋을 쏜 뒤 멈춰 서서 덧장전(HUD 탄창 — 열 틱마다 한 장).
    // 여섯째 인자로 무기를 고르면 그것만(예: … rb axe,light).
    const ARMS = [['axe', 'Sword', 22], ['light', 'Hammer', 40], ['javelin', 'Spear', 70], ['stone', 'Fists', 22], ['magic', 'Staff', 16], ['bow', 'Bow', 0], ['crossbow', 'Crossbow', 0],
      // 0274 ③: 쌍단검 — 투척 단검 셋.
      ['knives', 'Daggers', 22],
      // 0274 ④: 대검 — 땅 내려찍기.
      ['quake', 'Greatsword', 30],
      // 0274 ⑤: 낫 — 가시 씨앗(떨어지기까지 0.35초 + 솟는 뿌리).
      ['seed', 'Scythe', 40],
      // 0274 ⑥: 완드 — 저주탄(곧게 · 균열).
      ['curse', 'Wand', 26]];
    const names = process.argv[5] ? process.argv[5].split(',') : null;
    for (const [name, arm, n] of ARMS) {
      if (names && !names.includes(name)) continue;
      p = await armed(arm); await p.waitForTimeout(1200);
      await p.keyboard.press('ArrowUp'); await p.waitForTimeout(700); // 한 단계 가까운 쿼터뷰
      await hold(p, 'KeyS', 900); await hold(p, 'KeyW', 120); await p.waitForTimeout(400); // 줄에서 물러나 거리를 두고 다시 줄 쪽을 본다(날아가는 것이 보이게 — 사격은 찾지 않는다)
      await p.keyboard.press('F6'); await p.waitForTimeout(300);
      if (name === 'bow') {
        await p.keyboard.down('KeyU'); await burst(p, 'rb-bow-draw', 50); await p.keyboard.up('KeyU'); await burst(p, 'rb-bow-full', 24);
        await p.keyboard.press('KeyU'); await burst(p, 'rb-bow-tap', 16);
      } else if (name === 'crossbow') {
        for (let k = 0; k < 3; k++) { await p.keyboard.press('KeyU'); await burst(p, `rb-xbow-shot${k}`, 6); }
        for (let k = 0; k < 12; k++) { await p.screenshot({ path: path.join(OUT, `rb-xbow-topup-${k}.png`) }); for (let j = 0; j < 10; j++) await p.evaluate(() => new Promise((r) => requestAnimationFrame(r))); }
      } else {
        await p.keyboard.press('KeyU'); await burst(p, `rb-${name}-q`, n);
        await p.keyboard.down('KeyU'); await burst(p, `rb-${name}-charge`, 50); await p.keyboard.up('KeyU'); await burst(p, `rb-${name}-charged`, n);
      }
      // 숄더뷰: ↑를 더 눌러 자유 숄더뷰(카메라 앞 = 크로스헤어) — 쏘기 전 한 장 · 쏜 뒤 틱마다.
      await p.keyboard.press('F6'); await p.waitForTimeout(200);
      for (let k = 0; k < 4; k++) { await p.keyboard.press('ArrowUp'); await p.waitForTimeout(250); }
      await p.waitForTimeout(900); await hold(p, 'ArrowRight', 350); await p.waitForTimeout(500); /* 줄 쪽으로 시점을 조금 돌린다 */ await p.screenshot({ path: path.join(OUT, `rb-${name}-shoulder.png`) });
      await p.keyboard.press('F6'); await p.waitForTimeout(300);
      if (name === 'bow') { await p.keyboard.down('KeyU'); await burst(p, 'rb-bow-s-draw', 46); await p.keyboard.up('KeyU'); await burst(p, 'rb-bow-s', 10); }
      else { await p.keyboard.press('KeyU'); await burst(p, `rb-${name}-s`, Math.max(n, 12)); }
      await p.close();
    }
  }
  if (want('throw')) {
    // 0278: 수류탄을 던지는 순간(활의 공격 = 뿌리 씨앗 수류탄 — 빈 손의 결이 덫으로 바꾸는 쇠뇌가 아니라): 손끝의 불티 셋 + 흙먼지.
    p = await armed('Bow'); await p.waitForTimeout(1200);
    await p.keyboard.press('ArrowUp'); await p.waitForTimeout(700);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 2; k++) { await p.keyboard.press('KeyJ'); await burst(p, `throw${k}`, 14); }
    await p.close();
  }
  if (want('bare')) {
    // 0276 빈 손의 결과 방패: 무기마다 새 판 — 새 기록이라 보조무기 슬롯은 비어 있다(놀이터 = 빈 손). `Arm(<무기>)`(표의 둘째 칸 — 방패는
    // `Sword+Shield`) → "한 줄로 선 잔재"(`Scenario(Line)`). 틱마다(F6) 찍는다.
    // 망치의 기도하는 손은 맞아야 마실 수 있다 — 그 판만 적을 세우지 않고(still 없음) 맞게 둔다(unhurt 없음).
    // 다섯째 인자로 고르면 그것만(예: … bare sword,shield).
    const BARE = [['sword', 'Sword', true], ['hammer', 'Hammer', false], ['staff', 'Staff', true], ['crossbow', 'Crossbow', true], ['bow', 'Bow', true],
      ['spear', 'Spear', true], ['daggers', 'Daggers', true], ['shield', 'Sword+Shield', true],
      // 0274 ④: 대검 — 남김없이 휘두른다(체력이 바닥이 아니라 둘레의 넷으로 분노를 채운다 — 줄 안으로 걸어 들어가 휘두른다).
      ['greatsword', 'Greatsword', true],
      // 0274 ⑤: 낫 — 땅을 짚는 손(가시 씨앗이 솟는 둘레가 넓다 — 보통 씨앗 · 모은 씨앗).
      ['scythe', 'Scythe', true],
      // 0274 ⑥: 완드 — 계약의 표(저주탄의 표가 남은 적이 쓰러지면 체력 · 칸 — 그림은 쏘는 손과 회복의 빛).
      ['wand', 'Wand', true]];
    const names = process.argv[5] ? process.argv[5].split(',') : null;
    for (const [name, arm, quiet] of BARE) {
      if (names && !names.includes(name)) continue;
      // 개발 창의 "보조무기 비우기" 줄 설명(지금: 그대로 — 새 기록이라 이미 빈 손)을 한 장.
      p = await armed(arm, { quiet, shot: `bare-${name}-dev.png`, row: 'ArmBare' }); await p.waitForTimeout(1200);
      await p.keyboard.press('ArrowUp'); await p.waitForTimeout(700); // 한 단계 가까운 쿼터뷰
      if (name === 'hammer') {
        // 줄 쪽으로 걸어 들어가 맞고(한두 대) 뒤로 굴러 빠진 뒤 — 쫓아오는 떼 앞에서 마신다(삼킨 순간 둘레가 밀려난다).
        await hold(p, 'KeyW', 900); await p.waitForTimeout(1600);
        for (let k = 0; k < 2; k++) { await p.keyboard.down('KeyS'); await p.keyboard.press('Space'); await p.waitForTimeout(450); await p.keyboard.up('KeyS'); }
        await p.waitForTimeout(250);
        await p.keyboard.press('F6'); await p.waitForTimeout(300);
        await p.keyboard.press('KeyR'); await burst(p, 'bare-hammer-sip', 50);
      } else {
        await p.keyboard.press('F6'); await p.waitForTimeout(300);
        await burst(p, `bare-${name}-hud`, 1);
        if (name === 'sword') {
          await p.keyboard.down('KeyJ'); await burst(p, 'bare-sword-charge', 30); await p.keyboard.up('KeyJ'); await burst(p, 'bare-sword-heavy', 24);
          for (let k = 0; k < 3; k++) { await p.keyboard.press('KeyJ'); await burst(p, `bare-sword-hit${k}`, 10); }
        } else if (name === 'staff') {
          await p.keyboard.down('KeyU'); await burst(p, 'bare-staff-charge', 30); await p.keyboard.up('KeyU'); await burst(p, 'bare-staff-charged', 14);
        } else if (name === 'crossbow') {
          // 줄 바로 앞의 잡몹 발밑에 놓으면 곧 문다(물기) · 물러나 빈 땅에 놓으면 놓인 덫이 보인다.
          await p.keyboard.press('KeyJ'); await burst(p, 'bare-crossbow-set', 30);
          await p.keyboard.press('F6'); await hold(p, 'KeyS', 1200); await p.keyboard.press('F6'); await p.waitForTimeout(300);
          await p.keyboard.press('KeyJ'); await burst(p, 'bare-crossbow-lay', 30);
        } else if (name === 'bow') {
          // 당긴 채 줄 쪽으로 걷고(가벼운 걸음) 끝까지 당긴 화살을 놓는다(오금 — 발밑의 끊긴 연두 고리가 길게 남는다).
          await hold(p, 'KeyS', 900); await p.keyboard.press('F6'); await p.waitForTimeout(300);
          await p.keyboard.down('KeyU'); await p.keyboard.down('KeyW'); await burst(p, 'bare-bow-draw', 50); await p.keyboard.up('KeyW'); await p.keyboard.up('KeyU'); await burst(p, 'bare-bow-loose', 30);
        } else if (name === 'spear') {
          await p.keyboard.down('KeyJ'); await burst(p, 'bare-spear-charge', 40); await p.keyboard.up('KeyJ'); await burst(p, 'bare-spear-lunge', 40);
        } else if (name === 'daggers') {
          // 줄 쪽(오른쪽 위)으로 굴러 들어가 첫 타 = 급소(그림자 걸음).
          await p.keyboard.down('KeyW'); await p.keyboard.down('KeyD'); await p.keyboard.press('Space'); await burst(p, 'bare-daggers-dodge', 8);
          await p.keyboard.up('KeyW'); await p.keyboard.up('KeyD'); await p.keyboard.press('KeyJ'); await burst(p, 'bare-daggers-shadow', 14);
        } else if (name === 'greatsword') {
          // 줄 안으로 걸어 들어가(둘레의 적 — 분노가 두 배로 차 가득) 쉬지 않고 휘두른다: 끝 타 뒤에 둘째 타로 되돌아 이어진다.
          await p.keyboard.press('F6'); await hold(p, 'KeyW', 700); await p.keyboard.press('F6'); await p.waitForTimeout(300);
          for (let k = 0; k < 8; k++) { await p.keyboard.press('KeyJ'); await burst(p, `bare-gs-hit${k}`, 7); }
        } else if (name === 'wand') {
          for (let k = 0; k < 4; k++) { await p.keyboard.press('KeyU'); await burst(p, `bare-wand-curse${k}`, 12); }
          await p.keyboard.down('KeyU'); await burst(p, 'bare-wand-charge', 50); await p.keyboard.up('KeyU'); await burst(p, 'bare-wand-charged', 30);
        } else if (name === 'scythe') {
          await p.keyboard.press('KeyU'); await burst(p, 'bare-scythe-seed', 36);
          await p.keyboard.down('KeyU'); await burst(p, 'bare-scythe-charge', 50); await p.keyboard.up('KeyU'); await burst(p, 'bare-scythe-charged', 36);
        } else if (name === 'shield') {
          await p.keyboard.press('KeyU'); await burst(p, 'bare-shield-throw', 60);
          await p.keyboard.down('KeyK'); await burst(p, 'bare-shield-guard', 70); await p.keyboard.up('KeyK'); await burst(p, 'bare-shield-rush', 30);
        }
      }
      await p.close();
    }
  }
  if (want('stage')) {
    // 입구: 화로 불티 + 회피·달리기 먼지(진짜 바닥)
    p = await open('Entrance'); await p.waitForTimeout(6500); await hold(p, 'KeyW', 2200); await hold(p, 'KeyD', 2500); await p.waitForTimeout(500);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    await burst(p, 'brazier', 3); await p.keyboard.down('KeyD'); await p.keyboard.press('Space'); await burst(p, 'stagedodge', 10); await p.keyboard.up('KeyD');
    await p.close();
  }
  await b.close(); srv.close(); if (errors.length) { console.error('console errors:\n' + errors.slice(0, 5).join('\n')); process.exit(1); } console.log('ok');
})();
