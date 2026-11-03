# Knellroot(넬루트) — 레트로 로우폴리 3D ARPG PoC

쿼터뷰로 떼를 상대하다가, 락온을 누르면 숄더뷰로 내려가 보스와 1:1. Rust + macroquad(첫 게임 [nail-under-the-roots](https://github.com/AimenJoe/nail-under-the-roots)와 같은 스택).
지금은 **PoC + Lab** 단계다: 상자 플레이어·보스·잡몹, 카메라 리그 둘, 실행 중 수치를 바꾸는 랩. 그림·소리·모델은 아직 없다.

- 무엇을 만드나: `docs/concept.md`
- 지금 어디까지·무엇을 판정하나: `docs/phase-0-poc.md`(**구현 상태**·**사람이 할 일**)
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
조작(키보드 / 패드): WASD·화살표 / 왼쪽 스틱 이동 · J·Space / X 공격 · K·Shift / A 회피 · Q·E / LB·RB 땅 돌리기 ↺↻ · L·Tab / R3 락온 · F1 / Select 랩 · 1~7 / Start 시나리오(6·7 = 성능 측정).
터치(폰·태블릿 브라우저, 가로): 왼쪽 반 = 스틱, 오른쪽 아래 = 공격·회피·락온, 오른쪽 위 = SC(시나리오)·LAB.
```sh
node tools/test/touch.js dist shots   # 가로 폰 터치 연기 시험
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
src/main.rs      루프: 입력 폴링 → 고정 틱 60Hz(월드 → 카메라) → 보간 렌더
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
