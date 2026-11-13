// 튜토리얼 시험(0167): 헤드리스 Chromium으로 한국어·영어 두 판 —
//   걸음 여덟의 힌트 줄을 하나씩 띄워 본다(걸음을 미리 적은 기록 `tut.step=N`으로 이어하기) — 큰 화면(1280×720)과 가로 폰(844×390).
//   개발판의 F4 넘침 검사(`ui::check`)를 켜 두고 콘솔의 `[ui-check]` 줄(켬/끔 말고)이 있으면 실패.
//   그리고 새 판에서 실제로 걸어 첫 걸음이 넘어가는지(힌트 [1/8] → [2/8], 기록 `tut.step`), 곁에 온 첫 적을 세 번 베고 한 번
//   굴러 공격·회피 걸음이 넘어가는지를 본다(월드 사건이 튜토리얼의 셈까지 이어져 있나).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/tutorial.js dist shots
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8143;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const KEY = 'knell.save.0';
// 문 앞 쉼터에 선 새 판의 기록(처음의 둘도 기술 점수도 없다) + 걸음마다의 사실.
const BASE = 'v=1\nrun.rest=0\nrun.gate=0\nplay=1\nweapons=\nmigrated.v04=1\nmigrated.v06=1\n';
const STEPS = [
  ['1-move', 'tut.step=0\n'], ['2-attack', 'tut.step=1\n'], ['3-dodge', 'tut.step=2\n'], ['4-parry', 'tut.step=3\n'],
  ['5-gather', 'tut.step=4\nembers=20\n'], ['6-level', 'tut.step=5\nembers=60\n'],
  ['7-learn', 'tut.step=6\nlv.life=1\n'], ['8-skill', 'tut.step=7\nlv.life=1\nskill.rank.whirl=1\n'],
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
  const press = async (key, n = 1) => { for (let i = 0; i < n; i++) { await page.keyboard.press(key); await page.waitForTimeout(90); } };
  const ready = () => page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 20000 });
  const record = () => page.evaluate((k) => localStorage.getItem(k), KEY);
  // 기록을 심고 다시 띄워 이어하기(타이틀 → 슬롯 창 → 이어하기).
  const boot = async (text) => {
    await page.evaluate(([k, v]) => localStorage.setItem(k, v), [KEY, text]);
    await page.reload();
    await ready();
    await page.waitForTimeout(600);
    await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
    await press('F4');
    await press('Enter'); await page.waitForTimeout(300);
    await press('Enter'); await page.waitForTimeout(1500);
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
    await press('ArrowDown', 2); await press('Enter'); await page.waitForTimeout(300);
    await press('ArrowRight'); await page.waitForTimeout(200);
    await press('Escape'); await page.waitForTimeout(300);
    await press('ArrowUp', 2);
  }
  for (const [name, facts] of STEPS) {
    await boot(BASE + facts);
    // 시작 이야기·구역 카드가 가라앉은 뒤.
    await page.waitForTimeout(2500);
    await shot(name);
  }
  // 새 판에서 실제로 걷는다: [1/8] → 넉 미터 걸으면 [2/8], 기록에 지난 걸음 수가 남는다.
  await boot(BASE + STEPS[0][1]);
  await page.keyboard.down('KeyS'); await page.waitForTimeout(1600); await page.keyboard.up('KeyS');
  await page.waitForTimeout(300);
  await shot('9-walked');
  let b = await record();
  if (!/tut\.step=[1-9]\n/.test(b)) errors.push(`walking must pass the first step: ${JSON.stringify(b)}`);
  // 큰 화면에서만: 곁에 온 첫 적과 실제로 싸운다 — 맞힌 타 셋이면 공격 걸음이, 한 번 구르면 회피 걸음이 지나간다
  // (월드 사건 → 튜토리얼의 셈이 이어져 있는지. 판정의 운에 기대지 않게 넉넉히 휘두른다).
  if (w >= 1280) {
    const step = async () => Number(/tut\.step=(\d+)\n/.exec(await record())[1]);
    for (let i = 0; i < 40 && (await step()) < 2; i++) {
      await page.mouse.click(w * 0.57, h * 0.47); await page.waitForTimeout(260);
    }
    await shot('9b-struck');
    if ((await step()) < 2) errors.push(`three hits must pass the attack step: ${JSON.stringify(await record())}`);
    await press('Space'); await page.waitForTimeout(700);
    await shot('9c-dodged');
    if ((await step()) < 3) errors.push(`a dodge must pass the dodge step: ${JSON.stringify(await record())}`);
  }
  // 멈춤 창 밑에는 힌트를 그리지 않는다.
  await press('Escape'); await page.waitForTimeout(400);
  await shot('9a-paused');
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
