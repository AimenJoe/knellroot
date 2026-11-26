# 0279 — 플랫폼과 무관한 수학: 초월 함수는 `libm` 크레이트로

- 날짜: 2026-11-26
- 상태: **함**(사장 2026-11-26 "수학 고정은 지금 바로 하자")
- 앞선 것: 0278(골든 · 시나리오 · 이펙트 트레이스)

## 무슨 일이 있었나
0278의 둘째 안전망(`scenario_traces`)을 넣은 push부터 CI의 `cargo test`가 네 번 내리 빨갰다(`184d168` · `f330762` · `1ec458b` · `5e25863`) — 개발판 웹이 `9e60393`에서 멈춰 있었다. 여기(세션 컨테이너)에서는 늘 통과했다.

- 갈린 것: 시나리오 65 가운데 넷(`Horde.Hammer` · `Stress40` · `Stress80` · `Elites`), 이펙트 429 가운데 지켜보기 열하나 — 모두 15초짜리 긴 판. 6초짜리 첫 골든 102는 같았다.
- 두 CI 실행의 해시가 서로 같았다 → 경합이 아니라 플랫폼 차이.
- CI 러너 `ubuntu-latest`는 Ubuntu 26.04, 여기는 24.04(glibc 2.39). 검사 잡만 `ubuntu-24.04`로 고정하니 통과했다(`4b0bae7`) → **시스템 libm의 판 차이**.
- Rust의 `f32::sin` · `atan2` …는 네이티브에서 시스템 libm(`sinf` · `atan2f`)을 부른다. 초월 함수의 끝자리는 IEEE가 정하지 않는다 — glibc가 구현을 바꾸면 달라지고, 적 수십이 서로 미는 판에서는 끝자리 하나가 15초 뒤 다른 판이 된다.

## 정한 것
1. **게임의 초월 함수는 모두 `libm` 크레이트**(Rust로 옮긴 musl의 libm — 웹(wasm)의 std가 쓰는 것과 같은 계열)를 지난다: `src/det.rs`의 `Det` 트레이트 — `x.sin_d()` · `y.atan2_d(x)` · `sin_cos_d` · `tan_d` · `asin_d` · `acos_d` · `atan_d` · `exp_d` · `powf_d` · `hypot_d`. 로직만이 아니라 크레이트 전체(520곳쯤 · 파일 57) — 규칙이 하나다.
2. **std의 것은 clippy가 막는다**(`clippy.toml` `disallowed-methods` — CI의 `-D warnings`). 새 함수가 필요하면 `Det`에 더한다.
3. **로직은 glam의 각도 함수를 부르지 않는다**(`Quat::from_rotation_y` · `from_angle` · `angle_between` … — 안에서 std를 부른다): 네 곳을 `det::turn_y`로, 시험 `logic_keeps_off_glams_trig`가 로직 폴더를 훑는다. 그림(렌더 · 카메라 · 모델)은 트레이스 밖이라 glam 그대로.
4. `sqrt` · `abs` · `floor` · `mul_add` · `to_radians` · `powi`는 그대로(IEEE가 결과를 정하거나 곱셈뿐).
5. CI 러너는 다시 `ubuntu-latest` — 26.04에서 통과하는 것이 이 결정의 증명이다.

## 달라진 것
- 값: std와 끝자리까지만 다르다(시험 `det_math_agrees_with_std_to_the_last_places_and_turns_like_glam`).
- 고정값: 골든 102 · 시나리오 65 · 사건 표는 **그대로**(다시 쓰지 않았다 — 여기의 glibc 2.39와 `libm` 크레이트가 이 판들에서 같은 해시를 냈다). 이펙트 트레이스는 지켜보기 일곱 줄만 다시 썼다(입자 쪽의 삼각함수 끝자리 — `Tray.Sword` · `Stress40.Sword` · `Stress80` 둘 · `Skills` 둘 · `Line.Sword`).
- 빠르기: 웹은 그대로(std가 이미 같은 구현). 네이티브는 `libm` 크레이트의 `sinf`가 glibc보다 조금 느리다 — 한 프레임의 호출은 수천 아래라 재지 않았다. 느려지면 `KNELL_PROF`로 재고, 그림 쪽만 std로 되돌릴 수 있다(그 모듈에 `#![allow(clippy::disallowed_methods)]`).
- 의존성: `libm`은 이미 다른 크레이트를 따라 들어와 있었다(`Cargo.lock` 0.2.16) — 직접 의존성으로 올렸을 뿐.

## 배운 것(KB로)
- 골든 트레이스는 **같은 기계**에서만 뜻이 있었다. 넣은 날 CI를 보지 않아 네 번의 push 동안 몰랐다 — 안전망을 넣으면 그 push의 CI 결과까지 본다.
- "두 번 돌려 같다"로 경합과 플랫폼 차이를 가른다.
