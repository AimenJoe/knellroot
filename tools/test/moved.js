// 옛 주소 → 새 주소 기록 넘김 시험(0246, nail 0294와 같은 시험). 헤드리스 Chromium에서 두 출처를 흉내 낸다(네트워크 없이 디스크에서):
//   https://knellroot.netlify.app/*  → 넘김 페이지(assemble.sh … moved)
//   https://knellroot.beardworks.io/play/*  → 운영판 게임(assemble.sh … release /play)
// 세 경우: 옛 기록이 넘어가고 # 뒤가 지워진다 · 새 주소에 기록이 이미 있으면 덮지 않는다 · 옛 기록이 없으면 그냥 넘어간다.
//   sh tools/web/assemble.sh dist <id> release /play && sh tools/web/assemble.sh dist-moved <id> moved
//   node tools/test/moved.js dist dist-moved
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const GAME = path.resolve(process.argv[2] || 'dist');
const MOVED = path.resolve(process.argv[3] || 'dist-moved');
const OLD = 'https://knellroot.netlify.app';
const NEW = 'https://knellroot.beardworks.io/play/';
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm', '.png': 'image/png',
  '.ttf': 'font/ttf', '.mp3': 'audio/mpeg', '.txt': 'text/plain; charset=utf-8', '.webmanifest': 'application/manifest+json' };

function file(root, rel) {
  if (rel === '' || rel.endsWith('/')) rel += 'index.html';
  const f = path.join(root, decodeURIComponent(rel));
  return f.startsWith(root) && fs.existsSync(f) && fs.statSync(f).isFile() ? f : null;
}

async function run() {
  const browser = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
  const failures = [];
  const check = (cond, msg) => { if (!cond) { failures.push(msg); console.error('FAIL ' + msg); } else console.log('ok   ' + msg); };

  async function context() {
    const ctx = await browser.newContext({ serviceWorkers: 'block', locale: 'ko-KR' });
    await ctx.route('**/*', (route) => {
      const u = new URL(route.request().url());
      // 기록을 심을 빈 페이지(두 출처 다). 넘김 페이지는 열자마자 떠나므로 거기서는 심을 수 없다.
      if (u.pathname.endsWith('/__seed')) return route.fulfill({ status: 200, contentType: 'text/html', body: '<!doctype html><title>seed</title>' });
      let f = null;
      if (u.origin === OLD) f = file(MOVED, u.pathname.slice(1)) || file(MOVED, '404.html');
      else if (u.origin === 'https://knellroot.beardworks.io' && u.pathname.startsWith('/play/')) f = file(GAME, u.pathname.slice('/play/'.length));
      if (!f) return route.fulfill({ status: 404, body: '' });
      return route.fulfill({ status: 200, contentType: MIME[path.extname(f)] || 'application/octet-stream', body: fs.readFileSync(f) });
    });
    return ctx;
  }
  const storage = (page) => page.evaluate(() => { const o = {}; for (let i = 0; i < localStorage.length; i++) { const k = localStorage.key(i); o[k] = localStorage.getItem(k); } return o; });
  const seed = async (page, url, data) => { await page.goto(url + (url === OLD ? '/__seed' : '__seed')); await page.evaluate((d) => { localStorage.clear(); for (const k in d) localStorage.setItem(k, d[k]); }, data); };

  // 한글·여러 줄이 든 기록(UTF-8 왕복을 본다).
  const OLD_SAVE = { 'knell.save.0': 'v=1\nname=방랑자\nembers=42\n', 'knell.slot': '0', 'knell.pref.3': '7', 'knell.save.1': 'v=1\nlv=3\n', 'other.key': 'x' };

  // 1) 옛 기록이 넘어간다.
  {
    const ctx = await context(); const page = await ctx.newPage();
    await seed(page, OLD, OLD_SAVE);
    await seed(page, NEW, {});
    await page.goto(OLD + '/', { waitUntil: 'load' });
    await page.waitForURL((u) => u.href.startsWith(NEW), { timeout: 10000 });
    await page.waitForFunction(() => !/#/.test(location.href), null, { timeout: 30000 });
    const got = await storage(page);
    check(page.url() === NEW, `넘어온 뒤 주소에 # 뒤가 남지 않는다 (${page.url()})`);
    for (const k of ['knell.save.0', 'knell.slot', 'knell.pref.3', 'knell.save.1']) check(got[k] === OLD_SAVE[k], `${k}가 그대로 넘어온다`);
    check(!('other.key' in got), 'knell.* 밖의 칸은 넘기지 않는다');
    await page.waitForFunction(() => !document.getElementById('knell-load'), null, { timeout: 60000 })
      .then(() => check(true, '넘어온 뒤 게임이 첫 화면을 그린다'), () => check(false, '넘어온 뒤 게임이 첫 화면을 그린다'));
    await ctx.close();
  }
  // 2) 새 주소에 기록이 이미 있으면 덮지 않는다.
  {
    const ctx = await context(); const page = await ctx.newPage();
    await seed(page, OLD, OLD_SAVE);
    await seed(page, NEW, { 'knell.save.0': 'v=1\nname=new\n' });
    await page.goto(OLD + '/', { waitUntil: 'load' });
    await page.waitForURL((u) => u.href.startsWith(NEW), { timeout: 10000 });
    await page.waitForFunction(() => !/#/.test(location.href), null, { timeout: 30000 });
    const got = await storage(page);
    check(got['knell.save.0'] === 'v=1\nname=new\n', '새 주소의 기록을 덮지 않는다');
    check(!('knell.save.1' in got), '덮지 않을 때는 아무 칸도 넣지 않는다');
    await ctx.close();
  }
  // 3) 옛 기록이 없으면 # 없이 넘어간다. 옛 주소의 다른 길(예 /knellroot.wasm)도 넘김 페이지로.
  {
    const ctx = await context(); const page = await ctx.newPage();
    await seed(page, OLD, {});
    await page.goto(OLD + '/assets/old-deep-link.png', { waitUntil: 'load' });
    await page.waitForURL((u) => u.href.startsWith(NEW), { timeout: 10000 });
    check(page.url() === NEW, `기록이 없으면 그냥 새 주소로 (${page.url()})`);
    await ctx.close();
  }

  await browser.close();
  if (failures.length) { console.error(`\n${failures.length}개 실패`); process.exit(1); }
  console.log('\n모두 통과');
}

run().catch((e) => { console.error(e); process.exit(1); });
