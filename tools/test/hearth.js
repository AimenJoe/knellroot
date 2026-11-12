// 화로·레벨 화면 시험(0104, v0.4.0 G1): 헤드리스 Chromium으로 한국어·영어 두 판 —
//   쉼터 기록(잔불 넉넉)을 심고 이어하기 → 쉬기 → 화로 → 레벨(줄 셋을 차례로 올려 보고 묻는 창) → 가게, 화면마다 스크린숏.
//   기술 판(0158 G0·G1): 화로 넷째 줄 "기술" → 한 단 사기(묻는 창) → 3단 고르기 뒤집기 → 줄마다 설명 → 되돌리기(묻는 창).
//   그다음 단을 미리 찍은 기록 열(0158 G2: 스킬 여덟이라 점수 23으로는 한 기록에 다 못 올린다 — 첫 넷·새 넷을 다른 단으로 짝지어
//   2단 · 3·4·5단 × ㄱ/ㄴ을 모두, 큰 기술 셋의 단도)으로 판의 열세 줄과 멈춤의 기술 창(칸 1 · 큰 기술 줄)을 돌며 설명 칸의 넘침을 본다.
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
// 문 앞 쉼터(0)에서 쉰 기록, 잔불은 레벨 몇 단 + 가게 + 기술 되돌리기(40)를 볼 만큼. 기력·힘은 이미 4단(다음이 마지막 단 — "다 올림" 줄·평타 +1도 본다).
const SEED = 'v=1\nembers=700\nclears=1\nrun.rest=0\nrun.gate=1\ncharm.owned=shell\ncharm.worn=shell,,\nlv.life=1\nlv.stamina=4\nlv.might=4\ncore.warden=1\nfinds=sanctuary.shard.0\nmigrated.v04=1\n';
// 기술 판의 단을 미리 찍은 기록(0158): 점수 23(레벨 15 + 엘리트 6 + 문지기 둘). 첫 넷(휘돌이·가로지르기·끌어모으기·땅울림)과
// 새 넷(버팀·되울림·던지기·잔불 흩기, G2)을 따로 같은 단·같은 고르기에, 큰 기술은 (뒤집기, 몰아치기, 한 줄기)의 단.
// 설명 칸은 "지금 고르기 + 다음 단"이라 단마다 글이 다르다 — 넘침 검사(F4)가 모든 조합을 보게. 쓴 점수는 기록마다 23 안.
const RICH = 'v=1\nembers=600\nclears=1\nrun.rest=0\nrun.gate=1\nlv.life=5\nlv.stamina=5\nlv.might=5\ncore.warden=1\ncore.bellkeeper=1\n'
  + 'elites=t.elite.0,t.elite.1,t.elite.2,t.elite.3,t.elite.4,t.elite.5\nmigrated.v04=1\n';
const set = (keys, rank, pick) => keys.map((k) =>
  (rank > 0 ? `skill.rank.${k}=${rank}\n` : '') + (rank >= 3 ? `skill.pick.${k}=${pick}\n` : '')).join('');
const OLD = ['whirl', 'cut', 'gather', 'quake'];
const NEW = ['brace', 'echo', 'throw', 'embers'];
const board = ([ro, po], [rn, pn], [upend, flurry, line]) => RICH + set(OLD, ro, po) + set(NEW, rn, pn)
  + `ult.rank.upend=${upend}\n` + (flurry ? `ult.rank.flurry=${flurry}\n` : '') + (line ? `ult.rank.line=${line}\n` : '');
const BOARDS = [
  ['o3a-n3b', board([3, 'a'], [3, 'b'], [1, 0, 0])], ['o3b-n3a', board([3, 'b'], [3, 'a'], [1, 0, 0])],
  ['o4a-n2', board([4, 'a'], [2, 'a'], [1, 0, 0])], ['o2-n4b', board([2, 'a'], [4, 'b'], [1, 0, 0])],
  ['o4b-n1-u', board([4, 'b'], [1, 'a'], [1, 1, 1])], ['n4a-u', board([1, 'a'], [4, 'a'], [3, 2, 0])],
  ['o5a-n1', board([5, 'a'], [1, 'a'], [1, 0, 0])], ['o5b-n1', board([5, 'b'], [1, 'a'], [1, 0, 0])],
  ['n5a-l1', board([1, 'a'], [5, 'a'], [1, 0, 1])], ['n5b-f1', board([1, 'a'], [5, 'b'], [1, 1, 0])]];

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
    await press('Escape'); await page.waitForTimeout(300);
    await press('ArrowUp', 2);
  }
  // 이어하기 → 슬롯 창 → 이어하기 → 문 앞 쉼터.
  await press('Enter'); await page.waitForTimeout(300);
  await press('Enter'); await page.waitForTimeout(1500);
  // 쉬기 → 화로.
  await press('KeyF'); await page.waitForTimeout(600);
  await shot('1-hearth');
  // 레벨 창: 줄마다 커서를 옮겨 미리보기 글을 본다(생명 1단 · 기력 4단(다음이 끝) · 힘 4단(다음 단에 평타 +1)).
  await press('Enter'); await page.waitForTimeout(300);
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
  // 다 올린 줄을 다시 고르면 알림만(묻는 창 없음).
  await press('Enter'); await page.waitForTimeout(300);
  await shot('7-level-full-toast');
  // 화로로 → 나무(둘째 줄, 0106): 점수(핵 1 + 조각 1)로 박자 첫 칸을 연다, 셋째 갈래 끝 칸에 목표 핀.
  await press('Escape'); await page.waitForTimeout(300);
  await press('ArrowDown'); await press('Enter'); await page.waitForTimeout(300);
  await shot('8a-tree');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('8b-tree-ask');
  await press('Enter'); await page.waitForTimeout(400);
  await press('ArrowRight', 2); await press('ArrowDown', 4); await press('Enter'); await page.waitForTimeout(300);
  await shot('8c-tree-pinned');
  const t = await record();
  if (!/tree\.beat=1\n/.test(t) || !/tree\.pin=2\.4\n/.test(t)) errors.push(`tree not saved: ${JSON.stringify(t)}`);
  // 화로로 → 가게(셋째 줄).
  await press('Escape'); await page.waitForTimeout(300);
  await press('ArrowDown'); await press('Enter'); await page.waitForTimeout(300);
  await shot('8-shop');
  // 가게 → 화로 → 기술(넷째 줄, 0158): 줄 값 = 남은 기술 점수(레벨 11 + 문지기 1 = 12).
  await press('Escape'); await page.waitForTimeout(300);
  await press('ArrowDown'); await page.waitForTimeout(150);
  await shot('11-hearth-skills');
  await press('Enter'); await page.waitForTimeout(300);
  // 기술 판: 휘돌이(1단) — 설명은 다음 단(2단 피해 +15%). 사기 = 묻는 창(커서 "올리기").
  await shot('12-board');
  await press('Enter'); await page.waitForTimeout(300);
  await shot('12a-board-ask');
  await press('Enter'); await page.waitForTimeout(400);
  // 산 직후: 설명 자리에 알림("휘돌이 2단")이 잠깐 — 지나면 다음 단의 설명(3단의 고르기 둘).
  await shot('12b-board-bought');
  await page.waitForTimeout(1800);
  await shot('12b-board-picks');
  await press('Enter'); await page.waitForTimeout(300); await press('Enter'); await page.waitForTimeout(2200);
  // 3단: 줄이 ◀ ㄱ 도는 걸음 ▶ — 오른쪽으로 값 없이 ㄴ으로(설명이 바로 바뀐다).
  await shot('12c-board-rank3');
  await press('ArrowRight'); await page.waitForTimeout(300);
  await shot('12d-board-flipped');
  let b = await record();
  if (!/skill\.rank\.whirl=3\n/.test(b) || !/skill\.pick\.whirl=b\n/.test(b)) errors.push(`board not saved: ${JSON.stringify(b)}`);
  // 줄마다 설명(0158 G2 — 첫 줄 넷 · 둘째 줄 넷(잠김) · 큰 기술 셋(잠김) · 되돌리기 · 돌아가기).
  for (const name of ['cut', 'gather', 'brace', 'quake-locked', 'echo-locked', 'throw-locked', 'embers-locked',
    'upend-locked', 'flurry-locked', 'line-locked', 'respec', 'back']) {
    await press('ArrowDown'); await page.waitForTimeout(200);
    await shot(`12e-board-${name}`);
  }
  // 잠긴 줄(땅울림)을 고르면 알림만(묻는 창 없음).
  await press('ArrowUp', 8); await press('Enter'); await page.waitForTimeout(300);
  await shot('12f-board-locked-toast');
  // 되돌리기: 묻는 창(커서 "취소") → 왼쪽 = "되돌리기" → 쓴 점수 2를 돌려받고 잔불 40(30 + 5 × 2).
  const embers = Number(/embers=(\d+)\n/.exec(b)[1]);
  await press('ArrowDown', 7); await press('Enter'); await page.waitForTimeout(300);
  await shot('12g-respec-ask');
  await press('ArrowLeft'); await press('Enter'); await page.waitForTimeout(400);
  await shot('12h-respec-done');
  await page.waitForTimeout(1800);
  await shot('12i-board-after-respec');
  b = await record();
  if (/skill\.rank\./.test(b) || /skill\.pick\./.test(b)) errors.push(`respec must clear the board: ${JSON.stringify(b)}`);
  if (!new RegExp(`embers=${embers - 40}\\n`).test(b)) errors.push(`respec must cost 40 embers: ${JSON.stringify(b)}`);
  // 판 → 화로 → 싸움(HUD 기력 막대가 길어졌는지).
  await press('Escape'); await page.waitForTimeout(300);
  await press('Escape'); await page.waitForTimeout(400);
  await shot('9-play-hud');
  // 멈춤 → 나무(셋째 줄, 읽기 전용) → 장비(둘째 줄): 레벨 줄(최대 기력·기력 회복·평타·강타·자세)이 상태 줄에 더해져도 창 안에.
  await press('Escape'); await page.waitForTimeout(200);
  await press('ArrowDown', 2); await press('Enter'); await page.waitForTimeout(400);
  await shot('9b-tree-view');
  await press('Escape'); await page.waitForTimeout(300);
  await press('ArrowUp'); await press('Enter'); await page.waitForTimeout(400);
  await shot('10-equip');
  // 기술 판의 모든 단(0158): 단을 미리 찍은 기록마다 — 판의 줄 열셋을 다 지나고, 멈춤의 기술 창에서 칸 1을 배운 스킬로 돌리고
  // 큰 기술 줄을 배운 큰 기술로 돌린다(G2 — 끼우는 큰 기술은 하나).
  for (const [name, text] of BOARDS) {
    await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, text]);
    await page.reload();
    await ready();
    await page.waitForTimeout(600);
    await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
    await press('F4');
    await press('Enter'); await page.waitForTimeout(300);
    await press('Enter'); await page.waitForTimeout(1500);
    await press('KeyF'); await page.waitForTimeout(600);
    await press('ArrowDown', 3); await press('Enter'); await page.waitForTimeout(300);
    await shot(`20-${name}-board`);
    for (let i = 0; i < 12; i++) {
      await press('ArrowDown'); await page.waitForTimeout(150);
      if (i === 3 || i === 5 || i === 9) await shot(`20-${name}-board-row${i + 1}`);
    }
    await press('Escape'); await page.waitForTimeout(300);
    await press('Escape'); await page.waitForTimeout(400);
    // 멈춤 → 기술(다섯째 줄): 끼운 칸의 "이름 단 · 고르기", 칸 1을 오른쪽으로 돌려 스킬 넷의 설명을 다 본다.
    await press('Escape'); await page.waitForTimeout(200);
    await press('ArrowDown', 4); await press('Enter'); await page.waitForTimeout(300);
    await shot(`21-${name}-pause-skills`);
    for (let i = 0; i < 7; i++) { await press('ArrowRight'); await page.waitForTimeout(150); }
    await shot(`21-${name}-pause-skills-turned`);
    await press('ArrowDown', 2);
    for (let i = 0; i < 2; i++) { await press('ArrowRight'); await page.waitForTimeout(150); }
    await shot(`21-${name}-pause-ult`);
    await press('ArrowDown'); await page.waitForTimeout(150);
  }
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
