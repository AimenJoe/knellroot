# 0093. main 룰셋 — 운영 갈래는 관리자만 바로, 나머지는 PR

- 날짜: 2026-11-07(처음 0092로 적었으나 같은 날 다른 세션의 소개 페이지 0092와 겹쳐 0093으로 옮김)
- 상태: **채택·걸림**(2026-11-07, 사장이 nail 룰셋 둘을 그대로 가져옴 — `main-only` 26757362·`release-branches` 26757372)
- 사장: "main 룰셋 만들자, 내 레포 보호랄수 있도록."

## 전
- knellroot에는 룰셋이 하나도 없었다(공개 API `GET /repos/AimenJoe/knellroot/rulesets` = 빈 목록, 2026-11-07). 공개 레포라 쓰기 권한자만 push할 수 있지만, 운영 갈래(main)를 지우거나 강제 push로 덮는 것, 검사 없이 넣는 것을 막는 장치가 없었다.
- nail에는 2026-11-04부터 `main-only`(지우기·강제 push 막기 + PR 필수 + `build` 검사 통과, 우회 = 관리자)와 `release-branches`가 걸려 있다(KB `guides/protect-deploy-pipeline-in-public-repo.md`).

## 결정
- `.github/rulesets/main.json`(nail `main-only`와 같은 꼴):
  - 대상 `refs/heads/main`.
  - **지우기 막기**·**강제 push 막기**.
  - **PR 필수**(승인 0명 — 혼자 하는 레포라 본인이 Merge한다, 합치기는 merge만).
  - **필수 검사** `build-deploy`(GitHub Actions — `deploy-web.yml`의 잡: fmt·clippy·test·wasm·연기 시험).
  - **우회**: 레포 관리자(사장)만. Claude 앱은 우회 목록에 없다.
- 그래서 **운영 배포는 사장이 PR을 Merge할 때만** 일어난다("배포는 사장이 정한다"를 규칙이 지킨다). `docs/release.md` 절차를 바꿨다:
  - 세션은 버전을 올린 커밋을 `release/v0.3`에 push하고 PR(`release/v0.3` → `main`)을 연다. 사장이 검사 통과를 보고 Merge한다.
  - Merge 뒤 `release/v0.3`를 `origin/main`으로 앞으로 감는다(Merge 커밋만 더 있으므로 fast-forward).
- `release/**`는 이번엔 그대로 둔다. nail처럼 `release-branches`(만들기·바꾸기·지우기 막기, 우회 = 관리자 + Claude 앱)를 걸 수 있지만, 세션이 거기에 바로 push하므로 앱 아이디를 확인한 뒤에 한다.

## 거는 법(사장, 한 번)
GitHub → knellroot → Settings → Rules → Rulesets → New ruleset ▾ → **Import a ruleset** → `.github/rulesets/main.json` → 확인 → Create.

## 확인
- 건 뒤 공개 API `https://api.github.com/repos/AimenJoe/knellroot/rulesets`에 `main-only`가 보이는지.
- 다음 배포 때 PR에 `build-deploy` 검사가 붙고 통과해야 Merge 단추가 열리는지.

## 건 뒤(2026-11-07)
- 사장이 처음 제안한 JSON 대신 **nail 것을 그대로** 가져왔다. 공개 API로 읽은 실제 설정을 `.github/rulesets/main.json`·`release.json`에 옮겨 두었다(다음 레포에 그대로 가져오기).
  - `main-only`: 지우기·강제 push 막기, PR 필수(승인 0, 합치기 셋 다), 필수 검사 **`build`**, 우회 = 레포 관리자.
  - `release-branches`(`release/**`): 만들기·바꾸기·지우기·강제 push 막기, 우회 = 레포 관리자 + GitHub App 1236702(세션이 push하는 앱).
- 필수 검사 이름이 nail의 잡 이름 `build`라, knellroot 웹 CI 잡 `build-deploy`를 **`build`로 바꿨다** — 안 그러면 PR이 오지 않는 검사를 기다려 Merge가 막힌다.
