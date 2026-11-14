// 터치 연기 시험(0006): 가로 폰(844×390, DPR 3, 터치)으로 웹 빌드를 띄워 가상 스틱·버튼·랩 터치판을 눌러 보고 스크린숏을 남긴다.
//   node tools/test/touch.js dist shots
// 볼 것: 첫 화면부터 버튼이 보이는지(pointer: coarse), 스틱으로 걷는지, 락온 버튼이 노랗게 차는지, 랩 줄이 폰에 들어가는지.
// 손가락 끌기는 CDP Input.dispatchTouchEvent(Playwright touchscreen은 탭만 한다). 콘솔 에러가 있으면 종료 코드 1.
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8132;
const W = 844, H = 390;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm' };
// 둘째: CDP로 오래 누른 touchStart에 Chromium이 다는 개입 경고(우리 코드가 아니라 브라우저의 스크롤 판정, 무해).
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/, /Ignored attempt to cancel a touchstart event with cancelable=false/];
const server = http.createServer((req, res) => {
  const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});

// src/touch.rs layout()과 같은 식(논리 px, 0053). 바꾸면 함께. 락온·등잔은 상황 버튼(뜰 때만) — 자리는 늘 같다.
function layout() {
  const u = Math.min(W, H, 520), r = u * 0.09;
  const atk = [W - u * 0.238, H - u * 0.21];
  const at = (deg, dist) => { const a = deg * Math.PI / 180; return [atk[0] + Math.cos(a) * u * dist, atk[1] - Math.sin(a) * u * dist]; };
  // 0151: 예전 돌리기 캡슐 자리 = 스킬 둘, 돌리기는 오른쪽 위(멈춤과 마주 본다 — `touch::layout`).
  const skill = at(118, 0.44);
  const ps = r * 0.7, pc = ps * 1.4;
  const b = {
    attack: atk,
    dodge: at(188, 0.26),
    parry: at(146, 0.26),
    cast: at(104, 0.26),
    jump: at(62, 0.26),
    skill1: [skill[0] - u * 0.08, skill[1]],
    skill2: [skill[0] + u * 0.08, skill[1]],
    turn_left: [W - pc - ps * 2.5, pc],
    turn_right: [W - pc, pc],
    // 시점(0184): 돌리기 왼쪽, 한 칸 띄워 — 쿼터뷰 ↔ 자유 숄더뷰.
    view: [W - pc - ps * 5.4, pc],
    lock: at(80, 0.44),
    flask: at(150, 0.44),
    // 멈춤은 왼쪽 위(2026-11-06), 랩·시나리오 터치 버튼은 없다(랩 = 타이틀·개발 창 도구 탭).
    pause: [r * 0.7 * 1.4, r * 0.7 * 1.4],
  };
  return b;
}

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  await new Promise((r) => server.listen(PORT, r));
  const browser = await chromium.launch();
  const ctx = await browser.newContext({ viewport: { width: W, height: H }, deviceScaleFactor: 3, hasTouch: true, isMobile: true });
  const page = await ctx.newPage();
  const errors = [];
  page.on('console', (m) => { if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(m.text()))) errors.push(m.text()); });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(String(e)); });
  const cdp = await ctx.newCDPSession(page);
  const touch = (type, pts) => cdp.send('Input.dispatchTouchEvent', { type, touchPoints: pts.map(([x, y], i) => ({ x, y, id: i + 1 })) });
  const tap = async ([x, y]) => { await touch('touchStart', [[x, y]]); await page.waitForTimeout(60); await touch('touchEnd', []); await page.waitForTimeout(60); };
  const shot = (n) => page.screenshot({ path: path.join(OUT, n + '.png') });

  await page.goto(`http://localhost:${PORT}/`);
  await page.waitForTimeout(2500);
  let b = layout();
  await shot('t00-title');
  // 타이틀(0019): 첫 줄 "시작"을 탭(뗄 때 누른다). 자리는 `app::draw_title`·`ui::menu`와 같은 셈.
  {
    // 화면에 맞춘 내부 해상도(0046, render::fit_internal): 배율 = 높이/360 반올림, 줄 수 = 높이/배율, UI 배율 = 그린 높이/270.
    const dpi = 3, s = Math.max(1, Math.min(Math.round(H * dpi / 360), Math.floor(W * dpi / 480)));
    const w = Math.floor(W * dpi / s), rows = Math.min(Math.floor(H * dpi / s), Math.floor(w * 3 / 4));
    const dh = rows * s / dpi, oy = Math.floor((H - dh) / 2), px = 13 * dh / 270;  // ui::GLYPH × Frame::ui
    await tap([W / 2, oy + dh * 0.37 + px * 0.5]);  // app::TITLE_MENU_Y
    // 시작하기 → 슬롯 창(0066): 첫 줄(슬롯 1, 비어 있음 = 시작)을 탭. 창은 480×270 기준 높이 106 —
    // 위 6·아래 안내 줄(2.8글) 사이 가운데, 제목 줄 뒤 첫 줄 가운데 ≈ 108/270(`app::menu` `draw_menu`·`ui::list`).
    await page.waitForTimeout(400);
    await tap([W / 2, oy + dh * 108 / 270]);
  }
  // 빈 기록의 첫 판은 깨어나는 장면(0171): 화면 버튼은 걷혀 있고(덮개만), 싸움 버튼 자리를 누르면 건너뛴다 — 안내 글이 선
  // 오른쪽 아래가 공격 버튼 자리다. 건너뛴 뒤에야 버튼들이 선다.
  await page.waitForTimeout(4200);
  await shot('t01a-wake');
  await tap(b.attack); await page.waitForTimeout(1500);
  {
    const rec = await page.evaluate(() => localStorage.getItem('knell.save.0'));
    if (!/woke=1\n/.test(rec || '')) errors.push(`a touch on the attack spot must skip the waking scene: ${JSON.stringify(rec)}`);
  }
  await shot('t01-first-screen');
  // 시점 버튼(0184 — 적이 아직 먼 첫 화면에서): 눈 버튼 = 자유 숄더뷰 → 오른쪽 반을 끌면 시점이 돈다 → 다시 눈 버튼 = 쿼터뷰.
  await tap(b.view); await page.waitForTimeout(800);
  await shot('t01b-shoulder');
  {
    const y = H * 0.4, x0 = W * 0.62;
    await touch('touchStart', [[x0, y]]); await page.waitForTimeout(40);
    for (let i = 1; i <= 8; i++) { await touch('touchMove', [[x0 + i * 20, y]]); await page.waitForTimeout(30); }
    await touch('touchEnd', []); await page.waitForTimeout(300);
  }
  await shot('t01c-shoulder-dragged');
  await tap(b.view); await page.waitForTimeout(800);
  await shot('t01d-back-to-quarter');
  // 스틱: 왼쪽 아래에 대고 위로 끌어 0.8초 걷기.
  const s0 = [W * 0.2, H * 0.7];
  await touch('touchStart', [s0]);
  for (let i = 1; i <= 6; i++) { await touch('touchMove', [[s0[0], s0[1] - i * 10]]); await page.waitForTimeout(30); }
  await page.waitForTimeout(800);
  await shot('t02-stick-walk');
  // 스틱을 쥔 채 공격(두 손가락).
  await touch('touchMove', [[s0[0], s0[1] - 60], b.attack]); await page.waitForTimeout(80);
  await touch('touchMove', [[s0[0], s0[1] - 60]]); await page.waitForTimeout(50);
  await touch('touchEnd', []); await page.waitForTimeout(100);
  await tap(b.lock); await page.waitForTimeout(700);
  await shot('t03-lock-on');
  await tap(b.dodge); await page.waitForTimeout(100);
  await shot('t04-dodge');
  // 땅 돌리기 오른쪽 ↷(0012·0017 — 카메라가 오른쪽으로): 도는 중 · 다 돈 뒤(버튼이 대기로 옅어짐).
  await tap(b.turn_right); await page.waitForTimeout(120);
  await shot('t08-turn-cw');
  await page.waitForTimeout(600);
  await shot('t09-after-turn');
  // 멈춤(2026-11-06): 버튼을 누른 채 창이 열리고 나서 떼도 창이 남아야 한다(뗀 자리가 "창 밖 탭"이 되어 닫히던 것).
  await touch('touchStart', [b.pause]); await page.waitForTimeout(300);
  await touch('touchEnd', []); await page.waitForTimeout(400);
  await shot('t10-pause-stays');
  // 장비 탭 터치(2026-11-06 → 0164): 멈춤 = 큰 창(상태 탭에서 열린다) → 탭 머리 "장비"를 누른다 → 칸은 두 걸음(누르면 고르기,
  // 다시 누르면 그 칸 곁에 후보의 작은 목록) → 후보도 두 걸음(고르기 = 미리 보기, 다시 = 끼우기).
  // 0076·0077 → 0117: 새 기록의 무기 후보 = 맨손 · 검(착용 중)뿐(무기는 상자에서 얻는다). 맨손은 첫 줄 — 같은 자리의 둘째 탭이 끼운다.
  // 자리는 코드와 같은 셈(480×270 기준 단위 u): 창 464×228(`app::frame` — 위 6u와 아래 안내 줄(2.8글) 사이 가운데), 머리 줄 22,
  // 몸의 원점 = 창 + (8, 27). 탭 머리는 왼쪽 10부터 폭 44씩(두 글자 26 + 여백 18, 틈 2 — `ui::window`).
  // 장비 탭의 칸(`app::equip`): 장신구 상자 34 × 3(오른쪽 끝에서 8 들여, 사이 6 — 세로 가운데), 무기 상자 40은 왼쪽 8에 장신구 열의
  // 위 끝과 같은 높이(그 밑은 덧옷 상자 — 0176), 후보 목록은 무기 상자 오른쪽 5에 상자와 같은 높이부터 폭 170 — 줄 높이 13 × 1.75, 위 여백 4.
  const u = H / 270;
  const win = (() => {
    const top = 6 * u, bottom = H - 13 * u * 2.8;
    const x = (W - 464 * u) / 2, y = top + (bottom - top - 228 * u) / 2;
    const body = (bx, by) => [x + (8 + bx) * u, y + (27 + by) * u];
    const boxesTop = (196 - (34 * 3 + 6 * 2)) / 2;
    return {
      tab: (i) => [x + (10 + 44 * i + 22) * u, y + 12 * u],
      weapon: body(8 + 20, boxesTop + 20),
      charm: (i) => body(270 - 8 - 17, boxesTop + (34 + 6) * i + 17),
      pick: (row) => body(8 + 40 + 5 + 85, boxesTop + 4 + 13 * 1.75 * (row + 0.5)),
    };
  })();
  const SWORD_BOX = win.weapon, PICK_ROW = win.pick(0), CHARM_BOX = win.charm(0);
  await shot('t10a-pause-status');
  await tap(win.tab(1)); await page.waitForTimeout(400);   // 탭 머리 "장비"(상태 · 장비 · 기술 · 도감 · 설정의 둘째)
  await shot('t10b-equip-tab');
  await tap(CHARM_BOX); await page.waitForTimeout(300);    // 고르기만(설명이 바뀜)
  await tap(SWORD_BOX); await page.waitForTimeout(300);    // 무기 칸 고르기
  await tap(SWORD_BOX); await page.waitForTimeout(400);    // 다시 = 후보 목록
  await tap(PICK_ROW); await page.waitForTimeout(300);     // 맨손 고르기(미리 보기)
  await shot('t12-equip-preview');
  await tap(PICK_ROW); await page.waitForTimeout(400);     // 다시 = 끼우기
  await shot('t13-equip-worn');
  const rec = await page.evaluate(() => Object.keys(localStorage).filter((k) => k.startsWith('knell.save')).map((k) => localStorage.getItem(k)).join('\n'));
  if (!/weapon=fists/i.test(rec)) errors.push('touch equip: two taps on bare hands should wear them: ' + JSON.stringify(rec));
  // 창 밖을 누르면 창이 닫힌다(어느 탭에서든 한 번에 — 0164).
  await tap([W - 4, H - 4]); await page.waitForTimeout(300);
  await shot('t11-pause-closed');
  // 세로로 든 폰(0082): 회전 안내(#rotate)가 덮인다. 누르면 전체 화면 + 가로 잠금을 해 보고, 잠금이 안 되면(헤드리스·iPhone) 세로로 계속.
  {
    const pctx = await browser.newContext({ viewport: { width: H, height: W }, deviceScaleFactor: 3, hasTouch: true, isMobile: true });
    const pp = await pctx.newPage();
    await pp.goto(`http://localhost:${PORT}/`);
    await pp.waitForTimeout(4000); // 로딩 덮개가 걷힌 뒤
    const on = () => pp.evaluate(() => document.getElementById('rotate').classList.contains('on'));
    if (!(await on())) errors.push('portrait: the rotate hint should cover the game');
    await pp.screenshot({ path: path.join(OUT, 't14-portrait-rotate.png') });
    await pp.tap('#rotate');
    await pp.waitForTimeout(1000);
    if (await on()) errors.push('portrait: tapping the hint should rotate or, if it cannot lock, let it go on upright');
    // 돌린 뒤 늦은 크기(0187): 폰은 회전 때 resize를 레이아웃이 끝나기 전에 보내기도 한다 — 번들의 처리기가 그 이벤트를 놓친(옛 크기를
    // 읽은) 것처럼, 처리기를 잠시 떼고 가로로 돌린 뒤 되돌린다. 조금 뒤 캔버스의 그리기 크기가 화면의 크기와 같아야 한다(안 그러면 늘어난다).
    await pp.evaluate(() => document.fullscreenElement ? document.exitFullscreen().catch(() => {}) : null);
    await pp.waitForTimeout(300);
    await pp.evaluate(() => { window.__mqResize = window.onresize; window.onresize = null; });
    await pp.setViewportSize({ width: W, height: H });
    await pp.waitForTimeout(30);
    await pp.evaluate(() => { window.onresize = window.__mqResize; });
    await pp.waitForTimeout(1500);
    const fit = await pp.evaluate(() => {
      const c = document.getElementById('glcanvas'); const d = window.high_dpi ? devicePixelRatio : 1;
      return [c.width, c.height, Math.floor(c.clientWidth * d), Math.floor(c.clientHeight * d)];
    });
    if (fit[0] !== fit[2] || fit[1] !== fit[3]) errors.push('rotate: canvas size ' + fit.slice(0, 2) + ' != box ' + fit.slice(2) + ' after a late resize (stretched)');
    await pp.screenshot({ path: path.join(OUT, 't15-landscape-after-rotate.png') });
    await pctx.close();
  }
  await browser.close();
  server.close();
  for (const n of fs.readdirSync(OUT).filter((n) => /^t\d/.test(n))) console.log(`${n}\t${fs.statSync(path.join(OUT, n)).size} bytes`);
  if (errors.length) { console.error('console errors:\n' + errors.join('\n')); process.exit(1); }
  console.log('ok');
}
main().catch((e) => { console.error(e); process.exit(1); });
