# 0278 — 리팩터링 점검 셋째: 추상화 수준 판정과 차례(R12~R25)

- 날짜: 2026-11-26
- 상태: **정함**(사장 2026-11-26: "0.9.0에 0-2 단계 하고, 이후 다시 측정해서 3~4 진행하자. 그 후 완료된 구조로 claude.md 업데이트 해줘") — 0~2단계(R12~R19)를 v0.9.0에, 끝나면 다시 재서 3~4단계(R20~R25), 마지막에 끝난 구조로 `CLAUDE.md`를 고친다. R 하나씩 커밋한다
- 앞선 점검: 0159(R1~R4) · 0251(R5~R11). 재는 법과 추상화 기준은 KB `concepts/solo-game-codebase-health-signals.md`, 첫 게임 0170
- 잰 곳: `release/v0.9` `8cba4df`. 수치는 `wc`·`grep`·짧은 파이썬(함수 길이 · `git log -L<줄 범위>`)으로 잰 것, 구조 읽기는 읽기 전용 조사 셋(스킬 · 무기/종류 · 앱/UI/입력) — 조사가 말한 것 가운데 아래 "확인함"은 코드를 다시 열어 본 것

## 1. 잰 것

| 신호 | 0251(11-19) | 지금(11-26) | 판정 |
|---|---|---|---|
| Rust 줄(시험 포함) · 시험 | 89,916 · 759 | **122,005 · 1,018**(제품 84,437 · 시험 37,724) | 일주일 +36%, 시험은 따라왔다 |
| 200줄 넘는 함수(제품) | 22 | **40**(150 넘는 것 57) | 거의 두 배 |
| 가장 긴 로직 함수 | `tick_skill` 438 | `fx::on_skill_event` **1,019** · `tick_skill` **856** · `fx::on_skill` **784** · `enemy::tick` 479 · `input::poll` 445 | R11에서 나눈 곳이 다시 자랐다 |
| 구조체 칸 | App 59 · Enemy 52 · StageRun 60 | App **68** · Enemy 58 · StageRun 62 · World **74** · Player 48 · Actions 53 | 묶은 뒤 다시 늘었다 |
| 새 무기 종류 하나에 고친 `src` 파일 | — | **40**(완드 `0c00bac`, +2,120줄) | "표 한 줄" 규칙과 멀다 |
| 로직 → 위층 참조 | 0 | 2(`Actions` · 카메라의 순수 함수 하나) | 좋음 |
| 월드 틱의 "이 무기면" 직접 비교 | — | 4(`keen` · `swinging_greatsword` · 덫 둘) | 좋음 — 한 곳에 묻기가 지켜진다 |
| 앱 → 월드 칸에 쓰기 | 20 | 24(잔불 10 — 여덟 파일에서) | 지켜볼 것 |
| TODO/FIXME · `allow(dead_code)` | — | 0 · 1 | 좋음 |
| `CLAUDE.md` | — | **120 KB** | 세션·에이전트마다 통째로 읽는다 |

**길면서 자주 고친 함수**(11-19 뒤 그 줄 범위를 고친 커밋 수):
`app/play.rs step_world` 234줄 · **38** / `world_sounds` 265 · **34** / `app/dev.rs dev_start` 257 · 30 / `World::new` 205 · 26 /
`fx::on_skill_event` · `on_skill` · `strike_lands` 268 · 각 22 / `tick_skill` 20 / `fly_bolts` 207 · 18 / `skill_shape` 305 · 16 / `at_grown` 296 · 14.

## 2. 추상화 판정(0170의 셋 — 깊이 · 방향 · 셋의 규칙)

**한 줄**: 경계와 방향은 좋다. 약한 곳은 **"한 개념이 여러 파일의 match에 나란히 적혀 있고, 그 가운데 일부가 `_ =>`로 끝나서 빠뜨려도 조용하다"**는 것이다.
크기(긴 함수)는 그 결과다.

### 2.1 잘된 것(지킬 것)
- 방향: 월드는 화면·소리·메뉴를 모른다(위층 참조 2). 소리·이펙트는 사건으로.
- 한 곳에 묻기: `World::bare()` · `knack()` · `rb_toss()` · `rb_bolts()` · `fury()` · `next_hit()` · `hit_of()` · `skill_eff()` — 깊은 메서드.
- `Id` · `Class` · `Hand` · `Grip` · `Form`(틱 · 치수 · 미리보기) · `Scene`(탭)의 match는 와일드카드가 없다 — 새 갈래를 컴파일러가 짚는다.
- 순수 규칙 모듈과 시험(`flow` · `anim` · `skills::Board` · `status` · `plates` …), 지도 불변식 시험, 글 넘침 시험(0275).

### 2.2 약한 것

| # | 신호 | 근거 | 무엇이 문제인가 |
|---|---|---|---|
| A | **조용한 와일드카드** | `skill_active`(`world/skill_tick.rs:2087` `_ => 0.0`) · 흔들림 예외 목록(`app/play.rs:671`) · `aims`/`sighted`(`!matches!` — 새 모양은 저절로 "겨눈다") · `anim::clip_period` · `fx::on_skill_start` · `rb_name`(`app/equip.rs:357`) · `toss_icon`(`touch.rs:1205`) · `Rhythm` 셋 · `Bare`(`world.rs:1689`) | **실제 버그(확인함)**: 시드는 그늘(`Blight` — 바닥에 남는 스킬)이 `skill_active`의 이름 목록에 없어 HUD·터치 칸의 남은 시간이 늘 0. 흔들림 예외에도 화약 덫 · 시드는 그늘이 빠졌다. 시험 하나(`touch_aim.rs:212`)는 같은 목록을 베껴 써서 깨질 수 없다 |
| B | **월드 사건의 갈래 넷** | 같은 `Event`를 소리(`world_sounds`) · 이펙트(`fx::on_*` 넷) · 카메라(흔들림 · 타격 줌 · 당김 — `step_world`) · 튜토리얼이 따로 match. 소리는 `_ => continue`, 줌 · 당김도 `_`. 소리 없는 사건 13종 · 이펙트 없는 사건 13종 — 일부러인지 빠뜨린 것인지 코드가 말하지 않는다 | 가장 자주 고친 두 함수(38 · 34)가 여기다. 0277에서 사건 셋을 더할 때 네 곳을 손으로 맞췄다 |
| C | **모양의 성질이 여덟 곳에** | "둥근가/겨누나 · 한 대상인가 · 무엇이 커지나 · 동작의 주기 · 바닥에 남나 · 움직임"이 `skills.rs:205/220/931` · `skill_tick.rs:312/2072` · `anim.rs:366` · `app/skills.rs:290` · `player_tick.rs:1525`에 따로 | 빠진 추상화. 새 모양 = 여덟 곳, 그 가운데 넷은 조용히 기본값 |
| D | **타입 없는 타입표** | 탄의 꼴 `Bolt::look: u8` — 마법 수 `look == 3`(`bolt_tick.rs:593` · `weapons.rs:419`, 확인함) · 약 35곳(월드 · 렌더 · fx · 터치 · 앱) · 와일드카드 둘 | 새 탄의 꼴이 틀린 이름·그림을 조용히 받는다 |
| E | **플래그로 구부러진 구조체** | 놓인 것 `Field`에 한 스킬만 쓰는 칸 여섯(`follow` · `kindle` · `drift` · `trap` · `iron` · `spire`), `Burst.toll`/`on`(종 치기만), `Toss.charges`/`refill`(완드만 — 상수 아홉에 찍었다), `Art`(종류마다 칸 하나 — 합 타입을 곱 타입으로), bool 둘 이상 받는 함수 12 | 잘못된 추상화의 첫 신호(모드 인자). 셋을 넘었다 |
| F | **`tick_skill` 안의 복사** | 같은 `skill_area(Sweep{…})` 블록 9번 · 휘청 맺음 4번 · 회복 블록 2번 — 쿨타임 돌려주기와 표지 걸기는 **복사본끼리 이미 달라졌다**(한쪽만 `refunded`를 보고, 한쪽은 덮어쓰고 한쪽은 `max`) | 어느 쪽이 규칙인지 시험이 말하지 않는다 |
| G | **표에서 나올 수 있는 손 목록** | 종류 하나에 고친 40 파일 가운데 12가 이것: 무기 이름·설명·도감 글의 `Id` → 글 match 셋, `item_view` 줄, 터치 그림 match, 개발 창의 종류별 상자 변종 일곱 자리 · `CYCLE`, `Class::index`, 손으로 지키는 `ONE`/`TWO` 비트, `charms::TABLE`에 새 `Mods` 칸을 열 번(`toss_dmg: 0` ×10, 확인함), `garb::MODS0`(= `Mods::NONE`의 복사), 종류마다 복사한 기록 왕복 시험 70줄 | 얕은 반복. 표의 줄에 칸을 두면 사라진다 |
| H | **그리기 안의 규칙** | 장비 탭의 상태 글·표지(`equip.rs:1026~1102`), 가방 칸의 상태 차례(`bag.rs:598`), 스킬 트리의 열린 줄(`skills.rs:1846` — **그리는 도중에 `self.skill_rows`를 바꾼다**), HUD의 쇠뇌 장전 셈(`hud.rs:1016` — 월드의 규칙을 다시 센다) | 시험이 붙을 수 없다(앱을 띄워야 한다). 진실이 둘 |
| I | **얕은 곳** | 잔불 `world.embers`를 앱 여덟 파일이 직접 고친다 · `App`의 탭 UI 칸 열하나(화면 상태는 스택 칸에 두기로 한 0066과 어긋남) · 종류의 상태가 `Player`에 납작하게(`cleaved` · `beat.fury` · `off.*` · `ranged.topup`) · 월드의 틱 깃발 열 | 묶음이 없다 — 0251 R10의 다음 바퀴 |
| J | **헤드리스 시험이 누른 횟수에 기댄다** | 스크립트 21개 3,449줄 · 세어 누르기 63곳(개발 창 길 38 — `fx.js` 25) · 스크립트마다 서버·띄우기 코드 복사 19번 · 프레임을 안 기다리는 무기 바꾸기 고리 일곱 | 개발 창에 줄 하나를 더하면 조용히 깨진다(0151 · 2026-11-19에 겪음) |

### 2.3 추상화하지 **않을** 곳(조사 셋이 같이 낸 결론)
- **종류마다의 훅을 트레이트로 묶지 않는다.** 이번 주의 종류 넷은 표의 줄만 더한 것이 아니라 저마다 새 훅 자리를 냈다(급소 · 쪼개기/분노/`next_hit` · 끌기/흡수 · 속성 싣기/`after_deaths`). `ClassHooks` 트레이트였다면 종류마다 메서드 셋이 늘고 그 메서드는 한 종류만 쓴다 — 셋의 규칙에 못 미친다. **표 + 그 동사의 자리에서 한 곳에 묻기**를 지킨다.
- **스킬마다의 이펙트·소리**는 정체성이다(서명이 서로 달라야 한다는 시험이 있다). 스킬별 match로 두고 `_ =>`만 없앤다.
- **큰 창의 탭**: 일곱 곳의 match가 나란하지만 와일드카드가 없어 컴파일러가 지킨다. 탭 트레이트는 이 보장을 동적 호출과 바꾼다 — 두지 않는다.
- **모양(`Form`)의 동작**(틱 · 미리보기)은 트레이트 객체가 아니라 **빠짐없는 match → 갈래마다 이름 있는 메서드**. 늘어나는 축은 모양·스킬이고(이번 마일스톤에 `Form` 글이 든 커밋 16), 동작의 수(열쯤)는 그대로다 — 갈래를 한곳에 모으되 enum의 빠짐 검사는 잃지 않는다.

## 3. 설계 — 원칙 넷

1. **빠뜨리면 컴파일이 깨지게**: 내용 enum(`Form` · `skills::Id` · `Ult` · `Event` · 탄의 꼴 · `Bare` · `Rhythm`)을 보는 제품 코드에 `_ =>`를 두지 않는다. 일부러 아무것도 안 하는 갈래는 이름을 적는다.
2. **성질은 서술자 한 줄로**: 갈래마다의 "값"(둥근가 · 이름 · 그림 · 소리 · 흔들림)은 `const fn …::info()`/표의 줄 한 곳 — 여러 파일의 작은 match를 없앤다(`Form::info()` · `Look` + `LookDef` · `Weapon` 줄의 글·그림 · `Event` → `Cue`).
3. **동작은 빠짐없는 match + 갈래마다 메서드**: 긴 match의 몸을 `Ctx` 하나를 받는 이름 있는 메서드로(0159 R1 · 첫 게임 0169 R5의 꼴). 공용 조각(`Sweep::of` · `cap_stagger`)은 셋 넘게 나란한 것만.
4. **규칙은 그리기 밖에**: 그리기가 계산하던 것은 기록·월드만 받는 순수한 뷰 구조체로(`EquipView` · `BagCellView` · `SkillRowsView`) — 시험이 붙는다. 잔불은 지갑 메서드로.

## 4. 차례(R 하나 = 동작을 바꾸지 않는 커밋 하나 — 기능 작업과 섞지 않는다. 버그 고침은 따로 적는다)

크기: S = 에이전트 한 번(어림 30분~1시간) · M = 1~2시간 · 어림은 이번 주 작업의 실제 시간에서.

### 0단계 — 안전망(먼저)
| R | 무엇 | 지킴 |
|---|---|---|
| R12 | **골든 트레이스 시험**: 스킬 34 · 궁극기 11 · 종류의 RB를 고정 마당에서 한 번씩 쓰고 틱마다의 사건 · 적 체력 · 자리를 해시 — 값은 파일에. **사건 덮개 시험**: 모든 `Event`가 소리 · 이펙트에서 "다룬다" 아니면 "일부러 없음" 목록에 있다 | 뒤의 R15~R19가 이 해시로 "그대로"를 증명한다(3D 스크린숏 비교는 잡음이 커서 쓰지 않는다 — KB, 0159에서 잰 것) |
| R13 | **헤드리스 길을 이름으로**(J): 웹 부팅 깃발(`?boot=<개발 창 줄 이름>` — PC의 `KNELL_BOOT`를 웹에도) · `sceneKit.openTab(이름)` · `devRow(이름)` · 서버·띄우기 한 곳. 세어 누르기 63 → 0 | CI 넷이 그대로 통과 · 찍힌 메뉴 화면이 앞뒤로 같다 |

### 1단계 — 조용한 빠짐 막기(작고 값이 크다)
| R | 무엇 | 크기 |
|---|---|---|
| R14 | **와일드카드 걷기**(A): 2.2 A의 자리들을 빠짐없는 match로. **버그 고침 둘**(동작이 바뀐다): 시드는 그늘의 남은 시간 표시 · 화약 덫/시드는 그늘의 시동 흔들림. 베껴 쓴 시험을 "놓는 스킬은 쓴 뒤 `skill_active > 0`"으로 | S |
| R15 | **`Event` → `Cue`**(B): 순수한 `cue.rs` — 사건마다 (소리 · 흔들림 · 타격 줌 · 당김)을 와일드카드 없이. `world_sounds` 265줄과 `step_world`의 카메라 match 셋이 표를 읽는다. 스킬마다의 소리는 `Cue` 안의 스킬별 match로 그대로. 이펙트 · 튜토리얼은 따로 둔다 | M |

### 2단계 — 서술자(빠진 추상화 메우기)
| R | 무엇 | 크기 |
|---|---|---|
| R16 | **`Form::info()`**(C): 모양의 성질 여덟 곳 → 서술자 하나. `aims`·`sighted`·`active`·`clip_period`·치수 이름·`skill_dir`의 "바라보는 쪽"이 그것을 읽는다 | S~M |
| R17 | **탄의 꼴 `Look` enum + `LookDef`**(D): 이름 · 그림 · 빛 · 곧게/내리꽂기/돌아옴/튕김. `look == 3` 같은 수와 35곳의 비교가 메서드로 | M |
| R18 | **무기 줄에 글·그림**(G): `Weapon`에 `name`·`desc`·`codex`(`Tr`)·터치 그림 — `Id` → 글 match 셋을 지운다. `charms::TABLE`은 `..Mods::NONE`, `garb::MODS0` 지움, `Toss`는 `..Toss::BASE` | S |
| R19 | **표에서 나오는 목록**(G): `Class::index` = 표의 자리 · `ONE`/`TWO` = 표의 `hand`에서 · `Id::class`는 무기 줄의 칸 · `DevStart::Chest(Id)` 하나 · `CYCLE`/`hold_for`는 `IDS`에서 · 기록 왕복 시험은 표를 도는 고리 하나 · `keen`/휘청 덜기는 종류 표의 칸 | M |

### 3단계 — 깊이(긴 함수와 얕은 구조체)
| R | 무엇 | 크기 |
|---|---|---|
| R20 | **`tick_skill` 갈래 → 메서드**(F): `SkillCx` + `Sweep::of` · `cap_stagger`. 달라진 복사본 둘(쿨타임 돌려주기 · 표지 걸기)은 **먼저 결정 문서에서 규칙을 찾아 시험으로 고정**한 뒤 하나로 — 규칙이 바뀌면 그 줄은 버그 고침으로 따로 | M |
| R21 | **`Field` 플래그 → `FieldKind`**(E) · `Burst`의 종 치기 칸 · `Art`를 합 타입으로 | M |
| R22 | **뷰 모델**(H): `EquipView` · `BagCellView` · `SkillRowsView`(그리기는 받기만 — 그리는 중의 상태 바꿈을 없앤다) · HUD의 장전 셈은 월드의 메서드로 · 시험 | M |
| R23 | **입력 나누기**: `input::poll` → 장치마다 모으기 + 합치기(터치의 공격/사격 맞바꿈은 합치기 앞) · `touch::read(&[Touch], w, h)`로 엔진 없이 시험 | M |
| R24 | **묶기**(I): 잔불 지갑(`World::earn`/`spend`) · `App`의 탭 UI 칸 → `TabUi` 또는 스택 칸 · `Player`의 종류 상태(`ClassState` · `BareState` · `ShieldState`) · 월드의 틱 깃발/결의 틱/놓인 것 — 쓰는 곳은 컴파일러 진단으로(0251 R10의 법) | M |

### 4단계 — 기계적 나누기(마지막 — 앞 단계가 줄여 놓은 뒤에)
| R | 무엇 | 크기 |
|---|---|---|
| R25 | `skills.rs` 3,700 → `skills/{form,table,ranks,ult,entries,board}.rs`(`pub use`) · `fx::on_skill`의 스킬별 갈래 → `fx/skills.rs` · `hud::draw` 370 → 구역마다 · `touch.rs`의 그림 데이터 1,328줄 → `touch/icons.rs` | M |

### 따로 — 문서
- **`CLAUDE.md` 120 KB**: 세션과 하위 에이전트가 시작할 때마다 다 읽는다. 뼈대(먼저 읽을 것 · 갈래 · 하지 않는 것 · 규칙의 머리말) + `docs/guide/<영역>.md`(입력 · 월드 · 스킬 · 무기 · UI · 그림 · 시험 — 그 영역을 만질 때 읽는다)로 나누는 안. 규칙의 글은 그대로 옮긴다. **사장 확인 뒤에**(지시 파일이라 먼저 묻는다).

## 5. 끝난 기준

| 신호 | 지금 | 목표 |
|---|---|---|
| 내용 enum의 조용한 와일드카드(제품 코드) | 15쯤 | 0 |
| 새 종류에 고치는 `src` 파일 | 40 | 28 아래(표에서 나오는 12가 2~3으로) — R19 뒤: 손으로 지키는 목록 12 → 1(+ 옛 눈금 표 한 줄), 완드 커밋을 지금 하면 31쯤(나머지는 내용) |
| 새 모양(`Form`)에 손으로 찾아야 하는 자리 | 8(넷은 조용) | 서술자 1 + 컴파일러가 짚는 match |
| 200줄 넘는 함수 | 40 | 30 아래(그림 데이터 · 글 표 빼고) |
| 헤드리스의 세어 누르기 | 63 | 0 |
| 그리기 안 규칙의 시험 | 0 | 뷰 셋에 시험 |
| 동작 | — | R12의 해시 · 시험 이름 목록 · clippy 다섯 · CI 넷이 단계마다 그대로 |

## 6. 사장이 정할 것
1. **시점**: (안) R14의 버그 고침 둘은 지금 v0.9에, 나머지는 **v0.9.0 배포 뒤 다음 마일스톤 갈래의 첫 일로** — 사장이 판정하는 동안 개발판의 동작이 발밑에서 바뀌지 않게. 지금 바로 시작할 수도 있다(동작은 안 바뀌지만 범위가 넓다).
2. **범위**: 0~2단계(안전망 · 조용한 빠짐 · 서술자)까지만 먼저 하고 3~4단계는 다음 점검에서 다시 재도 된다.
3. **`CLAUDE.md` 나누기**를 할지.

## 7. 한 것
- **R12**(안전망): `world/tests/golden.rs` + `golden.txt` — 골든 트레이스 102 경우(스킬 34 × 고르기 ㄱ/ㄴ 최대 레벨 · 궁극기 11 최대 레벨,
  그 종류의 무기 · 무기 11의 RB 톡/쥐었다 떼기(1.5초) · 방패 던지기). 같은 고정 마당(30 m · 보스 하나 + 잡몹 다섯, 처음엔 서 있다)에서 6초 —
  틱마다의 사건(Debug 글의 수를 1e-3으로 반올림)과 적 체력 · 자세 · 자리, 방랑자 자리 · 체력을 FNV-1a 64로 접는다. 다르면 경우 이름을
  말하고, 경우마다 그 일이 정말 나갔는지(시동 · 궁극기 · RB의 `Cast`)도 본다. 일부러 바꿨으면 `GOLDEN_WRITE=1 cargo test golden_traces`.
  결정적임: 세 번 돌려 같다(월드의 이 길에 씨앗 없는 난수가 없다 — 잔불 흩기의 씨앗은 월드 시각). 무는지: 시드는 그늘의 피해를 1 올리자
  `skill.Blight.A`·`.B` 둘만 깨졌다(되돌림). 고르기가 이 마당에서 갈리지 않는 일곱(되울림 · 버팀 · 재기 · 저주 새기기 · 방패 돌진 · 자루 치기 ·
  밀치기 — 패링 · 맞음 · 쓰러뜨림 같은 때가 있어야 갈린다)은 ㄱ·ㄴ의 해시가 같다.
  사건 덮개: `Event::ordinal`(와일드카드 없는 match — 새 갈래는 컴파일이 깨진다) · `Event::samples`(갈래마다 표본 하나, 86). 소리는
  `world_sounds`의 갈래를 순수한 `sound_of`로 떼어(사슬 끊김의 글 한 줄만 앞에서 — 동작 그대로) "소리가 나거나 `SILENT` 15", 이펙트는
  "묶음 하나가 맡거나 `NO_FX` 16" — 목록의 것은 정말 없는지도 본다. 소리 없음: 락온 셋 · 땅 돌리기 · 벽 쿵 · 적의 공격이 풀림 · 깨어남 ·
  피가 샘 · 흡수 · 무기 바꾸기 · 다 모임 · 수류탄 던짐 · 뿌리 하나 · 화상이 탐 · 저주의 표. 이펙트 없음: 휘두름 · 락온 셋 · 땅 돌리기 ·
  수레 둘 · 지연 공격 · 잔불 · 무기 바꾸기 · 안 됨 넷 · 수류탄 던짐 · 박자 타.
- **R14**(조용한 와일드카드 걷기 — **버그 고침 둘**): 내용 enum의 와일드카드 18 → 0 — `World::skill_active` · `Form::aims`/`sighted`(`!matches!`
  → 빠짐없는 match) · `anim::clip_period` · `fx::on_skill_start` · `fx::on_skill`(스킬 34의 `(id, 모양)` 튜플 → id마다 한 갈래, 모양은 갈래 안에서
  풀어 쓴다) · `Eff::active` · 스킬 걸음의 속도(`player_tick`) · 저주가 옮는 둘레 · `Rhythm` 넷(`player_tick` · `bolt_tick` 둘 · `hud`) ·
  `World::hit_of`의 `Bare` · 개발 창 상자 줄(`DevStart::chest`). 새 서술 `Form::leaves_field`(불티 · 눈발/그늘 · 덫)를 HUD 칸과 흔들림이 같이 읽는다.
  고친 것: 시드는 그늘의 칸 남은 몫이 늘 0이었다(이름 목록에 없었다) · 화약 덫과 시드는 그늘을 놓는 순간 화면이 흔들렸다(같은 까닭 —
  `skill_kick`). 시험 셋: `every_skill_that_leaves_a_field_reports_its_remaining_share`(옛 `skill_active`로 돌리자 "Blight placed something but
  reports 0"로 깨졌다) · `placing_skills_do_not_shake_the_screen` · `every_toss_look_has_a_name_and_an_icon`(탄의 꼴은 아직 `u8` — `rb_name_of` ·
  `toss_icon_of`의 `None`을 종류의 RB · 돌 · 방패의 꼴로 지킨다, R17에서 enum). 베껴 쓴 시험(`touch_aim.rs` `only_skills_with_a_direction_take_the_drag`)은
  월드에 묻는 것으로 — 바라보는 쪽(-z)과 다른 쪽으로 끌어 쓰면 `aims`인 모양만 끈 쪽으로 시작한다. 골든 해시 102는 그대로.
  남긴 것: `Event`의 와일드카드 열 — 카메라 킥 · 타격 줌(R15 `Cue`가 대신), 무기 바꾸기 찾기 · 튜토리얼의 셈 둘(거르는 것), `sound_of`의 끝(R15),
  `fx`의 묶음 넷(묶음마다 "내 사건이면 참" — 설계대로, 덮개 시험이 지킨다). 탄의 꼴 `u8` 셋(`overlays::bolts`의 던지는 것 그림 · 이름 · 그림의
  `None` — R17). 개발 창의 줄 이름 · 설명의 기본 글 둘, `Entry`(스킬 트리 칸)의 match 다섯 — 내용 enum 목록 밖.
  시험 1,018 → 1,026(무시 20 포함).
- **R13**(헤드리스 길을 이름으로): 개발 창의 줄 · 든 무기 · 큰 창의 탭을 세어 누르지 않는다.
  - **개발 명령**(`app/dev.rs` `DevCmd` — 웹 개발판만): 페이지의 `window.knellDev`(이름 배열)를 `App::frame`이 프레임마다 앞에서 꺼내
    차례로 한다(`screen::dev_command` ↔ `web/index.html` `knell_dev_next`). 이름은 모두 있는 표의 Debug — 개발 창 줄 `DevStart`
    (`Gate` · `Scenario(Line)` — 그 줄을 고른 것과 같다, PC `KNELL_BOOT`와 같은 이름) · `Dev(<줄>)`(개발 창을 그 줄에 커서를 두고 연다 —
    설명을 찍는 스크립트용) · `Arm(<무기>)` · `Arm(Sword+Shield)`(`ARM_CYCLE` — 옛 `CYCLE`을 모듈로) · `ArmSub(<무기>|None)`(`sub_ids`).
    모르는 이름은 콘솔 에러 `knellDev: …`(스크립트의 `dev`가 던진다). `KNELL_BOOT`도 같은 해석(쉼표로 여럿). 운영판(`--no-default-features`)
    wasm에는 `knell_dev_next` 가져오기 · `knellDev` 글이 없다(grep 0 — 개발판 2). 시험 `every_dev_command_name_round_trips`(이름 → 명령 →
    이름, 겹침 없음, 스크립트가 쓰는 이름 33개를 박아 둔다 — 줄 · 무기 이름을 바꾸면 스크립트가 아니라 이 시험이 깨진다). 개발 창 ·
    모션 보기는 고른 줄을 장면 이름 끝에 붙인다(`Dev { tab: 3 } @Gate` · `Motions { … } @attack`, 개발판 — `RowFocus`).
    옛 시험의 줄 자리 고정(전투 첫 줄 = 보스 아레나 …)은 걷었다 — 줄의 자리는 자유다.
  - **`tools/test/scene.js`**: `dev(…이름)`(쌓고 다 꺼낼 때까지 + 프레임 둘) · `boot(page, …이름)`(goto 전 — 첫 프레임에) ·
    `openTab(장면 이름, 무엇, 키)`(Q/E로 넘기며 장면 이름을 기다린다 — 한 바퀴에 안 오면 지금 장면을 말하며 던진다) · `devRow(줄 이름)`
    (개발 창을 메뉴로 지나는 길 — →로 묶음의 첫 줄들을 한 바퀴 본 뒤, 없으면 묶음마다 ↓로 훑는다) · `keyboardOnly()` · `serve(dist, {port, drip})` · `launch(args)` ·
    `watch(page, errors)`. 스크립트 열아홉이 서버 · 띄우기를 `serve` · `launch`로(베낀 `http.createServer` 19 → 0, 3,449줄 → 3,225줄).
    개발 창을 메뉴로 지나는 길은 `shot.js` 하나에 남겼다(`devRow('Gate')` · `devRow('Motion(0)')` · `devRow('Scenario(Boss)')` — 창 자체를 지킨다,
    그리고 `mouse.js`의 놀이터 Esc = 개발 창). 나머지(`fx` · `swap` · `leak` · `mouse` · `stress`)는 `dev`로.
  - **세어 누르기**(`press('<길 키>', N≥2)` · 고정 횟수 고리 · `open(묶음, 줄)`): 개발 창 · 무기 길 43 → 0(`grep`: `until('Dev { tab: N' …
    press('Arrow…', N)` 23 · "든/보조무기 바꾸기" Enter 고리 11(프레임을 안 기다리던 150 ms 고리 — `fx` 9 · `swap` 2) · `open(n, n)`/`pick(p, 4 …)`
    8 · 보조무기 비우기 줄 설명으로 ↓2 1). 큰 창의 탭 넘기기 Q/E × N 8 → `openTab`. 남긴 세어 누르기 41(넓은 grep 118 → 41)은 모두 개발 창 밖의 짧은 목록이고 장면을
    기다리거나 화면을 찍는다: 타이틀 → 설정(↓2 · 언어 스크립트 넷) · 조작 설정의 "키보드만 배치로"(↑2 — `keyboardOnly` 한 곳 + `view`/`shot`가
    부른다) · 설정 탭 끝 줄(`shot` ↑4/↓3 · `save` ↑2) · 큰 창 탭 안의 칸 커서(`hearth` · `items` · `shield`의 장비 칸 ↓2~5 — 장면 이름에 칸이 없다) ·
    연구소 창(F1 — `shot` →4 · ↓2) · 카메라 거리(`view` ↑2 · `fx` ↑4) · 기술 트리 칸 찾기(`fx` `wear` — 장면 이름 `@Skill(…)`을 보며 훑는다).
  - **잡은 것**(세던 길이 조용히 엉뚱한 화면을 찍고 있었다): ① `fx.js horde`는 전투 묶음 아홉째 줄을 눌러 **이끼 덮인 문지기 아레나**를
    찍고 있었다(줄이 늘어 떼가 열한째로 밀렸다) — 이제 `Scenario(Horde)`. ② `stress.js`는 타이틀에서 숫자 키 6·7을 눌러 `s40.png`·`s80.png`에
    **타이틀**을 찍고 있었다(숫자 키는 놀이터에서만 듣는다) — 이제 `Scenario(Stress40/80)`. ③ `fx.js`의 무기 바꾸기 고리 · `swap.js`의 보조무기
    고리는 프레임을 안 기다렸다(150 ms) — 이번 실행에서는 우연히 맞았다(찍힌 설명 칸의 무기 이름이 앞뒤로 같다).
  - **"그대로" 증명**: CI 넷(`shot` · `touch` · `save` · `pwa` + `away`)을 바꾸기 전 두 번 · 뒤 세 번(`shot`) — 모두 종료 0. 바꾸기 전 두 번 사이에
    바이트가 같았던 화면 10장(타이틀 · 슬롯 창 · 깨어남의 어둠 · 회전 뒤 · 로딩 · 오프라인)은 뒤에도 모두 바이트가 같다(`pwa_loading`은 한 번
    달랐고 — 로딩 막대의 때 — 다시 돌리니 같았다. 옛 스크립트 + 새 wasm, 새 스크립트 + 옛 wasm도 같다). 월드가 보이는 화면은 앞뒤를 눈으로
    (`00f-gate` · `00j2-motions-hit` · `horde0-0` · `spear-arm` · `sk-arm-echo` · `swap-sword-bow-arm`). 운영판에 `boot.js` · `pwa.js` 종료 0.
  - **시간(초, 로컬 4코어 — 소프트웨어 GL, 한 번에 하나)**: CI 넷 — `shot` 269·304 → 306(처음엔 328·359 — `devRow`가 묶음마다 ↓로 훑어
    보스 아레나까지 마흔 남짓을 눌렀다, 첫 줄들을 먼저 보게 고쳐 셋으로) · `touch` 123·131 → 109·120 · `save` 54 → 49·52 · `pwa` 16 → 15 · `away` 11 → 10·11. 나머지 — `fx` duel,horde 125 → 122 · spear 91 → 82 ·
    rb curse 136 → 126 · skills echo 115 → 105 · bare shield 135 → 128 · `swap` 319 → 310 · `leak` stress/stage(30초) 54/52 → 50/52 · `mouse` 24 → 28 ·
    `hearth` ko 307 → 288 · `items` 278 → 228 · `shield` 246 → 197 · `tabs` 215 → 192 · `view` 39 → 37 · `tutorial` 623 → 603 · `menu_res` 114 → 119 ·
    `prologue` 147 → 150 · `stress` → 13. 모두 종료 0, **`tabs.js`만 앞뒤로 1**(영어 844 px의 탭 안내 줄 `[ui-check] cut` — 이 일 전부터,
    그대로 둔다). `fx.js`의 다른 판(skills 마흔셋 전부 · daggers · greatsword · scythe · pull · wand · rb 나머지 · bare 나머지 · boss · stage)은
    돌리지 않았다 — 같은 `armed`/`open`을 지나는 길이다.
  - 시험 1,026 → 1,027(무시 20 포함). 골든 트레이스 · 월드 코드는 건드리지 않았다.
- **R15**(`Event` → `Cue`): `app/cue.rs`의 순수한 `cue(&Event, &CueCtx) -> Cue { sound, shake, punch, zoom, camera }` — `Event`를 보는 match
  하나에 와일드카드 없음(새 사건은 여기서 컴파일이 깨진다). `CueCtx`는 옛 match들이 읽던 것만(박자 이음 · 방랑자 자리 · 속도 · 버팀을 다
  받아냄). 스킬마다의 소리 · 흔들림 · 줌은 갈래 안의 빠짐없는 스킬별 match(`skill_start_sound` · `skill_sound` · `skill_kick` · `skill_zoom`)로
  두었다(2.3 — 표로 만들지 않는다). 타입(`Kind`) · 적의 큰 타(`Blow`)를 보던 와일드카드 셋도 빠짐없이. `world_sounds`(사건의 소리 —
  떼에서 틱에 한 번 · 맞힘 둘째 층 · 사슬 글은 그대로)와 `step_world`(흔들림 · 킥 · 줌 · 땅 돌리기/락온 잃음 = `Cue::camera`)가 표를 읽고,
  옛 `sound_of`와 match 셋은 지웠다. 이펙트 묶음 넷 · 튜토리얼의 셈은 따로.
  **"그대로" 증명**: 옛 match 넷을 그대로 옮긴 사본과 새 표를 860 경우(표본 86 + 스킬 34의 시동 · 판정 + 타입 아홉의 `Ail` · `FieldPulse` +
  탄의 꼴 16 + 적의 큰 타 8 × 보스/잡몹 + 여러 타 궁극기 다섯의 마지막/아님 + 땅 돌리기 −2, 각각 × 맥락 넷 — 박자 이음 0~3 · 방랑자 가까이 ·
  14~18 m · 40 m · 버팀 다 받아냄 켬/끔)에서 견줘 소리 · 흔들림 · 킥 · 줌 · 카메라가 모두 같았다(첫 시도에). 무는지: 벽 쿵의 흔들림 하나를 바꾸자
  깨졌다(되돌림). 그 뒤 사본을 지우고 옛 값을 `cue_snapshot.txt`로 남겨 `the_cue_table_matches_its_snapshot`이 지킨다(일부러 바꾸면
  `CUE_WRITE=1 cargo test cue_table`). `SILENT` 덮개 시험은 `Cue::sound`로. 골든 트레이스 · 월드 코드 그대로.
  `Event`의 와일드카드(앱 · fx): 10 → 7 — 남은 것은 무기 바꾸기 찾기(`find_map` — 한 갈래만 거른다) · 튜토리얼의 셈 둘(본 것만 센다) ·
  `fx`의 묶음 넷(묶음마다 "내 사건이면 참" — 덮개 시험이 한 사건 한 묶음을 지킨다). 시험 1,027 → 1,030(무시 20 포함 — 소리 하나 · 안내 줄 하나 · 반응 표 하나).
  CI 넷(`shot` 287초 · `touch` 108 · `save` 49 · `pwa` 15 + `away` 10) · `tabs` 180 · `fx.js throw`(새 판 — 활의 수류탄) 모두 종료 0.
- **R16**(`Form::info()` — 2.2 C): `skills.rs`의 `impl Form { pub const fn info(&self) -> FormInfo }` — 와일드카드 없는 match 하나가
  수(payload)와 상관없는 성질을 돌려준다: `aim: Aim`(`Round` 몸 둘레 · `Area` 쪽이 있는 범위 · `Sighted` 한 대상) · `grows: Grow`(4단에서 커지는
  치수 `Radius`/`Length`/`Range`/`Arc`) · `leaves: Leaves`(`Nothing` · `Aura` 되울림 · `Ground` 불티/눈발/그늘 · `Snare` 덫) · `mends`(재기 ·
  생기 거두기). 읽는 곳: `Form::aims`·`sighted`(얇은 메서드 — 부르는 곳 그대로) · `World::skill_active`(남은 몫 — 되울림 · 놓인 것 · 덫인가를
  `leaves`의 match 하나로) · `cue::skill_kick`(시동 흔들림 = `leaves != Nothing`) · 기술 탭 `app/skills.rs`의 4단 글(`grows`)과 재기/거두기 글(`mends`).
  `leaves_field`는 지웠다. 수에 따라 바뀌는 동작의 주기는 `anim.rs`에서 `Form::clip_period`(같은 impl)로 옮겼다. 쓰지 않는 칸은 두지 않았다 —
  움직임(`player_tick`) · 쪽(`skill_dir`) · 판정 시간(`Skill::active`·`Eff::active`)은 수와 고르기(`Twist`)를 쓰는 동작이라 빠짐없는 match 그대로(2.3).
  **셈**: 모양의 성질을 따로 말하던 match 9(`aims` · `sighted` · `leaves_field` · 4단 글 · `mends`의 `matches!` · 흔들림의 되울림 `matches!` ·
  `skill_active`의 되울림 `if let`과 덫 `matches!` · `anim::clip_period`) → 2(`info` · `clip_period`). **새 모양이 손댈 곳**: 컴파일러가 짚는 것 —
  `Form::info`의 한 줄 · `clip_period` · `Skill::active` · `Eff::active` · `skill_dir` · 움직임(`player_tick`) · 틱(`tick_skill`)과 미리보기 ·
  이펙트 · `Skill::at`의 4단 치수; 손으로 — `SKILLS`의 줄 · 이펙트의 서명 · 시험(아래 둘이 표가 아니라 월드에 묻는다).
  시험: 놓는다고 적힌 모양만 바닥에 무엇을 둔다(`every_skill_that_leaves_a_field_reports_its_remaining_share`에 더함) · 한 대상 모양만 숄더뷰
  크로스헤어 쪽으로 나간다(`only_sighted_skills_follow_the_shoulder_crosshair` — `aims`는 R14의 끌기 시험이 이미 월드에 묻는다). 골든 · 신호 표 그대로.
- **R17**(탄의 꼴 `Look` — 2.2 D): `Bolt::look` · `Smite::look` · `Event::Cast::look`이 `u8`(상수 열 + 마법 수 `2` · `3`)에서
  `weapons::Look`(Magic · Bolt · Arrow · Axe · Light · Javelin · Stone · Knife · Shield · Quake · Seed · Curse)으로. 성질은 `Look::def() -> LookDef`
  한 곳: `flight: Flight`(`Ground` 땅을 탄다 · `Straight` 곧게 · `Smite` 겨눈 자리에 내려 꽂힌다) · `shot`(사격 — 벼린 몫) · `thrown` · `returns` ·
  `bounces` · `name: Tr`(RB 이름) · `icon: touch::Kind`. `Bolt::is_shot`·`thrown`·`straight`·`returns`·`bounces`·`smites`는 `def()`를 읽고,
  `look_thrown` · `rb_name_of` · `toss_icon_of`는 지웠다. 소리(`cue::cast_sound`) · 빛(`overlays::bolt_colors` — 옛 수 차례 표 + 끝 줄에서 멈춤)
  · 던지는 몸(`overlays::thrown`)은 앱 · 렌더의 것이라 `LookDef`에 넣지 않고 그 자리의 빠짐없는 match로(월드 표에 소리를 두지 않는다 — CLAUDE.md).
  옛 수 0(라벤더) · 1(놋쇠)은 낼 줄이 없어 갈래를 두지 않았다. `Debug` = 옛 수(`Look::code`) — 사건의 글이 그대로라 **골든 102 해시 그대로**.
  **신호 표는 16줄만 빠졌다**: 시험이 돌던 수 0 · 1 · 14 · 255(대시 타 번호 `DASH`를 꼴로 넣던 것) × 맥락 넷의 `Cast` 줄 — 모두 "소리 없음 ·
  흔들림 0"이었고 이제 만들 수 없는 사건이다, 남은 줄은 바이트가 같다(`git diff --stat`: 16 deletions).
  **이행 증명**: 바꾸기 전, 옛 `u8` 술어(`is_shot` · `thrown` · `straight` · `smites` · `returns` · `bounces`) · `toss_icon` · `rb_name`과 새 `def()`를
  꼴 열둘에서 견준 시험 둘이 통과한 뒤 옛 것을 지웠고, 그 값을 `look_defs_keep_the_old_behaviour`에 표로 박았다(옛 수 · `Debug`까지).
  `every_toss_look_has_a_name_and_an_icon`(R14)은 만들어서 참이 되어 `every_look_is_thrown_by_some_row`(꼴마다 그것을 내는 무기 · 돌 · 방패 줄이
  있다 — 죽은 갈래 없음)로 바꿨다.
  **셈**: `u8` 꼴 비교 32곳(마법 수 2) + 수 → 값 표 셋(이름 · 그림 · 던지는 몸)의 넘어감 셋(`None` 둘 · `_ => {}`) + 빛 표의 끝 줄 멈춤 → `u8` 비교 0 ·
  넘어감 0. 남은 것은 타입이 있는 꼴 하나의 비교 20(던진 창/방패가 날고 있나 · 땅 내려찍기 · 저주탄 · 씨앗 · 볼트가 수류탄을 맞힘 · 이펙트 갈래 다섯 —
  성질이 아니라 그 꼴 자체를 묻는 곳)과 빠짐없는 match 다섯(`code` · `def` · `cast_sound` · `bolt_colors` · `thrown`).
  **새 꼴이 손댈 곳**: `Look` 갈래 + `ALL`(시험) + 컴파일러가 짚는 `code` · `def` · `cast_sound` · `bolt_colors` · `thrown`(던지는 꼴이면 몸); 손으로 —
  그것을 내는 무기 줄(`Toss`/`Shot`), 번쩍임을 낼 꼴이면 `fx`의 갈래, 터치 그림 마스크(새 `Kind`면). 시험 1,030 → 1,033(무시 20 포함 — R16 +1, R17 +2 −1, 아래 글꼴 +1).
  헤드리스(한 번에 하나): CI 넷(`shot` 291초 · `touch` 109 · `save` 50 · `pwa` 15 + `away` 10) · `fx.js rb`(꼴 열하나 전부 — 1,238초) ·
  `bare shield`(방패 던지기 114) · `throw`(36) · `tabs`(186) 모두 종료 0. 꼴마다 틱 사진을 눈으로 — 손도끼 · 창 · 단검 · 돌의 몸, 마법 탄 민트 ·
  저주탄 하늘빛, 땅 내려찍기의 물결, 신성한 빛 · 가시 씨앗의 고리와 솟는 뿌리, 볼트 · 화살의 빛 띠, 튕기는 방패가 그대로 그려진다.
- **R18**(무기 줄에 글 · 그림 — 2.2 G): `weapons::Weapon`에 `name` · `desc` · `place`(도감의 나오는 곳, `Tr`) · `icon`(`touch::Kind` — 탄의 꼴
  `LookDef::icon`(R17)과 같은 방향: 월드 쪽 표가 터치 그림의 이름표만 든다, 마스크는 `touch.rs`에) · `view`(아이템 그림의 각) · `twin`.
  지운 것: `Id` → 글 match 셋(`app/mod.rs` `weapon_name` · `equip.rs` `weapon_desc` · `codex.rs` `weapon_texts`), `touch::attack_icon`의 무기마다
  match, `item_view::view`의 무기마다 각 match(순수한 수라 줄로 — 렌더 타입이 없다). `weapon_name` · `attack_icon`은 줄을 읽는 얇은 함수로
  남겼다(부르는 곳 열넷 · 넷). `charms::TABLE`은 줄마다 `Mods` 칸 스물다섯을 다 적던 것을 `..Mods::NONE`으로(283줄 → 줄마다 바뀌는 칸만),
  `garb::MODS0`(= `Mods::NONE`의 복사) 지움. `Toss::BASE`(부채꼴 · 칸 · 차는 배 — 쌍단검 · 완드만 다르다) · `Heavy::BASE`(여진 · 깨기 · 끌기 ·
  속성 — 그 무기만의 것) · 사격 볼트의 `dose` · `lame`은 `..MAGIC`. **"그대로" 증명**: 바꾸기 전에 고정 파일 셋을 찍었다 —
  `save_fixture.txt`(모든 칸이 찬 기록 — 무기 열 · 장신구 열 · 방패 · 덧옷 · 무기마다 벼린 단 · 판마다 산 스킬 스물다섯 · 궁극기 · 주/부무기와
  그 칸 — 의 글, 바이트로) · `weapon_fixture.txt`(무기 · 종류 표의 수 Debug) · `charm_fixture.txt`(장신구 · 덧옷 · 보조특성의 `Mods`와
  `Stats::compute`). 셋 다 R18 · R19 뒤에 그대로. 골든 102 · 신호 표 · 글 맞춤 그대로.
- **R19**(표에서 나오는 목록 — 2.2 G): `Class::index` = `CLASSES`에서 찾는 const fn(손 match 지움) · `Weapon { key, class }`(`Id::key` ·
  `Id::class`의 match 지움 — 기록 키는 줄로 옮겼을 뿐, 고정 파일이 바이트로 본다) · `IDS` = `WEAPONS`의 차례 · `WEAPONS` · `CLASSES`는 길이를
  적지 않는 슬라이스 · `skills::ONE`/`TWO`/`RANGED` = `hand_bits(종류 표의 계열)`(옛 합과 같은지 시험 — 뒤에 오는 종류는 고치지 않게
  `& known`으로) · 종류 표의 `Quirks { keen, steady_swing }`(`World::keen` · `World::steady_swing` — 옛 `Class::Dagger` · `swinging_greatsword`의
  이름 비교) · 완드의 상황 힌트 = 저주탄을 쏘는 무기(탄의 꼴) · 활쏘기 동작(`anim` · 모션 보기) = 당기는 리듬(`Id::draws`). 개발 창: 상자 줄
  일곱 → `DevStart::WeaponChest(Id)` 하나(지역은 스테이지 표 `chest_loot`에서 — 상자에 든 무기마다 줄이 서서 망치 · 활의 상자도 생겼다,
  이름은 무기 줄의 이름 · 나오는 곳으로 채운다) · 모션 보기 줄 열하나는 `IDS`에서(`dev_list` = 손으로 적는 `FIXED_ROWS` + 무기마다의 줄) ·
  `ARM_CYCLE` → `arm_cycle`(`IDS` + 한손 종류는 방패와 함께 — 망치 + 방패 · 낫 + 방패가 새로 든다) · 개발 창 설명의 무기 차례 글(두 언어의
  산문 목록 넷)은 표에서 채운다. 시험 스크립트가 쓰는 이름 서른셋은 그대로(바뀐 이름 없음). `i18n::all`의 무기 · 종류 글 83줄은 표의 줄에서
  (글 모음은 언어마다 앞뒤로 같다 — 견줘 봄). 시험: 종류마다 베낀 기록 왕복 넷(쌍단검 · 대검 · 낫 · 완드) → 종류 표를 도는 고리 하나
  (`every_class_weapon_and_its_plate_round_trip` — 열 종류 모두, 전엔 넷), 쌍단검 시험의 옛 기록 부분은 `the_moved_rogue_arts_keep_through_a_save`로.
  `hold_for`는 `IDS` 차례(마법력의 배가 붙는 종류는 끝에 — "속성 스킬은 배 없는 쇠뇌 · 활로" 그대로). 남긴 것(일부러): 판 차례 시험의
  줄(`the_strip_is_common_then_every_class_then_the_shield` — 읽는 차례를 박아 둔 것) · 옛 눈금 표 `OLD_WEAPONS`(0158 N0 전의 수 — 새 무기는
  한 줄을 적는다) · 스킬 표의 종류 비트 이름(`skills::SWORD` … — 스킬이 이름으로 부른다) · 렌더 시험의 동작 이름 목록(무기가 아니라 동작마다의 예외).
  **시험 이름**: 1,033 → 1,036(무시 20 포함) — 더함 다섯(고정 파일 셋 · `every_class_weapon_and_its_plate_round_trip` ·
  `hand_masks_from_the_class_table_equal_the_old_unions` · `weapon_chest_rows_come_from_the_stage_tables`) − 지움 셋(`the_greatsword_and_its_arts_round_trip` ·
  `the_scythe_and_its_arts_round_trip` · `the_wand_and_its_arts_round_trip` — 고리가 대신) · 이름 바꿈 하나(`the_moved_rogue_arts_and_the_daggers_keep_through_a_save`
  → `the_moved_rogue_arts_keep_through_a_save`).
  **잰 것 — 열한째 종류 + 무기 하나(가짜 `Class::Flail` + `Id::Flail`, 망치의 줄을 베껴 — 커밋하지 않고 되돌림)**: `cargo check`까지 고친 파일
  **1**(`weapons.rs`: `Class` 갈래 · `CLASSES` 줄 · `Id` 갈래 · `WEAPONS` 줄). `cargo test`까지: 손으로 지키는 목록은 **`plates.rs`의 판 차례 시험 한 줄 +
  같은 `weapons.rs`의 `OLD_WEAPONS` 한 줄**, 다시 쓰는 고정 파일 넷(`golden.txt` · `save_fixture.txt` · `weapon_fixture.txt` · `charm_fixture.txt` —
  명령 하나씩). 나머지 실패 열넷은 모두 **내용**을 묻는다(시험이 짚는다): 종류만의 궁극기 · 빈 손의 결 · RB 꼴 · 터치 그림 · 아이템 부위 ·
  상자 · 첫 줄 스킬 둘(스킬 표의 비트) · 이름 글(글 맞춤의 키 수). 아무것도 짚지 않는 손일: `tools/test/hearth.js`의 `PLATE_ORDER` · 보스 모의 줄
  (`balance.rs`) · Blender 부위 · 동작(`wanderer.py` · `wanderer_skin.py` · `check_glb.py`) · 문서. 견줌: 완드 커밋 `0c00bac`의 `src` 40 파일 가운데
  표에서 나오게 된 아홉(`app/codex` · `app/dev` · `app/equip` · `app/mod` · `charms` · `garb` · `item_view` · `i18n/dev` · `save`)은 이제 손대지 않는다
  — 같은 일을 지금 하면 31쯤(나머지는 새 동작 · 내용). 손으로 지키는 목록 12 → 1(+ 같은 파일의 옛 눈금 표).
  헤드리스(한 번에 하나): CI 넷(`shot` 259초 · `touch` 100 · `save` 46 · `pwa` + `away` 24) · `hearth` ko(287 — 장비 탭 · 도감의 무기 글이 줄에서) ·
  `items`(235) · `fx.js wand`(129 — `Arm(Wand)` · 개발 창 `Dev(Arm)`의 차례 글) · `swap`(264 — 든 무기 차례) 모두 종료 0, 그리고 `WeaponChest(Wand)` ·
  `(Spear)` · `(Hammer)`를 개발 명령으로 열어 상자 곁에 서는 것을 한 번(스크립트에 두지 않은 손 확인).
- 함께(R16 · R17과 따로 된 커밋): **가로 줄 기호가 작은 글에서 빈칸처럼 보이던 것**(`tabs.js` 영어 844 px 멈춤 창 아래 줄 "View only — raise at a
  hearth"). 글자가 없거나 `Face`를 비껴간 것이 아니었다 — 고운바탕 원본의 엠 대시 · 엔 대시는 0.05 em(굵게 0.065) 두께라 9~11 px 글에서
  반 화소 밑으로 녹았다. `subset_ui.py`의 `thicken_rules`가 — – −를 가운데 그대로 0.09 em으로(원본을 받아 다시 만들었다 — 그 단계 전까지는 레포의
  파일과 바이트가 같았다), 시험 `rules_are_thick_enough_to_draw`.
- 함께(리팩터링이 아닌 따로 된 커밋 둘): **빠진 소리 다섯**(다 모임 · 2단계 · 벽 쿵 · 수류탄 던짐 · 흡수 — R12 덮개가 찾은 것, 사장 "넣자"
  — 합성음, 흡수는 0.25초에 한 번, 수류탄 던짐의 이펙트도 — 소리 없는 사건 15 → 10, 이펙트 없는 사건 16 → 15) · **`tabs.js` 넘침 고침**
  (키 안내 줄을 꼭 맞게 줄이던 배가 반올림으로 칸을 넘어 끝 낱말이 잘렸다 — `ui::hint_scale`이 반 px를 남긴다).

## 8. 다시 잰 것(2026-11-26, 0~2단계 뒤 `9e60393`) — 3~4단계의 차례

| 신호 | 시작(8cba4df) | 지금 | 목표(5절) |
|---|---|---|---|
| 내용 enum의 조용한 와일드카드 | 18 | **0**(사건 쪽은 10 → 7 — 이펙트 묶음 넷 · 튜토리얼 둘 · 교체 하나: "내 사건인가"만 묻는 자리, 덮개 시험이 지킨다) | 0 |
| 모양의 성질을 묻는 match | 9 | **2**(`Form::info` · `clip_period`) | 서술자 1 |
| 탄의 꼴 수 비교 | 35 | **0**(`Look` + `LookDef`) | — |
| 새 종류에 고치는 `src` 파일(가짜 종류로 잼) | 40 | **약 31** — 손 목록 12 → 1(+ 옛 값 표 한 줄), `cargo check`까지는 `weapons.rs` 하나 | 28 아래 |
| 헤드리스의 세어 누르기(개발 창 · 무기 · 탭) | 43 | **0** | 0 |
| 200줄 넘는 함수 | 40 | **38** | 30 아래 |
| 시험 | 1,018 | 1,036(+ 골든 102 경우 · Cue 스냅샷 · 기록/무기/장신구 고정값) | — |
| Rust 줄 | 122,005 | 123,737 | — |

**읽기**: 조용한 빠짐과 손 목록은 목표에 닿았다. 긴 함수는 거의 그대로다 — 0~2단계는 길이를 줄이는 일이 아니었다. `step_world` · `world_sounds`는 표로 빠졌지만(R15),
`fx::on_skill`은 스킬마다 갈래 하나가 되면서 784 → 883줄, `fx::on_skill_event` 1,019 → 1,066줄로 **가장 긴 로직 함수 둘이 이펙트**가 되었다. 그 다음이 `tick_skill` 856 ·
`enemy::tick` 479 · `input::poll` 445 · `hud::draw` 370 · `stage::tick` 336. 구조체 칸(World 74 · App 68 · StageRun 62 · Enemy 58 · Player 48)과 앱 → 월드 쓰기(잔불 10)는 손대지 않았다.

**3~4단계의 차례(다시 정함 — 긴 것 · 자주 고치는 것 먼저, 묶음 하나 = 에이전트 한 번)**:
1. **R20 · R21 + 안전망 하나 더**: 시나리오를 돌린 사건 해시(적의 틱까지 덮는 둘째 골든) → `tick_skill` 갈래를 메서드로(`SkillCx` · `Sweep::of` · `cap_stagger`, 달라진 복사본 둘은 규칙을 시험으로 고정한 뒤 하나로) → `Field` 플래그를 `FieldKind`로 · `Burst`의 종 치기 칸 · `Art`를 합 타입으로.
2. **R25a(앞당김) · `enemy::tick`**: 이펙트의 안전망(사건 표본마다 낸 입자 목록의 해시) → `fx::on_skill` · `on_skill_event`의 갈래를 스킬/사건마다 이름 있는 함수로 `fx/`에(정체성은 그대로 — 묶지 않는다) → `enemy::tick`의 상태 갈래를 메서드로 → `hud::draw`를 구역마다.
3. **R22 · R24**: 뷰 모델 셋(장비 · 가방 칸 · 스킬 트리의 열린 줄 — 그리는 중의 상태 바꿈 없애기) · HUD의 장전 셈은 월드의 메서드로 → 잔불 지갑 · `App`의 탭 UI 칸 · `Player`의 종류 상태 · 월드의 틱 깃발 묶기.
4. **R23 · R25b**: `input::poll` 장치마다 + 합치기 · `touch::read(&[Touch], w, h)` → 파일 나누기(`skills.rs` · 터치 그림 데이터).
5. 끝난 구조로 `CLAUDE.md`(뼈대 + `docs/guide/<영역>.md`).
