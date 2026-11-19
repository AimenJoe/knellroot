// 헤드리스 시험의 기다림(2026-11-19): 시간이 아니라 게임의 장면을 기다린다.
// 게임이 맨 위 장면이 바뀐 프레임에 window.knellScene에 이름(Scene의 Debug — "Title"·"Slots { load: false }"·"Ask(Overwrite(0))"·"Play" …)을
// 적는다(src/screen.rs report_scene, web/index.html knell_scene). CI의 소프트웨어 GL은 10 fps 안팎이라 "300ms"가 몇 프레임인지 정해지지 않는다 —
// 장면이 바뀌는 누름 뒤에는 until(앞머리)로 기다리고, 못 오면 지금 장면을 말하며 멈춘다(실패가 곧 진단).
//   const { scene, until, step, press } = require('./scene')(page);
//   SCENES=1 node tools/test/<시험>.js … → 누름과 장면 바뀜을 차례대로 찍는다(새 길을 짤 때 무엇을 기다릴지 본다).
//   THROTTLE=10 node … → CPU를 그만큼 느리게(느린 CI 흉내, 따로 throttle(ctx, page)를 부른 스크립트만).
const LOG = !!process.env.SCENES;

module.exports = (page, gap = 80) => {
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
  return { scene, until, leave, step, flip, press, frames };
};

module.exports.throttle = async (ctx, page) => {
  if (!process.env.THROTTLE) return;
  const cdp = await ctx.newCDPSession(page);
  await cdp.send('Emulation.setCPUThrottlingRate', { rate: Number(process.env.THROTTLE) });
};

// 방랑자가 맞지 않게(개발판만 — src/screen.rs test_flags, World::unhurt): 긴 스크립트가 도중에 쓰러지면 그 뒤의 화면이 모두
// 엉뚱하다(2026-11-19 touch.js — 멈춤 버튼을 누를 때 이미 결과 화면이었다). 맞는 모습을 찍는 스크립트는 부르지 않는다. goto 전에.
module.exports.unhurt = (page) => page.addInitScript(() => { window.knellTestFlags = 1; });
