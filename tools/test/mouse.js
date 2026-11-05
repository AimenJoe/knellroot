// 키보드+마우스 배치(0058) 헤드리스 확인: 기본 배치 그대로(새로 설치한 판), 개발 PoC 보스 아레나에서
//   빈 방에서 커서를 플레이어 오른쪽·왼쪽에 두고 오른쪽 클릭(마법) → 탄이 커서 쪽으로, 조준 표시, 휠 클릭 락온.
//   캔버스의 오른쪽 클릭 메뉴·휠 클릭 자동 스크롤이 막혔는지(web/index.html).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/mouse.js dist shots
// 콘솔 에러·패닉이나 막히지 않은 브라우저 동작이 있으면 종료 코드 1. 눈으로 볼 것: m1·m2의 탄 방향, 조준 표시, m3 락온.
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8133;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm' };
const KNOWN_NOISE = [/ReferenceError: register_plugin is not defined/];

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
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  const errors = [];
  page.on('console', (m) => { if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(m.text()))) errors.push(m.text()); });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(String(e)); });
  await page.goto(`http://localhost:${PORT}/`);
  await page.waitForTimeout(2500);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  const shot = (name) => page.screenshot({ path: path.join(OUT, name + '.png') });
  const press = async (key, n = 1) => { for (let i = 0; i < n; i++) { await page.keyboard.press(key); await page.waitForTimeout(60); } };

  // 브라우저 기본 동작이 막혔나(캔버스에 보낸 이벤트의 defaultPrevented).
  const blocked = await page.evaluate(() => {
    const c = document.getElementById('glcanvas');
    const ev = (type, button) => { const e = new MouseEvent(type, { button, bubbles: true, cancelable: true }); c.dispatchEvent(e); return e.defaultPrevented; };
    return { contextmenu: ev('contextmenu', 2), middle: ev('mousedown', 1) };
  });
  if (!blocked.contextmenu) errors.push('right-click menu is not blocked on the canvas');
  if (!blocked.middle) errors.push('middle-click autoscroll is not blocked on the canvas');
  await page.mouse.up({ button: 'middle' });

  // 설정에 마우스 줄이 보이나(타이틀 → 설정).
  await press('ArrowDown'); await press('Enter'); await page.waitForTimeout(300);
  await shot('m0-settings');
  await press('Escape'); await page.waitForTimeout(300);
  // DEV → PoC 첫째(보스 아레나). 타이틀: 시작·설정·DEV, DEV: 문 앞·무기·PoC·돌아가기.
  await press('ArrowDown', 2); await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowDown', 2); await press('Enter'); await page.waitForTimeout(1200);
  const cast = async (x, y, name) => {
    // 커서를 옮겨 "만졌다"로 만들고 바로 오른쪽 클릭(쉼 시간 1초 안).
    await page.mouse.move(x - 40, y); await page.waitForTimeout(50);
    await page.mouse.move(x, y, { steps: 4 }); await page.waitForTimeout(80);
    await page.mouse.down({ button: 'right' }); await page.waitForTimeout(40); await page.mouse.up({ button: 'right' });
    await page.waitForTimeout(220); await shot(name);
    await page.waitForTimeout(900);
  };
  // 빈 방(시나리오 4)에서 오른쪽·왼쪽 — 적이 없어 탄이 끝까지 보인다.
  await press('Digit4'); await page.waitForTimeout(800);
  await cast(1050, 400, 'm1-cast-right');
  await cast(230, 400, 'm2-cast-left');
  await press('Digit1'); await page.waitForTimeout(800);
  // 휠 클릭 = 락온, 락온 중에는 조준 표시가 없다.
  await page.mouse.move(700, 300, { steps: 3 });
  await page.mouse.down({ button: 'middle' }); await page.waitForTimeout(40); await page.mouse.up({ button: 'middle' });
  await page.waitForTimeout(900); await shot('m3-lock');
  // 왼쪽 클릭 = 공격.
  await page.mouse.down(); await page.waitForTimeout(40); await page.mouse.up();
  await page.waitForTimeout(150); await shot('m4-attack');

  await browser.close();
  server.close();
  if (errors.length) { console.error(errors.join('\n')); process.exit(1); }
  console.log('ok', OUT);
}

main().catch((e) => { console.error(e); process.exit(1); });
