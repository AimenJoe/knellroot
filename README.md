# Knellroot(넬루트) — 레트로 로우폴리 3D ARPG PoC

쿼터뷰로 떼를 상대하다가, 락온을 누르면 숄더뷰로 내려가 보스와 1:1. Rust + macroquad(첫 게임 [nail-under-the-roots](https://github.com/AimenJoe/nail-under-the-roots)와 같은 스택).
지금은 **1단계: 첫 스테이지 "침묵의 성소" 초안**이다(0019): 타이틀·멈춤·결과 화면, 입구 → 뿌리의 뜰(물결 셋) → 성소 문 → 잠든 문지기와 결투, Blender로 만든 로우폴리 모델과 화로 불빛. 소리는 아직 없다. PoC(카메라 리그 둘·땅 돌리기·랩)는 닫고 개발판에 남겼다.

- 무엇을 만드나: `docs/concept.md`
- 지금 어디까지·무엇을 판정하나: `docs/phase-1-first-stage.md`(**구현 상태**·**사람이 할 일**), 닫은 PoC: `docs/phase-0-poc.md`
- 랩: `docs/lab.md` · 배포: `docs/release.md` · 왜 그렇게 정했나: `docs/decisions/`
- 이 대화에서 나온 것: `docs/research/chatgpt-2026-11-03-brainstorm.md`

## 실행
```sh
# 데스크톱(리눅스 의존성: libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev libudev-dev pkg-config)
cargo run --release
# 웹
cargo build --release --target wasm32-unknown-unknown && sh tools/web/assemble.sh dist && python3 -m http.server -d dist 8080
# 헤드리스 스크린숏(Playwright + Chromium): 시나리오 다섯·락온·랩 패널·땅 돌리기
node tools/test/shot.js dist shots
```
조작(키보드 / 패드): WASD·화살표 / 왼쪽 스틱 이동 · 오른쪽 스틱 조준(공격은 가까운 적 자동 조준) · J·Space / X 공격 · K·Shift / A 회피 · Q·E / LB·RB 땅 돌리기 왼쪽·오른쪽(누르는 쪽으로 카메라가 돈다, 0017) · L·Tab / R3 락온 · Esc·P / Start 멈춤 · 메뉴는 위·아래 + Enter / A. 개발판: F1 / Select 랩 · F3 개발 줄 · 타이틀의 "DEV" 항목(성소 문 앞에서 시작, PoC 시나리오 1~7 — 6·7 = 성능 측정).
터치(폰·태블릿 브라우저, 가로): 왼쪽 반 = 스틱, 오른쪽 아래 = 공격·회피·락온, 오른쪽 위 = SC(시나리오)·LAB.
```sh
node tools/test/touch.js dist shots   # 가로 폰 터치 연기 시험
node tools/test/leak.js dist 120 stage   # 오래 켜기: 살아 있는 GL 텍스처·버퍼·wasm 메모리(0020). stress = 잡몹 80
```

## 커밋 전
```sh
cargo fmt --all -- --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo clippy --target wasm32-unknown-unknown --no-default-features -- -D warnings
cargo test
cargo build --release --target wasm32-unknown-unknown
```

## 구조
```
src/main.rs      창 + App 하나
src/app/         장면(타이틀·싸움·멈춤·끝·결과)과 루프, flow.rs = 흐름 순수 함수
src/stage/       글자 지도 → 레벨 + 진행 대본, sanctuary.rs = 첫 스테이지
src/species.rs   적 종류 표, enemy.rs = 행동(근접·돌진·문지기)
src/scenery.rs   지형 굽기(바닥·벽·소품 + 빛), models.rs = GLB 목록(build.rs)
src/hud.rs ui.rs i18n/   HUD·메뉴·글(한국어·영어, 고운바탕 서브셋 — tools/fonts/subset_ui.py)
src/input.rs     Actions (KeyCode·패드 버튼은 이 파일 밖에 없음)
src/tuning.rs    수치 전부 + 랩이 바꾸는 표 SPECS
src/world.rs     규칙 하나: 플레이어·적·락온·타격 (카메라·소리·메뉴를 모른다)
src/camera.rs    리그 둘(쿼터뷰·락온) + 보간 + 당기기
src/render.rs    내부 해상도 렌더 타깃 → 정수 배율, 바닥 예고·그림자·HUD
src/lab.rs       실행 중 수치 바꾸기·A/B·내보내기
src/scenario.rs  개발 시나리오 다섯
src/touch.rs     터치: 가상 스틱·버튼·랩 터치판(0006)
web/             index.html + macroquad JS 번들 복사본(핫링크 금지)
tools/           web/assemble.sh · test/shot.js
```
