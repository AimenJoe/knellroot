# CLAUDE.md — Knellroot

레트로 로우폴리 3D ARPG. 쿼터뷰 떼싸움 + 락온 숄더뷰 보스전. Rust + macroquad. 웹 먼저, PC, 나중에 Android 핸드헬드(컨트롤러).

## 먼저 읽을 것
1. `docs/concept.md` — 무엇을 만드는가(세계관은 초안, 바뀔 수 있다)
2. `docs/phase-1-first-stage.md` — 지금 단계(첫 스테이지 "침묵의 성소" 초안, 0019 — v0.2.0으로 닫음 0060, 다음 마일스톤 v0.3.0 "남는 판" 제안 0064). 끝의 **"구현 상태"와 "사람이 할 일"**이 현재 위치다. PoC(`docs/phase-0-poc.md`)는 닫았다. **기본값은 얼려 있다(0013, `docs/tuning-frozen.txt`) — 판정 없이 바꾸지 않는다**
3. `docs/lab.md` — 랩이 무엇을 재고 값이 `tuning.rs`로 어떻게 가는가
4. `docs/decisions/` — 왜 그렇게 정했나. 바꾸려면 새 번호로

세션이 끝나면 "구현 상태"·"사람이 할 일"을 갱신하고 커밋한다. 배포는 `docs/release.md`(웹 = Netlify, Android APK = GitHub Release, main push = 배포, **배포는 사장이 정한다**). **갈래(0022)**: `main` = 운영판(패치만, `--no-default-features`), `release/v0.3` = 개발판이자 작업 갈래(v0.2.0 배포 2026-11-05 뒤 — 옛 `release/v0.2`는 그대로 둠)(작업은 여기에 커밋·push — 웹 별칭 `dev`, APK 미리 보기 Release `dev`). 릴리스 = `Cargo.toml` 버전·`version_code` + `docs/releases/v<버전>.md`를 main에(`release.yml`이 태그·APK). 기여 규칙은 첫 게임 `CONTRIBUTING.md`를 따른다. 이름·v0.1.0 조건은 `docs/decisions/0005`.

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
- 입력은 `Actions`만. `KeyCode`·패드 버튼은 `input.rs` 밖에 없다(터치는 `touch.rs`, `input.rs`만 부른다, 0006). Android 패드는 패치한 miniquad(`third_party/miniquad`, `[patch.crates-io]`, nail에서 가져옴)가 버튼을 F13~F24 키로·스틱을 `joystick_axes`로 넘긴다 — miniquad를 올리면 `PATCHED.md`대로 다시 넣고 `cargo test`의 `android_*`를 본다(0022). 커밋 전에 `cargo clippy --target aarch64-linux-android -- -D warnings`(NDK 없이 됨). 게임 동작의 키·버튼은 `input::Bindings`(설정에서 바꿈, 0024 — 소울라이크식, 공격만 X·J. 0048: 마법 RB·U, 등잔 Y·R, 상호작용은 점프 버튼이 같이 — 스테이지가 `World::interact_here`로 자리를 알린다. 0058: 새로 설치한 판의 기본은 키보드+마우스(왼쪽 클릭 공격·오른쪽 클릭 마법·휠 클릭 락온·L-Shift 패링, 왼손은 그대로 — 땅 돌리기 Q·E도), 키 칸 번호는 `KEYS` 뒤에 마우스 버튼 `MICE`, "키보드만"은 `Bindings::keyboard_only`. **마우스는 패드의 오른쪽 스틱과 같은 일**: 쿼터뷰에서 커서를 땅에 비춘 점(`Actions::aim_at`, 앱이 `camera::screen_ray`로)이 조준, 최근 만진 커서만(`input::cursor_live`), 락온 중엔 안 쓴다, 감도 칸은 없다(OS 커서). 저장 칸 뜻을 바꾸면 prefs `MAGIC`을 올린다), 화면에 버튼 이름을 쓸 때는 `input::prompt(Act)`(지금 장치·지금 배치) — 글에 키 이름을 박지 않는다. 설정 저장은 `prefs.rs`(웹 localStorage·네이티브 `prefs.txt`, 칸을 뒤에만 붙인다). 눌림은 래치, 틱이 소비. 터치 버튼 자리를 바꾸면 `tools/test/touch.js`의 `layout()`도. 터치 배치는 공격 둘레 호 + 상황 버튼(`touch::Ctx` — 뜰 때만, 다른 버튼을 밀지 않는다, 0053). 타이틀·멈춤 메뉴 줄을 바꾸면 `tools/test/shot.js`·`leak.js`의 메뉴 길도(0052 — 둘은 처음에 "키보드만 배치로"를 고르고 J·K·L로 싸운다, 마우스 배치는 `tools/test/mouse.js`, 0058).
- 로직 고정 60Hz(`TICK_DT`), 렌더는 보간. 히트스톱은 로직 시계만(`World::hitstop`).
- 수치는 `tuning.rs` 한 곳. 랩이 바꾸는 값은 `P` + `SPECS` 한 줄씩. 랩의 export를 붙여 넣는 곳도 여기. 코드에 타별 숫자 금지.
- `world.rs`는 카메라·소리·메뉴를 모른다. `Basis`(입력 기준)와 `Event`로만 바깥과 만난다. 틱은 단계 메서드 순서(`tick_lock → tick_player → apply_strike → apply_turn → tick_enemies → collide → cleanup`) — 새 메커닉은 맞는 단계에.
- 카메라는 `camera.rs`: 원하는 리그를 매 프레임 계산 + 모드가 바뀌면 지금 리그에서 보간. **입력 기준은 월드가 즉시 바꾸고 카메라만 늦게 간다.** `Camera3D`는 `up: Vec3::Y`를 매번 적는다(기본 z-up).
- 좌표: y-up, XZ 평면 이동, 1 단위 = 1m, 플레이어 키 1.8. 쿼터뷰의 화면 위 = 월드 -z.
- 렌더는 월드를 읽기만. 렌더 타깃 → 정수 배율(물리 픽셀로 센다, 0006) → HUD는 화면 패스. 렌더 타깃 크기는 화면에 맞춘다(`render::fit_internal` — 배율은 높이 / `INTERNAL_H` 반올림, 크기는 화면 / 배율, 0046) — 내부 해상도를 상수로 박지 않는다. Android는 몰입 모드(패치한 miniquad `setFullScreen`, `PATCHED.md`).
- 장면은 `app/`(타이틀·싸움·멈춤·끝·결과·설정·조작·DEV — 0052). **기초 설계는 0066**: 장면은 스택(`app/stack.rs` — 창은 `Go::Open`으로 얹고 뒤로 = `Go::Back`, 닫으면 연 줄로 돌아온다, 돌아갈 곳 깃발을 두지 않는다), 무엇 위에 그리나는 `Scene::layer()`(`_` 없는 match), 처음 포커스는 `flow::focus`. 흐름은 `app/flow.rs` 순수 함수(시험), `app/mod.rs`는 그대로 한다. 화면 상태(커서·창)는 스택 칸에 — `App`에 화면별 필드를 늘리지 않는다. 새 장면 = `Scene` 한 줄 + `layer` 한 줄 + flow 함수 + 그림 함수.
- 스테이지는 `stage/`: 글자 지도 → `Level`(벽·소품·불빛·문·표지), 진행 대본 `StageRun`(구역·물결·문·보스·핵). 새 스테이지 = `stage/<이름>.rs`의 `StageDef` 하나. 월드는 스테이지를 모른다 — 스테이지가 월드를 지켜보며 적을 세우고 문을 여닫는다.
- 무기는 `weapons.rs` 표(0051): 줄 = (X 콤보·강타, RB 마법/사격, 수류탄), 시간·거리는 랩 `attack.*`·`heavy.*`에 곱하는 배율, 동작 이름은 `wanderer.py`, 모델 부위는 오른손 뼈(든 것만 보인다). 새 무기 = `Id` 한 줄 + 표 한 줄 + 동작 + 부위 + 글. 자세 막대는 `world::add_poise` 한 곳. 후딜레이 캔슬·누름 받아 두기는 `World::cancel_or_buffer` 한 곳(0057) — 새 동작은 캔슬 시점과 "직접 받는 누름"을 거기에 한 줄.
- 적 종류는 `species.rs` 표(수치·행동 방식·모델 이름), 행동은 `enemy.rs`. 새 종류 = `Kind` 한 줄 + 표 한 줄. 예고 모양은 `TeleShape`(원판·부채꼴·띠·고리). 잠든 적은 `Enemy::active()`가 거짓.
- 개발 시나리오는 `scenario.rs` 표 한 줄(`checks`에 그것이 보는 PoC 번호) — 타이틀의 개발 항목으로 들어간다. feature `dev`(기본 켬)로 가린다. 운영판 = `--no-default-features`.
- 새 메커닉 = 단위 테스트 하나 이상(규칙을 이름으로 읽히게). 소리·그림 핸들을 로직에 넣지 않는다 — 그래서 시나리오 일곱을 600틱씩 돌리는 시험이 있다.
- **UI 기본틀은 KB와 nail에서(0065, 사장)**: 새 화면·메뉴·알림 전에 KB UI 노트(`controller-layout-and-remapping`·`hud-readability-and-quick-slots`·`roguelite-save-slots-and-continue`·`game-audio-without-a-composer` 메뉴 소리)와 nail의 같은 화면을 읽고 같은 꼴로. 둘 다에 없는 꼴이면 스터디 → 제안 → 사장 확인 뒤. 되돌릴 수 없는 일은 `Scene::Ask`(하는 일 이름 / 취소, 커서는 취소에서).
- UI(0042, nail 방식): 안내는 짧게(메뉴 안내만, 전체 조작은 조작 설정 화면), 키 이름은 키캡 `ui::key`, 고정 칸은 `ui.fit`, 긴 글은 `ui::wrap`·`text_center_wrapped`, UI 크기는 `Frame::ui`(480×270 기준, 0041). 새 화면은 개발 빌드 **F4**(넘침 검사 `ui::check`)를 켜고 한국어·영어로 — 콘솔 `[ui-check]`가 없어야 한다. **넘치는 목록은 굴린다(0059)**: 메뉴는 `ui.menu(…, &mut ListView, cx, top, bottom)` — 위·아래 끝만 주면 넘칠 때 보이는 줄 + ▲▼, 키 커서는 창이 따라가고(가장자리 한 줄 여유) 끌기 `Actions::drag`·휠 `Actions::wheel`·▲▼ 탭은 창을 옮긴다. 화면마다 줄 간격을 줄여 맞추지 않는다.
- 글: 플레이어가 보는 글은 `i18n/<언어>.rs`(한국어·영어) 키-필드 표에만(`Strings`, 데이터 표는 `Tr`, 숫자는 `{0}` + `fill`). 글꼴은 고운바탕(보통·굵게, 0021) **서브셋 — 표에 있는 글자만 들어 있다**: 새 글자를 쓰면 `python3 tools/fonts/subset_ui.py <원본 Regular> <원본 Bold>`(원본은 Google Fonts, 레포에 넣지 않음), 시험이 모든 글자를 본다, 표 밖 기호는 `i18n::UI_GLYPHS`. 가변 글꼴은 쓰지 않는다(fontdue가 축을 못 고름). 화면 글·메뉴는 `ui.rs`(UI 배율 `Frame::ui`, 커서 없이, 터치는 뗄 때), 싸움 HUD는 `hud.rs`. 개발 전용 화면(랩·개발 줄 F3)만 영어 기본 글꼴.
- **그림 기준은 t3ssel8r 풍 3D 도트(0061, 사장: 늘)**: 모델링할 때마다 0061 체크리스트 — **귀여움 문법이 먼저**(0036: 큰 머리·큰 눈 낮게·입 없음·둥근 실루엣, 보스도), 큰 면 몇 개의 둥근 덩어리(상자 + 면 법선은 마인크래프트 인상), 각진 것은 뿌리만(0037), 실루엣 + 큰 색 덩어리, 밝은 선이 붙을 볼록 모서리는 뿌리·핵·테에, 게임 거리의 px 크기 먼저, 판정은 게임 안 스크린숏.
- 그림·모델: 손으로 PNG·GLB를 고치지 않는다. 모델 원본은 Blender bpy 스크립트 `tools/art/blender/<이름>.py`(공용 `common.py`: 상자 부위·뼈에 붙인 강체·64×64 아틀라스·동작) → `blender -b -P` → `assets/models/<이름>.glb`(+ 미리보기 `docs/art/previews/`, 검사 `check_glb.py`). GLB는 `build.rs`가 바이너리에 넣고 `models.rs`가 이름으로 찾는다 — 없으면 상자. 부위 이름 `glow_*` = 스스로 빛남. 상태 → 동작 이름은 `render::enemy_anim`·`player`. 지형은 `scenery.rs`가 한 번 굽는다(빛은 정점 색, `render::lit`). 에셋 출처는 `assets/LICENSES.md`. AI를 쓴 곳은 숨기지 않는다.
- 주위 환경(0063)은 `backdrop.rs`(렌더의 일부)·`flora.rs`(굽는 지형에 얹는 나무·덤불·벽 뿌리 — 잎은 흔든 이십면체, 각진 것은 뿌리만, 걷는 칸에는 두지 않는다): 높이 안개는 걷는 땅 아래에만(`Scenery::walk_min`), 하늘·산등성이·달은 땅보다 먼저·법선 패스 밖, 먼 클립(200m) 안에(시험). 아지랑이는 카메라 숙임으로(`haze_by_pitch`).
- 기록(0064 M2)은 `save.rs`: 키=값 줄(`save::key` 이름은 바꾸지 않는다, 모르는 키는 지킨다), 웹 localStorage `knell.save.0`·네이티브 `save0.txt`. 앱은 판에서 옮겨(`sync_progress`) 바뀌었을 때만 쓴다(`persist`). 판을 넘어 남는 것을 새로 만들면 `Progress`에 칸 + 키 하나 + 시험. 설정(`prefs`)과 섞지 않는다.
- 능력치(0064 M3)는 `stats.rs` 한 곳: `Stats::compute(&[Mods])`(합·곱·상한), 월드는 `World::stats`만 — 최대 체력·등잔은 `Player::max_hp`·`flask_max`(상수 `PLAYER_HP`·`FLASK_CHARGES`는 기본값일 뿐), 랩 값은 `Stats::tune`이 휜 사본. 장신구는 `charms.rs` 표(기록 키는 바꾸지 않는다). 새 보정 = `Mods` 칸 + compute 한 줄 + 쓰는 곳 + 시험.
- 성장(0064 M4)은 `growth.rs`(핵 고르기 갈래·상한, 화로 가게 `SHOP` 표): 이기면 `Scene::Core`, 쉬면 `Scene::Shop`. 능력치는 장신구와 같은 `Stats::compute`로.
- 소리(0064 M5)는 `sfx.rs`(앱만 안다): 파일 표 `FILES`(`assets/sfx`, CC0 — 출처는 LICENSES) + 합성(`synth`·`ambience`, 순수 함수라 시험). 앱 `world_sounds`가 `Event`·상태 변화(회피 시작·예고 시작·깨어남)를 듣고 낸다 — 월드에 소리를 넣지 않는다. 예고 소리는 모양마다 달라야 한다(시험).
- 웹 로딩·PWA(0062): 덮개는 `web/index.html`(글 없음, wasm 바이트로 진행, 첫 프레임 뒤 `main.rs` `loaded`), 서비스 워커 `web/sw.js`는 캐시 = 빌드 아이디·`skipWaiting` 없음. `web/`에 파일을 더하면 `sw.js` `PRECACHE`와 `assemble.sh`도, 시험은 `tools/test/pwa.js`. 아이콘은 `tools/icons/make_icons.py`.

## 하지 않는 것
- PoC 통과 기준(`docs/phase-0-poc.md`)을 뒤에 고치지 않는다. 상한을 넘기면 대안으로 간다고 적어 둔 대로.
- 랩 값을 저장 파일에서 게임이 읽게 하지 않는다(진실은 `tuning.rs` 하나).
- 콘텐츠를 표·파일 밖에서 쌓지 않는다(적 = 종류 표, 스테이지 = 파일 하나, 글 = i18n 표, 모델 = GLB 하나). 마일스톤 밖의 큰 기능은 사장 지시 없이 먼저 만들지 않는다.
- 외부 JS 핫링크, 라이선스 미확인 에셋, 코드 안의 글 리터럴(플레이어가 보는 글).
- 언어: 대화·문서·커밋 한국어, 코드 식별자 영어. 모델 이름은 커밋에 쓰지 않는다.
