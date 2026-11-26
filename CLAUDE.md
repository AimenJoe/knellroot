# CLAUDE.md — Knellroot

레트로 로우폴리 3D ARPG. 쿼터뷰 떼싸움 + 락온 숄더뷰 보스전. Rust + macroquad. 웹 먼저, Android 핸드헬드(컨트롤러, v0.1.0부터 APK — 0022), PC(Windows·macOS·Linux 묶음, 0135), iOS는 서명 안 한 .ipa만(0136).

## 먼저 읽을 것
1. `docs/concept.md` — 무엇을 만드는가(세계관은 nail의 먼 뒷이야기 — 세부는 초안)
2. `docs/phase-1-first-stage.md` — 지금 단계. 운영 버전은 v0.9.0 "무기의 길"(0270 — 배포 2026-11-26, 사장 "운영에 내보내", 판정 전 — 고칠 것은 v0.9.x, 작업 갈래 `release/v0.9`), 다음 마일스톤은 "디테일링"(다듬기 + 라이선스 걱정 없는 음악·효과음·특수효과, 사장 2026-11-26). 지난 버전마다 무엇이 나갔는지는 `docs/guide/versions.md`. 끝의 **"구현 상태"와 "사람이 할 일"**이 현재 위치다. PoC(`docs/phase-0-poc.md`)는 닫았다. **기본값은 얼려 있다(0013, `docs/tuning-frozen.txt`) — 판정 없이 바꾸지 않는다**
3. `docs/lab.md` — 랩이 무엇을 재고 값이 `tuning.rs`로 어떻게 가는가
4. `docs/decisions/` — 왜 그렇게 정했나. 바꾸려면 새 번호로
5. `docs/guide/<영역>.md` — 영역마다의 규칙(아래 "영역 안내"). **고칠 영역의 안내를 고치기 전에 읽는다** — 이 파일에서 옮긴 것이라 같은 힘을 가진다.

세션이 끝나면 "구현 상태"·"사람이 할 일"을 갱신하고 커밋한다. 배포는 `docs/release.md`(웹 운영 = Cloudflare `knellroot.beardworks.io/play/` — 옛 Netlify 주소는 기록 넘김 페이지, 토큰은 Environment `web-production`에만 0246, 개발판 = Netlify, Android APK·PC 묶음 셋 = GitHub Release, main push = 배포, **배포는 사장이 정한다**). **갈래(0022)**: 원격에는 셋만 둔다(사장 2026-11-24 정리 — 다 합쳐진 옛 갈래·작업 갈래는 지웠다). `main` = 운영판(패치만, `--no-default-features`), `release/v0.9` = 개발판이자 작업 갈래(v0.9.0 "무기의 길" 0270 — 무기별 기술 판·새 피해 타입·필드 보스·지역마다의 바닥, 2026-11-23 `release/v0.8`에서 땄다, 사장 "응, 계속하고 배포해" — 작업은 여기에 커밋·push, 웹 별칭 `dev`, APK 미리 보기 Release `dev`), `release/v0.8` = v0.8.x 패치 갈래(패치는 여기서 하고 `release/v0.9`에 합친다 — v0.9.0을 배포하면 지운다). 새 마일스톤 갈래는 그 전 갈래에서 따고, 배포해서 다 합쳐진 갈래는 지운다. 백업이 필요하면 `wip/<이름>`에 올리고 합친 뒤 지운다(세션은 지우는 push를 못 한다 — 사장에게 알린다). 릴리스 = `Cargo.toml` 버전·`version_code` + `docs/releases/v<버전>.md`를 main에(`release.yml`이 태그·APK) — main 룰셋(0092)이 걸리면 바로 push가 아니라 PR `release/v0.x` → `main`, 사장이 Merge. 기여 규칙은 첫 게임 `CONTRIBUTING.md`를 따른다. 이름·v0.1.0 조건은 `docs/decisions/0005`.

소개 페이지는 `site/`(0093 — 개발 중이라 한 장만, nail 0069 꼴, `pages.yml`이 main에서 GitHub Pages로, 주소 knellroot.beardworks.io). 페이지 글을 고치면 두 언어를 다 고치고 `tools/site/subset_font.py`로 글꼴을 다시 줄인다.

## 지식 베이스(KB)
일반 지식은 **github.com/AimenJoe/KB**. 세션에 KB를 붙이고 `maps/gamedev.md` → 노트를 먼저 읽는다. 이 프로젝트와 바로 닿는 것:
- `reference/macroquad/3d-rendering.md` — Camera3D(z-up 기본!)·Mesh·배치·조명 없음
- `concepts/camera-mode-switch-and-lock-on.md` — 리그 둘·입력 기준 즉시·구도·가림
- `reference/assets/retro-3d-models-blockbench-gltf.md` — 상자 모델·glTF 파이프라인(P4)
- `guides/poc-and-lab-before-content.md` — PoC·Lab 절차
- `guides/second-game-project-day-one.md` — 첫 게임에서 가져온 규칙 15개(이 레포의 뼈대가 그것)
- 첫 게임 설계 원칙: `concepts/action-roguelite-for-solo-dev.md`, 손맛: `concepts/melee-light-and-charged-attack-feel.md`·`fixed-timestep-and-hitstop.md`·`timing-parry.md`

**프로젝트 기록(TODO·실험 결과·결정)은 이 레포에, 프로젝트가 사라져도 유용한 지식은 KB에**(KB `CLAUDE.md` 규칙: frontmatter·verified·지도·`tools/kb.py check && index`).

## 구조(0278 — 2026-11-26에 끝난 리팩터링 기준)

방향은 하나다: **표(내용) → 월드(규칙) → 사건 → 앱(소리·이펙트·장면) → 렌더(읽기만)**. 뒤의 것이 앞의 것을 읽고, 앞의 것은 뒤의 것을 모른다.

| 층 | 어디 | 지키는 것 |
|---|---|---|
| 내용 표 | `weapons.rs`(`WEAPONS`·`CLASSES`·`Look`) · `skills/`(`SKILLS`·`Form`·`ENTRIES`) · `species.rs` · `stage/<이름>.rs` · `items.rs` · `charms.rs` · `garb.rs` · `journey.rs` · `i18n/` | 새 내용 = 표 한 줄 + 컴파일러가 짚는 match. 목록은 표에서 선다(손으로 지키는 목록을 만들지 않는다) |
| 수치 | `tuning.rs`(랩 `P` + `SPECS`) · `stats.rs`(`Stats::compute`) | 코드에 타별 숫자 금지 |
| 규칙 | `world.rs` + `world/`(단계 파일 · `skill_tick/forms.rs`) · `enemy.rs` + `enemy/states.rs` · `status.rs` · 순수한 시간표와 규칙(`fall`·`wake`·`arrive`·`descent`·`knell`·`tutorial`·`anim`·`gait`·`idle`) | 카메라·소리·메뉴·그림을 모른다 |
| 입력 | `input/`(`collect.rs` 장치마다 모으기 → `merge`) · `touch/`(`read_fingers` 순수 · `icons.rs` 그림 데이터) | 밖으로는 `Actions`만 |
| 앱 | `app/`(장면 스택 `stack` · 흐름 `flow` · 탭마다 몸 파일 · 사건 → 소리·흔들림·줌의 표 `cue.rs`) · `fx.rs` + `fx/` · `sfx.rs` · `camera.rs` · `hud.rs` | 월드의 사건을 듣는다. 그리기는 읽기만 — 무엇을 보이나는 순수한 뷰가 정한다 |
| 렌더 | `render/` · `scenery.rs` · `backdrop.rs` · `flora.rs` | 월드를 읽기만 |
| 기록 | `save.rs`(`Progress`) · `prefs.rs` | 키 이름·칸의 뜻은 바꾸지 않는다 |

추상화의 원칙(0278 3절 — 새 코드도 이 꼴로):
1. **빠뜨리면 컴파일이 깨지게**: 내용 enum을 보는 제품 코드에 `_ =>`를 두지 않는다. 일부러 아무것도 안 하는 갈래는 이름을 적는다.
2. **성질은 서술자 한 줄로**: 갈래마다의 "값"(둥근가·이름·그림·소리·흔들림)은 `const fn …::info()`·표의 줄 한 곳(`Form::info()` · `Look::def()` · `Weapon` 줄의 글·그림 · `Event` → `Cue`) — 여러 파일에 작은 match를 흩지 않는다.
3. **동작은 빠짐없는 match + 갈래마다 이름 있는 메서드**: 긴 match의 몸은 `Ctx` 하나를 받는 메서드로(`tick_form` → `skill_tick/forms.rs` · `enemy::tick` → `enemy/states.rs` · `Fx::on_skill` → `fx/skills.rs` · `hud::draw` → `draw_<구역>`). 공용 조각은 셋 넘게 나란한 것만.
4. **규칙은 그리기 밖에**: 그리기가 셈하던 것은 기록·월드만 받는 순수한 뷰로(`EquipView` · `BagView` · `skill_rows`/`RowFades` · `hud::near_button`) — 시험이 붙는다. 앱이 월드의 칸을 바로 고치지 않는다(잔불은 지갑 메서드).

추상화하지 **않는** 곳(0278 2.3절): 종류마다의 손을 트레이트(`ClassHooks`)로 묶지 않는다 — 표 + 그 동사의 자리에서 한 곳에 묻는다. 스킬마다의 이펙트·소리는 정체성이라 스킬별 match로 둔다. 큰 창의 탭을 트레이트로 만들지 않는다(와일드카드 없는 match가 지킨다). 모양(`Form`)의 동작은 트레이트 객체가 아니다.

## 어디서나 지키는 규칙
- 로직 고정 60Hz(`TICK_DT`), 렌더는 보간. 히트스톱은 로직 시계만(`World::hitstop`).
- 수치는 `tuning.rs` 한 곳. 랩이 바꾸는 값은 `P` + `SPECS` 한 줄씩. 랩의 export를 붙여 넣는 곳도 여기. 코드에 타별 숫자 금지.
- `world.rs`는 카메라·소리·메뉴를 모른다. `Basis`(입력 기준)와 `Event`로만 바깥과 만난다. 틱은 단계 메서드 순서(`tick_lock → tick_player → apply_strike → apply_turn → tick_enemies → collide → cleanup`) — 새 메커닉은 맞는 단계에. 긴 단계는 `world/`의 단계 파일(`player_tick`·`enemy_tick`·`bolt_tick`·`strike`·`skill_tick`)에 이름 있는 메서드로, 월드 시험은 `world/tests/<주제>.rs`(0159 R2·R3 — 스킬 시험만 감춘 함수를 부르므로 `world/skill_tick/tests.rs`, 큰 기술·놓인 것의 틱은 `skill_tick/ult.rs`·`fields.rs`, 0251 R11). 방랑자의 상태는 뜻의 묶음으로(0159 R4 — `Player`의 `skill`·`parry`·`beat`·`ranged`·`hurt` + 0278 R24 `class`(종류마다의 손 — 분노 · 쪼개기 · 들이받기)·`bare`(빈 손의 결)·`swap`(바꾸기 · 교체 공격): 새 필드는 맞는 묶음에, 어디에도 안 맞는 것만 `Player`에 바로). 틱 사이의 쪽지는 `World::notes`(`TickNotes` — 걷는 단계가 저마다 `take`)·결의 이번 틱은 `World::veins`, 잔불은 지갑 메서드로만(`World::embers()`·`earn`·`spend`·`set_embers` — 칸은 감췄다).
- 쉬기·쓰러져 다시 서기가 방랑자가 놓거나 던진 것을 치우는 곳은 `World::clear_placed` 한 곳(0278) — 놓이거나 날아가는 것의 목록을 새로 만들면 거기에 한 줄(시험 `clearing_what_the_wanderer_placed_leaves_nothing_and_brings_thrown_weapons_home`).
- **초월 함수는 `crate::det::Det`로**(0279): `x.sin_d()`·`y.atan2_d(x)`·`sin_cos_d`·`exp_d`·`powf_d` … — std의 `sin`·`cos`·`atan2`…는 시스템 libm을 불러 glibc 판마다 끝자리가 달라 트레이스가 기계마다 갈렸다(`clippy.toml`이 막는다, 새 함수는 `det.rs`에 더한다). 로직(월드·적·스테이지·이펙트)은 glam의 각도 함수(`Quat::from_rotation_y`·`from_angle`·`angle_between`)도 쓰지 않는다 — `det::turn_y`(시험 `logic_keeps_off_glams_trig`). `sqrt`·`abs`·`powi`·`to_radians`는 그대로.
- 좌표: y-up, XZ 평면 이동, 1 단위 = 1m, 플레이어 키 1.8. 쿼터뷰의 화면 위 = 월드 -z.
- 새 메커닉 = 단위 테스트 하나 이상(규칙을 이름으로 읽히게). 소리·그림 핸들을 로직에 넣지 않는다 — 그래서 시나리오 일곱을 600틱씩 돌리는 시험이 있다.
- **내용 enum은 와일드카드 없이(0278 R14)**: `Form`·`skills::Id`·`Ult`·`Bare`·`Rhythm`·`DevStart`를 보는 제품 코드의 match에 `_ =>`를 두지 않는다 — 일부러 아무것도 안 하는 갈래는 이름을 적는다(새 갈래는 컴파일러가 짚는다). 모양의 성질은 `Form::info()`(R16), 탄의 꼴은 `weapons::Look` + `LookDef`(R17) 한 곳. **골든 트레이스(0278 R12)**: `world/tests/golden.rs`가 스킬(고르기 ㄱ/ㄴ)·궁극기·무기마다의 RB·방패 던지기를 고정 마당에서 6초 돌려 해시 — 리팩터링은 이 값이 그대로여야 한다, 일부러 동작을 바꿨으면 `GOLDEN_WRITE=1 cargo test golden_traces`로 `golden.txt`를 다시 쓴다(새 스킬·궁극기·무기는 저절로 든다). 그 곁의 **시나리오 트레이스**(`world/tests/scenario_golden.rs` — 0278 R20 준비): 개발 시나리오마다 정한 누름 차례로 15초(적이 깨어나 치고 받아쳐지고 쓰러진다) + 첫 마당이 못 보는 때(패링 · 맞음 · 쓰러뜨림)의 겨눈 경우 — 첫 골든에서 고르기 ㄱ/ㄴ이 같던 스킬은 여기서 갈려야 한다(시험), 다시 쓰기는 `GOLDEN_WRITE=1 cargo test scenario_traces`. 둘 다 그대로여야 리팩터링이다. **사건 덮개(0278 R12)**: 새 `Event` = `Event::ordinal`의 번호 + `samples` 한 줄 + `COUNT`, 그리고 소리(`app/cue.rs` `cue`의 갈래 — 컴파일러가 짚는다, 조용한 것은 `app/play.rs` 시험의 `SILENT`)·이펙트(`fx` 묶음 하나 또는 `NO_FX`).
- 입력은 `Actions`만 — `KeyCode`·패드 버튼은 `input/` 밖에 없다. 화면에 버튼 이름을 쓸 때는 `input::prompt(Act)`, 글에 키 이름을 박지 않는다(→ `docs/guide/input.md`).
- 렌더는 월드를 읽기만. 그림은 틱의 `rig`가 아니라 `cam.view`를 읽는다(→ `render.md`·`camera.md`).
- 플레이어가 보는 글은 `i18n/<언어>.rs` 표에만(한국어·영어 둘 다), 통용어로(0273), 글꼴은 서브셋이라 새 글자를 쓰면 다시 줄인다(→ `app-ui.md`).
- 기록 키(`save::key`)·prefs 칸의 뜻은 바꾸지 않는다. 판을 넘어 남는 것을 새로 만들면 `Progress`에 칸 + 키 하나 + 시험(→ `items.md`).
- 손으로 PNG·GLB를 고치지 않는다 — 원본은 `tools/art/`의 스크립트(→ `art.md`).
- 방랑자의 체력을 깎는 곳은 `wanderer_takes_hit` 하나, 적이 맞는 곳은 `Enemy::struck` 하나, 타입이 걸리는 곳은 `world::afflict` 하나(→ `feel.md`·`skills.md`).

## 영역 안내(`docs/guide/`)

| 고치는 곳 | 먼저 읽을 안내 | 안에 든 것 |
|---|---|---|
| `src/input/` · `src/touch/` · 조작 설정 | `input.md` | 배치(`Bindings`) · 마우스 = 오른쪽 스틱 · 터치 버튼과 dp · 끌어 조준 · 조준 보조 · 곁의 할 일 |
| `tools/test/` · `scenario.rs` · `app/dev.rs` | `testing.md` | 헤드리스 스크립트(장면을 기다린다 · 이름으로 연다 · 깃발) · 개발 시나리오와 개발 창 |
| `camera.rs` · `descent.rs` | `camera.md` | 리그와 보간 · 시점 셋 · 뿌리 굴 전환 |
| `weapons.rs` · `shield.rs` · `world/bolt_tick.rs` · `world/strike.rs` | `weapons.md` | 체력·피해의 눈금 · 무기 표 · 주/보조무기 · 교체 공격 · 방패 · 탄과 탄의 꼴 |
| 무기 종류 · `plates.rs` | `classes.md` | 종류 표 · 쥔 꼴 · 스킬 트리 · 주특성·보조특성·빈 손의 결 · 종류마다의 손(창·쌍단검·대검·낫·완드) |
| `skills/` · `world/skill_tick/` · `status.rs` · `app/skills.rs` | `skills.md` | 스킬 표 · 기술 판 · 무기 슬롯과 스킬 · 피해 타입 |
| `items.rs` · `app/bag.rs` · `forge.rs` · `garb.rs` · `stats.rs` · `charms.rs` · `growth.rs` · `progression.rs` · `save.rs` · `journey.rs` | `items.md` | 아이템 · 가방 · 강화 · 덧옷 · 능력치 · 성장 · 기록 · 여정 |
| `app/` · `ui.rs` · `hud.rs` · `i18n/` | `app-ui.md` | 장면 스택 · 큰 창과 탭 · UI · 글과 글꼴 · 용어 |
| `stage/` · `tools/maps/` · `species.rs` · `enemy.rs` · `tutorial.rs` · `wake.rs` · `arrive.rs` · `fall.rs` | `stage.md` | 스테이지 · 서막 · 필드 보스 · 적 · 튜토리얼 · 보는 장면 · 쓰러짐 |
| `render/` · `scenery.rs` · `backdrop.rs` · `flora.rs` | `render.md` | 그리는 차례 · 적의 빛 · 대비 · 색상대 · 바람 · 거르기와 재기 · 배치와 상주 지형 |
| `tools/art/` · `assets/models/` · `anim.rs` · `gait.rs` · `idle.rs` | `art.md` | 그림 기준 · Blender·Meshy 파이프라인 · 방랑자의 스킨 몸 · 걸음 |
| `fx.rs` · `fx/` · `sfx.rs` · `app/cue.rs` | `feel.md` | 타격 줌 · 휘두름 · 맞음 피드백 · 이펙트 · 소리 |
| `rust-toolchain.toml` · `.github/` · `web/` · `ios/` · `pc/` | `platform.md` | Rust 버전 · PC · iOS · 웹 로딩과 PWA |

안내의 구조를 바꾸는 코드를 고쳤으면 그 안내를 같이 고친다. 새 규칙은 맞는 안내에 적고, 어디서나 걸리는 것만 이 파일에 적는다 — 같은 규칙을 두 곳에 쓰지 않는다.

## 안전망(0278 — 리팩터링은 이것들이 그대로여야 한다)

| 시험 | 지키는 것 | 일부러 바꿨을 때 |
|---|---|---|
| `golden_traces`(`world/tests/golden.rs` + `golden.txt`) | 스킬·궁극기·무기마다의 RB를 고정 마당에서 돌린 해시 | `GOLDEN_WRITE=1 cargo test golden_traces` |
| `scenario_traces`(`world/tests/scenario_golden.rs`) | 개발 시나리오 15초 + 패링·맞음·쓰러뜨림의 겨눈 경우 | `GOLDEN_WRITE=1 cargo test scenario_traces` |
| `fx_traces`(`fx/traces.rs` + `fx_traces.txt`) | 사건·스킬·시나리오마다 나온 이펙트 | `FX_WRITE=1 cargo test fx_traces` |
| `the_cue_table_matches_its_snapshot`(`app/cue_snapshot.txt`) | 사건 → 소리·흔들림·킥·줌 | `CUE_WRITE=1 cargo test the_cue_table_matches_its_snapshot` |
| 기록·무기·장신구 고정값(`save_fixture.txt`·`weapon_fixture.txt`·`charm_fixture.txt`) | 기록의 글과 표의 수 | 키를 바꾸는 것이 아닌지 먼저 본다 |
| `app::text_fit` | 고정 칸의 글 넘침(두 언어) | 칸을 고친다 — 시험을 느슨하게 하지 않는다 |

다시 쓴 고정값은 바뀐 줄을 읽고 까닭을 커밋에 적는다. **안전망을 넣거나 고정값을 다시 쓴 push는 CI 결과까지 본다**(0279 — 여기서만 통과하고 CI에서 네 번 빨갰다). 동작을 바꾸지 않는 커밋과 동작을 바꾸는 커밋을 섞지 않는다.

## 커밋 전
```sh
cargo fmt --all -- --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo clippy --target wasm32-unknown-unknown --no-default-features -- -D warnings   # 운영판
cargo clippy --target aarch64-linux-android -- -D warnings                          # NDK 없이 됨(CI는 --no-default-features도)
cargo clippy --release --target x86_64-pc-windows-msvc --no-default-features -- -D warnings   # PC. macOS는 aarch64-apple-darwin
cargo clippy --release --target aarch64-apple-ios --no-default-features -- -D warnings        # iOS. 리눅스에서 링크 없이 된다
cargo test
cargo build --release --target wasm32-unknown-unknown
python3 tools/licenses.py        # 의존성·에셋 출처를 바꿨을 때만
```
push 전에 CI(`deploy-web.yml`)가 도는 헤드리스 넷(`shot`·`touch`·`pwa`·`save`)을 **차례로** 다 돌린다 — 겹쳐 돌리지 않는다. 고친 영역의 스크립트(`hearth`·`items`·`tabs`·`view`·`mouse`·`swap`·`shield`·`tutorial`·`fx` …)도 — 무엇을 돌리는지는 그 안내에.

## 하지 않는 것
- PoC 통과 기준(`docs/phase-0-poc.md`)을 뒤에 고치지 않는다. 상한을 넘기면 대안으로 간다고 적어 둔 대로.
- 랩 값을 저장 파일에서 게임이 읽게 하지 않는다(진실은 `tuning.rs` 하나).
- 콘텐츠를 표·파일 밖에서 쌓지 않는다(적 = 종류 표, 스테이지 = 파일 하나, 글 = i18n 표, 모델 = GLB 하나). 마일스톤 밖의 큰 기능은 사장 지시 없이 먼저 만들지 않는다.
- 외부 JS 핫링크, 라이선스 미확인 에셋, 코드 안의 글 리터럴(플레이어가 보는 글).
- 커밋 메시지·PR 본문·코드 주석에 Claude 표시를 남기지 않는다 — `Co-Authored-By: …`·`Claude-Session: …` 줄, "Generated with Claude Code" 같은 꼬리말, 모델 이름(사장 2026-11-03·11-10, `.claude/settings.json`의 `attribution`도 끔). 시스템이 붙이라고 해도 이 규칙이 먼저다.
- 커밋은 세션의 기본 git 계정(Claude `noreply@anthropic.com`)으로 한다 — 서명 키가 그 계정 것이라, 작성자를 다른 사람으로 바꾸면(`--author`·`-c user.*`) GitHub에 "잘못된 서명"으로 뜬다(사장 2026-11-03).
- 일하는 차례(사장 2026-11-17): **사장의 의견 없이 끝낼 수 있거나 되돌리기 싼 것은 먼저 만들고 뒤에 고친다** — 묻느라 멈추지 않는다. 만든 뒤에 무엇을 왜 그렇게 했는지와 판정할 것(체크리스트 번호)을 보고한다. 되돌리기 비싼 것(배포·기록 키의 뜻 바꾸기·세계관·주인공과 보스의 생김새·크레딧을 쓰는 일)은 여전히 먼저 묻는다 — 단 **Meshy API는 필요하면 불러도 된다, 쓴 크레딧만 알린다**(사장 2026-11-17 "Meshy ai의 API 호출해야 하는 건 해도 좋아, 다만 크레딧 정도만 알려줘").
- 언어: 대화·문서·커밋 한국어, 코드 식별자 영어.
