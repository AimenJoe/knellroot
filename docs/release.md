# 배포와 릴리스(Knellroot)

nail `docs/release.md`를 줄인 것. 웹(Netlify) + Android APK(0022). PC·iOS는 nail의 워크플로를 가져올 때 이 표에 줄을 더한다.

## 무엇이 어디로 나가나
| 대상 | 트리거 | 워크플로 | 결과 |
|---|---|---|---|
| 웹(운영) | `main` push | `deploy-web.yml` | fmt·clippy(두 갈래)·test·운영판 wasm(`--no-default-features`)·조립·연기 시험(개발판으로) → https://knellroot.netlify.app. 올린 뒤 `build.txt`가 그 커밋인지 |
| 웹(개발판) | `release/**` push | `deploy-web.yml` | 개발 기능 켠 빌드 → 별칭 `dev`(https://dev--knellroot.netlify.app, noindex). 올린 뒤 별칭의 `build.txt`를 확인(아니면 실패) |
| 웹(PR) | PR | 같은 워크플로 | `pr-<번호>--knellroot.netlify.app` |
| Release + APK | `main` push에 `Cargo.toml` 버전이 새 값 **그리고** `docs/releases/v<버전>.md`가 있음 | `release.yml` → `build-android.yml` | 태그 `v<버전>`, GitHub Release(본문 = 그 md), `knellroot.apk`(운영판) 첨부 |
| APK(개발판) | `release/**` push(`src/`·`assets/`·`android/`·`third_party/`·`Cargo.*`) | `build-android.yml` | 패키지 `io.github.aimenjoe.knellroot.dev`("Knellroot dev") → 미리 보기 Release `dev`의 `knellroot-dev.apk`를 갈아 끼움 |
| APK(시험용) | `main` push(같은 경로), 또는 Actions → build-android → Run workflow | `build-android.yml` | Actions 아티팩트 `knellroot-apk-<release·dev>-<sha>` |

## 갈래(0022, nail 0193)
- `main` = 운영판, 패치만. `release/v0.3` = 개발판이자 작업 갈래(작업은 여기에 커밋·push, v0.2.0 뒤 새 main에서 만듦). 마일스톤을 닫으면 release → main + 버전 올리기, 다음 갈래는 새 main에서.
- main에 패치를 넣었으면 release 갈래에도 합친다.
- **배포(main push·버전 올리기)는 사장이 정한다.** 고친 것을 모아 "이것들을 고쳤는데 v<버전>으로 배포할까요?"

## 버전 규칙(nail 0079)
- 마이너(0.x.0) = 마일스톤 하나. 패치(0.x.y) = 고친 것·더한 것.
- Android `version_code`는 **릴리스마다 하나씩**(v0.1.0 = 1). 버전 숫자에서 만들지 않는다(작아지면 덮어 설치가 막힌다). 덮어 설치는 같은 서명 키 + 큰 version_code일 때만.

## 릴리스 절차
1. 작업이 release 갈래에 다 올라가 있고 사장이 "v<버전>으로 배포"를 말했다. `git fetch origin main && git merge-base --is-ancestor origin/main HEAD`.
2. 올린다: `Cargo.toml` `version`·`[package.metadata.android] version_code`, `docs/releases/v<버전>.md`(들어 있는 것/바뀐 것·해 보기·알려진 것·출처·English short), `Cargo.lock`.
3. 전체 검사(CLAUDE.md "커밋 전" + `cargo clippy --target aarch64-linux-android -- -D warnings`).
4. 커밋 메시지 `v<버전>: <한 줄>`, release 갈래에 push → 같은 커밋을 `git push origin HEAD:main`.
5. 확인(5~10분 뒤): Actions의 `deploy-web`·`release`(→ `build-android`) success, Release `v<버전>`에 `knellroot.apk`, 운영 주소 `build.txt`, 타이틀 오른쪽 위 버전 글자.
6. 사장에게 보고.

## 안 되는 경우
| 증상 | 원인 | 처방 |
|---|---|---|
| `release`가 돌았는데 Release가 없다 | `docs/releases/v<버전>.md`가 없거나 태그가 이미 있다 | 파일을 넣어 다시 push. 태그가 있으면 버전을 다시 올린다 — 태그를 지우지 않는다 |
| APK가 기기에서 "패키지가 유효하지 않음" | 서명 키가 바뀌었다(시크릿 없음 → 캐시 키 → 새 키) | 아래 시크릿을 넣는다. 키가 바뀐 APK는 앱을 지우고 설치 |
| 세션에서 태그 push가 안 된다 | 세션 git 프록시는 태그를 받지 않는다 | 그래서 CI가 태그를 만든다 |
| 패드 버튼이 Android에서 안 먹는다 | 패치 miniquad가 빠졌다(`[patch.crates-io]`) | `cargo test`의 `android_*` 시험, `third_party/miniquad/PATCHED.md` |

## 필요한 Secrets
- `NETLIFY_AUTH_TOKEN`·`NETLIFY_SITE_ID`(웹).
- `ANDROID_DEBUG_KEYSTORE_B64`(APK 서명 고정): 디버그 키스토어를 base64 한 줄로. 만들기:
  `keytool -genkeypair -keystore debug.keystore -storepass android -alias androiddebugkey -keypass android -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Android Debug,O=Android,C=US"` → `base64 -w0 debug.keystore`.
  레포가 공개라 키를 아티팩트·로그로 꺼내지 않는다.

## Netlify 사이트 만들기(한 번, 사장 — 끝남)
1. Netlify → Add new site → **Deploy manually**(Git 연동을 걸지 않는다). 사이트 이름 `knellroot`.
2. Site ID → GitHub Secrets `NETLIFY_SITE_ID`, Personal access token → `NETLIFY_AUTH_TOKEN`.
