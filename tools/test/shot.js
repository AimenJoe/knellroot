// 헤드리스 Chromium으로 웹 빌드를 띄워 시나리오마다 키를 넣고 스크린숏을 남긴다(nail tools/test/smoke.js를 줄인 것).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/shot.js dist shots
// 콘솔 에러·패닉이 있으면 종료 코드 1. 눈으로 볼 것: 카메라 전환, 예고 부채꼴, 랩 패널.
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8131;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm' };
// 알려진 잡음: mq_js_bundle의 quad_net 플러그인이 strict 모드에서 register_plugin= 대입(무해, nail web/README.md).
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
  const canvas = page.locator('#glcanvas');
  await canvas.click({ position: { x: 640, y: 360 } });
  const shot = (name) => page.screenshot({ path: path.join(OUT, name + '.png') });
  // 1 보스 아레나: 쿼터뷰 → 앞으로 걷기 → 락온 → 전환 중·전환 뒤 → 공격 → 회피 → 해제.
  await page.keyboard.press('Digit1'); await page.waitForTimeout(400);
  await shot('01-quarter');
  // 쿼터뷰 요 비교(랩 값 cam.quarter.yaw, 표 다섯째 줄 = 0부터 4): 기본 35 → 0 → 45 → 35.
  // 누름은 틱(16ms)마다 하나만 들어가므로 사이를 띄운다. Ctrl(10배)은 헤드리스에서 안 잡혀 쓰지 않는다.
  const tap = async (key, n) => { for (let i = 0; i < n; i++) { await page.keyboard.press(key); await page.waitForTimeout(30); } };
  await page.keyboard.press('F1'); await page.waitForTimeout(100);
  await tap('ArrowDown', 4);
  await tap('ArrowLeft', 35);
  await page.keyboard.press('F1'); await page.waitForTimeout(400); await shot('01a-yaw0');
  await page.keyboard.press('F1'); await page.waitForTimeout(100);
  await tap('ArrowRight', 45);
  await page.keyboard.press('F1'); await page.waitForTimeout(400); await shot('01b-yaw45');
  await page.keyboard.press('F1'); await page.waitForTimeout(100);
  await page.keyboard.press('KeyR'); // 기본값(35)으로
  await page.keyboard.press('F1'); await page.waitForTimeout(400); await shot('01c-yaw35');
  await page.keyboard.down('KeyW'); await page.waitForTimeout(900); await page.keyboard.up('KeyW');
  await page.keyboard.press('KeyL'); await page.waitForTimeout(120); await shot('02-lock-blending');
  await page.waitForTimeout(700); await shot('03-lock-on');
  await page.keyboard.down('KeyD'); await page.waitForTimeout(600); await page.keyboard.up('KeyD'); await shot('04-lock-strafe');
  for (let i = 0; i < 6; i++) { await page.keyboard.press('KeyJ'); await page.waitForTimeout(250); }
  await shot('05-attack');
  await page.keyboard.press('KeyK'); await page.waitForTimeout(120); await shot('06-dodge');
  await page.waitForTimeout(1500); await shot('07-boss-telegraph');
  await page.keyboard.press('KeyL'); await page.waitForTimeout(150); await shot('08-unlock-blending');
  // 2 떼.
  await page.keyboard.press('Digit2'); await page.waitForTimeout(1500); await shot('09-horde');
  // 3 기둥 + 락온(카메라 당기기).
  await page.keyboard.press('Digit3'); await page.waitForTimeout(300);
  await page.keyboard.down('KeyS'); await page.waitForTimeout(1200); await page.keyboard.up('KeyS');
  await page.keyboard.press('KeyL'); await page.waitForTimeout(800); await shot('10-pillars-lock');
  // 랩 패널.
  await page.keyboard.press('F1'); await page.waitForTimeout(200);
  for (let i = 0; i < 4; i++) await page.keyboard.press('ArrowRight');
  await page.waitForTimeout(300); await shot('11-lab');
  await page.keyboard.press('KeyX'); await page.waitForTimeout(200);
  await page.keyboard.press('F1'); await page.waitForTimeout(100);
  // 쟁반 돌리기(0012): 시나리오 5에서 E(↻) — 도는 중 · 쏟아지는 중 · 다 돈 뒤, 대기 중 Q(↺)는 안 먹는다.
  await page.keyboard.press('Digit5'); await page.waitForTimeout(500); await shot('12-tray');
  await page.keyboard.press('KeyE'); await page.waitForTimeout(100); await shot('13-turn-mid');
  await page.waitForTimeout(200); await shot('14-turn-spill');
  await page.waitForTimeout(500); await shot('15-turn-done');
  await page.keyboard.press('KeyQ'); await page.waitForTimeout(300); await shot('16-turn-cooldown');
  await browser.close();
  server.close();
  for (const n of fs.readdirSync(OUT)) {
    const sz = fs.statSync(path.join(OUT, n)).size;
    console.log(`${n}\t${sz} bytes`);
  }
  if (errors.length) { console.error('console errors:\n' + errors.join('\n')); process.exit(1); }
  console.log('ok');
}
main().catch((e) => { console.error(e); process.exit(1); });
