// 큰 창(화로·멈춤) 시험(0104 → 0164 "창 하나 + 탭" → 0275 흐름만): 헤드리스 Chromium으로 한국어·영어 두 판 —
//   쉼터 기록(잔불 넉넉)을 심고 이어하기 → 쉬기 → 화로 창(0263 — 성장 · 장비 · 가방 · 기술 · 가게 · 설정; 성장 탭 — 레벨 줄을 올려 보고
//   묻는 창, 기술 줄에서 확인 = 기술 탭의 그 칸, 0173) → 화로의 Select(Tab) = 같은 창의 장비 탭(후보의 작은 목록) → 가방(작은 목록 ·
//   여벌 정리) → 가게 · 설정 · 기술, 그리고 싸움에서 연 멈춤 창(장비 · 가방 · 기술 · 기록(여정·도감의 보상 받기) · 설정), 화면마다 스크린숏.
//   기술 탭(0270 C2): 위의 판 줄(공통 · 한손검 · 망치 · 지팡이 · 쇠뇌 · 활 · 창 · 쌍단검 · 대검 · 낫 · 완드 · 방패 — 넘치면 가로로 굴린다, 0275) + 그 판의 칸 격자 +
//   오른쪽 상세. 장면 이름이 보이는 판과 고른 칸을 말한다(`Skills { host: Hearth } [Class(Sword)] @Skill(Whirl)`) — 칸은 이름으로 찾는다.
//   한 단 사기 · 3단 고르기 뒤집기 · 패시브 칸 열기 · 결 열기 · 잠긴 칸 · 되돌리기(묻는 창), 판마다 판 줄에서 한 장(눈으로 본다),
//   멈춤의 기술 탭(끼우기만). 끝으로 새 판의 기록(0166·0167·0169 — 아는 스킬 없음, 튜토리얼의 상 한 점): 빈 스킬 칸 → 쉬기 = 기술 탭 →
//   배움 = 빈 칸에 끼워짐 → 첫 레벨 = 둘째 점수 → 멈춤에서 끼우기, 그리고 맨손(공통 판에서 연다).
//   0275: 글이 칸에 드는지는 `cargo test`(`app::text_fit` — 고정 칸의 모든 글을 글꼴로 잰다)가 본다. 이 스크립트는 흐름만 — 예전처럼
//   모든 판의 모든 칸·단을 미리 찍은 기록 넷을 돌며 상세를 F4로 보던 걸음(언어마다 25분)은 걷었다. 칸에 닿는 걸음은 Rust 시험
//   (`walking_reaches_every_cell_of_every_plate_from_anywhere`). F4 넘침 검사(`ui::check`)는 켜 둔다 — 지나는 화면에서 콘솔의
//   `[ui-check]` 줄(켬/끔 말고)이 있으면 실패.
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/hearth.js dist shots
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');
const sceneKit = require('./scene');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8137;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const KEY = 'knell.save.0';
// 문 앞 쉼터(0)에서 쉰 기록(뿌리껍질 덧옷을 걸쳤다 — 0176), 잔불은 레벨 몇 단 + 가게 + 기술 되돌리기를 볼 만큼. 기력·힘은 이미 4단(다음이 마지막 단 — "다 올림" 줄·평타 +1도 본다).
const SEED = 'v=1\nembers=700\nclears=1\nrun.rest=0\nrun.gate=1\ncharm.owned=shell\ncharm.worn=shell,,\nlv.life=1\nlv.stamina=4\nlv.might=4\ncore.warden=1\nfinds=sanctuary.garb.1,sanctuary.shard.0,sanctuary.stone.0,village.stone.0\nforge.sword=1\ngarb.worn=bark\nmigrated.v04=1\nweapon=sword\n';
// 새 판의 기록(0166·0167·0169 — `migrated.v06`이 있다: 처음의 둘을 받지 않는다). 튜토리얼의 몸 걸음 넷을 마쳐 상(기술 점수 1)을
// 받은 참이다. 문 앞 쉼터에 서 있고 잔불은 첫 레벨 값보다 넉넉히.
// 0272: 맨손은 무기 판이 없다(기술 탭이 공통 판에서 열린다) — 여기 기록들은 검을 들었다(`weapon=sword`), 맨손의 길은 끝에서 따로 본다.
const FRESH = 'v=1\nembers=130\nrun.rest=0\nrun.gate=0\nplay=1\nweapons=sword\nweapon=sword\nmigrated.v04=1\nmigrated.v06=1\nmigrated.ult=1\ntut.step=4\nskill.gift=1\n';

const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});

async function run(browser, lang) {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const page = await ctx.newPage();
  const errors = [];
  const checks = [];
  page.on('console', (m) => {
    const t = m.text();
    if (/\[ui-check\]/.test(t) && !/\[ui-check\] (on|off)$/.test(t)) checks.push(t);
    if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(t))) errors.push(t);
  });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(String(e)); });
  const shot = (name) => page.screenshot({ path: path.join(OUT, `hearth-${lang}-${name}.png`) });
  // 누름마다 게임 프레임 둘(창 안의 커서 옮기기는 그것으로 넉넉하다). 판이 서고·화로가 열리고·창이 닫히는 곳은 장면을 기다린다
  // (tools/test/scene.js — 느린 CI에서 판이 서기 전에 F가 들어가 화로 대신 싸움 화면을 찍던 길).
  const { press, until } = sceneKit(page, 90);
  await sceneKit.cheap(page);
  const toPlay = async () => { await until('Title', '타이틀'); await press('F4'); await press('Enter'); await until('Slots', '이어하기 → 슬롯 창'); await press('Enter'); await until('Play', '슬롯 → 판'); await page.waitForTimeout(1500); };
  const rest = async (what) => { await press('KeyF'); await until(/host: Hearth/, what || '화로에서 쉬기'); await page.waitForTimeout(300); };
  const close = async (what) => { await press('Escape'); await until('Play', what || '창 닫기'); await page.waitForTimeout(200); };
  const pause = async (what) => { await press('Escape'); await until(/host: Pause/, what || '멈춤'); await page.waitForTimeout(200); };
  // 기술 화면의 걸음(0161 → 0270 C2): 가장자리에서 돌지 않는다. 맨 윗줄에서 위 = 판 줄(`@Strip`), 판 줄에서 ← → = 판 바꾸기,
  // ↓ = 그 판의 맨 윗줄. 칸은 장면 이름 끝의 `@칸`으로 찾는다(자리 숫자는 판이 바뀔 때마다 낡았다).
  const at = () => page.evaluate(() => window.knellScene || '');
  // 누를 때마다 장면 이름을 보고 멈춘다(헛누름 없이 — 판을 도는 길이 길어 누름 하나하나가 시간이다).
  const toStrip = async () => {
    for (let i = 0; i < 12 && !(await at()).endsWith('@Strip'); i++) await press('ArrowUp');
    if (!(await at()).endsWith('@Strip')) throw new Error(`판 줄에 못 올랐다 — 지금 "${await at()}"`);
  };
  const toLeft = async () => {
    for (let i = 0; i < 10; i++) { const here = await at(); await press('ArrowLeft'); if ((await at()) === here) break; }
  };
  const PLATE_ORDER = ['Body', 'Class(Sword)', 'Class(Hammer)', 'Class(Staff)', 'Class(Crossbow)', 'Class(Bow)', 'Class(Spear)', 'Class(Dagger)', 'Class(Greatsword)', 'Class(Scythe)', 'Class(Wand)', 'Shield'];
  const toPlate = async (plate) => {
    await toStrip();
    const want = PLATE_ORDER.indexOf(plate);
    for (let i = 0; i < PLATE_ORDER.length; i++) {
      const s = await at();
      const now = PLATE_ORDER.findIndex((p) => s.includes(`[${p}]`));
      if (now === want) break;
      await press(now < want ? 'ArrowRight' : 'ArrowLeft');
    }
    if (!(await at()).includes(`[${plate}]`)) throw new Error(`${plate} 판을 못 골랐다 — 지금 "${await at()}"`);
  };
  // 보이는 판의 왼쪽 위 칸: 판 줄로 올라가 ↓(맨 윗줄) → 왼쪽 끝.
  const home = async () => { await toStrip(); await press('ArrowDown'); await toLeft(); };
  // 판 하나의 칸을 줄마다 왼쪽 끝 → 오른쪽으로 지난다(끝은 장면 이름이 안 바뀌는 것으로 — 옆 걸음은 줄을 떠나지 않는다). `each(row, col)`을 칸마다 부르고 참이면 거기서 멈춘다.
  // 지난 칸 이름들을 돌려준다. 맨 아랫줄에서 ↓가 되돌리기 단추(화로)면 한 번 올라와 끝낸다.
  const walkPlate = async (each) => {
    await home();
    const seen = new Set();
    for (let row = 0; row < 12; row++) {
      // ↓는 그 줄의 가로로 가장 가까운 칸에 내려선다 — 줄마다 왼쪽 끝부터.
      if (row > 0) await toLeft();
      for (let col = 0; col < 10; col++) {
        const here = await at();
        seen.add(here.replace(/^.*@/, ''));
        if (await each(row, col, here)) return { seen, stop: true };
        await press('ArrowRight');
        if ((await at()) === here) break;
      }
      const before = await at();
      await press('ArrowDown');
      const next = await at();
      if (next.endsWith('@Respec')) { await press('ArrowUp'); break; }
      if (next === before) break;
    }
    return { seen, stop: false };
  };
  // 보이는 판에서 칸 하나를 이름으로(예: 'Skill(Cut)').
  const goto = async (target) => {
    const { stop } = await walkPlate(async (r, c, here) => here.endsWith(`@${target}`));
    if (!stop) throw new Error(`${target} 칸을 못 찾았다 — 지금 "${await at()}"`);
  };
  // 판마다 판 줄에서 한 장(0275 — 판 줄이 넘치면 굴린다: 고른 판이 창 안에 보이는지 눈으로 본다).
  const strips = async (name) => {
    for (const plate of PLATE_ORDER) {
      await toPlate(plate);
      await shot(`${name}-${plate.replace(/[^A-Za-z]/g, '')}`);
    }
  };
  const ready = () => page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 20000 });
  const record = () => page.evaluate((k) => localStorage.getItem(k), KEY);

  await page.goto(`http://localhost:${PORT}/`);
  await ready();
  await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, SEED]);
  await page.reload();
  await ready();
  await page.waitForTimeout(600);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  await until('Title', '타이틀');
  await press('F4'); // 넘침 검사 켬(개발판)
  // 타이틀(이어하기·시작하기·설정·개발) → 설정 → 언어(첫 줄): 영어 판이면 → 한 번(한국어 → English).
  if (lang === 'en') {
    await press('ArrowDown', 2); await press('Enter'); await until('Settings { host: Title', '타이틀 → 설정');
    await press('ArrowRight'); await page.waitForTimeout(200);
    await shot('0-title-settings');
    await press('Escape'); await until('Title', '설정 → 타이틀');
    await press('ArrowUp', 2);
  }
  // 이어하기 → 슬롯 창 → 이어하기 → 문 앞 쉼터.
  await press('Enter'); await until('Slots', '이어하기 → 슬롯 창');
  await press('Enter'); await until('Play', '슬롯 → 판'); await page.waitForTimeout(1500);
  // 쉬기 → 큰 창(0164)이 성장(상태) 탭에서 열린다. 탭은 E(오른쪽)·Q(왼쪽)로 넘긴다 — 화로 = 성장 · 장비 · 가방 · 기술 · 가게 · 설정(0263).
  await rest(); await until('Status { host: Hearth', '화로 = 성장 탭');
  await shot('1-hearth');
  // 상태 탭의 레벨 줄: 줄마다 커서를 옮겨 미리보기 글을 본다(생명 1단 · 기력 4단(다음이 끝) · 힘 4단(다음 단에 평타 +1)).
  await shot('2-level-life');
  await press('ArrowDown'); await page.waitForTimeout(150);
  await shot('3-level-stamina');
  await press('ArrowDown'); await page.waitForTimeout(150);
  await shot('4-level-might');
  // 힘 5단 올리기 → 묻는 창 → 올림.
  await press('Enter'); await page.waitForTimeout(300);
  await shot('5-level-ask');
  await press('Enter'); await page.waitForTimeout(400);
  // 기력 5단(끝) 올리기 → "다 올림" 줄.
  await press('ArrowUp'); await press('Enter'); await page.waitForTimeout(300);
  await press('Enter'); await page.waitForTimeout(400);
  await shot('6-level-after');
  const r = await record();
  if (!/lv\.might=5\n/.test(r) || !/lv\.stamina=5\n/.test(r)) errors.push(`levels not saved: ${JSON.stringify(r)}`);
  // 다 올린 줄을 다시 고르면 안 됨 소리만(묻는 창 없음 — 까닭은 오른쪽 칸의 상태 줄 "다 올렸다").
  await press('Enter'); await page.waitForTimeout(300);
  await shot('7-level-full');
  await page.waitForTimeout(1800);
  // 상태 탭의 기술 줄 넷(레벨 줄 아래, 0173 — 그전에는 장신구 세 칸): 스킬 두 칸 · 큰 기술 · 패시브의 상세 → 패시브 줄에서 확인 =
  // 기술 탭의 그 칸으로(탭 사이의 이음). 거기서 Select(Tab) = 같은 화로 창의 장비 탭(무기 칸, 0263 `flow::hearth_select`).
  await press('ArrowDown', 2); await page.waitForTimeout(150);
  await shot('7a-status-skill-1');
  await press('ArrowDown'); await page.waitForTimeout(150);
  await shot('7b-status-skill-2');
  await press('ArrowDown'); await page.waitForTimeout(150);
  await shot('7c-status-great');
  await press('ArrowDown'); await page.waitForTimeout(150);
  await shot('7d-status-passives');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('7e-status-to-skills');
  await until('Skills { host: Hearth', '성장의 패시브 줄 → 화로의 기술 탭');
  await press('Tab'); await until('Equip { host: Hearth', '화로에서 Select = 화로 창의 장비 탭');
  // 탭마다 한 바퀴(0164): 장비(무기 칸에 서 있다 — → 같은 높이의 첫 장신구 칸, 0176) — 칸을 걷고 후보의 작은 목록(장신구 칸·무기 칸)을 열어 본다.
  // 장신구 칸은 종류마다 하나(0224 — 목걸이 · 반지 · 팔찌): 가진 것(단단한 껍질)은 팔찌라 셋째 칸에 끼워져 있다 — → 뒤 ↓ 둘.
  await shot('8a-equip');
  await press('ArrowRight'); await page.waitForTimeout(150);
  await shot('8b0-equip-neck-empty');
  await press('ArrowDown', 2); await page.waitForTimeout(150);
  await shot('8b-equip-charm');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('8c-equip-charm-pick');
  await press('ArrowUp'); await page.waitForTimeout(150);
  await shot('8d-equip-charm-pick-empty');
  await press('Escape'); await page.waitForTimeout(300);
  // 아래로 한 번 = 첫 칸(목걸이 — 그 종류는 아직 없다: "이 자리에 낄 것을 아직 얻지 못했다").
  await press('ArrowDown'); await page.waitForTimeout(150);
  await shot('8e-equip-empty-slot');
  // 덧옷 칸(0176 — 왼쪽 열 아래): 첫 장신구 칸 ← = 주무기 칸, ↓ 둘 = 보조무기 칸을 지나 덧옷 칸(뿌리껍질을 걸쳤다) → 목록(벗기 · 뿌리껍질), 벗기에서 견주는 줄.
  await press('ArrowLeft'); await page.waitForTimeout(150); await press('ArrowDown', 2); await page.waitForTimeout(150);
  await shot('8e2-equip-garb-slot');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('8e3-equip-garb-pick');
  await press('ArrowUp'); await page.waitForTimeout(150);
  await shot('8e4-equip-garb-pick-bare');
  await press('Escape'); await page.waitForTimeout(300);
  // 보조무기 칸(0223 → 0224 — 왼쪽 열: 주무기 · 보조무기 · 덧옷, 방패는 이 칸의 후보): 덧옷에서 ↑ → 목록(비우기 · 맨손 · 가진 무기 · 가진 방패).
  await press('ArrowUp'); await page.waitForTimeout(150);
  await shot('8e5-equip-sub-slot');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('8e6-equip-sub-pick');
  await press('Escape'); await page.waitForTimeout(300);
  await press('ArrowUp'); await page.waitForTimeout(150); await press('Enter'); await page.waitForTimeout(300);
  await shot('8f-equip-weapon-pick');
  await press('ArrowUp'); await page.waitForTimeout(150);
  await shot('8g-equip-weapon-pick-fists');
  await press('Escape'); await page.waitForTimeout(300);
  // 가방(0226 — 장비 다음 탭): 칸마다(첫 줄 여섯 + 빈 칸), 아이템의 작은 목록(화로의 가방 = 잔불로 줄, 0263 `Host::edits`), 맨 아랫줄 아래 "여벌 정리".
  await press('KeyE'); await until('Bag { host: Hearth', '장비 → 가방 탭(화로)');
  await shot('8n-bag-0');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('8n-bag-acts');
  await press('Escape'); await page.waitForTimeout(300);
  await press('ArrowDown', 5); await page.waitForTimeout(200);
  await shot('8n-bag-tidy');
  await press('Enter'); await page.waitForTimeout(400); // 심은 기록에는 여벌이 없다 — 묻는 창 없이 "정리할 여벌이 없다" 알림
  await shot('8n-bag-tidy-enter');
  // 기술 → 가게(0263 — 기록(도감·여정)은 화로에 없다: 아래 싸움에서 연 멈춤 창에서 모든 줄을 지난다).
  await press('KeyE'); await until('Skills { host: Hearth', '가방 → 기술 탭');
  // 가게(기술에서 E): 살 것의 줄마다(끝에서 돈다).
  await press('KeyE'); await until('Shop', '화로 → 가게 탭');
  await shot('8-shop');
  // 설정: 줄마다(0261 — 화로의 설정 = 멈춤의 설정: 끝에 개발 · 처음부터 · 타이틀로).
  await press('KeyE'); await until('Settings { host: Hearth', '가게 → 설정 탭');
  await shot('8j-settings');
  // 기술 탭(설정에서 Q 둘 — 가게를 지나): 탭 머리의 민트 점 = 남은 기술 점수로 올릴 것이 있다.
  await press('KeyQ', 2); await until('Skills { host: Hearth', '설정 → 기술 탭');
  // 기술 화면(0270 C2): 든 무기(검)의 판 — 한손검 — 에서 열리고, 처음 서는 칸 = 그 판에서 올릴 수 있는 첫 칸(0274 ② — 왼쪽 충격 갈래의 자루 치기).
  await until(/\[Class\(Sword\)\] @Skill\(Pommel\)$/, '기술 탭은 한손검 판의 자루 치기에서 연다');
  await shot('12-skills');
  // 판 줄: 맨 윗줄에서 ↑ → 판 줄(한손검, 오른쪽 위 금빛 점 = 든 무기의 판) → ← = 공통 판 → ↓ = 그 판의 맨 윗줄(왼쪽 위) → ↑ → 다시 한손검.
  await press('ArrowUp'); await until(/\[Class\(Sword\)\] @Strip$/, '자루 치기에서 ↑ = 판 줄');
  await shot('12-plate-strip-sword');
  await press('ArrowLeft'); await until(/\[Body\] @Strip$/, '판 줄에서 ← = 공통 판');
  await shot('12-plate-strip-body');
  await press('ArrowDown'); await until(/\[Body\] @Cell\(EvenBreath\)$/, '공통 판에서 ↓ = 왼쪽 위 칸');
  await shot('12-plate-body');
  await toPlate('Class(Sword)'); await press('ArrowDown');
  await until(/\[Class\(Sword\)\] @Skill\(Pommel\)$/, '한손검 판에서 ↓ = 맨 윗줄(왼쪽)');
  await goto('Skill(Whirl)'); await page.waitForTimeout(200);
  // 확인 = 작은 목록(올리기 · 칸 2에 끼우기) → 올리기 = 묻는 창(커서 "올리기") → 2단.
  await press('Enter'); await page.waitForTimeout(300);
  await shot('12a-skills-acts');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('12b-skills-ask');
  await press('Enter'); await page.waitForTimeout(400);
  // 산 직후: 상태 줄 자리에 알림("휘돌이 2단")이 잠깐.
  await shot('12c-skills-bought');
  await page.waitForTimeout(1800);
  // 한 번 더 → 3단(고르기가 열린다) → 목록의 끝 줄 = 고르기 바꾸기(값 없이, 묻지 않는다): ㄱ → ㄴ.
  await press('Enter'); await page.waitForTimeout(300); await press('Enter'); await page.waitForTimeout(300);
  await press('Enter'); await page.waitForTimeout(2200);
  await shot('12d-skills-rank3');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('12e-skills-acts-flip');
  await press('ArrowDown', 2); await press('Enter'); await page.waitForTimeout(400);
  await shot('12f-skills-flipped');
  let b = await record();
  if (!/skill\.rank\.whirl=3\n/.test(b) || !/skill\.pick\.whirl=b\n/.test(b)) errors.push(`skills not saved: ${JSON.stringify(b)}`);
  // 패시브 칸(공통 판 — 휘돌이 밑에서 이어지는 고른 호흡): 목록(열기) → 묻는 창 → 열림. 칸마다 따로 남는다(`tree.cells.beat`).
  await toPlate('Body'); await goto('Cell(EvenBreath)'); await page.waitForTimeout(200);
  await shot('12g-skills-cell');
  await press('Enter'); await page.waitForTimeout(300); await press('Enter'); await page.waitForTimeout(300);
  await shot('12h-skills-cell-ask');
  await press('Enter'); await page.waitForTimeout(400);
  await shot('12i-skills-cell-open');
  b = await record();
  if (!/tree\.cells\.beat=1\n/.test(b)) errors.push(`cell not saved: ${JSON.stringify(b)}`);
  await page.waitForTimeout(1800);
  // 판마다 판 줄에서 한 장(칸마다의 상세는 `cargo test`의 넘침 시험이 본다, 0275).
  await strips('13-strip');
  // 결의 칸(0223 11절 → 0270 C2 — 무기 판의 셋째 줄): 한손검 판은 충격의 결(0274 ② — 왼쪽 갈래). 그 판에 쓴 점수 6부터 — 지금 3
  // (휘돌이 3 — 0274 ③부터 가로지르기는 쌍단검 판): 확인하면 목록 없이 "이 판에 점수를 3 더 쓰면 열린다".
  await toPlate('Class(Sword)'); await goto('Vein(Shock)');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('13v-vein-plate-locked');
  await page.waitForTimeout(1600);
  // 자루 치기를 세 단 배워(0 → 3) 판의 셋째 줄을 연다 — 그 판에 쓴 점수 6, 충격의 스킬을 안다.
  await goto('Skill(Pommel)');
  for (let i = 0; i < 3; i++) { await press('Enter'); await page.waitForTimeout(300); await press('Enter'); await page.waitForTimeout(300); await press('Enter'); await page.waitForTimeout(1800); }
  b = await record();
  if (!/skill\.rank\.pommel=3\n/.test(b)) errors.push(`pommel not raised on the sword plate: ${JSON.stringify(b)}`);
  // 충격의 결: 목록(열기) → 묻는 창 → 열림(`tree.veins`).
  await goto('Vein(Shock)'); await page.waitForTimeout(200);
  await shot('13w-vein');
  await press('Enter'); await page.waitForTimeout(300); await press('Enter'); await page.waitForTimeout(300);
  await shot('13x-vein-ask');
  await press('Enter'); await page.waitForTimeout(400);
  await shot('13y-vein-open');
  b = await record();
  if (!/tree\.veins=shock\n/.test(b)) errors.push(`vein not saved: ${JSON.stringify(b)}`);
  await page.waitForTimeout(1800);
  // 울림의 결(0274 ② — 망치 판의 오른쪽): 고르기 둘(I 울림 모으기 · II 울려서 되찾기)이 상세에 나란히.
  await toPlate('Class(Hammer)'); await goto('Vein(Knell)'); await page.waitForTimeout(300);
  await shot('13z-knell-vein-picks');
  // 잠긴 줄(공통 판의 맺음 — 쓴 점수 합 22부터)의 칸은 확인해도 목록이 없다(안 됨 소리만).
  await toPlate('Body'); await goto('Key(Measure)'); await page.waitForTimeout(200);
  await press('Enter'); await page.waitForTimeout(300);
  await shot('14-skills-locked');
  // 되돌리기 단추(창 아래 오른쪽 — 맨 아래 줄에서 아래): 묻는 창(커서 "취소") → 왼쪽 = "되돌리기" → 쓴 점수 11(휘돌이 3 · 가로지르기 1 —
  // 옛 기록이 지닌 처음의 둘 0166 · 자루 치기 3 · 쟁반 뒤집기의 배움 2 0172 · 패시브 하나 · 결 하나)을 돌려받고 잔불 85(30 + 5 × 11).
  const embers = Number(/embers=(\d+)\n/.exec(b)[1]);
  await page.waitForTimeout(1600);
  await press('ArrowDown'); await until(/@Respec$/, '맺음 줄에서 ↓ = 되돌리기 단추');
  await shot('15-respec-focus');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('15a-respec-ask');
  await press('ArrowLeft'); await press('Enter'); await page.waitForTimeout(400);
  await shot('15b-respec-done');
  await page.waitForTimeout(1800);
  await shot('15c-skills-after-respec');
  b = await record();
  if (/skill\.rank\./.test(b) || /skill\.pick\./.test(b) || /tree\.cells\./.test(b) || /tree\.veins/.test(b)) errors.push(`respec must clear the board: ${JSON.stringify(b)}`);
  if (!new RegExp(`embers=${embers - 85}\\n`).test(b)) errors.push(`respec must cost 85 embers: ${JSON.stringify(b)}`);
  if (/ult\.rank\./.test(b)) errors.push(`respec must forget the great skill too (0172): ${JSON.stringify(b)}`);
  // 창을 닫으면 떠난다(한 번에) → 싸움(HUD 기력 막대가 길어졌는지).
  await close('화로 창 닫기');
  await shot('9-play-hud');
  // 멈춤 = 탭 다섯(장비 · 가방 · 기술 · 기록 · 설정, 0261 — 상태 탭은 화로에만, 그래서 옛 "멈춤의 상태 탭 읽기만" 장면은 없다).
  // 멈춤은 이 판에서 마지막에 본 탭에서 열리니 Select(Tab)로 장비 탭부터 연다(0263부터 화로 위에 멈춤 창을 열지 않아 처음이라 어차피 장비).
  await press('Tab'); await until('Equip { host: Pause', '싸움에서 Select = 장비 탭');
  // 장비: 지금의 수치(레벨 줄까지 — 최대 기력·기력 회복·평타·강타·자세, 0261)가 상태 줄에 더해져도 칸 안에(넘치면 굴린다).
  await shot('10-equip');
  // 가방(0226): 싸움에서 연 멈춤에서는 "여벌 정리" 단추가 없다.
  await press('KeyE'); await until('Bag { host: Pause', '멈춤: 가방 탭');
  await shot('16-pause-bag');
  // 기술: 같은 화면, 끼우기만(되돌리기 단추 없음).
  await press('KeyE'); await until('Skills { host: Pause', '멈춤: 기술 탭');
  await until(/\[Class\(Sword\)\]/, '멈춤의 기술 탭도 든 무기의 판'); await shot('16-pause-skills');
  // 되돌린 뒤라 아는 스킬이 없다(점수는 있다): 안 배운 칸에서 확인 — 목록이 없고 상태 줄 자리에 "올리기는 쉼터 화로에서만" 알림(0173).
  // 여기서 Escape를 누르면 목록이 아니라 창이 닫힌다(예전 길이 그래서 뒤의 도감·설정을 싸움 화면에서 찍고 있었다) — 누르지 않는다.
  await home(); await press('ArrowRight'); await press('Enter'); await page.waitForTimeout(300);
  await shot('16a-pause-skills-away');
  await page.waitForTimeout(1600);
  // 기록(기술에서 E, 0261 도감 → 기록 — 0263부터 멈춤에만): 첫 묶음 여정(0265 — 맨 앞) + 도감 묶음 넷(좌우)의 모든 줄.
  // 여정: 줄마다. 심은 기록은 성소의 문지기를 넘었다(core.warden=1) — 둘째 줄 "성소의 문지기를 넘어라"를
  // 받으면(확인) 기록에 journey=s.warden, 잔불 +60. 마을의 목표도 보인다.
  await press('KeyE'); await until('Codex { host: Pause, tab: 0', '멈춤: 기록 탭 = 여정 묶음');
  await shot('8m-journey');
  // 한 줄 아래가 문지기 목표.
  await press('ArrowDown'); await page.waitForTimeout(150);
  {
    const before = Number(/embers=(\d+)\n/.exec(await record())[1]);
    await press('Enter'); await page.waitForTimeout(400);
    await shot('8m-journey-taken');
    const j = await record();
    if (!/journey=s\.warden\n/.test(j)) errors.push(`journey: taking a goal must be saved: ${JSON.stringify(j)}`);
    const after = Number(/embers=(\d+)\n/.exec(j)[1]);
    if (after !== before + 60) errors.push(`journey: the warden goal gives 60 embers (${before} → ${after})`);
    await press('Enter'); await page.waitForTimeout(300);
    if (!/journey=s\.warden\n/.test(await record())) errors.push('journey: a taken goal is taken once');
  }
  await press('ArrowRight'); await until('Codex { host: Pause, tab: 1', '멈춤: 기록 탭의 적 묶음');
  await shot('8h-codex-foe');
  // 장신구 묶음: 첫 줄(가진 껍질 부적)의 보상을 받는다(0265 — 확인 = 받기, 잔불 +15, 한 번만).
  await press('ArrowRight'); await until('Codex { host: Pause, tab: 2', '멈춤: 기록 탭의 장신구 묶음');
  {
    await shot('8i-codex-charm-bounty');
    const before = Number(/embers=(\d+)\n/.exec(await record())[1]);
    await press('Enter'); await page.waitForTimeout(400);
    await shot('8i-codex-charm-taken');
    const j = await record();
    if (!/codex\.paid=charm\.shell\n/.test(j)) errors.push(`codex: taking a bounty must be saved: ${JSON.stringify(j)}`);
    const after = Number(/embers=(\d+)\n/.exec(j)[1]);
    if (after !== before + 15) errors.push(`codex: an owned charm gives 15 embers (${before} → ${after})`);
    await press('Enter'); await page.waitForTimeout(300);
    if (Number(/embers=(\d+)\n/.exec(await record())[1]) !== after) errors.push('codex: a bounty is taken once');
  }
  // 덧옷(0176) · 무기(0191 — 맨손은 없다, 0265).
  await press('ArrowRight'); await until('Codex { host: Pause, tab: 3', '멈춤: 기록 탭의 덧옷 묶음');
  await shot('8k-codex-garb');
  await press('ArrowRight'); await until('Codex { host: Pause, tab: 4', '멈춤: 기록 탭의 무기 묶음');
  await shot('8l-codex-weapon');
  // 설정(끝에 개발 · 처음부터 · 타이틀로): 첫 줄에서 ↑ = 끝 줄(타이틀로 — 목록은 돈다) → 묻는 창을 열어 보고 취소.
  await press('KeyE'); await until('Settings { host: Pause', '멈춤: 설정 탭');
  await shot('18-pause-settings');
  await press('ArrowUp'); await press('Enter'); await until(/Ask/, '멈춤: 타이틀로 → 묻는 창');
  await shot('18a-pause-to-title-ask');
  await press('Escape'); await until(/host: Pause/, '묻는 창 → 설정 탭');
  await close('멈춤 닫기');
  // 새 판(0166 → 0167 → 0169): 스킬 칸 둘이 비어 있다. 튜토리얼의 몸 걸음 넷을 마친 상으로 기술 점수 하나를 들고 있다
  // (기록의 몫 `skill.gift=1`) — 쉬면 기술 탭에서 열리고(아는 스킬이 없고 배울 점수가 있는 동안만), 배우면 빈 칸에 저절로
  // 끼워진다. 둘째 점수는 첫 레벨에서(잔불 50).
  const boot = async (text) => {
    // 게임이 도는 쪽에서 심으면 다시 띄우기 전에 게임이 제 기록을 덮어쓸 수 있다 — 같은 출처의 빈 쪽으로 나가서 심는다.
    await page.goto(`http://localhost:${PORT}/manifest.webmanifest`);
    await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, text]);
    await page.goto(`http://localhost:${PORT}/`);
    await ready();
    await page.waitForTimeout(600);
    await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
    await toPlay();
  };
  await boot(FRESH);
  await shot('30-fresh-hud');
  // 빈 칸은 눌러도 아무 일 없다(Q·E = 스킬 둘).
  await press('KeyQ'); await page.waitForTimeout(300);
  await shot('30a-fresh-press-empty');
  await rest('새 판: 화로'); await until('Skills { host: Hearth', '새 판: 화로 = 기술 탭');
  await shot('31-fresh-rest-skills');
  // 처음 서는 칸 = 든 무기의 판(이 기록은 검을 지닌다 — 맨손이면 공통 판, 0272)에서 살 수 있는 첫 칸(0274 ② — 자루 치기): 목록(배우기뿐) → 묻는 창 → 배움.
  await until(/\[Class\(Sword\)\] @Skill\(Pommel\)$/, '새 판: 한손검 판의 자루 치기');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('31a-fresh-acts');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('31b-fresh-ask');
  await press('Enter'); await page.waitForTimeout(400);
  await shot('31c-fresh-learned');
  b = await record();
  if (!/skill\.rank\.pommel=1\n/.test(b) || !/skill\.gift=1\n/.test(b)) errors.push(`fresh: pommel not learned: ${JSON.stringify(b)}`);
  await page.waitForTimeout(1800);
  // 한 단 올린다(Q 셋 = 가방·장비를 지나 성장 탭, 0263, 잔불 50 → 기술 점수 +1) → 기술 탭(한손검 판)의 버팀 → 남은 빈 칸(칸 2)에.
  await press('KeyQ', 3); await until('Status { host: Hearth', '새 판: 기술 → 성장 탭');
  await press('Enter'); await page.waitForTimeout(300); await press('Enter'); await page.waitForTimeout(500);
  await shot('31d-fresh-level-one');
  b = await record();
  if (!/lv\.life=1\n/.test(b) || !/embers=80\n/.test(b)) errors.push(`fresh: the first level costs 50: ${JSON.stringify(b)}`);
  await press('KeyE', 3); await until('Skills { host: Hearth', '새 판: 성장 → 기술 탭');
  await goto('Skill(Brace)'); await page.waitForTimeout(200);
  await press('Enter'); await page.waitForTimeout(300); await press('Enter'); await page.waitForTimeout(300);
  await press('Enter'); await page.waitForTimeout(2200);
  await shot('31e-fresh-second');
  b = await record();
  if (!/skill\.rank\.brace=1\n/.test(b) || !/skill\.2=brace\n/.test(b)) errors.push(`fresh: the second skill must fill slot 2: ${JSON.stringify(b)}`);
  if (/skill\.rank\.cut=/.test(b)) errors.push(`fresh: cut was never learned: ${JSON.stringify(b)}`);
  await close('새 판: 화로 창 닫기');
  await shot('32-fresh-hud-two');
  // 멈춤의 기술 탭(끼우기만, 0173): 멈춤(장비 탭) → E 둘 = 기술(가방을 지나). 배운 자루 치기(칸 1)에서 확인 = "칸 2에 끼우기" 목록.
  await pause('새 판: 멈춤'); await until('Equip { host: Pause', '새 판: 멈춤 = 장비 탭');
  await press('KeyE', 2); await until('Skills { host: Pause', '새 판: 멈춤의 기술 탭');
  await goto('Skill(Pommel)'); await press('Enter'); await page.waitForTimeout(300);
  await shot('32a-fresh-pause-acts');
  await press('Escape'); await page.waitForTimeout(300);
  await close('새 판: 멈춤 닫기');
  // 다시 쉬면 성장 탭(아는 스킬이 있다).
  await rest('새 판: 다시 쉬기'); await until('Status { host: Hearth', '새 판: 다시 쉬면 성장 탭');
  await shot('33-fresh-rest-status');
  await press('Escape'); await page.waitForTimeout(300);
  // 0272 맨손: 무기 판이 없다 — 첫 쉼의 기술 탭은 공통 판에서 열린다(한손검의 판을 빌려 쓰지 않는다), 판 줄에 금빛 점이 없다.
  await boot(FRESH.replace('weapon=sword\n', 'weapon=fists\n'));
  await rest('맨손: 화로'); await until('Skills { host: Hearth', '맨손: 화로 = 기술 탭');
  await until(/\[Body\] @/, '맨손: 기술 탭은 공통 판에서 연다');
  await shot('34-bare-common');
  await press('Tab'); await until('Equip { host: Hearth', '맨손: 장비 탭'); await shot('34a-bare-equip');
  await press('Escape'); await page.waitForTimeout(300);
  await ctx.close();
  return { errors, checks };
}

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(PORT, r));
  const browser = await chromium.launch();
  let bad = false;
  // 언어 하나만 돌리려면 `HEARTH_LANGS=en`.
  for (const lang of (process.env.HEARTH_LANGS || 'ko,en').split(',')) {
    const { errors, checks } = await run(browser, lang);
    for (const e of errors) { console.error(`[${lang}] ERROR ${e}`); bad = true; }
    for (const c of checks) { console.error(`[${lang}] ${c}`); bad = true; }
    console.log(`[${lang}] errors ${errors.length}, ui-check ${checks.length}`);
  }
  await browser.close();
  server.close();
  process.exitCode = bad ? 1 : 0;
}

// 도중에 멈추면 브라우저가 열린 채라 node가 끝나지 않는다(2026-11-21 — 실패한 판이 두 시간을 붙들었다) — 바로 끝낸다.
main().catch((e) => { console.error(e); process.exit(1); });
