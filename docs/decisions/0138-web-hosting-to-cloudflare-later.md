# 0138. 웹 호스팅은 나중에 Netlify → Cloudflare로 — 방향만, 시점은 따로

- 날짜: 2026-11-10
- 상태: **방향 채택, 시점 미정**(사장 2026-11-10: "Nail 도, knellroot 도 나중에는 cloudflare 로 가야겠다"). 옮기는 일 자체는 정할 때 새 번호로.
- 근거(KB): `reference/netlify/free-plan-limits.md`·`reference/cloudflare/static-site-extras.md`, 결정 0001 덧붙임(2026-11-10, 공식 문서 라이브 대조).

## 왜
- Netlify 무료판은 2025-09-04부터 크레딧제다. 크레딧제 무료판은 대역폭 ≈ 15 GB(1 GB = 20크레딧)·운영 배포 1회 15크레딧이고, 다 쓰면 **팀의 모든 사이트가 멈춘다**("Site not available"). 우리 팀(2023년 생성)은 **Legacy**(사장 확인 2026-11-10 — 월 100 GB, 넘으면 멈춤, 크레딧제로 바꾸지 않는다: 바꾸면 되돌릴 수 없고 정적 사이트에 불리하다)다. 그래도 **넘으면 멈춘다** — 출시·홍보로 트래픽이 갑자기 오를 때 게임 둘(nail·knellroot)이 같은 팀이라 함께 내려간다.
- Cloudflare는 정적 파일 요청이 무료·무제한이다. 도메인·DNS·메일이 이미 Cloudflare에 있다(beardworks.io).

## 어디로
- **Workers 정적 파일**(Cloudflare가 새 프로젝트에 Pages보다 권한다 — Pages 문서 첫 쪽 2026-08-25). 요청·저장 무료, 파일 하나 25 MiB, 버전당 2만 파일(Free).
- knellroot 웹판의 가장 큰 파일은 `knellroot.wasm` 4.4 MB(모델·소리·글꼴을 바이너리에 넣음) — 25 MiB와 거리가 멀다(2026-11-10 운영 사이트에서 잼).

## 옮길 때 바뀌는 것(지금은 하지 않는다)
- `deploy-web.yml`의 `publish` 잡: `netlify deploy` → `wrangler deploy`(지은 `dist` 그대로 — "빌드는 CI에서, 올리기는 CLI로"는 같다). 시크릿 `CLOUDFLARE_API_TOKEN`·`CLOUDFLARE_ACCOUNT_ID`를 같은 Environment(운영·개발)에, 토큰은 그 Worker만.
- 개발판 별칭 `dev` → Workers 미리 보기(버전·미리 보기 URL) 또는 개발판 Worker 따로. 배포 뒤 `build.txt` 확인은 그대로.
- `_headers`(캐시·wasm Content-Type)·서비스 워커 캐시가 같은지, Range 요청에도 파일 전체(200)를 주는데 소리·큰 파일 받기에 문제없는지 확인.
- 주소: 게임 주소를 `*.netlify.app`에서 우리 하위 도메인으로 옮기면 PWA·웹 기록(localStorage)은 출처가 바뀌어 끊긴다(KB `reference/web/pwa-install-fullscreen-orientation.md`) — 옮기는 결정에서 옛 주소를 어떻게 둘지 함께 정한다.

## 사람이 할 일
- [x] Netlify 요금제: **Legacy**(사장 2026-11-10). 지금 규모에선 급하지 않다 — 옮길 때는 트래픽이 크게 오를 일(상점 출시·홍보) 전에.
- [ ] (때때로) Netlify 대시보드의 이번 달 대역폭 — 100 GB에 가까워지면 옮길 시점.
- [ ] 옮길 시점을 정하면 새 결정 번호로.
