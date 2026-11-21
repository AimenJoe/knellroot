// 서막 시험(0262): 헤드리스 Chromium으로
//   1) 빈 기록 → 타이틀 바탕은 숲(엎드린 방랑자) → 시작하기 → 숲의 빈터에서 깨어나는 장면(0171) → 멈춤 키로 건너뛴다(woke=1, 아직 prologue=0).
//   2) 숲길을 숲 끝까지 걷는다 — 시험의 도움: 개발판 시험 깃발 4(`App::prologue_autopilot` — 방랑자가 적의 길찾기 거리 지도를 따라
//      숲 끝으로 저절로 걷고 턱 앞에서 뛴다. 쿼터뷰가 35° 돌아 있어 키를 시간으로 누르면 느린 러너에서 길을 벗어난다). 잔재가 쓰러뜨리지
//      않게 unhurt(1).
//   3) 문 앞 표지를 밟으면 "성소에 들다": 성소 바깥 문이 열리고 걸어 든다(0265) → 다 어두울 때 성소로 바뀌며 기록에 prologue=1 → 높이 보는 장면 → 내려옴 → 이름 카드 → 조작이
//      돌아온다(멈춤 키 = 멈춤 창이 열린다 — 장면 동안이면 건너뛰기일 뿐이다).
//   4) 다시 빈 기록: 숲 끝에서 장면을 멈춤 키로 건너뛰어도 성소에서 조작이 돌아온다.
//   5) 옛 기록(키 없는 판, `v=1`)은 숲을 보지 않는다 — 이어하기가 성소에서(prologue=1로 읽힌다).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/prologue.js dist shots
const { chromium } = require('playwright');
const sceneKit = require('./scene');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8162;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];
const KEY = 'knell.save.0';

const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(PORT, r));
  const browser = await chromium.launch();
  const errors = [];
  const fail = (msg) => { console.error('FAIL ' + msg); errors.push(msg); };

  const open = async (name) => {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await ctx.newPage();
    await sceneKit.throttle(ctx, page);
    page.on('console', (m) => { if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(m.text()))) errors.push(`${name}: ${m.text()}`); });
    page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(`${name}: ${e}`); });
    const kit = sceneKit(page, 80);
    await sceneKit.unhurt(page);
    await sceneKit.prologue(page);
    const shot = (n) => page.screenshot({ path: path.join(OUT, `prologue-${n}.png`) });
    const record = () => page.evaluate((k) => localStorage.getItem(k) || '', KEY);
    const recorded = (re, what, ms = 90000) => page.waitForFunction(([k, src]) => new RegExp(src).test(localStorage.getItem(k) || ''), [KEY, re.source], { timeout: ms, polling: 100 })
      .catch(async () => { throw new Error(`${what}: 기록에 ${re} 이 오지 않았다 — ${JSON.stringify(await record())}`); });
    const autopilot = (on) => page.evaluate((v) => {
      window.knellTestFlags = v ? (window.knellTestFlags | 4) : (window.knellTestFlags & ~4);
    }, on);
    // 빈 쪽에서 기록을 심고(또는 지우고) 띄운다.
    const boot = async (text) => {
      await page.goto(`http://localhost:${PORT}/manifest.webmanifest`);
      await page.evaluate(([k, v]) => { if (v === null) localStorage.removeItem(k); else localStorage.setItem(k, v); }, [KEY, text]);
      await page.goto(`http://localhost:${PORT}/`);
      await page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 60000 });
      await page.waitForTimeout(600);
      await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
      await kit.until('Title', `${name}: 타이틀`, 60000);
    };
    return { ctx, page, kit, shot, record, recorded, autopilot, boot };
  };

  // 1)~3) 새 기록: 숲에서 깨어나 숲길을 지나 성소에 든다.
  {
    const g = await open('new');
    const { page, kit, shot, record, recorded, autopilot } = g;
    await g.boot(null);
    await page.waitForTimeout(1200);
    await shot('0-title-wood');
    await kit.press('Enter'); await kit.until('Slots', '시작하기 → 슬롯 창');
    await kit.press('Enter'); await kit.until('Play', '빈 슬롯 → 새 판');
    await page.waitForTimeout(7500);
    await shot('1-wake-in-the-wood');
    await kit.press('Escape'); await recorded(/woke=1\n/, '깨어나는 장면 건너뛰기');
    if (!/prologue=0\n/.test(await record())) fail('woke in the wood: prologue must still be 0');
    await page.waitForTimeout(2600);
    await shot('2-glade');
    await page.waitForTimeout(4500);
    await shot('2b-glade-objective');
    await autopilot(true);
    // 숲길(잔재 곁을 지나는 동안 몇 장 — 생물이 바닥에서 떠 보이나, 0170).
    for (const n of ['3a', '3b', '3c', '3d', '3e', '3f']) {
      await page.waitForTimeout(1600);
      await shot(`${n}-forest-path`);
    }
    await kit.until('Play ~arrive', '숲 끝 → 성소에 들다', 90000);
    await autopilot(false);
    // 문 앞(0265): 서서 문을 올려다본다 → 종 → 문이 열린다 → 걸어 든다 → 어두워진다.
    if (/prologue=1\n/.test(await record())) fail('the gate scene comes before the swap: prologue must still be 0');
    await page.waitForTimeout(1500);
    await shot('3g-at-the-gate');
    await page.waitForTimeout(1300);
    await shot('3h-gate-opening');
    await page.waitForTimeout(1500);
    await shot('3i-walking-in');
    await recorded(/prologue=1\n/, '다 어두울 때 성소로 — 기록');
    await page.waitForTimeout(1300);
    await shot('4-establishing');
    await page.waitForTimeout(1800);
    await shot('4b-establishing');
    await page.waitForTimeout(2600);
    await shot('5-descending');
    await page.waitForTimeout(1800);
    await shot('6-name-card');
    // 장면이 끝나면 이름에서 ` ~arrive`가 빠진다(src/app/mod.rs frame).
    await kit.until(/^Play$/, '"성소에 들다"가 끝난다', 60000);
    await page.waitForTimeout(800);
    await shot('7-arrived');
    // 조작이 돌아왔다: 멈춤 키 = 멈춤 창(장면 동안이면 건너뛰기일 뿐 창이 열리지 않는다).
    await kit.press('Escape'); await kit.until(/host: Pause/, '성소에서 멈춤 창');
    await kit.press('Escape'); await kit.until('Play', '멈춤 창 닫기');
    const r = await record();
    if (!/woke=1\n/.test(r) || !/prologue=1\n/.test(r)) fail('after the arrival the record must hold woke=1 and prologue=1: ' + JSON.stringify(r));
    await g.ctx.close();
  }

  // 4) 장면을 건너뛰어도 성소에서 조작이 돌아온다.
  {
    const g = await open('skip');
    const { page, kit, shot, recorded, autopilot } = g;
    await g.boot(null);
    await kit.press('Enter'); await kit.until('Slots', '시작하기 → 슬롯 창');
    await kit.press('Enter'); await kit.until('Play', '빈 슬롯 → 새 판');
    await page.waitForTimeout(1500);
    await kit.press('Escape'); await recorded(/woke=1\n/, '깨어나는 장면 건너뛰기');
    await autopilot(true);
    await kit.until('Play ~arrive', '숲 끝 → 성소에 들다', 90000);
    await autopilot(false);
    await kit.press('Escape');
    await recorded(/prologue=1\n/, '건너뛰어도 다 어두울 때 성소로');
    await kit.until(/^Play$/, '건너뛴 장면이 짧게 끝난다', 15000);
    await page.waitForTimeout(800);
    await shot('8-skipped');
    await kit.press('Escape'); await kit.until(/host: Pause/, '건너뛴 뒤 멈춤 창');
    await g.ctx.close();
  }

  // 5) 옛 기록은 숲을 보지 않는다(이어하기 = 성소 — 깨어나는 장면도 없다).
  {
    const g = await open('old');
    const { page, kit, shot, record } = g;
    await g.boot('v=1\nembers=30\n');
    await kit.press('Enter'); await kit.until('Slots', '이어하기 → 슬롯 창');
    await kit.press('Enter'); await kit.until('Play', '슬롯 → 판');
    await page.waitForTimeout(3000);
    await shot('9-old-record-sanctuary');
    // 처음 쓴 기록에서 prologue=1(woke를 따른다).
    await kit.press('Escape'); await kit.until(/host: Pause/, '옛 기록 멈춤 창');
    const r = await record();
    if (/prologue=0\n/.test(r)) fail('an old record must read as past the wood: ' + JSON.stringify(r));
    await g.ctx.close();
  }

  await browser.close();
  server.close();
  if (errors.length) {
    console.error('errors:\n' + errors.join('\n'));
    process.exitCode = 1;
  } else {
    console.log('prologue: ok — shots in ' + OUT);
  }
}

main().catch((e) => { console.error(e); process.exitCode = 1; server.close(); process.exit(1); });
