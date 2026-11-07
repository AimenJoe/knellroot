// 화로·레벨 화면 시험(0104, v0.4.0 G1): 헤드리스 Chromium으로 한국어·영어 두 판 —
//   쉼터 기록(잔불 넉넉)을 심고 이어하기 → 쉬기 → 화로 → 레벨(줄 셋을 차례로 올려 보고 묻는 창) → 가게, 화면마다 스크린숏.
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
// 문 앞 쉼터(0)에서 쉰 기록, 잔불은 레벨 몇 단 + 가게를 볼 만큼. 기력·힘은 이미 4단(다음이 마지막 단 — "다 올림" 줄·평타 +1도 본다).
const SEED = 'v=1\nembers=400\nclears=1\nrun.rest=0\nrun.gate=1\ncharm.owned=shell\ncharm.worn=shell,,\nlv.life=1\nlv.stamina=4\nlv.might=4\n';

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
  // 화로로 → 가게.
  await press('Escape'); await page.waitForTimeout(300);
  await press('ArrowDown'); await press('Enter'); await page.waitForTimeout(300);
  await shot('8-shop');
  // 가게 → 화로 → 싸움(HUD 기력 막대가 길어졌는지).
  await press('Escape'); await page.waitForTimeout(300);
  await press('Escape'); await page.waitForTimeout(400);
  await shot('9-play-hud');
  // 장비 창(멈춤 → 둘째 줄): 레벨 줄(최대 기력·기력 회복·평타·강타·자세)이 상태 줄에 더해져도 창 안에.
  await press('Escape'); await page.waitForTimeout(200);
  await press('ArrowDown'); await press('Enter'); await page.waitForTimeout(400);
  await shot('10-equip');
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
