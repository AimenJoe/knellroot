# 0066. 앱·UI 기초 설계: 장면 스택 · 화면 = 줄 표 · 누를 자리 목록 · 창 틀 · 기록 슬롯

- 날짜: 2026-11-05
- 상태: 채택(사장: "Nail 구조를 참조해서 우리 스타일로 만들면 됨. 우리가 스터디한 내용이 있으면 그걸 기반으로 하고, 더 스터디 해야할 케이스들이 있으면 검색해보고 best practice 를 찾아. 그 과정에서 코드에 대한 리펙토리가 필요하다면 설계 및 계획 세워서 진행 하면 됨")
- 계기: v0.3.0(0064)에서 화면이 다섯 늘었고(장비·핵·가게·묻는 창·알림) 다음에 슬롯 창·도감·저널이 온다. 새 화면을 붙일 때마다 `app/mod.rs`의 큰 match 두 곳(입력·그림)에 몸통이 쌓였다. 0065는 "UI는 KB·nail 꼴로"를 정했지만 그 꼴을 담을 뼈대가 없었다.
- 대신하는 것: 0065의 "아직 다른 것" 두 줄(슬롯 하나, nail 창 틀 대신 가운데 목록), 0064 "넣지 않는 것: 저장 슬롯 여럿".

## 잰 것(2026-11-05, release/v0.3 `c491513`)
KB `concepts/solo-game-codebase-health-signals`의 신호로.

| 신호 | knellroot | 뜻 |
|---|---|---|
| `App::tick_scene` | 340줄, 11-01 뒤 8번 고침 | "자주 고치는 긴 함수는 지금 쪼갠다" |
| `app/flow.rs` 시험 | 3 | 최근 UI 고침(묻는 창·커서·소리)이 모인 곳과 같다 — "시험 0개 + 최근 버그" |
| 화면의 줄 목록 | 입력과 그림이 따로 만든다(설정·장비·가게·멈춤) | 같은 사실이 둘 — 줄을 하나 더하면 두 곳 |
| 돌아갈 곳 | `Settings{from_pause}`·`Controls{from_pause}`·`back()`, 장면이 바뀌면 커서 0 | 설정에서 멈춤으로 돌아오면 "계속"에 선다 |
| `App` 필드 | 34(nail 91) | 아직 작다 — 화면 상태를 접두어 필드로 늘리지 않을 때다 |
| `World::tick_player` | 795줄 | UI 밖. 다음 후보(R6) |

## nail에서 가져오는 것(검증된 꼴)
nail 조사(2026-11-05, 읽기만): 장면은 납작한 `enum Scene` 24개, 화면은 `app/*.rs` 자식 모듈의 `impl App`(`update_X`·`draw_X`), 터치는 그림이 채우고 입력이 읽는 `hits: Vec<(Rect, Hit)>`.
- `Scene::layer()` — `_` 없는 match 하나가 "뒤에 무엇을 그리나·어느 곡·창 닫기 때 저장"을 정한다. 새 장면을 넣으면 컴파일러가 묻는다(nail 0235 R21: `matches!` 셋에 손으로 넣다 빠뜨렸다).
- 누를 자리 목록 `Vec<(Rect, Hit)>`: 그림이 채우고 입력이 읽는다, 먼저 넣은 칸이 이긴다(▲▼는 앞에), 묻는 창은 목록을 비우고 제 단추만(nail `draw_shop_ask`).
- 닫기 ✕(그림은 작게·누르는 칸은 넉넉히) + 창 밖 탭 = 닫기, 창 안 빈 곳은 닫지 않는다(nail 0198·0239).
- 탭 창은 (이름·입력 함수·그림 함수) 표 — 새 탭 = 표 한 줄 + 함수 둘(nail 0248 R30 `PauseTab`).
- 설정 줄은 이름 enum + `label()`·`group()`(nail 0269: 번호 상수가 줄을 더할 때마다 밀렸다), 값 줄은 "이름 ◀ 값 ▶"(nail `setting_row`).
- 흐름의 결정은 순수 함수, 앱은 그대로 한다(nail 0230 R15). 메뉴 소리는 한 곳(이미 있음, 0065).
- 기록 슬롯 셋 + 마지막 슬롯 칸, 옛 열쇠 = 슬롯 1, 설정은 슬롯 밖, 두 번 눌러 덮어쓰기·지우기(nail 0078, KB `roguelite-save-slots-and-continue`).

## nail과 다르게 하는 것(까닭 = nail이 남긴 아쉬움 + 스터디)
1. **장면 스택**(pushdown automaton). nail은 납작한 enum에 돌아갈 곳을 값으로 들고 다닌다(`Settings{back}`·`Lab{from_pause}`·`after_cutscene`). 열고 닫는 메뉴는 "아래 장면을 멈춰 두고, 닫으면 아래가 이어진다"가 본질이라 스택이 맞다(Nystrom *Game Programming Patterns* State 장 "Pushdown Automata": 평범한 상태 기계는 지난 상태를 잊는다). 스택이 주는 것:
   - 돌아갈 곳 깃발이 없어진다(`from_pause` 둘 삭제). 뒤로 = 꺼내기.
   - **커서 되찾기**: 스택 칸마다 커서·굴림 창을 들고 있어 닫으면 연 줄로 돌아온다. Microsoft 패드 지침(창을 닫으면 포커스를 원래 자리로)과 지금의 "설정 → 멈춤이면 계속에 선다"를 함께 고친다.
   - 묻는 창은 위에 얹힌다 — 아래 창을 그대로 깔고 어둡게(nail 가게 묻는 창 꼴)가 저절로 된다.
2. **화면 상태는 스택 칸에**. nail은 `*_cursor` 18개·`ListView` 8개가 `App` 필드로 늘었다(91개, 0230 "묶기만의 작업은 안 한다"). knellroot는 처음부터 칸(`Entry { scene, cursor, view, armed }`)에 둬 `App`에 화면 필드가 붙지 않게 한다.
3. **목록 화면 = 줄 표 하나**. 입력과 그림이 같은 `Menu`(제목·줄·닫을 수 있나·설명)를 한 함수에서 받는다 — 줄을 더하면 한 곳. 공용 몰이꾼이 그리고(창 틀·목록·✕), 입력은 `Out`(고름 i · 좌우 i±1 · 뒤로)으로 바꿔 화면 함수 하나(`act`)에 넘긴다. 표가 맞지 않는 화면(조작 표·슬롯·타이틀·묻는 창)만 제 그림을 갖되 같은 `Hit`·`Out` 길을 쓴다. 트레이트 대신 `_` 없는 match 둘(`menu_of`·`act`) — 새 장면이면 컴파일러가 묻는다.

## 포커스 규칙(스터디: Xbox 접근성 지침 112, Microsoft "Gamepad and remote interactions")
- 키·패드만으로 모든 화면을 다룬다(커서 없이, 0019). 마우스·터치는 같은 `Out`으로.
- 창을 열면 **처음 포커스는 뜻이 있는 자리**: 묻는 창은 취소(KB·0065), 시작하기 슬롯 창은 첫 빈 슬롯, 이어하기 창은 마지막 슬롯(KB). 그 밖은 첫 줄. `flow::focus(scene, …)` 한 곳.
- 창을 닫으면 **연 자리로** 돌아온다(스택).
- 한 줄 목록은 끝에서 돌아 처음으로(지침 112 — 격자는 돌지 않는다: 조작 표).
- 포커스는 늘 보인다(밝은 면 + ▶). 글만 있는 창은 만들지 않는다(누를 것 하나는 있게 — Microsoft).
- 뒤로는 어디서나 같은 버튼, 갈 곳이 늘 있다. 예외: 골라야 끝나는 창(핵 고르기·결과)은 ✕가 없고 뒤로를 받지 않는다(까닭: 고르지 않고 나가면 보상을 잃는다).
- 같은 입력이면 들어간 곳에서 나온다(지침 112 "reversible") — 좌우가 값을 바꾸는 줄과 탭 넘기기를 한 화면에 겹치지 않는다(탭이 생기면 Q·E/LB·RB).

## 누를 자리(스터디: IMGUI hot/active — Muratori 2005, Sol IMGUI 3장)
- 지금처럼 **뗄 때** 정한다(0059). 누른 칸에서 떼야 누른 것이다(IMGUI "active인 채 hot에서 떼면 눌림") — 끌기로 바뀌면 누름이 아니다(이미 3% 문턱).
- 누를 자리는 지난 프레임 그림에서. 장면이 바뀌면 비운다(스택 칸이 바뀔 때).

## 창 틀(nail `ui::panel`·`button`·close, `wip/ui-panels`에 먼저 만든 것)
- 면 `PANEL` + 청동 테두리 `BORDER`, 제목은 왼쪽 위 호박색 + 밑줄, 고른 줄은 `PANEL_LIGHT` + ▶, 값은 오른쪽(◀ 값 ▶ · → · 값만), 흐린 줄(못 고름)은 옅게, 아래 설명 두 줄, 넘치면 굴림(0059 그대로).
- 묻는 창: 아래 창을 깔고 어둡게 + 작은 창(무엇을 잃는지 + 가로 단추 둘, 좌우·위아래 둘 다).
- 크기는 480×270 기준 px × UI 배율(0041), 화면 안으로 줄인다. 개발 F4 넘침 검사에 "창 테두리 밖"을 더한다.
- 타이틀은 창 없이 그대로(키 아트 위 목록), 조작 표도 그대로(격자).

## 기록 슬롯(KB `roguelite-save-slots-and-continue` 그대로)
- 슬롯 셋. 웹 `knell.save.0`(= 슬롯 1, 옛 한 칸 그대로 — 옮기는 코드 없음)·`knell.save.1`·`knell.save.2`, 마지막 슬롯 `knell.slot`. 네이티브 `save0.txt`·`save1.txt`·`save2.txt`·`slot.txt`. 플러그인 함수에 칸 번호 인자가 붙으니 플러그인 버전을 올린다.
- 타이틀: 시작하기(늘)·이어하기(기록 있는 슬롯이 하나라도) → 슬롯 창. "새로 하기" 줄과 그 묻는 창은 없어진다(덮어쓰기가 대신).
  - 시작하기 창: 커서는 첫 빈 슬롯, 기록 있는 슬롯은 "덮어쓰기"(두 번 눌러야).
  - 이어하기 창: 기록 있는 슬롯만(커서가 빈 슬롯을 건너뜀), 단추 이어하기 / 지우기(두 번), 마지막 기록을 지우면 타이틀로.
  - 줄마다 요약: 완주 수 · 잔불 · 장신구 수 · 마지막 쉼터.
- 설정(prefs)은 슬롯 밖. **든 무기는 prefs 칸 21에서 기록 키 `weapon`으로 옮긴다**(결정 감사: 슬롯 사이에 새지 않게). 옛 prefs 값은 기록에 무기가 없을 때의 기본으로.

## 리팩터링 계획(KB 순서: 동작을 바꾸지 않는 커밋 하나씩, 기능과 섞지 않는다)

| # | 무엇 | 동작 변화 | 확인 |
|---|---|---|---|
| R1 | 장면 스택 `app/stack.rs`(순수, 시험) + `Scene::layer()`(`_` 없는 match) + `from_pause` 삭제 + 열 때 포커스 `flow::focus` | 닫으면 연 줄로 돌아온다(의도한 고침) | 스택·layer·focus 시험, 헤드리스 여섯 |
| R2 | 누를 자리 `Vec<(Rect, Hit)>` + `Menu`/`Out` 몰이꾼 — 목록 화면의 줄 표를 한 곳에서, `tick_scene` → 장면당 한 줄 위임 | 없음 | 단위 시험(줄 표: 장면마다 줄·뒤로·좌우 줄), 헤드리스 여섯 |
| R3 | `app/` 파일 나누기(기계적: title·pause·settings·equip·shop·core·ask·result·dev + 기존 controls·flow) | 없음 | 빌드·시험 그대로, 옛 경로 grep |
| R4 | 창 틀 적용(`wip/ui-panels`의 부품을 몰이꾼이 그린다), 묻는 창은 아래 창 위 | 그림(의도) | 스크린숏, F4 넘침 검사 한국어·영어, 사장 확인 |
| R5 | 기록 슬롯 셋 + 슬롯 창 + 무기를 기록으로 | 기능(의도) | save 시험(왕복·옛 열쇠 = 슬롯 1·마지막 칸·지우기), `save.js` 다시 씀 |
| R6 | (제안, 따로) `World::tick_player` 795줄 → 상태마다 메서드 + `Ctx`(nail 0170 R9) | 없음 | 시나리오 600틱 시험 |

R6은 UI 밖이고 v0.3.0 판정(R1~R4 손맛)과 섞이면 안 되니 판정 뒤에 한다(KB 함정 "마일스톤 직전 리팩터링").

## 하지 않는 것
- 장면 트레이트·위젯 객체 트리(retained UI)·ECS 같은 다시 짜기(KB "문제는 크기와 시험 가능성"). 즉시 모드 그대로.
- `App` 필드를 화면별 접두어로 늘리기(nail의 아쉬움).

## 출처
- Robert Nystrom, *Game Programming Patterns*, State — https://gameprogrammingpatterns.com/state.html
- Xbox Accessibility Guideline 112 — https://devdocs.xbox.com/gaming/accessibility/xbox-accessibility-guidelines/112
- Microsoft, Gamepad and remote control interactions — https://learn.microsoft.com/windows/uwp/design/input/gamepad-and-remote-interactions
- Casey Muratori, Immediate-Mode GUI (2005) — https://caseymuratori.com/blog_0001 · Sol, IMGUI 3장 — https://solhsa.com/imgui/ch03.html

## 구현(2026-11-05)
- **R1** `app/stack.rs`(순수, 시험 둘) + `flow::Go`·`Layer`·`focus`. 닫으면 연 줄로 돌아오는 고침 때문에 헤드리스 길 넷을 고쳤다(`save.js` 새로 하기 취소 뒤, `shot.js`·`leak.js` "키보드만" 뒤 타이틀 커서가 "설정", `shot.js` 조작 → 멈춤 뒤, `mouse.js` 설정 → DEV).
- **R2 + R4를 한 커밋으로**: 옛 가운데 목록은 "이름 … 값"을 나눠 그릴 수 없어(글이 "언어: {0}" 한 덩어리) R2만 따로 하면 임시 글을 만들어야 했다. 그래서 줄 표 몰이꾼과 창 틀을 같이 넣었다.
  - `app/menu.rs`: `Menu`(제목·줄·폭·닫힘·지갑·제목 아래 한 줄·설명), `Out`(고름·좌우), `Hit`(줄·◀▶·▲▼·닫기·조작 칸), `hits_of`·`first_hit`(순수, 시험 — ◀▶·▲▼가 줄을 이긴다), `menu_input`(뒤로는 여기서 `flow::back`), `draw_menu`.
  - 화면 파일: `pause.rs`(멈춤·장비), `settings.rs`(설정·DEV), `shop.rs`(가게·핵·결과), `ask.rs`(묻는 창 — 아래 칸을 그 커서로 깔고, 누를 자리는 제 단추 + 창 밖만). `App::menu_of`·`act` 두 match에 `_` 없음.
  - `tick_scene` 340줄 → 목록 화면은 한 줄(`tick_menu`). 입력과 그림이 같은 `Menu`를 쓴다.
  - `ui.rs`: `panel`·`close`·`button(s)`·`list`(`Row`: 그냥·◀ 값 ▶·→, 흐림)·`title_arrows`(넘치면 ▲▼를 제목 줄에 — nail 0239, 줄 자리를 먹지 않게)·`u`, F4 검사에 `past-panel`·`forget`(묻는 창이 덮은 글은 겹침이 아니다).
  - 글: 값 줄은 이름만(`menu_language` "언어" …) + 값 글(`value_on/off`·`value_secs`·`grow_count`·`shop_item` "잔불 {0}"), 기호 ◀ → 를 `UI_GLYPHS`에.
- 덤으로 고친 것: 시작 이야기 글이 창 밑에 비쳤다(`hud` — 창 밑에서는 그리지 않는다). 창이 열린 동안 카메라 따라가기는 한 규칙("판 위의 멈춤 아래가 아니면")으로 — 결과·타이틀에서 연 조작 창에서도 카메라가 마저 자리 잡는다.
- **R3** 파일 나누기(함수 몸 그대로, 이름 목록으로 스크립트가 옮김): `tick_scene`의 남은 갈래를 `tick_title`·`tick_controls`·`tick_play`·`tick_sandbox`로, 파일 `title.rs`·`controls_screen.rs`(조작 표 화면 — 표 규칙은 `controls.rs`)·`play.rs`(싸움 틱·조준·랩·카메라·길 안내·개발 줄·월드 소리)·`progress.rs`(이어하기·기록·장신구·사기·저장). `app/mod.rs` 1,676 → 545줄, `tick_scene` 340 → 35줄. 옛 경로는 `docs/phase-1-first-stage.md` 일곱 곳을 고쳤다.
- 확인: 단위 시험 203, 헤드리스 여섯(shot·save·touch·mouse·pwa·leak), F4 넘침 검사 한국어·영어 0건, 스크린숏 `docs/art/ui-0066/`.
