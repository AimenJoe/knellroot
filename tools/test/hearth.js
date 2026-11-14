// 큰 창(화로·멈춤) 시험(0104 → 0164 "창 하나 + 탭"): 헤드리스 Chromium으로 한국어·영어 두 판 —
//   쉼터 기록(잔불 넉넉)을 심고 이어하기 → 쉬기 → 창(상태 탭 — 레벨 줄 셋을 차례로 올려 보고 묻는 창, 기술 줄에서 확인 = 기술 탭의
//   그 칸, 0173) → 탭마다(E·Q로 넘긴다):
//   장비(칸·후보의 작은 목록) · 도감(묶음 둘의 모든 줄) · 가게 · 설정(모든 줄) · 기술, 그리고 멈춤의 같은 창(탭 다섯), 화면마다 스크린숏.
//   기술 탭(0161 — 옛 "나무" 줄과 기술 판이 하나가 되었다): 칸 그림 격자(갈래 셋 × 줄 셋) + 오른쪽 상세.
//   확인 = 그 칸에 할 수 있는 일의 작은 목록(올리기·열기 / 끼우기 / 고르기 바꾸기) → 사는 일은 묻는 창. 한 단 사기 · 3단 고르기 뒤집기 ·
//   패시브 칸 열기 · 모든 칸의 상세 · 잠긴 칸 · 되돌리기(묻는 창)를 지나고, 멈춤의 기술 화면(끼우기만)도 본다.
//   그다음 단을 미리 찍은 기록 열(스킬 여덟이라 점수 23으로는 한 기록에 다 못 올린다 — 첫 넷·새 넷을 다른 단으로 짝지어
//   2단 · 3·4·5단 × ㄱ/ㄴ을 모두, 큰 기술 셋의 단도)으로 화로·멈춤의 모든 칸을 돌며 상세 칸의 넘침을 본다.
//   끝으로 새 판의 기록(0166·0167·0169 — 아는 스킬 없음, 튜토리얼의 상 한 점): 빈 스킬 칸 → 쉬기 = 기술 탭 → 배움 = 빈 칸에 끼워짐 → 첫 레벨 = 둘째 점수.
//   개발판의 F4 넘침 검사(`ui::check`)를 켜 두고 콘솔의 `[ui-check]` 줄(켬/끔 말고)이 있으면 실패(CLAUDE.md "새 화면은 F4").
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/hearth.js dist shots
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8137;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const KEY = 'knell.save.0';
// 문 앞 쉼터(0)에서 쉰 기록(뿌리껍질 덧옷을 걸쳤다 — 0176), 잔불은 레벨 몇 단 + 가게 + 기술 되돌리기를 볼 만큼. 기력·힘은 이미 4단(다음이 마지막 단 — "다 올림" 줄·평타 +1도 본다).
const SEED = 'v=1\nembers=700\nclears=1\nrun.rest=0\nrun.gate=1\ncharm.owned=shell\ncharm.worn=shell,,\nlv.life=1\nlv.stamina=4\nlv.might=4\ncore.warden=1\nfinds=sanctuary.garb.1,sanctuary.shard.0,sanctuary.stone.0,village.stone.0\nforge.sword=1\ngarb.worn=bark\nmigrated.v04=1\n';
// 단을 미리 찍은 기록(0158 → 0161): 점수 23(레벨 15 + 엘리트 6 + 핵 둘). 첫 넷(휘돌이·가로지르기·끌어모으기·땅울림)과
// 새 넷(버팀·되울림·던지기·잔불 흩기, G2)을 따로 같은 단·같은 고르기에, 큰 기술은 (뒤집기, 몰아치기, 한 줄기)의 단.
// 기술 화면의 상세는 다섯 단의 줄과 고르기 둘을 늘 다 보여 주므로(0161) 단마다 글이 갈리지 않는다 — 기록은 상태 줄과 표지가
// 갈리는 넷이면 된다: 3단(고르기 ㄱ/ㄴ) · 4단 + 큰 기술 셋 · 5단(다 올림) · 새 넷의 5단 + 몰아치기.
const RICH = 'v=1\nembers=600\nclears=1\nrun.rest=0\nrun.gate=1\nlv.life=5\nlv.stamina=5\nlv.might=5\ncore.warden=1\ncore.bellkeeper=1\n'
  + 'elites=t.elite.0,t.elite.1,t.elite.2,t.elite.3,t.elite.4,t.elite.5\nmigrated.v04=1\n';
// 새 판의 기록(0166·0167·0169 — `migrated.v06`이 있다: 처음의 둘을 받지 않는다). 튜토리얼의 몸 걸음 넷을 마쳐 상(기술 점수 1)을
// 받은 참이다. 문 앞 쉼터에 서 있고 잔불은 첫 레벨 값보다 넉넉히.
const FRESH = 'v=1\nembers=130\nrun.rest=0\nrun.gate=0\nplay=1\nweapons=\nmigrated.v04=1\nmigrated.v06=1\nmigrated.ult=1\ntut.step=4\nskill.gift=1\n';
const set = (keys, rank, pick) => keys.map((k) =>
  (rank > 0 ? `skill.rank.${k}=${rank}\n` : '') + (rank >= 3 ? `skill.pick.${k}=${pick}\n` : '')).join('');
const OLD = ['whirl', 'cut', 'gather', 'quake'];
const NEW = ['brace', 'echo', 'throw', 'embers'];
const board = ([ro, po], [rn, pn], [upend, flurry, line]) => RICH + set(OLD, ro, po) + set(NEW, rn, pn)
  + `ult.rank.upend=${upend}\n` + (flurry ? `ult.rank.flurry=${flurry}\n` : '') + (line ? `ult.rank.line=${line}\n` : '');
const BOARDS = [
  ['o3a-n3b', board([3, 'a'], [3, 'b'], [1, 0, 0])], ['o4b-n1-u', board([4, 'b'], [1, 'a'], [1, 1, 1])],
  ['o5a-n1', board([5, 'a'], [1, 'a'], [1, 0, 0])], ['n5b-f1', board([1, 'a'], [5, 'b'], [1, 1, 0])]];

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
  const press = async (key, n = 1) => { for (let i = 0; i < n; i++) { await page.keyboard.press(key); await page.waitForTimeout(90); } };
  // 기술 화면의 걸음(0161): 가장자리에서 돌지 않는다 — 왼쪽·위로 넉넉히 누르면 왼쪽 위 칸(휘돌이).
  const home = async () => { await press('ArrowLeft', 6); await press('ArrowUp', 6); };
  // 모든 칸을 지난다: 보이는 줄 다섯(스킬 넷 · 패시브 여섯 · 스킬 넷 · 패시브 여섯 · 큰 기술과 패시브 여섯)을 왼쪽에서 오른쪽으로.
  // 오른쪽 끝에서 더 누르지 않는다(화로에서는 거기서 되돌리기 단추로 간다). `shots` = 찍을 줄(없으면 모두) — 줄마다 칸 하나씩 돌려 가며.
  // 끝나면 맨 아래 줄의 왼쪽 칸(몰아치기)에 선다.
  const sweep = async (name, shots) => {
    await home();
    const COLS = [4, 6, 4, 6, 6];
    for (let row = 0; row < COLS.length; row++) {
      for (let col = 0; col < COLS[row]; col++) {
        if (!shots || shots.includes(row)) await shot(`${name}-r${row}c${col}`);
        if (col + 1 < COLS[row]) { await press('ArrowRight'); await page.waitForTimeout(120); }
      }
      await press('ArrowLeft', 6);
      if (row + 1 < COLS.length) { await press('ArrowDown'); await page.waitForTimeout(120); }
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
  await press('F4'); // 넘침 검사 켬(개발판)
  // 타이틀(이어하기·시작하기·설정·개발) → 설정 → 언어(첫 줄): 영어 판이면 → 한 번(한국어 → English).
  if (lang === 'en') {
    await press('ArrowDown', 2); await press('Enter'); await page.waitForTimeout(300);
    await press('ArrowRight'); await page.waitForTimeout(200);
    await shot('0-title-settings');
    await press('Escape'); await page.waitForTimeout(300);
    await press('ArrowUp', 2);
  }
  // 이어하기 → 슬롯 창 → 이어하기 → 문 앞 쉼터.
  await press('Enter'); await page.waitForTimeout(300);
  await press('Enter'); await page.waitForTimeout(1500);
  // 쉬기 → 큰 창(0164)이 상태 탭에서 열린다. 탭은 E(오른쪽)·Q(왼쪽)로 넘긴다 — 상태 · 장비 · 기술 · 도감 · 가게 · 설정.
  await press('KeyF'); await page.waitForTimeout(600);
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
  // 기술 탭의 그 칸으로(탭 사이의 이음). 거기서 Q = 장비 탭(무기 칸).
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
  await press('KeyQ'); await page.waitForTimeout(300);
  // 탭마다 한 바퀴(0164): 장비(무기 칸에 서 있다 — → 같은 높이의 첫 장신구 칸, 0176) — 칸을 걷고 후보의 작은 목록(장신구 칸·무기 칸)을 열어 본다.
  await shot('8a-equip');
  await press('ArrowRight'); await page.waitForTimeout(150);
  await shot('8b-equip-charm');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('8c-equip-charm-pick');
  await press('ArrowUp'); await page.waitForTimeout(150);
  await shot('8d-equip-charm-pick-empty');
  await press('Escape'); await page.waitForTimeout(300);
  await press('ArrowDown'); await page.waitForTimeout(150);
  await shot('8e-equip-empty-slot');
  // 덧옷 칸(0176 — 무기 밑): 가운데 장신구 칸 ← = 무기 칸, ↓ = 덧옷 칸(뿌리껍질을 걸쳤다) → 목록(벗기 · 뿌리껍질), 벗기에서 견주는 줄.
  await press('ArrowLeft'); await page.waitForTimeout(150); await press('ArrowDown'); await page.waitForTimeout(150);
  await shot('8e2-equip-garb-slot');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('8e3-equip-garb-pick');
  await press('ArrowUp'); await page.waitForTimeout(150);
  await shot('8e4-equip-garb-pick-bare');
  await press('Escape'); await page.waitForTimeout(300);
  await press('ArrowUp'); await page.waitForTimeout(150); await press('Enter'); await page.waitForTimeout(300);
  await shot('8f-equip-weapon-pick');
  await press('ArrowUp'); await page.waitForTimeout(150);
  await shot('8g-equip-weapon-pick-fists');
  await press('Escape'); await page.waitForTimeout(300);
  // 도감(장비에서 E 둘 — 기술을 지나): 묶음 셋(좌우)의 모든 줄.
  await press('KeyE', 2); await page.waitForTimeout(300);
  for (let i = 0; i < 6; i++) { await shot(`8h-codex-foe-${i}`); await press('ArrowDown'); await page.waitForTimeout(120); }
  await press('ArrowRight'); await page.waitForTimeout(300);
  for (let i = 0; i < 9; i++) { await shot(`8i-codex-charm-${i}`); await press('ArrowDown'); await page.waitForTimeout(120); }
  // 셋째 묶음 덧옷(0176).
  await press('ArrowRight'); await page.waitForTimeout(300);
  for (let i = 0; i < 3; i++) { await shot(`8k-codex-garb-${i}`); await press('ArrowDown'); await page.waitForTimeout(120); }
  // 여정(0183 — 도감 다음 탭): 줄마다. 심은 기록은 성소의 문지기를 넘었다(core.warden=1) — 둘째 줄 "성소의 문지기를 넘어라"를
  // 받으면(확인) 기록에 journey=s.warden, 잔불 +60. 마을의 목표도 보인다.
  await press('KeyE'); await page.waitForTimeout(300);
  for (let i = 0; i < 10; i++) { await shot(`8m-journey-${i}`); await press('ArrowDown'); await page.waitForTimeout(120); }
  // 열 줄을 다 지나면 목록은 첫 줄로 돈다 — 한 줄 아래가 문지기 목표.
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
  // 가게: 살 것의 줄마다(끝에서 돈다).
  await press('KeyE'); await page.waitForTimeout(300);
  // 진열 다섯 + 벼리기 넷 + 거두기 하나(0180 — 숫돌 둘·벼린 검 +1을 심었다).
  for (let i = 0; i < 10; i++) { await shot(`8-shop-${i}`); await press('ArrowDown'); await page.waitForTimeout(120); }
  // 설정: 줄마다(화로에서는 처음부터·타이틀로가 없다).
  await press('KeyE'); await page.waitForTimeout(300);
  for (let i = 0; i < 11; i++) { await shot(`8j-settings-${i}`); await press('ArrowDown'); await page.waitForTimeout(120); }
  // 기술 탭(설정에서 Q 넷 — 가게 · 여정 · 도감을 지나): 탭 머리의 민트 점 = 남은 기술 점수로 올릴 것이 있다.
  await press('KeyQ', 4); await page.waitForTimeout(300);
  // 기술 화면: 처음 서는 칸 = 올릴 수 있는 첫 칸(휘돌이). 왼쪽 위로 가도 같은 칸 — 가장자리에서 돌지 않는다.
  await shot('12-skills');
  await home();
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
  // 패시브 칸(휘돌이 아래 — 고른 호흡): 목록(열기) → 묻는 창 → 열림. 칸마다 따로 남는다(`tree.cells.beat`).
  await press('ArrowDown'); await page.waitForTimeout(200);
  await shot('12g-skills-cell');
  await press('Enter'); await page.waitForTimeout(300); await press('Enter'); await page.waitForTimeout(300);
  await shot('12h-skills-cell-ask');
  await press('Enter'); await page.waitForTimeout(400);
  await shot('12i-skills-cell-open');
  b = await record();
  if (!/tree\.cells\.beat=1\n/.test(b)) errors.push(`cell not saved: ${JSON.stringify(b)}`);
  await page.waitForTimeout(1800);
  // 모든 칸의 상세(줄 다섯 — 스킬 넷 · 패시브 여섯 · 스킬 넷 · 패시브 여섯 · 큰 기술과 패시브 여섯).
  await sweep('13-skills');
  // 잠긴 줄(셋째 — 쓴 점수 8부터)의 칸은 확인해도 목록이 없다(안 됨 소리만).
  await press('Enter'); await page.waitForTimeout(300);
  await shot('14-skills-locked');
  // 되돌리기 단추(창 아래 오른쪽 — 맨 아래 줄에서 아래): 묻는 창(커서 "취소") → 왼쪽 = "되돌리기" → 쓴 점수 7(산 것 셋 + 옛 기록이 지닌 처음의 둘 0166 + 쟁반 뒤집기의 배움 2 0172)을 돌려받고 잔불 65(30 + 5 × 7).
  const embers = Number(/embers=(\d+)\n/.exec(b)[1]);
  await press('ArrowDown'); await page.waitForTimeout(200);
  await shot('15-respec-focus');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('15a-respec-ask');
  await press('ArrowLeft'); await press('Enter'); await page.waitForTimeout(400);
  await shot('15b-respec-done');
  await page.waitForTimeout(1800);
  await shot('15c-skills-after-respec');
  b = await record();
  if (/skill\.rank\./.test(b) || /skill\.pick\./.test(b) || /tree\.cells\./.test(b)) errors.push(`respec must clear the board: ${JSON.stringify(b)}`);
  if (!new RegExp(`embers=${embers - 65}\\n`).test(b)) errors.push(`respec must cost 65 embers: ${JSON.stringify(b)}`);
  if (/ult\.rank\./.test(b)) errors.push(`respec must forget the great skill too (0172): ${JSON.stringify(b)}`);
  // 창을 닫으면 떠난다(한 번에) → 싸움(HUD 기력 막대가 길어졌는지).
  await press('Escape'); await page.waitForTimeout(400);
  await shot('9-play-hud');
  // 멈춤 = 같은 창(탭 다섯 — 가게가 없다), 상태 탭에서 열린다: 읽기만(확인해도 묻는 창이 없다).
  await press('Escape'); await page.waitForTimeout(300);
  await shot('10a-pause-status');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('10b-pause-status-deny');
  // 장비: 레벨 줄(최대 기력·기력 회복·평타·강타·자세)이 상태 줄에 더해져도 칸 안에(넘치면 굴린다).
  await press('KeyE'); await page.waitForTimeout(400);
  await shot('10-equip');
  // 기술: 같은 화면, 끼우기만(되돌리기 단추 없음).
  await press('KeyE'); await page.waitForTimeout(400);
  await shot('16-pause-skills');
  // 되돌린 뒤라 아는 스킬이 없다(점수는 있다): 안 배운 칸에서 확인 — 목록이 없고 상태 줄 자리에 "올리기는 쉼터 화로에서만" 알림(0173).
  // 여기서 Escape를 누르면 목록이 아니라 창이 닫힌다(예전 길이 그래서 뒤의 도감·설정을 싸움 화면에서 찍고 있었다) — 누르지 않는다.
  await home(); await press('ArrowRight'); await press('Enter'); await page.waitForTimeout(300);
  await shot('16a-pause-skills-away');
  await page.waitForTimeout(1600);
  // 도감 → 설정(멈춤에서는 끝에 개발 · 처음부터 · 타이틀로): 줄마다, 끝 줄에서 묻는 창을 열어 보고 취소.
  await press('KeyE'); await page.waitForTimeout(300);
  await shot('17-pause-codex');
  await press('KeyE'); await page.waitForTimeout(300);
  await shot('17a-pause-journey');
  await press('KeyE'); await page.waitForTimeout(300);
  for (let i = 0; i < 13; i++) { await shot(`18-pause-settings-${i}`); await press('ArrowDown'); await page.waitForTimeout(120); }
  await press('ArrowUp'); await press('Enter'); await page.waitForTimeout(300);
  await shot('18a-pause-to-title-ask');
  await press('Escape'); await page.waitForTimeout(300);
  await press('Escape'); await page.waitForTimeout(300);
  // 모든 단(0158): 단을 미리 찍은 기록마다 — 화로의 기술 화면에서 모든 칸의 상세를 지나고, 멈춤의 기술 화면에서도 한 번 더
  // (멈춤은 상태 줄·목록이 다르다 — 끼우기만).
  for (const [name, text] of BOARDS) {
    // 게임이 도는 쪽에서 심으면 다시 띄우기 전에 게임이 제 기록을 덮어쓸 수 있다 — 같은 출처의 빈 쪽으로 나가서 심는다.
    await page.goto(`http://localhost:${PORT}/manifest.webmanifest`);
    await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, text]);
    await page.goto(`http://localhost:${PORT}/`);
    await ready();
    await page.waitForTimeout(600);
    await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
    await press('F4');
    await press('Enter'); await page.waitForTimeout(300);
    await press('Enter'); await page.waitForTimeout(1500);
    await press('KeyF'); await page.waitForTimeout(600);
    await shot(`20-${name}-status`);
    await press('KeyE', 2); await page.waitForTimeout(300);
    await shot(`20-${name}-skills`);
    await sweep(`20-${name}`, [0, 4]);
    await press('Escape'); await page.waitForTimeout(400);
    // 멈춤(다시 띄운 판이라 상태 탭) → E 둘 = 기술.
    await press('Escape'); await page.waitForTimeout(300);
    await press('KeyE', 2); await page.waitForTimeout(300);
    await shot(`21-${name}-pause-skills`);
    // 배운 스킬(휘돌이 — 칸 1에 끼움)에서 확인: "칸 2에 끼우기" + 올릴 수 있으면 끝에 흐린 "올리기 — 쉼터 화로에서"(0173) → 닫기.
    await home(); await press('Enter'); await page.waitForTimeout(300);
    await shot(`21-${name}-pause-acts`);
    await press('Escape'); await page.waitForTimeout(300);
    // 멈춤의 상세는 상태 줄만 다르다("쉼터 화로에서 올린다") — 모든 칸은 첫 기록에서 한 번만 돈다.
    if (name === BOARDS[0][0]) await sweep(`21-${name}-pause`, []); else await press('ArrowDown', 4);
    // 큰 기술 칸에서 확인(배웠고 끼우지 않았으면 "끼우기" 목록, 아니면 아무 일 없음) → 닫기.
    await press('Enter'); await page.waitForTimeout(250);
    await shot(`21-${name}-pause-ult`);
    await press('Escape'); await page.waitForTimeout(300);
  }
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
    await press('F4');
    await press('Enter'); await page.waitForTimeout(300);
    await press('Enter'); await page.waitForTimeout(1500);
  };
  await boot(FRESH);
  await shot('30-fresh-hud');
  // 빈 칸은 눌러도 아무 일 없다(Q·E = 스킬 둘).
  await press('KeyQ'); await page.waitForTimeout(300);
  await shot('30a-fresh-press-empty');
  await press('KeyF'); await page.waitForTimeout(600);
  await shot('31-fresh-rest-skills');
  // 처음 서는 칸 = 살 수 있는 첫 칸(휘돌이): 목록(배우기뿐) → 묻는 창 → 배움.
  await press('Enter'); await page.waitForTimeout(300);
  await shot('31a-fresh-acts');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('31b-fresh-ask');
  await press('Enter'); await page.waitForTimeout(400);
  await shot('31c-fresh-learned');
  b = await record();
  if (!/skill\.rank\.whirl=1\n/.test(b) || !/skill\.gift=1\n/.test(b)) errors.push(`fresh: whirl not learned: ${JSON.stringify(b)}`);
  await page.waitForTimeout(1800);
  // 한 단 올린다(Q 둘 = 상태 탭, 잔불 50 → 기술 점수 +1) → 기술 탭에서 오른쪽으로 둘째 칸(맞받기 갈래의 버팀) → 남은 빈 칸(칸 2)에.
  await press('KeyQ', 2); await page.waitForTimeout(300);
  await press('Enter'); await page.waitForTimeout(300); await press('Enter'); await page.waitForTimeout(500);
  await shot('31d-fresh-level-one');
  b = await record();
  if (!/lv\.life=1\n/.test(b) || !/embers=80\n/.test(b)) errors.push(`fresh: the first level costs 50: ${JSON.stringify(b)}`);
  await press('KeyE', 2); await page.waitForTimeout(300);
  await home(); await press('ArrowRight', 2); await page.waitForTimeout(200);
  await press('Enter'); await page.waitForTimeout(300); await press('Enter'); await page.waitForTimeout(300);
  await press('Enter'); await page.waitForTimeout(2200);
  await shot('31e-fresh-second');
  b = await record();
  if (!/skill\.rank\.brace=1\n/.test(b) || !/skill\.2=brace\n/.test(b)) errors.push(`fresh: the second skill must fill slot 2: ${JSON.stringify(b)}`);
  if (/skill\.rank\.cut=/.test(b)) errors.push(`fresh: cut was never learned: ${JSON.stringify(b)}`);
  await press('Escape'); await page.waitForTimeout(400);
  await shot('32-fresh-hud-two');
  // 다시 쉬면 상태 탭(아는 스킬이 있다).
  await press('KeyF'); await page.waitForTimeout(600);
  await shot('33-fresh-rest-status');
  await press('Escape'); await page.waitForTimeout(300);
  await ctx.close();
  return { errors, checks };
}

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(PORT, r));
  const browser = await chromium.launch();
  let bad = false;
  for (const lang of ['ko', 'en']) {
    const { errors, checks } = await run(browser, lang);
    for (const e of errors) { console.error(`[${lang}] ERROR ${e}`); bad = true; }
    for (const c of checks) { console.error(`[${lang}] ${c}`); bad = true; }
    console.log(`[${lang}] errors ${errors.length}, ui-check ${checks.length}`);
  }
  await browser.close();
  server.close();
  process.exitCode = bad ? 1 : 0;
}

main().catch((e) => { console.error(e); process.exitCode = 1; server.close(); });
