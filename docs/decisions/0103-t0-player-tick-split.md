# 0103. T0 — `tick_player`를 단계·상태마다 메서드로(동작 그대로)

- 날짜: 2026-11-07
- 상태: **채택·구현**(개발판). 0096 T0, 0066 R6, nail 0170 R9. 사장 "진행해, 지금부터는 그냥 release/v0.4에 그대로 작업해도 좋아"(2026-11-07).

## 전
- `World::tick_player`가 925줄 한 함수였다(`world.rs` 1076~1997): 입력 기준·조준 → 시계·스태미나(클로저 `spend`) → 땅 돌리기 → 상태 전이 `match` 15갈래(520줄) → 달리기 → 이동 `match` → 바라보기 → 떨어지기 → 마무리.
- 같은 "회피 시작" 블록이 일곱 번(서기·마시기·모으기·강타·쏘기·모은 사격·던지기), "패링 시작"이 두 번 복사돼 있었다. 나무 칸(0102)이 박자·패링·대시 공격·땅 돌리기·내려찍기를 건드리면 이 함수가 더 깊어진다(KB `solo-game-codebase-health-signals` — 새 메커닉 전에 깊어진 틱 함수부터).

## 결정
- `src/world/player_tick.rs`(world.rs의 하위 모듈 — `World`의 비공개 필드를 그대로 본다):
  - `Ctx` — 이번 틱의 입력(캔슬 반영)·`Tuning`·`dt`·대상·입력 기준·조준·자동 조준·쏘는 쪽·무기·떨굴 자리·월드 이동, 그리고 이번 틱에 생긴 것(탄·던진 자리·내려찍기 착지). `player_ctx`가 한 번 만든다.
  - 단계 메서드: `tick_player_clocks`(시계·스태미나, 끝에 `world_move`·`can_charge_cast`) → `try_turn` → 상태마다 `tick_idle`·`tick_drink`·`tick_attack`·`tick_charging`·`tick_heavy`·`tick_cast`·`tick_cast_charge`·`tick_throw`·`tick_dodge`·`tick_jump`·`tick_turning`(짧은 셋 — 맞음·내려앉음·패링 — 은 `match` 안에 그대로) → `tick_run` → `move_player` → `face_player` → `fall_player` → `finish_player_tick`.
  - 서기의 갈래: `throw_grenade`·`swing_pays`(박자 무료·스태미나)·`start_swing`(평타·대시 공격·패링 반격)·`start_shot`.
  - 공용 시작: `try_dodge`(일곱 곳)·`try_parry`(두 곳). 상태마다 다른 앞 조건(삼키기 전·쏜 뒤·활성 뒤 …)은 부르는 쪽이 `&&`로 앞에 둔다 — 부작용은 스태미나 쓰기 하나라 조건 순서를 바꿔도 같다.
  - `spend` 클로저 → `Player::spend`.
- 계산 순서·조건 평가 순서·값은 그대로. `world.rs` 6,816 → 5,896줄.
- 나무 칸(0102 G5)은 그 동사의 메서드에 한 줄씩 넣는다(예: 끝맺음 = `swing_pays`·`tick_charging`, 비껴서기 = `tick_dodge`, 기우는 땅 = `apply_turn`).

## 확인
- 시험 257개 그대로 통과. fmt·clippy(wasm 기본·운영판, android 운영판)·wasm 릴리스 빌드.
- **흔적 비교**(커밋하지 않은 임시 탐침): 시나리오 8 × 무기 5 × 시드 3, 각 1,200틱 의사 난수 입력(이동·조준·공격·쥐기·회피·패링·점프·등잔·마법·락온·땅 돌리기)으로 틱마다 플레이어 상태·위치·속도·바라봄·스태미나·체력·락온·적 체력 합을 해시 → 나누기 전후 **120/120 같음**. 탐침의 민감도: 회피 대기 +0.15 → +0.16 하나만 바꾸면 22/120이 달라졌다.
