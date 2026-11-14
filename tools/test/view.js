// 시점(0181): 십자 ↑·↓(키보드 ↑·↓) = 쿼터뷰 거리 단계 → 가장 가까운 단계에서 한 번 더 = 자유 숄더뷰, ←·→ = 그 시점 돌리기.
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/view.js dist shots/view
// 콘솔 에러·패닉이 있으면 종료 코드 1. 눈으로 볼 것: 거리 셋이 다른지, 숄더뷰로 넘어가는지, ←·→로 도는지, W가 카메라 앞으로 걷는지.
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots/view');
const PORT = 8139;
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
  const press = async (key, n = 1) => { for (let i = 0; i < n; i++) { await page.keyboard.press(key); await page.waitForTimeout(80); } };
  const hold = async (key, ms) => { await page.keyboard.down(key); await page.waitForTimeout(ms); await page.keyboard.up(key); };
  // 키보드만 배치로(shot.js와 같은 길) → 시작하기 → 첫 슬롯 → 깨어나는 장면 건너뛰기.
  await press('ArrowDown'); await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowUp'); await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowUp', 2); await press('Enter'); await page.waitForTimeout(200);
  await press('Escape'); await page.waitForTimeout(200); await press('Escape'); await page.waitForTimeout(300);
  await press('ArrowUp'); await page.waitForTimeout(100);
  await press('Enter'); await page.waitForTimeout(300);
  await press('Enter'); await page.waitForTimeout(1500);
  await press('Escape'); await page.waitForTimeout(6500);
  await shot('v0-quarter-home');
  await press('ArrowDown'); await page.waitForTimeout(700); await shot('v1-quarter-far');
  await press('ArrowUp', 2); await page.waitForTimeout(700); await shot('v2-quarter-near');
  await press('ArrowUp'); await page.waitForTimeout(900); await shot('v3-free');
  await hold('ArrowRight', 700); await page.waitForTimeout(300); await shot('v4-free-look-right');
  await hold('KeyW', 1200); await page.waitForTimeout(300); await shot('v5-free-walk-forward');
  await press('ArrowDown'); await page.waitForTimeout(900); await shot('v6-back-to-quarter');
  await browser.close();
  server.close();
  console.log(errors.length ? errors.join('\n') : 'ok');
  process.exitCode = errors.length ? 1 : 0;
}
main().catch((e) => { console.error(e); process.exitCode = 1; server.close(); });
