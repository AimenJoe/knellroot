# 0093. 소개 페이지는 GitHub Pages에 작게 — 개발 중이라 한 장만

- 날짜: 2026-11-07
- 상태: **채택·구현**(`site/`, 배포는 사장이 main에 넣은 뒤)
- 사장: "knellroot도 under the roots처럼 홈페이지 만들어줘, 대신 이 게임은 아직 개발중이니 상세하게 만들지 말고."

- 번호: 처음엔 0091 → 0092로 붙였으나 같은 날 다른 기록(0091 문 가림·0092 main 룰셋)과 겹쳐 0093. 페이지는 게임 코드와 상관없어 `site/`·`pages.yml`만 main에 패치로 먼저 넣는다(`pages.yml`은 main push에서만 돈다).

## 결정
1. **꼴은 첫 게임 nail 0069 그대로**: 원본은 이 레포 `site/`(한국어 `index.html`, 영어 `en/`), `pages.yml`이 main의 `site/**`가 바뀔 때 GitHub Pages에 올린다. 게임은 Netlify 그대로.
2. **주소**: https://knellroot.beardworks.io (nail = nail.beardworks.io와 같은 꼴). 회사 페이지 beardworks.io의 게임 카드가 여기로 온다. 도메인을 붙이기 전에는 https://aimenjoe.github.io/knellroot/.
3. **내용은 넷만**: 첫 화면(게임 장면 + 이름·발음 + 게임 첫 이야기 두 줄 + "개발 중" 표지) · 두 시점(쿼터뷰 떼 / 락온 결투, 스크린숏 둘) · 해 보기(개발 중 안내 + 브라우저·APK·의견) · 만드는 곳(Beardworks·소스·nail과 같은 세계).
   - **넣지 않은 것**: 조작 표·적·무기·장신구 목록·로드맵·소식·버전 숫자. 판마다 바뀌어 페이지가 금방 낡는다(KB `guides/indie-game-landing-page.md` "낡은 숫자를 박지 않는다"). 그래서 nail의 `site.js`(Releases에서 버전·소식 읽기)도 두지 않았다 — 스크립트 없는 정적 한 장.
   - 스포일러(핵·보라 뿌리의 뜻·nail 이웃의 흔적)는 넣지 않았다. nail과의 관계는 "같은 세계, 아주 먼 뒤의 이야기" 한 줄만(0004 D2 = a).
4. **그림**: 운영판 v0.3.3 웹 빌드를 헤드리스 Chromium(1280×720)으로 찍은 장면 셋 — 문지기 결투(`s_duel`, 첫 화면은 HUD를 잘라 낸 `s_hero`)·뜰의 떼(`s_horde`)·락온(`s_lock`). HUD는 한국어 하나뿐이라 영어 페이지에 그렇다고 적었다. 파비콘은 앱 아이콘 `web/icons/icon-192.png`.
5. **글꼴**: 제목은 게임 UI와 같은 고운바탕 굵게, 페이지 글자만 남긴 woff2(`tools/site/subset_font.py`, 원본은 Google Fonts — 레포에 넣지 않음, 32KB). 본문은 시스템 글꼴. 색은 0037의 해 질 녘 남보라 + 등잔빛.
6. **AI·출처 고지**: 꼬리말에 Gemini → Meshy 모델이라는 것과 Meshy 무료 플랜 결과물의 CC BY 4.0 표시(모델 이름 넷), CC0 소리·OFL 글꼴, 전체는 `assets/LICENSES.md`로. 개인정보처리방침은 회사 페이지 하나(beardworks.io/privacy.html)로 링크 — 그쪽 대상에 Knellroot를 넣었다(beardworks 레포).

## 왜
- 개발 중 게임의 페이지는 "무엇인지·지금 해 볼 수 있는지·아직 바뀐다"만 정확하면 된다. 자세히 쓸수록 다음 판에서 틀린 말이 된다.
- 첫 게임과 같은 배포 꼴이라 처음 켜기·도메인 붙이기 절차가 이미 검증되어 있다(KB `reference/github-actions/pages-static-site.md`).

## 확인
- 로컬에서 Pages와 같은 하위 경로(`/knellroot/`)로 띄워 1280·390·320px × 한국어·영어: 가로 넘침 없음, 404·콘솔 오류 없음, 고운바탕 불러옴.
- 확인 못 한 것: 실제 Pages 배포·도메인(사람이 할 일).

## 다시 볼 조건
v0.4.0 "내려가는 길"(0069)처럼 구역이 늘거나 판매 준비(첫 게임 0232)에 들어가면 nail 페이지처럼 넓힌다(조작·적·소식).
