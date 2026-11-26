// P5 측정(0013): 시나리오 6(잡몹 40)·7(80)을 띄워 HUD 측정 줄(cpu ms·draws·verts)을 스크린숏으로 남긴다.
//   node tools/test/stress.js dist shots
// 헤드리스 Chromium은 소프트웨어 렌더라 fps·frame ms는 판정 근거가 아니다 — cpu ms(우리 코드)와 draws(그리기 호출 수)를 본다.
// 실기 판정은 사람이 폰·PC 브라우저로(개발판 → 6/7, 개발 줄 F3). 상자/glTF 스위치(render.model)는 0070에서 걷었다.
// 판은 개발 명령의 이름으로 연다(0278 R13 — 예전에는 타이틀에서 숫자 키 6·7을 눌러 타이틀을 찍고 있었다: 숫자 키는 놀이터에서만 듣는다).
const fs = require('fs'), path = require('path');
const sceneKit = require('./scene');
const DIST = path.resolve(process.argv[2] || 'dist'); const OUT = path.resolve(process.argv[3] || 'shots');
(async () => {
  fs.mkdirSync(OUT, { recursive: true });
  const server = await sceneKit.serve(DIST, { port: 8151 });
  const browser = await sceneKit.launch();
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  const errors = [];
  sceneKit.watch(page, errors);
  const { until, dev } = sceneKit(page);
  await page.goto(server.url); await until('Title', '타이틀', 60000);
  for (const [row, name] of [['Scenario(Stress40)', 's40'], ['Scenario(Stress80)', 's80']]) {
    await dev(row); await until('Sandbox', row); await page.waitForTimeout(2500);
    await page.screenshot({ path: path.join(OUT, `${name}.png`) });
  }
  await browser.close(); server.close();
  for (const n of fs.readdirSync(OUT)) console.log(n);
  if (errors.length) { console.error('console errors:\n' + errors.join('\n')); process.exit(1); }
  console.log('ok');
})().catch((e) => { console.error(e); process.exit(1); });
