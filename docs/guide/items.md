# 아이템 · 가방 · 성장 · 기록

`CLAUDE.md`에서 옮긴 규칙이다(0278) — 문장은 그대로이고 같은 힘을 가진다. 고치는 곳: `src/items.rs` · `src/app/bag.rs` · `src/forge.rs` · `src/garb.rs` · `src/stats.rs` · `src/charms.rs` · `src/growth.rs` · `src/progression.rs` · `src/save.rs` · `src/journey.rs`.
이 영역의 파일을 고치기 전에 읽고, 구조를 바꿨으면 여기를 고친다. 굵은 머리말에서 문단만 갈랐다.

## 아이템 인스턴스

아이템 인스턴스(0223 V4 — 규칙·가방·얻는 길·장비 탭·화로에서 잔불로 바꾸기까지; 아이템마다의 벼린 단·바닥의 빛은 남김)는 `items.rs`: 얻는 길은 앱의 `gain_charm`·`gain_weapon`(`roll` = (급, 씨앗)이 있으면 굴린다 — 상자의 것·가게)과 `drop_spare`(엘리트·문지기가 떨구는 **이미 가진 바탕의 여벌**)·문지기의 장신구·상자의 여벌(`stage::Loot::Spare` — 가진 바탕의 고급 확정, 0274 ⑦ — 상자의 것이라 줍는 순간 가방에) — **아이템은 바로 얻지 않는다(0226, 사장 "습득의 재미")**: 바닥에 떨어지고(`App::lay_drop` — 등급의 빛으로 번쩍 `Fx::loot_drop` + 소리) 주워야 든다(`take_item` — `Fx::loot_take`). 새로 아이템을 주는 길도 `lay_drop`을 지난다(사는 것만 바로), 씨앗은 `App::loot_seed`. 장비 탭은 후보 목록에 여벌 줄 `Cand::Item(번호)`를 바탕의 줄 아래에 세운다(`equip::with_spares` — 바탕의 줄 = 그 바탕의 지금 것 `item_at`), 여벌을 끼는 길은 `App::wear_item`(그 밖의 길 — 바탕 줄·주/부무기 맞바꾸기·장신구 칸 옮기기 — 은 `Bag::settle`이 끼던 아이템을 따라가게 한다: 낀 자리를 바꾸는 새 길도 `gear.*`를 손으로 옮기지 않아도 되지만, 주·부무기를 맞바꾸는 길은 스킬 칸도 `App::swap_kits`로), 여벌의 바탕은 `items::pick_base`(낀 바탕이 무게 3 — 씨앗을 `>>`로 잘라 쓰지 않는다), 같은 틱에 다른 알림과 겹치는 알림은 `App::notice.toast_wait`에 잇는다, 장비 탭의 무기 숫자는 고른 아이템의 옵션까지 든 능력치로 세고 견주는 상대는 **그 칸에 든 것**(부무기 칸 = 부무기 — `equip_lines`의 `numbers`·`held`), 이름은 `i18n::item_label`(등급 말 + 바탕 이름)·옵션 줄은 `i18n::affix_line`. 여벌을 잔불로 바꾸는 곳은 화로의 가방 탭(0226 — `Ask::Salvage` → `App::salvage_item` · 한꺼번에 `Ask::Tidy` → `tidy_bag`: 거둘 수 있는 것은 `Bag::loose` = 끼지 않은 것 `spares`(그 바탕의 마지막 하나도 — 0266, 치우면 `Progress::part_with`가 가진 바탕에서 빼고 도감은 `codex_had`로 기억, 같은 바탕이 남는 진짜 여벌은 `extras`) 가운데 지키지 않는 것). 헤드리스 `tools/test/items.js`. 그 밖에: **가방** `Bag`(`Progress::bag` — 가진 아이템 하나하나와 낀 자리 `Gear`)은 기록의 옛 칸(가진 무기·방패·장신구와 낀 것 — 바탕) 위에 얹혀 "그 바탕의 어느 것인지"를 말한다: `Progress::settle_items`(읽은 뒤·`App::persist` 앞)가 가진 바탕마다 아이템 하나를 보장하고(옛 기록이 보통 것으로 옮겨진다) 낀 자리를 맞춘다. 옵션은 낀 것만 — 무기는 든 것만(`Bag::mods` → `Progress::stats`, 바꿔 들면 `Bag::swap_weapons`). 기록 키 `item.<번호>`·`item.next`·`gear.*`(바꾸지 않는다). 새 얻는 길·새 끼는 자리를 만들면 옛 칸을 바꾼 뒤 `settle_items`가 돌게 한다(저장 앞에서 저절로 돈다). 아이템 = 바탕(`Base` 무기·방패·장신구) + 등급(`Rarity` — 옵션 줄 수 0·1·2) + 옵션(`Affix` = `Stat` + 단 1~3) + 벼린 단. 옵션은 **숫자만**(표 `Stat::values`·붙는 곳 `Stat::fits`·보정 `Stat::mods` → `stats::Mods`), 받는 피해·패링 창·회피를 건드리는 옵션은 만들지 않는다(시험). 지역이 세기의 상한(`tier_cap`), 굴리기는 씨앗으로(`roll` — 같은 씨앗 = 같은 아이템), 기록의 글은 `Item::encode`/`decode`(키는 바꾸지 않는다). 새 옵션 = `Stat` 한 갈래 + `STATS` + `key`·`values`·`fits`·`mods`(+ 새 `Mods` 칸이면 `Stats::compute`와 쓰는 곳) + 시험. `Mods::skill_cd`(스킬 대기 배)·`Mods::ail`(타입 게이지 배 — 스킬의 타가 나가는 자리에서 `World::dosed`)가 이 옵션들의 칸이다.

## 가방

가방(0226 — 사장 2026-11-17 "가득이면 못 주워야지, 바닥에 떨어져 있어야지"): 한도 `items::BAG_CAP` 30(무기·방패·장신구 아이템 전부, 낀 것 포함 — 덧옷·숫돌 같은 숨은 물건은 밖).

**가득이면 칸이 드는 것은 줍지 못한다**: 앱이 스테이지의 틱 앞에서 `App::tell_bag`으로 알리고(`StageRun::bag_full` · 상자의 것은 `chest_block` — 아직 없는 바탕만 막는다), 스테이지는 곁의 것을 줍기 대신 `Near::Full`로 말한다(버튼은 그대로 점프, HUD가 까닭을). 아이템을 가방에 넣는 새 길은 가득일 때를 정한다(막는다 — 가게 · 바닥에 둔다 — 떨굼: 엘리트·문지기의 여벌과 문지기의 장신구). 바닥의 아이템은 `StageRun::item_drops`(`ItemDrop` — `drop_item(자리, 아이템, wait)` → 주우면 `take_item` → 앱 `App::take_item`; `wait` = 내려놓은 것은 한 번 곁을 떠나야 줍는다, 높이는 부른 쪽이 `World::ground_y`로 땅에), **기록에도 남는다**(`Progress::ground` = `items::Ground`, 키 `ground.<n>` — 앱이 `note_ground`로 이 지역의 것을 옮기고 지역을 세울 때 `seed_run` → `lay_ground`로 되놓는다: 판을 새로 세우는 길은 `seed_run`을 지나야 한다), 그림은 `render/props.rs` — 몸은 **그 아이템의 모델**(`drop_models`: `item_view`의 부위를 `DROP_MODEL_R` 크기로, 배우 앞 `stage_props`), 빛기둥·빛무리는 `item_drops`(빛 = `Rarity::glow`, 드물수록 굵고 높다 `Rarity::rank`, 배우 뒤 `stage_glows`), 문지기가 쓰러진 자리는 `StageRun::take_boss_down`. 화면은 큰 창의 가방 탭 `app/bag.rs`(`Scene::Bag { host }` + 작은 목록 `Scene::BagAct`): 칸의 차례 `bag::order`(저절로 — 손으로 옮기지 않는다) · 커서 `bag::nav` · 그 아이템에 할 수 있는 일 `bag::acts`(끼기 = `App::wear_item` — 장비 탭과 같은 길 · 잠그기(0235 — 화면 글 "잠근다", 그 전 "지킨다") `Bag::toggle_lock`(기록 `item.lock`) · 내려놓기 `put_down`(어디서나) · 잔불로(화로에서만 — 멈춤은 흐린 줄)) · 화로의 "여벌 정리" `Bag::clearable`(지키지 않는 여벌 가운데 그 바탕의 가장 값진 하나가 아닌 것 — 낀 것보다 값진 여벌은 남는다). 새 것의 점은 `Bag::fresh`(기록 `item.seen` — 탭을 떠날 때 `leave_bag`). 탭 차례는 멈춤 장비 · 가방 · 기술 · 기록 · 설정 / 화로 성장 · 장비 · 가방 · 기술 · 가게 · 설정(0263) — 헤드리스 스크립트는 세지 않고 장면 이름으로 넘긴다(`sceneKit.openTab(장면 이름)`, 0278 R13). 헤드리스 `tools/test/items.js`.

## 무기 강화

무기 강화(0180)는 `forge.rs`(순수 셈 — 무기마다 세 단 `Forged`, 값 `COST` = 숫돌 + 잔불, 거두면 숫돌만 돌아온다)와 무기 표의 `Weapon::forge`(근접 = 타마다 피해, 쇠뇌 = 탄창 먼저·`Stats::shot_dmg`·`mag`): 숫돌은 숨은 물건 `Find::Stone`(지도 `O`, 기록 `<스테이지>.stone.<n>` — 가진 수는 `finds`에서, 쓴 수는 벼린 단에서 센다) 또는 상자의 숫돌(`stage::Loot::Stone` — 기록 이름 `stage::chest_stone_id` = `<스테이지>.stone.chest<n>`: 숫돌로 세고 숨은 물건 · 여정의 셈에는 들지 않는다, 0274 ⑦), 벼린 단은 기록 `forge.<무기 키>`, 단은 **든 무기에만**(`Progress::stats`), 탄창은 `World::mag_of` 한 곳(HUD·장전·회피 장전). 화면은 화로의 가게 탭 줄 `app/shop.rs` `ShopRow`(진열 → 벼리기 → 거두기). 단의 값을 바꾸면 `forged_dps_probe`·보스 모의 `SIM_FORGE=<단>`으로 재고, 쇠뇌가 같은 단 검의 일곱 할 밑인지 시험이 본다.

## 덧옷

덧옷(0176)은 `garb.rs` 표(줍는 숨은 물건 `Find::Garb` = 지도 `M`, 기록 이름 `<스테이지>.garb.<n>` — 가진 것은 `finds`에서 센다, 걸친 것은 기록 `garb.worn`): **방어력·받는 피해 숫자를 쓰지 않는다**(사장 "너무 RPG식은 말고") — 맞을 때 밀림 `Mods::knock`·휘청임 `flinch`(`Stats::tune`이 `hurt.knock`·`hurt.stun`을 휜다 — 월드는 덧옷을 모른다)와 회피 무게만, 덜 밀리면 회피가 무겁다(시험). 설명은 말로만. 장비 탭 다섯째 칸(`equip::GARB_SLOT`, 왼쪽 열 무기 밑)·도감 셋째 묶음. 몸에 걸친 모습은 그리지 않는다(0157 — 그림 먼저). 새 덧옷 = `TABLE` 한 줄 + 지도 `M` + 글 넷(이름·설명·말·놓인 곳) + `charms.py`의 `garb_*` 부위(시험이 지도의 `M`과 표를 맞춘다).

## 능력치와 장신구

능력치(0064 M3)는 `stats.rs` 한 곳: `Stats::compute(&[Mods])`(합·곱·상한), 월드는 `World::stats`만 — 최대 체력·등잔은 `Player::max_hp`·`flask_max`(상수 `PLAYER_HP`·`FLASK_CHARGES`는 기본값일 뿐), 랩 값은 `Stats::tune`이 휜 사본. 장신구는 `charms.rs` 표(기록 키는 바꾸지 않는다) — **종류 셋(0224)**: 칸마다 종류 하나(`SLOT_KINDS` 목걸이·반지·팔찌), 장신구는 제 종류의 칸에만(`Id::kind` — `_` 없는 match, 같은 일을 하는 것끼리 한 종류: 겹쳐 쌓지 못하게), 읽을 때·낄 때 제 칸으로 앉힌다(`charms::seat`·`equip::wear`), 장비 탭의 후보는 그 칸의 종류만. 새 장신구 = 표 한 줄 + 종류(종류끼리의 수는 하나까지만 다르게 — 시험, 0270 D에서 열이 되었다). 새 보정 = `Mods` 칸 + compute 한 줄 + 쓰는 곳 + 시험.

## 성장

성장(0064 M4)은 `growth.rs`(화로 가게 `SHOP` 표 + 옛 핵 고르기의 남은 몫 — 핵 고르기는 0105에서 걷었다, 옛 기록은 `save::migrate_v04`가 한 번 옮긴다). 핵은 나무 마디(이길 때마다·셋까지 `core.warden`), 등잔·한 모금은 숨은 물건(`stage::Find`, 지도 `F`·`H`, 기록 `finds` = 이름 `<스테이지>.<종류>.<n>`, 상자와 같은 꼴로 줍기). v0.4.0 성장(0102)은 `progression.rs` 표 한 곳 — 잔불 레벨 생명·기력·힘(0104, 기록 `lv.*` — 값은 총 레벨로 `LEVEL_BASE + LEVEL_STEP × 레벨`, 0109)·패시브 열다섯 칸 `TREE`(갈래 셋 × 다섯 — 0161부터 기술 판의 칸이다: 기술 점수로 칸마다 따로 열고(`Tree`는 칸마다의 비트, 기록 `tree.cells.<갈래>` = 연 자리들 — 옛 `tree.<갈래>` = N은 앞에서부터 N칸으로 읽는다), 핵·종소리 조각 `Find::Shard`는 기술 점수의 샘 — 칸 동작은 `Tree::has`로 그 동사의 메서드에 한 줄씩). 쉬면 큰 창이 화로로 열린다(0164 — `flow::HEARTH_OPENS` = 상태 탭, 아는 스킬이 없는 새 판만 기술 탭 `flow::hearth_opens` 0166: 레벨은 상태 탭 `app/status.rs`, 기술·가게는 그 탭, 옛 `Scene::Hearth`·`Level`과 줄 목록은 없다). 능력치는 장신구와 같은 `Stats::compute`로 — 판정 창을 넓히는 성장은 없다(0095 원칙 2, 시험). 장비 탭의 상태 줄은 장신구가 바꾸는 줄만(`stats::lines_from`), 레벨까지 든 지금의 수치는 상태 탭. 큰 창의 탭(상태·장비·기술·도감·가게·설정)을 고치면 `tools/test/hearth.js`(한국어·영어 + F4 — 흐름만, 언어마다 4분 반: 칸마다의 글은 `app::text_fit`이 본다, 0275), 스킬을 끼우는 길은 `tools/test/fx.js`도 쓴다. 개발 창 글은 `i18n::dev_texts`에 — 글자 미리 굽기가 같이 쓴다(빠지면 아틀라스가 그 프레임에 커져 지운 텍스처 오류, 0106).

**벌이를 재는 시험(2026-11-27)**: `src/stage/economy.rs`의 `economy_probe`(무시 시험 — `cargo test economy_probe -- --ignored --nocapture`)가 실제 배치에서 지역마다의 잔불(처음 다 깨기 · 쉬고 한 바퀴 · 다음 판) · 도착 레벨 · 기술 점수의 상한 · 트리마다 다 올리는 값을 찍는다. 잔불이 들어오는 곳(적의 값 `Enemy::ember_worth` · 처음 보상 · 여정 · 도감)이나 쓸 곳을 더하거나 값을 바꾸면 이 시험을 같이 고치고 다시 찍어 본다 — 문서에 적은 벌이는 낡는다.

## 기록

기록(0064 M2)은 `save.rs`: 키=값 줄(`save::key` 이름은 바꾸지 않는다, 모르는 키는 지킨다).

**슬롯 셋(0066)**: 웹 localStorage `knell.save.0..2` + 마지막 슬롯 `knell.slot`, 네이티브 `save0..2.txt` + `slot.txt`(슬롯 1 = 옛 한 칸). 타이틀 [이어하기]·시작하기 → 슬롯 창(`app/slots.rs`, 결정은 `flow::slot_focus`·`slot_step`·`slot_pick`), 덮어쓰기·지우기는 묻는 창(`flow::Ask::Overwrite`·`Erase`, 커서는 취소에서 — 0192). 설정은 슬롯 밖, 든 무기는 기록(`weapon`). 앱은 판에서 옮겨(`sync_progress`) 바뀌었을 때만 쓴다(`persist`). 판을 넘어 남는 것을 새로 만들면 `Progress`에 칸 + 키 하나 + 시험. 설정(`prefs`)과 섞지 않는다.

## 여정과 도감 보상

여정(0183 — 퀘스트 대신 지역마다의 목표)은 `journey.rs` 표 `GOALS`(지역·갈래 갈 길/발견·이름·풀이·세는 것 `Need`·보상 `Reward`): **진행은 이미 있는 기록에서 다시 센다**(문·핵·숨은 물건·엘리트·도감 관찰·읽은 새긴 돌 `read`), 기록에는 받은 것만(`journey`). 보상은 잔불 조금 또는 기술 점수 하나(지역마다 하나까지 — `Progress::skill_points`가 받은 목표에서 센다, 새 화폐 없음 — 시험). 화면은 기록 탭의 첫 묶음(`app/journey.rs`·`codex::JOURNEY` — 확인 = 받기, 멈춤에서도, 0261 → 0265 맨 앞).

**도감 칸의 보상(0265)**: 적힌 적·얻은 장신구·덧옷·무기마다 한 번 잔불 조금(`codex::FOE_EMBERS`·`THING_EMBERS`), 기록 탭에서 확인 = 받기(여정과 같은 꼴 — ◆·민트 점), 기록 `codex.paid`(`codex::paid_key` — 바꾸지 않는다). 도감의 무기 묶음에 맨손은 없다, 새로 이룬 것의 알림은 `App::journey_watch`(불러온 기록의 지난 일은 조용히). 주민·핵의 반응·종소리의 근원은 목표로 삼지 않는다(lore). 새 지역 = 표에 줄 + 글 둘(이름·풀이, 한·영).
