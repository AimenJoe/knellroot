// 튜토리얼 시험(0167): 헤드리스 Chromium으로 한국어·영어 두 판 —
//   걸음 일곱의 힌트 줄을 하나씩 띄워 본다(걸음을 미리 적은 기록 `tut.step=N`으로 이어하기) — 큰 화면(1280×720)과 가로 폰(844×390).
//   개발판의 F4 넘침 검사(`ui::check`)를 켜 두고 콘솔의 `[ui-check]` 줄(켬/끔 말고)이 있으면 실패.
//   그리고 새 판에서 실제로 걸어 첫 걸음이 넘어가는지(기록 `tut.step`), 회피 걸음에서 한 번 구르면 넘어가는지, 몸의 걸음을 지난
//   판에 상(기술 점수 1)이 오는지를 본다. 공격·패링 걸음의 셈은 단위 시험이 본다(0209 T5 — 헤드리스의 손 싸움은 화면의
//   고정 자리를 눌러 헛치거나 맞아 쓰러졌다: 게임이 아니라 시험의 문제).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/tutorial.js dist shots
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');
const sceneKit = require('./scene');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8143;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const KEY = 'knell.save.0';
// 문 앞 쉼터에 선 새 판의 기록(처음의 둘도 기술 점수도 없다) + 걸음마다의 사실.
const BASE = 'v=1\nrun.rest=0\nrun.gate=0\nplay=1\nweapons=\nmigrated.v04=1\nmigrated.v06=1\nmigrated.ult=1\n';
// 0169: 몸의 걸음 넷 뒤에 상(기술 점수 1 — `skill.gift=1`) → 쉬고 배우기 → 쓰기 → 잔불이 첫 레벨 값(50)의 반부터 뜨는
// "한 단 올리기"(0209 T4 · 0213: 반~값 사이는 "잔불을 모은다 n / 값" — `7q`, 반 아래는 조용하다).
const STEPS = [
  ['1-move', 'tut.step=0\n'], ['2-attack', 'tut.step=1\n'], ['3-dodge', 'tut.step=2\n'], ['4-parry', 'tut.step=3\n'],
  ['5-learn', 'tut.step=4\nskill.gift=1\n'], ['6-skill', 'tut.step=5\nskill.gift=1\nskill.rank.whirl=1\n'],
  ['7-level', 'tut.step=6\nskill.gift=1\nskill.rank.whirl=1\nembers=60\n'],
  ['7q-level-gather', 'tut.step=6\nskill.gift=1\nskill.rank.whirl=1\nembers=30\n'],
];

const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});

async function run(browser, lang, size) {
  const [w, h] = size;
  const ctx = await browser.newContext({ viewport: { width: w, height: h } });
  const page = await ctx.newPage();
  const errors = [];
  const checks = [];
  page.on('console', (m) => {
    const t = m.text();
    if (/\[ui-check\]/.test(t) && !/\[ui-check\] (on|off)$/.test(t)) checks.push(t);
    if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(t))) errors.push(t);
  });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(String(e)); });
  const tag = `${lang}-${w}x${h}`;
  const shot = (name) => page.screenshot({ path: path.join(OUT, `tut-${tag}-${name}.png`) });
  // 장면이 바뀌는 누름 뒤에는 장면을 기다린다(tools/test/scene.js). 힌트를 찍는 동안 둘레의 적에게 쓰러지지 않게(개발판).
  const { press, until } = sceneKit(page, 90);
  await sceneKit.unhurt(page);
  // 기록에 그 줄이 적힐 때까지(장면이 끝나야 적는다 — 느린 러너에서는 고정 시간이 모자란다).
  const recorded = (re, what) => page.waitForFunction(([k, src]) => new RegExp(src).test(localStorage.getItem(k) || ''), [KEY, re.source], { timeout: 20000, polling: 100 })
    .catch(() => { throw new Error(`${what}: 기록에 ${re} 이 오지 않았다`); });
  const ready = () => page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 20000 });
  const record = () => page.evaluate((k) => localStorage.getItem(k), KEY);
  // 기록을 심고 다시 띄워 이어하기(타이틀 → 슬롯 창 → 이어하기).
  const boot = async (text) => {
    // 게임이 도는 쪽에서 심으면 다시 띄우기 전에 게임이 제 기록을 덮어쓸 수 있다 — 같은 출처의 빈 쪽으로 나가서 심는다.
    await page.goto(`http://localhost:${PORT}/manifest.webmanifest`);
    await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, text]);
    await page.goto(`http://localhost:${PORT}/`);
    await ready();
    await page.waitForTimeout(600);
    await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
    await until('Title', '타이틀');
    await press('F4');
    await press('Enter'); await until('Slots', '이어하기 → 슬롯 창');
    await press('Enter'); await until('Play', '슬롯 → 판'); await page.waitForTimeout(1500);
  };

  await page.goto(`http://localhost:${PORT}/`);
  await ready();
  await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, BASE + STEPS[0][1]]);
  await page.reload();
  await ready();
  await page.waitForTimeout(600);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  // 영어 판: 타이틀 → 설정 → 언어(첫 줄) 오른쪽 한 번.
  if (lang === 'en') {
    await until('Title', '타이틀');
    await press('ArrowDown', 2); await press('Enter'); await until('Settings { host: Title', '타이틀 → 설정');
    await press('ArrowRight'); await page.waitForTimeout(200);
    await press('Escape'); await until('Title', '설정 → 타이틀');
    await press('ArrowUp', 2);
  }
  for (const [name, facts] of STEPS) {
    await boot(BASE + facts);
    // 시작 이야기·구역 카드가 가라앉은 뒤.
    await page.waitForTimeout(2500);
    await shot(name);
  }
  // 새 판에서 실제로 걷는다: [1/7] → 넉 미터 걸으면 [2/7], 기록에 지난 걸음 수가 남는다.
  await boot(BASE + STEPS[0][1]);
  // 2026-11-15: 1.6초는 걷기 걸음의 4 m에 못 미쳤다(첫 몇 프레임의 가속·몸 돌림) — 넉넉히.
  await page.keyboard.down('KeyS'); await page.waitForTimeout(2600); await page.keyboard.up('KeyS');
  await page.waitForTimeout(300);
  await shot('9-walked');
  let b = await record();
  if (!/tut\.step=[1-9]\n/.test(b)) errors.push(`walking must pass the first step: ${JSON.stringify(b)}`);
  // 회피 걸음: 한 번 구르면 넘어간다(상태 변화 → 튜토리얼의 셈이 이어져 있나 — 적이 없어도 된다).
  await boot(BASE + STEPS[2][1]);
  await page.waitForTimeout(2500);
  await press('Space'); await recorded(/tut\.step=[3-9]\n/, '구르면 회피 걸음이 지나간다').catch(() => {}); await page.waitForTimeout(300);
  await shot('9c-dodged');
  b = await record();
  if (!/tut\.step=[3-9]\n/.test(b)) errors.push(`a dodge must pass the dodge step: ${JSON.stringify(b)}`);
  // 상(0169): 몸의 걸음 넷을 지난 판(패링은 헤드리스로 받아치기 어렵다 — 걸음이 넘어가는 것은 단위 시험이 본다)을 이어하면
  // 첫 틱에 기술 점수 하나가 온다 — 기록의 몫 `skill.gift=1`, 화면 위 알림, 힌트는 "쉬고 배운다"로.
  await boot(BASE + 'tut.step=4\n');
  await recorded(/skill\.gift=1\n/, '몸의 걸음 넷의 상').catch(() => {}); await page.waitForTimeout(300);
  await shot('9d-reward');
  b = await record();
  if (!/skill\.gift=1\n/.test(b)) errors.push(`the body steps must pay one skill point: ${JSON.stringify(b)}`);
  // 멈춤 창 밑에는 힌트를 그리지 않는다.
  await press('Escape'); await until('Status { host: Pause', '멈춤 = 상태 탭'); await page.waitForTimeout(200);
  await shot('9a-paused');
  // 깨어나는 첫 장면(0171): 빈 기록으로 시작하면 타이틀 바탕부터 방랑자가 꺼진 화로 곁에 엎드려 있고, 판은 그 장면으로 열린다 —
  // 서술 두 줄이 넘치지 않고(F4), 멈춤 버튼으로 건너뛰면 기록에 남고(`woke=1`) 튜토리얼의 첫 걸음이 뜬다. 화로는 아직 꺼져 있다.
  await page.goto(`http://localhost:${PORT}/manifest.webmanifest`);
  await page.evaluate((k) => localStorage.removeItem(k), KEY);
  await page.goto(`http://localhost:${PORT}/`);
  await ready();
  await page.waitForTimeout(600);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  await until('Title', '타이틀(빈 기록)');
  await press('F4');
  await shot('w0-title-asleep');
  await press('Enter'); await until('Slots', '시작하기 → 슬롯 창');
  await press('Enter'); await until('Play', '빈 슬롯 → 새 판');
  await page.waitForTimeout(4600);
  await shot('w1-eyes-open');
  await page.waitForTimeout(5200);
  await shot('w2-rising');
  // (장면이 아직 도는지는 보지 않는다 — 느린 헤드리스의 좁은 화면에서 시각이 어긋났다, 0209 T5. 시간표는 `wake.rs` 단위 시험.)
  await press('Escape'); await recorded(/woke=1\n/, '건너뛴 깨어나는 장면').catch(() => {}); await page.waitForTimeout(800);
  await shot('w3-awake');
  b = await record();
  if (!/woke=1\n/.test(b || '')) errors.push(`skipping the waking scene must be recorded: ${JSON.stringify(b)}`);
  if (!/hearth\.lit=0\n/.test(b || '')) errors.push(`the first hearth must still be cold: ${JSON.stringify(b)}`);
  await ctx.close();
  return { tag, errors, checks };
}

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(PORT, r));
  const browser = await chromium.launch();
  let bad = false;
  for (const lang of ['ko', 'en']) {
    for (const size of [[1280, 720], [844, 390]]) {
      const { tag, errors, checks } = await run(browser, lang, size);
      for (const e of errors) { console.error(`[${tag}] ERROR ${e}`); bad = true; }
      for (const c of checks) { console.error(`[${tag}] ${c}`); bad = true; }
      console.log(`[${tag}] errors ${errors.length}, ui-check ${checks.length}`);
    }
  }
  await browser.close();
  server.close();
  process.exitCode = bad ? 1 : 0;
}

main().catch((e) => { console.error(e); process.exitCode = 1; server.close(); });
