# 배포와 릴리스(Knellroot)

nail `docs/release.md`를 줄인 것. 지금은 **웹(Netlify) 하나**다. APK·PC·iOS는 nail의 워크플로를 가져올 때 이 표에 줄을 더한다.

## 무엇이 어디로 나가나
| 대상 | 트리거 | 워크플로 | 결과 |
|---|---|---|---|
| 웹 | `main` push | `deploy-web.yml` | fmt·clippy(두 갈래)·test·wasm·조립·헤드리스 연기 시험 → Netlify 운영 주소(시크릿 `NETLIFY_AUTH_TOKEN`·`NETLIFY_SITE_ID`가 있을 때만). 올린 뒤 `<주소>/build.txt`가 그 커밋 SHA인지 확인 |
| 웹(PR 미리보기) | PR | 같은 워크플로 | `pr-<번호>--<사이트>.netlify.app` |

## 갈래(0005)
- **v0.1.0 전**: `main` 하나. push = Netlify 운영 주소에 개발판(랩·시나리오 켬). 아직 플레이어가 없으니 운영 = 개발.
- **v0.1.0 뒤**: nail 0193대로 `main` = 운영(패치만), `release/v0.x` = 개발판 별칭 `dev`. 그때 `deploy-web.yml`에 nail의 "Deploy dev (alias)" 단계를 가져온다.
- 배포(main push·버전 올리기)는 사장이 정한다. 고친 것을 모아 "이것들을 고쳤는데 v<버전>으로 배포할까요?"

## 버전 규칙(nail 0079 그대로)
- 마이너(0.x.0) = 마일스톤 하나. 패치(0.x.y) = 고친 것·더한 것.
- **v0.1.0 "첫 웹 판"**(0005): 조건표를 마치면 `Cargo.toml` 0.1.0 + `docs/releases/v0.1.0.md`를 main에.

## 릴리스 절차
1. 작업이 main에 다 올라가 있고 사장이 "v<버전>으로 배포"를 말했다.
2. `Cargo.toml` 버전, `docs/releases/v<버전>.md`(바뀐 것·해 보기·출처), `Cargo.lock` 함께 커밋.
3. 커밋 메시지 `v<버전>: <한 줄>`, push → Actions `deploy-web` 성공, 운영 주소 `build.txt` 확인, 타이틀의 버전 글자.
4. 사장에게 보고: 들어간 것, 볼 것.

## Netlify 사이트 만들기(한 번, 사장)
1. Netlify → Add new site → **Deploy manually**(Git 연동을 걸지 않는다 — CLI 배포를 Netlify 자체 빌드가 덮어쓴다, KB `reference/netlify/deploy-from-ci.md`). 사이트 이름 `knellroot`.
2. Site configuration → General → **Site ID** 복사 → GitHub 레포 Settings → Secrets and variables → Actions → `NETLIFY_SITE_ID`.
3. Netlify User settings → Applications → **Personal access token** → `NETLIFY_AUTH_TOKEN`.
4. main에 아무거나 push → Actions에서 `Deploy production` 단계가 돌았는지, `https://knellroot.netlify.app/build.txt`가 그 SHA인지.
