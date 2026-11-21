# 0264 — CI를 덜 돌린다: 문서만 바뀐 push · main의 APK 두 번

- 날짜: 2026-11-21
- 상태: 채택
- 사장(2026-11-21): "워크플로우가 너무 많이 돌고 있는거 아닌가?" → 줄이는 안에 "응".

## 왜
하루(2026-11-21)에 push·머지 열네 번쯤으로 실행이 수십 개였다. 같은 일을 두 번 한 것이 셋:
1. main에 머지하면 `release.yml`이 build-android를 부르는데, build-android도 main push에 따로 돌아 운영 APK를 두 번 만들었다.
2. 문서만 고친 커밋(CLAUDE.md·결정 문서)에도 deploy-web이 fmt·clippy·test·wasm·헤드리스 시험을 다 돌았다(다른 셋은 이미 경로로 거른다).
3. 배포 뒤 `release/v0.7`을 main과 같은 커밋으로 맞추는 push가 이미 빌드한 커밋을 다시 돌렸다.

## 정한 것
- build-android: push는 `release/**`만(main 운영 APK는 `release.yml`이 부를 때만). build-ios는 그대로 main에서도 돈다 — 운영 .ipa는 main의 아티팩트다.
- deploy-web: **push에만** 경로 거르개 — `docs/**`·`*.md`만 바뀌면 돌지 않는다. `docs/tuning-frozen.txt`는 `cargo test`가 읽으니 넣는다.
  pull_request에는 두지 않는다: main 룰셋(0092)의 필수 검사 `build`는 문서만 고친 PR에서도 와야 Merge가 막히지 않는다.
- 일하는 습관: 배포 뒤 release 갈래를 main 커밋까지 앞으로 보내는 push는 다음 작업 커밋과 함께(혼자 push하지 않는다), v0.7 → v0.8 합치기·문서 커밋은 모아서 push.

## 하지 않은 것
- PR과 push가 같은 커밋을 두 번 빌드하는 것(release → main PR)은 그대로 둔다: PR 쪽을 건너뛰면 필수 검사 `build`가 건너뜀·취소로 남아
  Merge가 막히거나 검사 전에 열릴 수 있다. 겹침은 배포마다 한두 번이라 작다. release 갈래로 오는 다른 PR(#33·#35)은 이 PR 빌드가 유일한 검사라 남긴다.
