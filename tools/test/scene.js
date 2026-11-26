// 헤드리스 시험의 기다림(2026-11-19): 시간이 아니라 게임의 장면을 기다린다.
// 게임이 맨 위 장면이 바뀐 프레임에 window.knellScene에 이름(Scene의 Debug — "Title"·"Slots { load: false }"·"Ask(Overwrite(0))"·"Play" …)을
// 적는다(src/screen.rs report_scene, web/index.html knell_scene). CI의 소프트웨어 GL은 10 fps 안팎이라 "300ms"가 몇 프레임인지 정해지지 않는다 —
// 장면이 바뀌는 누름 뒤에는 until(앞머리)로 기다리고, 못 오면 지금 장면을 말하며 멈춘다(실패가 곧 진단).
//   const { scene, until, step, press } = require('./scene')(page);
//   SCENES=1 node tools/test/<시험>.js … → 누름과 장면 바뀜을 차례대로 찍는다(새 길을 짤 때 무엇을 기다릴지 본다).
//   THROTTLE=10 node … → CPU를 그만큼 느리게(느린 CI 흉내, 따로 throttle(ctx, page)를 부른 스크립트만).
// 길은 이름으로(0278 R13): 개발 창의 줄 · 든 무기는 세어 누르지 않고 dev(이름)/boot(page, 이름), 큰 창의 탭은 openTab(장면 이름),
// 개발 창을 메뉴로 지날 때는 devRow(줄 이름). 서버 · 브라우저 · 콘솔 에러는 serve · launch · watch 한 곳.
const LOG = !!process.env.SCENES;

module.exports = (page, gap = 80) => {
  // 서막 숲(0262)은 기본으로 건너뛴다 — 새 기록도 성소에서 연다(옛 스크립트의 길). 숲을 보는 스크립트는 `sceneKit.prologue(page)`.
  // 깃발은 다른 비트와 OR로(unhurt 1과 함께 쓴다), 서막을 켠 페이지(`__knellPrologue`)에는 걸지 않는다 — 부르는 차례와 상관없이.
  page.addInitScript(() => { if (!window.__knellPrologue) window.knellTestFlags = (window.knellTestFlags | 0) | 2; });
  if (LOG) {
    page.on('console', (m) => { if (m.text().startsWith('[scene] ')) console.log('   ' + m.text()); });
    page.addInitScript(() => { window.knellSceneLog = true; });
  }
  const scene = () => page.evaluate(() => window.knellScene || '');
  const frames = (n = 2) => page.evaluate((k) => new Promise((r) => {
    const tick = (i) => (i <= 0 ? r() : requestAnimationFrame(() => tick(i - 1)));
    tick(k);
  }), n);
  // 키 하나를 누르고 게임 프레임 둘을 기다린다(macroquad는 한 프레임에 같은 키 두 번을 한 번으로 센다 — KB wasm-build).
  const press = async (key, n = 1) => {
    for (let i = 0; i < n; i++) {
      if (LOG) console.log('  > ' + key);
      await page.keyboard.press(key);
      await frames(2);
      await page.waitForTimeout(gap);
    }
  };
  // want: 앞머리 글·정규식 또는 그 목록(하나라도 맞으면 — 화로의 큰 창은 /host: Hearth/처럼). 못 오면 지금 장면을 말하며 던진다.
  const until = async (want, what, ms = 20000) => {
    const list = (Array.isArray(want) ? want : [want]).map((w) => (w instanceof RegExp ? { re: w.source } : { pre: w }));
    try {
      await page.waitForFunction((ws) => ws.some((w) => {
        const s = window.knellScene || '';
        return w.re !== undefined ? new RegExp(w.re).test(s) : s.startsWith(w.pre);
      }), list, { timeout: ms, polling: 50 });
    } catch (e) {
      throw new Error(`${what}: "${list.map((w) => w.re || w.pre).join('" / "')}" 장면이 오지 않았다 — 지금 "${await scene()}"`);
    }
  };
  // 키를 누르고 장면이 (무엇으로든) 바뀔 때까지 — 탭 넘기기처럼 어디로 가는지는 알지만 이름을 적기 번거로울 때.
  const flip = async (key, what, ms = 20000) => {
    const before = await scene();
    await press(key);
    try {
      await page.waitForFunction((b) => (window.knellScene || '') !== b, before, { timeout: ms, polling: 50 });
    } catch (e) {
      throw new Error(`${what || key}: "${before}"에서 바뀌지 않았다`);
    }
  };
  // 지금 장면에서 벗어날 때까지(어디로 가는지 정해지지 않은 누름 — 멈춤이 마지막 탭에서 열린다 등).
  const leave = async (from, what, ms = 20000) => {
    try {
      await page.waitForFunction((f) => !(window.knellScene || '').startsWith(f), from, { timeout: ms, polling: 50 });
    } catch (e) {
      throw new Error(`${what}: "${from}"에서 벗어나지 않았다`);
    }
  };
  // 누르고 그 장면이 올 때까지.
  const step = async (key, want, what, n = 1) => { await press(key, n); await until(want, what || key); };
  // 큰 창의 탭을 이름으로(0278 R13): Q(또는 E)로 넘기며 장면 이름이 want(앞머리·정규식)에 닿을 때까지 — 몇 번인지 세지 않는다.
  // 한 바퀴(max)를 돌아도 안 오면 지금 장면을 말하며 던진다. 넘기는 쪽은 옛 스크립트가 누르던 쪽 그대로(지나는 탭이 상태를 바꾼다 —
  // 가방 탭을 떠나면 새 것의 점이 지워진다).
  const openTab = async (want, what, key = 'KeyE', max = 8) => {
    const ok = (s) => (want instanceof RegExp ? want.test(s) : s.startsWith(want));
    for (let i = 0; i < max && !ok(await scene()); i++) await flip(key, `${what || want} — ${key} ${i + 1}`);
    await until(want, what || String(want));
  };
  // 개발 명령(0278 R13 — src/app/dev.rs `DevCmd`): 이름을 window.knellDev에 쌓고 게임이 다 꺼낼 때까지 + 프레임 둘. 이름은 개발 창 줄
  // (`Gate` · `Scenario(Line)`) · `Dev(<줄>)`(그 줄에 커서를 두고 개발 창을 연다) · `Arm(<무기>)` · `Arm(Sword+Shield)` · `ArmSub(<무기>|None)`.
  // 모르는 이름이면 게임이 콘솔에 `knellDev: …`를 남기고 여기서 던진다. 장면이 바뀌는 명령 뒤에는 until로 그 장면을 기다린다.
  const devLog = [];
  page.on('console', (m) => { if (m.text().startsWith('knellDev:')) devLog.push(m.text()); });
  const dev = async (...names) => {
    const before = devLog.length;
    if (LOG) console.log('  > knellDev ' + names.join(' · '));
    await page.evaluate((ns) => { window.knellDev = (window.knellDev || []).concat(ns); }, names);
    await page.waitForFunction(() => !(window.knellDev && window.knellDev.length), null, { timeout: 20000, polling: 50 });
    await frames(2);
    if (devLog.length > before) throw new Error(devLog.slice(before).join(' / '));
  };
  // 개발 창의 줄을 이름으로 고른다(창을 메뉴로 지나는 길 — shot.js가 개발 창 자체를 지킨다): 게임이 고른 줄을 장면 이름 끝에
  // 붙인다(`Dev { tab: 3 } @Gate`, 개발판). 먼저 →로 묶음을 한 바퀴 돌며 첫 줄만 보고(묶음을 넘기면 커서는 첫 줄 — 싸다), 없으면
  // 묶음마다 ↓로 훑는다(돌아오면 그만). 고르기만 — 확인(Enter)은 부른 쪽이.
  const devRow = async (name) => {
    const hit = (s) => s.endsWith(` @${name}`);
    await until('Dev {', `개발 창 (${name})`);
    const tabOf = (s) => (s.match(/^Dev \{ tab: (\d+)/) || [])[1];
    const start = tabOf(await scene());
    for (let t = 0; t < 8; t++) {
      if (hit(await scene())) return;
      await flip('ArrowRight', `개발 창 묶음 넘기기 (${name})`);
      if (tabOf(await scene()) === start) break;
    }
    for (let t = 0; t < 8; t++) {
      let s = await scene();
      const seen = new Set([s]);
      while (!hit(s)) {
        await press('ArrowDown');
        s = await scene();
        if (seen.has(s)) break;
        seen.add(s);
      }
      if (hit(s)) return;
      await flip('ArrowRight', `개발 창 묶음 넘기기 (${name})`);
    }
    throw new Error(`개발 창에 ${name} 줄이 없다 — 지금 "${await scene()}"`);
  };
  // "키보드만 배치로"(0058 — J·K·L로 싸우는 스크립트): 타이틀 → 설정(둘째) → 조작(타이틀의 설정에서는 맨 끝 줄 — 위로 한 번, 0164) →
  // "키보드만 배치로"(끝에서 둘째) → 돌아가기 둘. 끝나면 타이틀이고 커서는 "설정"에 있다(0066 — 닫으면 연 줄로).
  const keyboardOnly = async () => {
    await press('ArrowDown'); await press('Enter'); await until('Settings { host: Title', '타이틀 → 설정');
    await press('ArrowUp'); await press('Enter'); await until('Controls', '설정 → 조작');
    await press('ArrowUp', 2); await press('Enter'); await page.waitForTimeout(200);
    await press('Escape'); await until('Settings { host: Title', '조작 → 설정'); await press('Escape'); await until('Title', '설정 → 타이틀');
  };
  return { scene, until, leave, step, flip, press, frames, openTab, dev, devRow, keyboardOnly };
};

// 개발 명령을 처음부터(0278 R13): goto 전에 부르면 게임의 첫 프레임이 그 이름들을 차례로 한다 — 타이틀을 거치지 않고 판에 선다
// (`sceneKit.boot(page, 'Arm(Wand)', 'Scenario(Line)')`). PC의 `KNELL_BOOT=Arm(Wand),Scenario(Line)`과 같은 이름. 다시 불러올 때마다
// 또 하니 reload하는 페이지에는 쓰지 않는다.
module.exports.boot = (page, ...names) => page.addInitScript((ns) => { window.knellDev = ns; }, names);

// 서버 하나(0278 R13 — 스크립트마다 베껴 쓰던 것): dist를 그대로 낸다. drip = wasm을 조각씩 천천히(pwa.js — 로딩 막대를 본다).
//   const srv = await sceneKit.serve(DIST, { port: 8131 }); … page.goto(srv.url) … srv.close()
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain', '.json': 'application/json' };
module.exports.serve = async (dist, { port = 0, drip = null } = {}) => {
  const http = require('http'); const fs = require('fs'); const path = require('path');
  const root = path.resolve(dist);
  const server = http.createServer((req, res) => {
    const f = path.join(root, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
    if (!f.startsWith(root) || !fs.existsSync(f) || !fs.statSync(f).isFile()) { res.writeHead(404); return res.end(); }
    const body = fs.readFileSync(f);
    res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream', 'Content-Length': body.length });
    if (!drip || !f.endsWith('.wasm')) return res.end(body);
    let at = 0;
    const step = () => { if (at >= body.length) return res.end(); res.write(body.subarray(at, at + drip.size)); at += drip.size; setTimeout(step, drip.ms); };
    step();
  });
  await new Promise((r) => server.listen(port, r));
  const p = server.address().port;
  return { port: p, url: `http://localhost:${p}/`, close: () => server.close() };
};

// 브라우저 하나(0278 R13): headless Chromium. args는 스크립트가 따로 필요한 것만(away.js — 소리 자동 재생).
module.exports.launch = (args = []) => require('playwright').chromium.launch(args.length ? { args } : {});

// 콘솔 에러 · 페이지 에러를 모은다(알려진 잡음 — mq_js_bundle quad_net의 register_plugin 대입 — 은 빼고).
const NOISE = [/register_plugin/];
module.exports.watch = (page, errors, noise = []) => {
  const quiet = (t) => NOISE.concat(noise).some((k) => k.test(t));
  page.on('console', (m) => { if (m.type() === 'error' && !quiet(m.text())) errors.push(m.text()); });
  page.on('pageerror', (e) => { if (!quiet(String(e))) errors.push(String(e)); });
};

module.exports.throttle = async (ctx, page) => {
  if (!process.env.THROTTLE) return;
  const cdp = await ctx.newCDPSession(page);
  await cdp.send('Emulation.setCPUThrottlingRate', { rate: Number(process.env.THROTTLE) });
};

// 방랑자가 맞지 않게(개발판만 — src/screen.rs test_flags, World::unhurt): 긴 스크립트가 도중에 쓰러지면 그 뒤의 화면이 모두
// 엉뚱하다(2026-11-19 touch.js — 멈춤 버튼을 누를 때 이미 결과 화면이었다). 맞는 모습을 찍는 스크립트는 부르지 않는다. goto 전에.
module.exports.unhurt = (page) => page.addInitScript(() => { window.knellTestFlags = (window.knellTestFlags | 0) | 1; });

// 적을 세워 둔다(개발판만 — test_flags 비트 16): 동작·이펙트를 틱마다 찍을 때 몰려든 적이 방랑자를 가리지 않게. 휘청임은 그대로. goto 전에.
module.exports.still = (page) => page.addInitScript(() => { window.knellTestFlags = (window.knellTestFlags | 0) | 16; });

// 싸게 그리기(개발판만 — test_flags 비트 8: 월드의 줄 수 270·2배 그리기 끔): 흐름·기록·터치 자리만 보는 스크립트용. 헤드리스의
// 소프트웨어 GL은 화소 수가 곧 시간이다 — 화면 크기와 UI 배치는 그대로다. 그림을 보는 스크립트(shot.js·fx.js·menu_res.js)는 부르지
// 않는다. `FULL=1`이면 걸지 않는다(찍힌 그림을 제 화질로 보고 싶을 때). goto 전에.
module.exports.cheap = (page) => page.addInitScript(() => {
  if (!window.__knellFull) window.knellTestFlags = (window.knellTestFlags | 0) | 8;
});
if (process.env.FULL) {
  const cheap = module.exports.cheap;
  module.exports.cheap = (page) => page.addInitScript(() => { window.__knellFull = true; }).then(() => cheap(page));
}

// 서막 숲부터(0262 — 개발판만, src/screen.rs test_flags 비트 2를 푼다): 새 기록이 숲의 빈터에서 깨어난다. goto 전에, sceneKit(page) 앞이든 뒤든.
module.exports.prologue = (page) => page.addInitScript(() => {
  window.__knellPrologue = true;
  window.knellTestFlags = (window.knellTestFlags | 0) & ~2;
});
