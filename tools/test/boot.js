// 운영 빌드(`--no-default-features`)를 띄워 본다 — main에서 배포 직전에 도는 가벼운 연기 시험(0271).
//   cargo build --release --target wasm32-unknown-unknown --no-default-features && sh tools/web/assemble.sh dist local release
//   node tools/test/boot.js dist shots
// 보는 것: 1) 타이틀까지 뜬다, 2) 개발 메뉴가 없다(타이틀의 끝 줄이 설정이다), 3) 새 판이 열리고 깨어나는 장면을 넘겨 몇 초 돈다,
// 4) 그동안 콘솔 에러·패닉이 없다. 긴 시나리오(shot·touch·save)는 개발판 빌드로 PR 검사에서 이미 돌았다 — 여기는 실제로 나가는 것만.
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');
const sceneKit = require('./scene');

const DIST = path.resolve(process.argv[2] || 'dist');
const OUT = path.resolve(process.argv[3] || 'shots');
const PORT = 8163;
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.webmanifest': 'application/manifest+json', '.png': 'image/png', '.txt': 'text/plain' };
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
  const { until, press, scene } = sceneKit(page, 60);
  page.on('console', (m) => { if (m.type() === 'error' && !KNOWN_NOISE.some((k) => k.test(m.text()))) errors.push(m.text()); });
  page.on('pageerror', (e) => { if (!KNOWN_NOISE.some((k) => k.test(String(e)))) errors.push(String(e)); });
  const shot = (name) => page.screenshot({ path: path.join(OUT, name + '.png') });

  await page.goto(`http://localhost:${PORT}/`);
  await until('Title', '타이틀', 60000);
  await page.waitForTimeout(500);
  await page.locator('#glcanvas').click({ position: { x: 4, y: 4 } });
  await shot('boot-title');

  // 타이틀의 끝 줄: 운영판은 설정, 개발판은 개발 — 위로 한 번(돌아서) 누르면 갈린다.
  await press('ArrowUp'); await press('Enter');
  await until(/^(Settings|Dev)/, '타이틀의 끝 줄');
  const last = await scene();
  if (!last.startsWith('Settings')) throw new Error(`운영 빌드가 아니다 — 타이틀의 끝 줄이 ${last}(개발 메뉴가 실려 있다)`);
  await press('Escape'); await until('Title', '설정 닫기');

  // 새 판: 시작(설정에서 위로 한 번) → 슬롯 창 → 빈 슬롯 → 깨어나는 장면 → 멈춤 키로 넘긴다 → 몇 초 돈다.
  await press('ArrowUp'); await press('Enter'); await until('Slots { load: false', '시작하기 → 슬롯 창');
  await press('Enter'); await until('Play', '빈 슬롯 → 새 판');
  await page.waitForTimeout(2500); await shot('boot-wake');
  await press('Escape'); await page.waitForTimeout(4000); await shot('boot-play');
  const end = await scene();
  if (!end.startsWith('Play')) throw new Error(`새 판이 돌다가 장면이 ${end}가 됐다`);

  await browser.close();
  server.close();
  if (errors.length) { console.error('console errors:\n' + errors.join('\n')); process.exit(1); }
  console.log('ok');
}
main().catch((e) => { console.error(e); process.exit(1); });
