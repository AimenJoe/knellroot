# 입력 · 터치 · 조준

`CLAUDE.md`에서 옮긴 규칙이다(0278) — 문장은 그대로이고 같은 힘을 가진다. 고치는 곳: `src/input/` · `src/touch/` · `src/prefs.rs` · 조작 설정 화면.
이 영역의 파일을 고치기 전에 읽고, 구조를 바꿨으면 여기를 고친다. 굵은 머리말에서 문단만 갈랐다.

## 입력은 `Actions`만 — 배치 · 마우스 · 터치 · 조준

입력은 `Actions`만. `KeyCode`·패드 버튼은 `input/` 밖에 없다(장치마다 모으기 `input/collect.rs` → 합치기 `merge` · 래치 `fold_latch`, 0278 R23 — 터치는 `touch/`, `input/`만 부른다, 0006 — 터치의 몸은 엔진 없는 `Touch::read_fingers(&[Finger], Surface, …)`). Android·iOS 패드는 패치한 miniquad(`third_party/miniquad`, `[patch.crates-io]`, nail에서 가져옴 — iOS는 0137)가 버튼을 F13~F24 키로·스틱을 `joystick_axes`로 넘긴다(게임은 `cfg(pad_keys)`로 가른다) — miniquad를 올리면 `PATCHED.md`대로 다시 넣고 `cargo test`의 `android_*`를 본다(0022). 커밋 전에 `cargo clippy --target aarch64-linux-android -- -D warnings`(NDK 없이 됨). 게임 동작의 키·버튼은 `input::Bindings`(설정에서 바꿈, 0024 — 소울라이크식, 근접 공격 X·J · 원거리 공격(마법·사격) RB·U(0255 — 0223의 "평타 RB"를 되돌림). 0151: 스킬 둘 LT·RT / Q·E(함께 = 큰 기술), 땅 돌리기는 십자 ←·→ / Z·C — 십자 키는 싸움에서 걷지 않는다(`Pad::DLeft`·`DRight`, Android·iOS는 화살표 키코드·HAT 축), 큰 창의 탭 넘기기는 패드 LB·RB(물리 — 메뉴의 십자·A·B처럼 고정)·키보드 스킬 버튼 둘 Q·E(0164 → 0175 — `Actions::tab_prev`·`tab_next`, `Act::Tabs`), 장비 칸 빨리 바꾸기는 공격 버튼(`Act::Swap` — 원거리 → 근접 공격 차례로 메뉴에서 다른 일을 안 하는 첫 버튼 `input::swap_bind` — 패드 X, 0255). 0048: 마법 RB·U, 등잔 Y·R, 상호작용은 점프 버튼이 같이 — 스테이지가 `World::interact_here`로 자리를 알린다. 0058: 새로 설치한 판의 기본은 키보드+마우스(왼쪽 클릭 공격·오른쪽 클릭 마법·휠 클릭 락온·L-Shift 패링, 왼손은 그대로 — 0151부터 Q·E는 스킬, 땅 돌리기는 Z·C), 키 칸 번호는 `KEYS` 뒤에 마우스 버튼 `MICE`, "키보드만"은 `Bindings::keyboard_only`.

**마우스는 패드의 오른쪽 스틱과 같은 일**: 쿼터뷰에서 커서를 땅에 비춘 점(`Actions::aim_at`, 앱이 `camera::screen_ray`로)이 조준, 최근 만진 커서만(`input::cursor_live`), 락온 중엔 안 쓴다, 감도 칸은 없다(OS 커서). 저장 칸 뜻을 바꾸면 prefs `MAGIC`을 올린다), 화면에 버튼 이름을 쓸 때는 `input::prompt(Act)`(지금 장치·지금 배치) — 글에 키 이름을 박지 않는다. 설정 저장은 `prefs.rs`(웹 localStorage·네이티브 `prefs.txt`, 칸을 뒤에만 붙인다). 눌림은 래치, 틱이 소비. 터치 버튼 자리를 바꾸면 `tools/test/touch.js`의 `layout()`도.

**터치의 크기·거리는 dp로**(0252 — Android의 miniquad는 dpi 배율이 늘 1이라 화면 단위가 물리 화소: 상한·탭 범위·꼬집기는 `touch::dp`(= `screen::dp()`, 패치한 miniquad의 `display_density`)를 곱하고 끈 시점 거리는 나눈다 — 새 화소 상수도 그렇게). 터치 버튼의 그림은 선이 아니라 13×13 도트 픽토그램(`touch::icon_mask` — 데이터는 `touch/icons.rs`, 흰 몸 + 어두운 한 칸 테, 칸은 정수 px, 0146 — nail 0172 꼴): 새 버튼 = 마스크 하나 + 시험의 목록 한 줄(서로 12칸 이상 달라야 한다). 공격 버튼의 그림은 든 무기(`touch::attack_icon` = 무기 줄의 `Weapon::icon` — 새 무기 = 마스크 하나 + 줄의 칸, 0147 · 0278 R18). 터치 배치는 공격 둘레 호 + 상황 버튼(`touch::Ctx` — 뜰 때만, 다른 버튼을 밀지 않는다, 0053).

**숄더뷰는 덜어낸다(0236)**: 땅 돌리기 둘은 쿼터뷰 + 락온 아닐 때만(`Ctx::hide_turn`), 스킬 칸은 끼운 스킬이 있을 때만(`Ctx::bare`) — 숄더뷰에서 버튼을 누른 손가락은 탭 범위를 넘게 끌면 그 뒤로 시점도 돈다(`touch::Rider`, 누른 일은 그대로), 엄지 묶음 크기는 설정 "터치 버튼 크기"(`prefs::TOUCH_SIZES`, `touch::set_size` — 자리도 같은 배로 모서리 쪽, 위 가장자리 줄은 그대로, 시험이 단계마다 겹침·최소 크기를 본다).

**터치 끌어 조준(0217 — 브롤스타즈 꼴)**: 겨눌 수 있는 일이 올라간 버튼(원거리 늘 · 공격은 수류탄 무기일 때 · 스킬 칸은 끼운 꼴이 쪽을 가질 때 `Form::aims` — 앱이 `set_touch_ctx`로 매 프레임 `touch::Ctx`에)은 짚으면 겨누고 **떼는 순간** 그 일(`touch::aim_end`: 톡 = `Actions::auto_aim` 가까운 적 · 끈 쪽 = `aim` + `aim_drag`(락온보다 앞선다) · 가운데로 되돌리면 취소), 원거리를 쥔 채면 첫 발 없이 모은다(`aim_hold`).

**숄더뷰: 한 대상은 크로스헤어, 범위는 바닥 표시(0229 → 0230)**: `touch::Ctx::looks(버튼)`(숄더뷰 `look_aim` + 쇠뇌의 평타·근접 무기의 특수·한 대상 스킬 `skills::Form::sighted` — 던지기·제비걸음, 앱이 `sight_skills`로)이면 끈 만큼 `Actions::look`(오른쪽 반 끌기와 같은 셈), 떼면 크로스헤어로(원거리는 `aim_sight`·`aim_ahead`, 한 대상 스킬은 월드 `Ctx::sight` — 어느 장치나), 취소 자리·땅 위 미리보기 없음. 수류탄·범위 스킬(길·줄·부채꼴)은 숄더뷰에서도 0217의 땅 위 끌기 — 락온·쿼터뷰는 0217 그대로. 버튼 둘레 그림 `touch::draw_aim`, 땅 위 미리보기 `render::overlays::aim_preview`(`View::aim_show` + `World::drag_aim` — 버튼이 아니라 그 버튼의 지금 일로 고른다 `touch_aim_show`, 0227) — 겨눌 일이 새로 생기면 `Ctx::aims`의 갈래 + 미리보기 꼴 한 줄.

**패드 조준 보조·숄더뷰 조준(0219)**: 앱이 틱마다 `Actions::aim_assist`(패드 = `PAD_AIM_ASSIST_DEG` 8°, 그 밖 0)·`aim_ahead`(자유 숄더뷰)를 채운다 — 원거리 쪽은 손이 겨눈 것(끌기·스틱·커서) > 숄더뷰의 카메라 앞, 패드면 그 쪽 ±8° 안의 적에 붙인다(`World::assist_aim`, 스틱을 안 기울이면 없음 — 0213), 근접 자동 조준(`aim`)에는 섞지 않는다. 겨누는 쪽은 `World::aim_line`, 무엇을 그릴지는 `App::aim_show`(쿼터뷰에서 조준할 때만 — 쏘는·모으는 동안·패드 스틱·터치 끌기 — 연한 띠, 마우스 커서는 없음).

**숄더뷰는 바닥에 그리지 않고 화면 가운데 크로스헤어(0222)**: `hud::draw_crosshair`, 원거리는 가운데 시선이 지나는 적(`App::crosshair_target` → `aim_sight` — 원거리의 쪽만, 몸·근접은 모른다: `aim_at`은 커서의 칸이라 몸을 돌린다, 0227)으로.

**조준 보정은 탄이 닿는 적에게만(0228)**: 패드 보조·크로스헤어·마법 탄과 터치 톡의 찾기(`nearest_shootable`)는 `World::shot_clear`(그 쪽으로 쏘면 멈추는 거리 `shot_reach`가 몸에 닿나)로 거른다 — 벽 너머의 적에게 붙지 않는다, 새 보정도 이것을 지난다, 없으면 카메라 앞으로(0221의 늘 떠 있는 바닥 헤어라인은 걷음). 곁에 할 일이 있을 때의 점프 버튼은 **그 일의 그림**을 쓴다(0166 — 쉬기·줍기·열기·당기기·말 걸기·읽기: 무엇이 곁에 있는지는 `StageRun::near()` 한 곳이 말하고 HUD의 글과 터치의 그림(`touch::Near`·`near_icon`)이 같이 읽는다 — 새 할 일 = `stage::Near` 한 갈래 + 차례 한 줄 + 그림 마스크 하나 + 글).
