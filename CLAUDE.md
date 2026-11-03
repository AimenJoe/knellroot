# CLAUDE.md — blight (임시 이름)

레트로 로우폴리 3D ARPG. 쿼터뷰 떼싸움 + 락온 숄더뷰 보스전. Rust + macroquad. 웹 먼저, PC, 나중에 Android 핸드헬드(컨트롤러).

## 먼저 읽을 것
1. `docs/concept.md` — 무엇을 만드는가(세계관은 초안, 바뀔 수 있다)
2. `docs/phase-0-poc.md` — PoC 질문 P1~P5·통과 기준·시간 상한. 끝의 **"구현 상태"와 "사람이 할 일"**이 현재 위치다
3. `docs/lab.md` — 랩이 무엇을 재고 값이 `tuning.rs`로 어떻게 가는가
4. `docs/decisions/` — 왜 그렇게 정했나. 바꾸려면 새 번호로

세션이 끝나면 "구현 상태"·"사람이 할 일"을 갱신하고 커밋한다. 기여·배포 규칙은 첫 게임(`nail-under-the-roots`의 `CONTRIBUTING.md`·`docs/release.md`)을 따르되 아직 배포는 없다.

## 지식 베이스(KB)
일반 지식은 **github.com/AimenJoe/KB**. 세션에 KB를 붙이고 `maps/gamedev.md` → 노트를 먼저 읽는다. 이 프로젝트와 바로 닿는 것:
- `reference/macroquad/3d-rendering.md` — Camera3D(z-up 기본!)·Mesh·배치·조명 없음
- `concepts/camera-mode-switch-and-lock-on.md` — 리그 둘·입력 기준 즉시·구도·가림
- `reference/assets/retro-3d-models-blockbench-gltf.md` — 상자 모델·glTF 파이프라인(P4)
- `guides/poc-and-lab-before-content.md` — PoC·Lab 절차
- `guides/second-game-project-day-one.md` — 첫 게임에서 가져온 규칙 15개(이 레포의 뼈대가 그것)
- 첫 게임 설계 원칙: `concepts/action-roguelite-for-solo-dev.md`, 손맛: `concepts/melee-light-and-charged-attack-feel.md`·`fixed-timestep-and-hitstop.md`·`timing-parry.md`

**프로젝트 기록(TODO·실험 결과·결정)은 이 레포에, 프로젝트가 사라져도 유용한 지식은 KB에**(KB `CLAUDE.md` 규칙: frontmatter·verified·지도·`tools/kb.py check && index`).

## 코드 규칙(첫 게임에서 가져옴 — KB `second-game-project-day-one`)
- 입력은 `Actions`만. `KeyCode`·패드 버튼은 `input.rs` 밖에 없다. 눌림은 래치, 틱이 소비.
- 로직 고정 60Hz(`TICK_DT`), 렌더는 보간. 히트스톱은 로직 시계만(`World::hitstop`).
- 수치는 `tuning.rs` 한 곳. 랩이 바꾸는 값은 `P` + `SPECS` 한 줄씩. 랩의 export를 붙여 넣는 곳도 여기. 코드에 타별 숫자 금지.
- `world.rs`는 카메라·소리·메뉴를 모른다. `Basis`(입력 기준)와 `Event`로만 바깥과 만난다. 틱은 단계 메서드 순서(`tick_lock → tick_player → apply_strike → tick_enemies → collide → cleanup`) — 새 메커닉은 맞는 단계에.
- 카메라는 `camera.rs`: 원하는 리그를 매 프레임 계산 + 모드가 바뀌면 지금 리그에서 보간. **입력 기준은 월드가 즉시 바꾸고 카메라만 늦게 간다.** `Camera3D`는 `up: Vec3::Y`를 매번 적는다(기본 z-up).
- 좌표: y-up, XZ 평면 이동, 1 단위 = 1m, 플레이어 키 1.8. 쿼터뷰의 화면 위 = 월드 -z.
- 렌더는 월드를 읽기만. 내부 해상도 `INTERNAL_W×H` 렌더 타깃 → 정수 배율 → HUD는 화면 패스.
- 개발 시나리오는 `scenario.rs` 표 한 줄(`checks`에 그것이 보는 PoC 번호). feature `dev`(기본 켬)로 가린다. 운영판 = `--no-default-features`.
- 새 메커닉 = 단위 테스트 하나 이상(규칙을 이름으로 읽히게). 소리·그림 핸들을 로직에 넣지 않는다 — 그래서 시나리오 넷을 600틱씩 돌리는 시험이 있다.
- 글: 지금은 개발 전용 화면뿐이라 영어 ASCII(기본 폰트). 플레이어가 보는 글이 생기는 순간 `i18n/<언어>.rs` 키-필드 구조로(첫 게임 규칙).
- 그림·모델: 손으로 PNG를 고치지 않는다. 모델은 Blockbench 원본(`tools/art/sources/`) → 내보낸 glTF → 로더. 에셋 출처는 `assets/LICENSES.md`. AI를 쓴 곳은 숨기지 않는다.

## 하지 않는 것
- PoC 통과 기준(`docs/phase-0-poc.md`)을 뒤에 고치지 않는다. 상한을 넘기면 대안으로 간다고 적어 둔 대로.
- 랩 값을 저장 파일에서 게임이 읽게 하지 않는다(진실은 `tuning.rs` 하나).
- 콘텐츠(적 종류·방·이야기·그림)를 P1~P3이 닫히기 전에 쌓지 않는다.
- 외부 JS 핫링크, 라이선스 미확인 에셋, 코드 안의 글 리터럴(플레이어가 보는 글).
- 언어: 대화·문서·커밋 한국어, 코드 식별자 영어. 모델 이름은 커밋에 쓰지 않는다.
