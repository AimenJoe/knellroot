# 0246. 운영 웹판은 https://knellroot.beardworks.io/play/ — Cloudflare Workers 정적 파일, 옛 주소는 기록을 들고 넘긴다

- 날짜: 2026-11-19
- 상태: 채택(사장 2026-11-19: "knellroot도 /play로 옮기자"). 0138(언젠가 Cloudflare로)의 운영판 몫을 실행한다. nail 0294(2026-11-18 배포)와 같은 꼴 — 그 결정의 "왜"를 그대로 가져온다. 개발판은 Netlify 별칭 `dev` 그대로.
- 근거(KB): `reference/web/pwa-install-fullscreen-orientation.md`(웹 기록·PWA·서비스 워커는 출처마다 따로, 옮길 때의 넘김 페이지), `reference/cloudflare/static-site-extras.md`(하위 경로에 게임 — `assets.base_path`, 다른 Worker의 와일드카드 route 함정).
- 번호: `release/v0.7`이 0245까지 써서 main에서 갈라 만든 이 패치는 0246.

## 왜
- `knellroot.netlify.app`은 Netlify에 묶인 출처다. 언젠가 옮기면 플레이어 기록(localStorage `knell.save.*`)·설치한 앱이 끊긴다. 우리 도메인 아래 경로로 두면 그 뒤로는 무엇이 내주든 출처가 바뀌지 않는다.
- 소개 페이지와 같은 주소 아래(공유·검색), Cloudflare 정적 파일은 요청·저장 무료·무제한(Netlify Legacy 월 100 GB는 팀 전체에 걸린다).
- Worker는 nail의 `web-game`과 따로 `web-game-knellroot` — 한 Worker에 두면 한 게임을 올릴 때 다른 게임도 바뀐다.

## 무엇을(nail 0294와 같은 꼴)
- `tools/web/wrangler.jsonc`: Worker `web-game-knellroot`, route `knellroot.beardworks.io/play`·`/play/*`, `assets.base_path: "/play"`, `workers_dev`·`preview_urls` 끔.
- `tools/web/assemble.sh <폴더> <id> release /play`: `_headers` 경로 앞에 `/play`. `moved`: 옛 주소의 넘김 페이지(`web/moved/`).
- 서비스 워커는 페이지를 `./`로만 담는다(Cloudflare의 `index.html → ./` 307 응답을 담으면 오프라인 실행이 깨진다).
- **기록 넘김**: 옛 운영 주소의 넘김 페이지가 그 출처의 `localStorage "knell.*"`를 `https://knellroot.beardworks.io/play/#knell-import=…`에 실어 보내고, `web/index.html`이 게임이 저장소를 읽기 전에(번들보다 앞의 동기 스크립트) 받아 넣고 `#`을 지운다. 새 주소에 `knell.save.*`가 있으면 아무것도 넣지 않는다. 옛 서비스 워커·캐시도 걷는다. 홈 화면에 설치한 iPhone 앱은 그 앱으로 한 번 열어야 넘어간다.
- 배포(`deploy-web.yml`): `build` 잡은 그대로(시험·연기 시험) + `/play` 조립과 `wrangler deploy --dry-run`(토큰 없이, 모든 갈래·PR). 새 **`publish` 잡**(main push만, Environment `web-production`): ① `wrangler deploy` → 새 주소의 `build.txt`가 이 커밋인지(`tools/web/check_live.sh`) → ② 그 뒤에만 Netlify 프로덕션을 넘김 페이지로. 개발판·PR 미리 보기는 Netlify 그대로.
- **Cloudflare 토큰은 Environment `web-production`에만**(Deployment branches = main) — 레포 시크릿에 두면 어느 갈래의 워크플로든 운영 Worker를 바꿀 수 있다.
- 시험: `tools/test/moved.js`(두 출처를 흉내 내 셋 — 넘어오고 `#`이 지워짐·덮지 않음·옛 기록 없음).

## 사람이 할 일
- [ ] Cloudflare: Worker **`web-game-knellroot`** 만들기(빈 Worker, nail `web-game`처럼). API 토큰 = 그 Worker의 **Editor** + **Zone → Workers Routes → Edit**(beardworks.io).
- [ ] GitHub knellroot → Settings → Environments → **`web-production`** 새로 만들기(Deployment branches = `main`만) → 시크릿 `CLOUDFLARE_API_TOKEN`·`CLOUDFLARE_ACCOUNT_ID`. Netlify 시크릿은 레포 시크릿 그대로(넘김 페이지를 올린다).
- [ ] 이 PR Merge = 운영 반영(공식 페이지 링크 포함). 그 뒤 `release/v0.7`에 main을 합친다. beardworks `ops/src/config.js`의 knellroot `prod`를 새 주소로(PAGES에 옛 주소).
- [ ] 머지 전에 Cloudflare 영역 **Workers Routes**에 `knellroot.beardworks.io/*` 같은 넓은 route가 없는지(2026-11-18 와일드카드 route가 소개 페이지를 가로챘다).
