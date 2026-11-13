// 헤드리스 Chromium으로 웹 빌드를 띄워 시나리오마다 키를 넣고 스크린숏을 남긴다(nail tools/test/smoke.js를 줄인 것).
//   cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && node tools/test/shot.js dist shots
// 콘솔 에러·패닉이 있으면 종료 코드 1. 눈으로 볼 것: 첫 스테이지(타이틀·이야기·뜰·멈춤·문지기), 카메라 전환, 예고, 랩 패널.
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
  // 모서리를 눌러 초점만(가운데를 누르면 메뉴 줄을 탭한다 — 마우스도 메뉴를 뗄 때 탭, 0058).
  await canvas.click({ position: { x: 4, y: 4 } });
  const shot = (name) => page.screenshot({ path: path.join(OUT, name + '.png') });
  const press = async (key, n = 1) => { for (let i = 0; i < n; i++) { await page.keyboard.press(key); await page.waitForTimeout(60); } };
  // 키보드만 배치로(0058 — 이 스크립트는 J·K·L로 싸운다. 새로 설치한 판의 기본은 키보드+마우스):
  // 타이틀 → 설정(둘째) → 조작(타이틀의 설정에서는 맨 끝 줄 — 위로 한 번, 0164) → "키보드만 배치로"(끝에서 둘째) → 돌아가기 둘.
  const keyboardOnly = async () => {
    await press('ArrowDown'); await press('Enter'); await page.waitForTimeout(300);
    await press('ArrowUp'); await press('Enter'); await page.waitForTimeout(300);
    await press('ArrowUp', 2); await press('Enter'); await page.waitForTimeout(200);
    await press('Escape'); await page.waitForTimeout(200); await press('Escape'); await page.waitForTimeout(300);
    // 닫으면 연 줄로 돌아온다(0066 장면 스택): 타이틀 커서는 "설정"에 있다 → 위로 한 번 = 시작.
    await press('ArrowUp'); await page.waitForTimeout(100);
  };
  await keyboardOnly();
  // 0 첫 스테이지(0019): 타이틀 → 시작 이야기 → 입구 → 뜰(첫 물결) → 멈춤 → 타이틀 → 개발: 문 앞 → 깨어남 → 결투(락온) → 멈춤 → 조작 설정.
  await shot('00-title');
  // 시작하기 → 슬롯 창(0066, 커서는 첫 빈 슬롯) → "시작".
  await press('Enter'); await page.waitForTimeout(300); await shot('00-slots-new');
  await press('Enter'); await page.waitForTimeout(1500); await shot('00a-intro');
  await page.waitForTimeout(5000); await shot('00b-entrance');
  await page.keyboard.down('KeyW'); await page.waitForTimeout(3500); await page.keyboard.up('KeyW');
  await page.waitForTimeout(900); await shot('00c-courtyard');
  for (let i = 0; i < 5; i++) { await page.keyboard.press('KeyJ'); await page.waitForTimeout(260); }
  await shot('00d-fight');
  await press('Escape'); await page.waitForTimeout(300); await shot('00e-pause');
  // 멈춤 = 큰 창(0164): 탭 상태·장비·기술·도감·설정(Q·E로 넘긴다 — 끝에서 돈다), 이 판에서 마지막에 보던 탭에서 열린다(새 판은 상태).
  // 설정 탭의 끝 줄들: … 조작 설정·개발(개발판)·처음부터·타이틀로. 타이틀: 시작·설정·개발(0094).
  // 개발 창(0083, nail 구조): 위 묶음 탭(도구 · 스테이지 · 전투·적 · 성능, 좌우로) + 그 묶음의 줄 — 스테이지 둘째 줄 = 문 앞, 전투·적 첫 줄 = 보스 아레나.
  // 타이틀로는 묻는 창(0064 — 커서가 "취소"에서 시작): 위로 한 번 = "타이틀로".
  // 타이틀의 DEV는 맨 끝 — 기록이 생겨 "이어하기"가 붙어도 위로 한 번(돌아서)이면 DEV(0066).
  // 상태 탭에서 Q = 설정 탭(끝에서 돈다) → 위로 한 번 = 맨 끝 줄 "타이틀로".
  await press('KeyQ'); await page.waitForTimeout(200); await shot('00e1-pause-settings');
  await press('ArrowUp'); await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowUp'); await press('Enter'); await page.waitForTimeout(300);
  await shot('00e2-title-continue');
  await press('ArrowUp'); await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowRight'); await page.waitForTimeout(150); await press('ArrowDown', 1); await press('Enter'); await page.waitForTimeout(800); await shot('00f-gate');
  await page.keyboard.down('KeyW'); await page.waitForTimeout(2200); await page.keyboard.up('KeyW');
  await page.waitForTimeout(800); await shot('00g-waking');
  await page.waitForTimeout(1500); await press('KeyL'); await page.waitForTimeout(900); await shot('00h-boss-lock');
  // 기술(0151 → 0161 기술 화면 → 0164 탭): 멈춤(새 판이라 상태 탭) → E 둘 = 기술 탭 — 칸을 오른쪽으로 한 번 옮겨 본 뒤 되돌리고 닫는다.
  await press('Escape'); await page.waitForTimeout(300); await shot('00h0-status');
  await press('KeyE'); await page.waitForTimeout(200); await shot('00h0-equip');
  await press('KeyE'); await page.waitForTimeout(300);
  await press('ArrowRight'); await page.waitForTimeout(200); await shot('00h1-skills');
  await press('ArrowLeft'); await page.waitForTimeout(150); await press('Escape'); await page.waitForTimeout(300);
  // 설정(0052 → 0164 탭): 다시 멈추면 마지막에 보던 기술 탭에서 열린다 → E 둘(도감 → 설정). 조작 설정은 끝에서 넷째
  // (개발·처음부터·타이틀로 위 — 위로 네 번, 전체 화면 줄이 있든 없든).
  await press('Escape'); await page.waitForTimeout(300);
  await press('KeyE'); await page.waitForTimeout(200); await shot('00h1-codex');
  await press('KeyE'); await page.waitForTimeout(300);
  await shot('00h2-settings');
  await press('ArrowUp', 4); await press('Enter'); await page.waitForTimeout(300);
  await shot('00i-controls');
  await press('Escape'); await page.waitForTimeout(200);
  // 개발 시나리오로(설정 탭 → 타이틀로 → 묻는 창 → DEV → 첫 PoC).
  // 닫으면 연 줄로(0066): 조작 → 설정 탭의 "조작 설정"에 커서가 있다 → 아래로 셋 = 타이틀로(개발·처음부터 다음, 0090).
  await press('ArrowDown', 3); await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowUp'); await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowUp'); await press('Enter'); await page.waitForTimeout(300);
  await shot('00j-dev');
  await press('ArrowRight', 2); await press('Enter'); await page.waitForTimeout(400);  // 개발 창: 전투·적 묶음 첫 줄 = 보스 아레나(0083)
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
  await page.keyboard.press('Space'); await page.waitForTimeout(120); await shot('06-dodge');
  await page.waitForTimeout(400); await page.keyboard.press('KeyK'); await page.waitForTimeout(60); await shot('06b-parry');
  await page.waitForTimeout(600); await page.keyboard.press('KeyF'); await page.waitForTimeout(200); await shot('06c-jump');
  await page.keyboard.press('KeyJ'); await page.waitForTimeout(250); await shot('06d-plunge');
  // 강공격(0030): J를 꾹 — 모으기 고리, 떼면 강타.
  await page.waitForTimeout(600); await page.keyboard.down('KeyJ'); await page.waitForTimeout(800); await shot('06e-charge');
  await page.keyboard.up('KeyJ'); await page.waitForTimeout(130); await shot('06f-heavy');
  // 등잔(0031): R — 다쳤으면 마시고, 가득이면 HUD 등잔이 깜빡인다.
  await page.waitForTimeout(700); await page.keyboard.press('KeyR'); await page.waitForTimeout(380); await shot('06g-flask');
  await page.waitForTimeout(1500); await shot('07-boss-telegraph');
  await page.keyboard.press('KeyL'); await page.waitForTimeout(150); await shot('08-unlock-blending');
  // 2 떼.
  await page.keyboard.press('Digit2'); await page.waitForTimeout(1500); await shot('09-horde');
  // 3 기둥 + 락온(카메라 당기기).
  await page.keyboard.press('Digit3'); await page.waitForTimeout(300);
  await page.keyboard.down('KeyS'); await page.waitForTimeout(1200); await page.keyboard.up('KeyS');
  await page.keyboard.press('KeyL'); await page.waitForTimeout(800); await shot('10-pillars-lock');
  // 랩 창(0070): F1 → 묶음 목록(위 넷: A/B·담기·내보내기·되돌리기) → 첫 묶음(카메라 쿼터뷰) → → 넷 → 내보내기 → F1로 닫기.
  await page.keyboard.press('F1'); await page.waitForTimeout(300); await shot('11-lab');
  await press('ArrowDown', 4); await press('Enter'); await page.waitForTimeout(300);
  await press('ArrowRight', 4); await page.waitForTimeout(200); await shot('11b-lab-group');
  await press('Escape'); await page.waitForTimeout(200);
  await press('ArrowUp', 2); await press('Enter'); await page.waitForTimeout(200);
  await page.keyboard.press('F1'); await page.waitForTimeout(200);
  // 쟁반 돌리기(0012): 시나리오 5에서 C(↻, 0151 — 예전 E) — 도는 중(적은 그대로) · 멈춘 순간 쏟아짐·휘청 · 대기 중 Z(↺)는 안 먹는다.
  await page.keyboard.press('Digit5'); await page.waitForTimeout(500); await shot('12-tray');
  await page.keyboard.press('KeyC'); await page.waitForTimeout(100); await shot('13-turn-mid');
  await page.waitForTimeout(400); await shot('14-turn-turning');
  await page.waitForTimeout(700); await shot('15-turn-landed-spill');
  await page.keyboard.press('KeyZ'); await page.waitForTimeout(300); await shot('16-turn-cooldown');
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
