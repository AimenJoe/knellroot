// 이펙트를 틱마다 찍는다(0141): 헤드리스 Chromium으로 개발 시나리오를 띄우고, 개발 키 F6(한 프레임 = 한 틱)을 켠 뒤
// 동작마다 연달아 스크린숏 — 0.1초짜리 번쩍임·먼지를 실제 시간으로는 못 잡는다(헤드리스는 초당 8프레임 남짓).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/fx.js dist shots-fx [duel,horde,boss,stage]
// 눈으로 볼 것: 맞힌 자리의 별·불똥, 휘두름의 초승달, 쓰러진 적의 재, 뛰기·회피·적 내리침의 먼지, 마법 탄의 마름모, 화로 불티.
// 메뉴 길은 shot.js와 같다(처음에 "키보드만 배치로" — J 공격·K 패링·L 락온·Space 회피·F 점프·U 마법). 콘솔 에러가 있으면 종료 코드 1.
const { chromium } = require('playwright'); const http = require('http'); const fs = require('fs'); const path = require('path');
const DIST = path.resolve(process.argv[2] || 'dist'); const OUT = path.resolve(process.argv[3] || 'shots-fx'); const PORT = 8141;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm' };
const server = http.createServer((req, res) => { const f = path.join(DIST, req.url === '/' ? 'index.html' : decodeURIComponent(req.url.split('?')[0]));
  if (!f.startsWith(DIST) || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] || 'application/octet-stream' }); fs.createReadStream(f).pipe(res); });
(async () => {
  fs.mkdirSync(OUT, { recursive: true }); await new Promise(r => server.listen(PORT, r));
  const b = await chromium.launch(); const ctx = await b.newContext({ viewport: { width: 1280, height: 720 } });
  const errors = []; let first = true;
  const open = async (tab, row) => {
    const p = await ctx.newPage();
    p.on('console', m => { if (m.type() === 'error' && !/register_plugin/.test(m.text())) errors.push(m.text()); });
    p.on('pageerror', e => { if (!/register_plugin/.test(String(e))) errors.push(String(e)); });
    await p.goto(`http://localhost:${PORT}/`); await p.waitForTimeout(2500); await p.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
    const press = async (k, n = 1) => { for (let i = 0; i < n; i++) { await p.keyboard.press(k); await p.waitForTimeout(70); } };
    if (first) { first = false;
      await press('ArrowDown'); await press('Enter'); await p.waitForTimeout(300); await press('ArrowUp', 2); await press('Enter'); await p.waitForTimeout(300);
      await press('ArrowUp', 2); await press('Enter'); await p.waitForTimeout(200); await press('Escape'); await p.waitForTimeout(200); await press('Escape'); await p.waitForTimeout(300); await press('ArrowUp'); }
    await press('ArrowUp'); await press('Enter'); await p.waitForTimeout(300); await press('ArrowRight', tab); await p.waitForTimeout(150); await press('ArrowDown', row); await press('Enter');
    return p;
  };
  const burst = async (p, name, n) => { for (let i = 0; i < n; i++) await p.screenshot({ path: path.join(OUT, `${name}-${i}.png`) }); };
  const hold = async (p, k, ms) => { await p.keyboard.down(k); await p.waitForTimeout(ms); await p.keyboard.up(k); };
  const only = process.argv[4] ? process.argv[4].split(',') : null;
  const want = (n) => !only || only.includes(n);
  let p;
  if (want('duel')) {
    // 결투(잔재 + 가시짐승): 락온 → 한 프레임 = 한 틱(F6) → 평타·마법·회피·내려찍기·강타
    p = await open(2, 2); await p.waitForTimeout(900); await p.keyboard.press('KeyL'); await p.waitForTimeout(600);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 3; k++) { await p.keyboard.press('KeyJ'); await burst(p, `hit${k}`, 12); }
    await p.keyboard.press('KeyU'); await burst(p, 'cast', 6);
    await p.keyboard.press('Space'); await burst(p, 'dodge', 8);
    await p.keyboard.press('KeyF'); await burst(p, 'jump', 5); await p.keyboard.press('KeyJ'); await burst(p, 'plunge', 16);
    await p.keyboard.down('KeyJ'); await burst(p, 'charge', 14); await p.keyboard.up('KeyJ'); await burst(p, 'heavy', 14);
    await p.close();
  }
  if (want('horde')) {
    p = await open(2, 8); await p.waitForTimeout(1500); await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 5; k++) { await p.keyboard.press('KeyJ'); await burst(p, `horde${k}`, 10); }
    await p.close();
  }
  if (want('boss')) {
    p = await open(2, 0); await p.waitForTimeout(800); await hold(p, 'KeyW', 900); await p.keyboard.press('KeyL'); await p.waitForTimeout(700);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    for (let k = 0; k < 10; k++) { await p.keyboard.press('KeyK'); await burst(p, `parry${k}`, 6); }
    await p.close();
  }
  if (want('stage')) {
    // 입구: 화로 불티 + 회피·달리기 먼지(진짜 바닥)
    p = await open(1, 0); await p.waitForTimeout(6500); await hold(p, 'KeyW', 2200); await hold(p, 'KeyD', 2500); await p.waitForTimeout(500);
    await p.keyboard.press('F6'); await p.waitForTimeout(300);
    await burst(p, 'brazier', 3); await p.keyboard.down('KeyD'); await p.keyboard.press('Space'); await burst(p, 'stagedodge', 10); await p.keyboard.up('KeyD');
    await p.close();
  }
  await b.close(); server.close(); if (errors.length) { console.error('console errors:\n' + errors.slice(0, 5).join('\n')); process.exit(1); } console.log('ok');
})();
