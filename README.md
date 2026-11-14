# Knellroot(넬루트) — 레트로 로우폴리 3D ARPG

쿼터뷰로 떼를 상대하다가, 락온을 누르면 숄더뷰로 내려가 보스와 1:1. Rust + macroquad(첫 게임 [nail-under-the-roots](https://github.com/AimenJoe/nail-under-the-roots)와 같은 스택). 세계관은 nail의 먼 뒷이야기(0004).

- **운영판 v0.6.0 "다듬기"**(2026-11-14 — 전투 스킬·기술 판·창 하나 + 탭·첫 장면과 튜토리얼·덧옷·뿌리 굴·PC 빌드): 웹 https://knellroot.netlify.app · Android APK·PC 묶음(Windows·macOS·Linux)은 GitHub Releases · 소개 페이지 https://knellroot.beardworks.io
- **개발판**(`release/v0.6`, 다음 v0.6.1 = 무기 강화, 0179): https://dev--knellroot.netlify.app · 개발판 APK는 미리 보기 Release `dev` · PC 묶음은 Actions 아티팩트
- iOS는 서명하지 않은 .ipa를 Actions 아티팩트로만(0136)

## 지금 있는 것
- **지역 둘**: 「침묵의 성소」(문지기 "잠들지 않는 문지기", 뿌리의 잔재·가시짐승) → 바닥이 열리면 내려가는 「떠난 마을」(문지기 "깨진 종지기", 등딱지짐승·꼬투리, 남은 주민 셋, 0116~0133). 보통 스테이지는 물결이 아니라 넓은 지도에 서 있는 적과 엘리트, 문 장치로 여는 지름길(0113). 두 문지기 모두 체력 반에서 2단계(0133).
- **싸움**: 무기 넷(검·망치·지팡이·쇠뇌) + 맨손 — 콤보·강공격·박자·패링·마법·점프 내려찍기·땅 돌리기, 락온 숄더뷰(0051·0076).
- **자라기**: 잔불 레벨(생명·기력·힘)·기술 나무 3갈래 × 5칸·화로 가게·장신구와 장비 창·숨은 물건(0102~0108·0067).
- **그 밖**: 저장 슬롯 셋(0066), 도감(0130), 소리(효과음 CC0 + 합성, 0064 M5), 한국어·영어.
- 그림은 Blender bpy로 만든 로우폴리 모델과, 적·문지기·주민은 AI 그림 → Meshy(AI) 메시를 줄여 부위로 자른 것(0038·0060·0132). t3ssel8r 풍 3D 도트(0061).

자세한 현재 위치는 `docs/phase-1-first-stage.md`의 **구현 상태**·**사람이 할 일**, 판정 체크리스트는 `docs/testing/v0-6-judgment-checklist.md`.

## 문서
- 무엇을 만드나: `docs/concept.md` · 세계관: `docs/lore/`
- 지금 어디까지: `docs/phase-1-first-stage.md` (닫은 PoC: `docs/phase-0-poc.md`)
- 왜 그렇게 정했나: `docs/decisions/` · 랩(실행 중 수치 바꾸기): `docs/lab.md` · 배포: `docs/release.md`
- 세션 규칙·코드 구조: `CLAUDE.md`
- 지식 베이스: [AimenJoe/KB](https://github.com/AimenJoe/KB) (`maps/gamedev.md`)

## 실행
```sh
# 데스크톱(리눅스 의존성: libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev libudev-dev pkg-config)
cargo run --release
# 웹
cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && python3 -m http.server -d dist 8080
```

## 조작
새로 설치한 판의 기본은 **키보드+마우스**(0058). 타이틀·멈춤의 "조작 설정"에서 바꾸고, "키보드만 배치로"도 고를 수 있다(0024).

| 동작 | 키보드+마우스(기본) | 키보드만 | 패드 |
|---|---|---|---|
| 이동 | WASD·화살표 | WASD·화살표 | 왼쪽 스틱(십자 키는 싸움에서 걷지 않는다, 0151) |
| 조준 | 마우스 커서(쿼터뷰) | 가까운 적 자동 | 오른쪽 스틱 |
| 공격(꾹 쥐었다 떼면 강공격, 0030) | 왼쪽 클릭 | J | X |
| 무기 기술(0048 마법 → 0151: 근접 = 마법 탄, 쇠뇌 = 수류탄 — 쇠뇌는 공격 버튼이 사격) | 오른쪽 클릭 | U | RB |
| 회피 | Space | Space | B |
| 패링(0025) | L-Shift | K | LB |
| 점프(공중에서 공격 = 내려찍기, 0027) · 상호작용(쉬기·열기·말 걸기) | F | F | A |
| 등잔(회복, 0031) | R | R | Y |
| 기술 1·2(0151, 개발판 — 함께 누르면 큰 기술) | Q·E | Q·E | LT·RT |
| 땅 돌리기 왼쪽·오른쪽(0028 → 0151) | Z·C | Z·C | 십자 ←·→ |
| 락온 | 휠 클릭 | L | R3 |
| 장비 창(0070) | Tab | Tab | Select |
| 멈춤 | Esc·P | Esc·P | Start |

터치(폰·태블릿 브라우저, 가로): 왼쪽 반 = 스틱, 오른쪽 아래 = 공격과 둘레의 호(회피·패링·무기 기술·점프)·기술 둘, 오른쪽 위 = 땅 돌리기, 락온·등잔·큰 기술은 쓸 수 있을 때만 뜬다(0053·0151).
개발판: F1 랩 · F3 개발 줄 · F4 글자 넘침 검사 · 타이틀·멈춤 끝 줄 "개발"(개발 시나리오·스테이지 시작점, `src/scenario.rs`·`src/app/dev.rs`).

## 시험
```sh
cargo test
# 헤드리스 Chromium(Playwright) — 먼저 dist를 조립한다
node tools/test/shot.js dist shots      # 스크린숏 연기 시험
node tools/test/touch.js dist shots     # 가로 폰 터치
node tools/test/mouse.js dist shots     # 키보드+마우스 배치
node tools/test/hearth.js dist shots    # 화로·레벨·나무 화면(한국어·영어 + F4)
node tools/test/save.js dist shots      # 저장 슬롯
node tools/test/pwa.js dist shots       # 로딩 덮개·서비스 워커
node tools/test/stress.js dist shots    # 잡몹 많이
node tools/test/fx.js dist shots-fx     # 이펙트를 틱마다(개발 키 F6 = 한 프레임에 한 틱)
node tools/test/leak.js dist 120 stage  # 오래 켜기: GL 텍스처·버퍼·wasm 메모리(stress = 잡몹 80)
sh tools/test/android-smoke.sh <apk> <폴더> [패키지]   # 에뮬레이터에서 APK가 켜지나(CI android-smoke.yml)
```

## 커밋 전
```sh
cargo fmt --all -- --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo clippy --target wasm32-unknown-unknown --no-default-features -- -D warnings   # 운영판
cargo clippy --target aarch64-linux-android -- -D warnings                         # NDK 없이 됨
cargo clippy --release --target x86_64-pc-windows-msvc --no-default-features -- -D warnings   # PC(macOS는 aarch64-apple-darwin)
cargo test
cargo build --release --target wasm32-unknown-unknown
```

## 구조(큰 덩어리만 — 자세한 규칙은 `CLAUDE.md`)
```
src/main.rs          창 + App 하나
src/app/             장면 스택(타이틀·싸움·멈춤·결과·설정·조작·장비·화로·나무·가게·도감·슬롯·개발), flow.rs = 흐름 순수 함수
src/world.rs         규칙 하나: 플레이어·적·락온·타격(카메라·소리·메뉴를 모른다)
src/stage/           글자 지도 → 레벨 + 진행 대본(sanctuary.rs 성소 · village.rs 떠난 마을)
src/species.rs enemy.rs weapons.rs   적 종류 표·행동, 무기 표
src/stats.rs charms.rs growth.rs progression.rs   능력치·장신구·화로 가게·레벨과 기술 나무
src/save.rs prefs.rs  기록(슬롯 셋)·설정
src/camera.rs render.rs scenery.rs backdrop.rs flora.rs models.rs   카메라 리그 둘·렌더·지형 굽기·주위 환경·GLB(build.rs)
src/input.rs touch.rs   Actions(KeyCode·패드 버튼은 input.rs 밖에 없다)·터치
src/hud.rs ui.rs i18n/  HUD·메뉴·글(한국어·영어, 고운바탕 서브셋 — tools/fonts/subset_ui.py)
src/sfx.rs           소리(앱만 안다)
src/tuning.rs lab.rs scenario.rs   수치 전부·랩·개발 시나리오
web/                 index.html·macroquad JS 번들 복사본(핫링크 금지)·manifest·sw.js
pc/                  PC 아이콘(0135)
tools/               web/assemble.sh · test/ · art/blender/(모델 원본) · maps/(지도 생성기) · icons/ · fonts/ · site/
```
