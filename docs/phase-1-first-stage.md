# 1단계: 첫 스테이지 초안 — "침묵의 성소"

PoC를 닫고(0019) 이야기 첫 장을 한 판으로 만든다. **초안이다** — 그림·소리·수치는 모두 바뀐다. 목적은 (1) 장면·스테이지·종류·글·모델이 표와 파일로 늘어나는 뼈대가 서는지, (2) 쿼터뷰 떼싸움 → 문 → 락온 보스전이 한 판 안에서 이어지는지 보는 것.

## 범위
- 장면: 타이틀(시작·언어·개발판 항목 둘) → 시작 이야기 → 싸움 → 멈춤(계속·처음부터·타이틀로) → 끝(1.6초) → 결과(클리어 = 시간·쓰러뜨린 수 / 쓰러짐)
- 스테이지: 입구 → 좁은 길 → 뿌리의 뜰(물결 셋) → 성소 문 → 잠든 문지기 깨어남 → 결투 → 핵 거두기
- 적: 뿌리의 잔재(느린 근접 떼), 가시짐승(곧은 돌진), 잠들지 않는 문지기(보스: 부채꼴 내려치기·돌진·뿌리 파동, 반 아래에서 빨라짐)
- 그림: 바닥 석판·무너진 벽·기둥·뿌리·화로 불빛(정점 빛), Blender 모델(방랑자·잔재·가시짐승·문지기·소품), 예고 모양 넷(원판·부채꼴·띠·고리)
- 글: 한국어·영어(고운바탕 서브셋, 0021), 키 그림은 지금 장치에 맞춰
- 하지 않는 것(1단계 처음 범위): 소리, 저장, 스테이지 둘째, 레벨 편집기, 개발 메뉴 안의 랩(랩은 F1 그대로). 소리·저장은 v0.3.0(0064)에서 들어왔다.

## 판정(사람)
| # | 질문 | 기준 |
|---|---|---|
| S1 | 한 판이 이어지나 | 처음 하는 사람이 설명 없이 입구 → 뜰 → 문 → 보스 → 핵까지 간다(어디로 가야 할지 헤매지 않는다) |
| S2 | 뜰에서 쿼터뷰 떼싸움이 읽히나(P1 이어서) | 잔재와 가시짐승을 구분하고, 돌진 띠를 보고 피한다 |
| S3 | 보스전이 락온으로 할 만한가(P3 이어서) | 예고 셋(부채꼴·띠·고리)을 읽고 피한다. 2~4분 안에 이기거나 질 이유를 안다 |
| S4 | 분위기 | 어둡고 조용한 성소 + 호박색 불빛. 캐릭터가 바닥에 묻히지 않는다 |
| P5 | 실기 성능(PoC에서 넘어옴) | 뜰 셋째 물결에서 데스크톱 Chrome 60, Android Chrome 30 이상(개발 줄 F3) |

## 구현 상태(2026-11-07 — 첫 판 2026-11-04, v0.3.0 배포 2026-11-06, v0.3.4·v0.3.5 배포 2026-11-07·0100까지, v0.4.0 T0 0103·G1 0104·G3 0105·G2·G4 0106·G5 0107·표시 0108 — v0.4.0 배포 2026-11-08(PR #6), 판정 전)
| 항목 | 위치 | 비고 |
|---|---|---|
| 남은 주민(0122, v0.5.0 C7) | `residents.rs`(표·말·재방문 `settle`), `StageRun::residents`·`near_resident`·`take_talked`·`take_residents_seen`, `World::bodies`, `App::bubble`·`draw_bubble`, 기록 `runs`·`residents.seen`·`residents.changed`, `render.rs` 상자 배우·생활 물건 | 셋(아궁이·물·천), 공격이 지나감, 돌아봄, 말풍선(다시 누르면 다음 말), 처음 본 판 뒤 다른 판에서 한 번 바뀜 |
| 떠난 마을 지도(0121, v0.5.0 C1) | `tools/maps/village.py` → `stage/village.rs` MAP·HEIGHTS, `StageDef::gate_shortcut`·`card_lever`, 시험 `residents_live_beyond_every_enemy_leash`·`the_village_ladder_opens_only_from_the_lever_below` | 45×60칸, 비탈 아래로(거주지 +4 → 골목 +2.5 → 광장 0 → 종탑 터 −1m), 사다리 지름길, 화로 셋, 적 15 + 엘리트 셋, 주민 자리 셋(끈 밖) |
| 떠난 마을의 적(0120, v0.5.0 C2) | `species::SHELL`·`POD`(`guard_arc`·`plant_*`·`retreat`·`Behavior::Planter`), `enemy.rs` `Atk::Plant`·`guards`·`tele_at`·`docile`·`eat_t`, `world.rs` 튕김(`Event::Deflect`·`PState::Recoil`)·꼬투리 물러나기·청소자 알아차림, `StageDef::docile`, 시나리오 `Newcomers` | 등딱지짐승·꼬투리는 Meshy 모델(0132 — `shellback.py` 앞을 들고 쿵, `pod.py` 꼬투리가 부풀었다 꽉 오므림·뒷발로 깡충), 청소자는 잔재 모델. 등딱지 = 앞 평타·탄 튕김(강타·옆·뒤·내려찍기로 뚫음), 꼬투리 = 심은 원판 + 깡충 물러나기, 청소자 = 맞아야 싸움·먹으면 회복 |
| 구역 등급(0112·0116, v0.5.0 T3) | `StageDef::tier`, `tuning::TIERS`·`tier`·`TIER_HEAVY_HITS`, `Enemy::set_tier`, `world::poise_max`·잔불 떨굼·한 대 피해 | 성소 1·마을 2(체력 ×1.5·자세 ×1.3·잔불 ×1.6 올림), 등급 3부터 한 대 2, 보스·예고·움직임은 그대로 |
| 지역 환경(0119, v0.5.0 T2) | `backdrop::Env`(`SURFACE`·`UNDERGROUND`)·`StageDef::env`, `render::set_moonlight`, `Scenery::bake(…, trees)`, 지도 `S`(빛줄기 — `stage::SHAFT_GLYPH`, `render::shaft`) | 땅 위 = 하늘·달·반딧불·나무, 땅속 = 흙빛 동굴 원경·높은 벽·달 없음·달빛 0.4·떨어지는 흙·벽 뿌리만·천장 틈 빛줄기 |
| 지역 잇기(0118, v0.5.0 T1) | `stage::STAGES`·`StageDef::depth_m`·`Phase::Down`·`StageOut::{CoreTaken,Descend,Ascend}`·`arrive_from_above/below`, `stage/village.rs`(빈 상자 지도), `descent.rs`(지층 단면), `app/progress.rs` `enter_stage`·`cross_to`·`respawn_elsewhere`, 기록 `run.stage`·`run.down` | 판의 차례 = 성소 → 떠난 마을(지하 40m). 성소 핵 → 바닥 길(`D`) → 단면 전환(가운데에서 지역을 바꿈, 굽기가 그 뒤에) → 마을 어귀. 오르는 길(`U`)로 되돌아가면 이긴 문지기는 없고 바닥 길은 열린 채. 다른 지역에서 쉬었으면 거기서 다시 섬. 결과 화면은 마을 끝 문지기(자리만 — 첫 문지기 모델) 뒤. 개발 "둘째 지역(떠난 마을) 어귀부터" |
| v0.4.2 패치(0117) | `weapons::Weapon::stamina`, `world/player_tick.rs`(평타·강타·쏘기·수류탄·공중 볼트), `tuning.rs` 충전 28/s·0.8초, `save::Progress::weapons`·`owns_weapon`, `app::App::arms`, `stage::Loot`·`chest_loot`, `growth::SHOP`, `tools/maps/sanctuary.py` | 무기별 기력·원거리 기력·느린 충전·무기는 상자에서(옛 기록은 다 가짐)·적 39·입구 강화 덜기·가게 값. `Cargo.toml` 0.4.2·`version_code` 16, `docs/releases/v0.4.2.md` |
| 나무 칸 표시·v0.4.0(0108) | `render.rs`(끝맺음 금빛 두 겹·비껴서기 민트 고리·맞서는 자 호박 반달), `World::beat_cue`(맺음타 마지막 타 부채꼴), `cancel_or_buffer`(맺음타 캔슬 길), `Cargo.toml` 0.4.0·`version_code` 14, `docs/releases/v0.4.0.md` | 0107에서 맺음타가 캔슬 길로는 안 걸리던 것을 고침. 릴리스 PR `release/v0.4` → `main`(Merge는 사장) |
| 나무 칸 동작 15(0107, v0.4.0 G5) | `progression::cell`(값), `world.rs` `strike_cells`·`outer_ring`·패링 튕김, `world/player_tick.rs` `swing_pays`·`tick_attack`·`tick_charging`·`tick_dodge`·`tick_run` | 칸마다 `tree.has` 한 줄 + 시험 하나, 판정 창을 넓히는 칸 없음. 받아내기·맞서는 자는 지금 게임에 맞게 뜻을 고침(0107). 칸을 안 열면 싸움은 T0 전과 같음(흔적 120/120) |
| 기술 나무·종소리 조각·나무 창(0106, v0.4.0 G2·G4) | `progression::TREE`·`Tree`·`tree_points`, `stage::Find::Shard`(지도 `K`), `app/tree.rs`, 기록 `tree.*`, 멈춤 셋째 줄 "나무" | 갈래 셋 × 다섯 칸, 칸마다 1점(핵 + 조각), 칸 밝기 네 단·거기까지 n점·목표 핀, 화로에서 열기(묻는 창), 멈춤은 읽기 전용. 칸 동작은 G5(아직 — 열어도 싸움은 그대로) |
| 핵 점수·숨은 물건·옛 기록 옮기기(0105, v0.4.0 G3) | `stage::Find`·`take_picked`·`first_core`, `progression::CORE_CAP`·`find_mods`, `save::migrate_v04`, 기록 `finds`·`core.warden`·`migrated.v04` | 핵 고르기 화면 없음 — 이길 때마다 핵 1점(셋까지, 나무는 G2), 처음 이긴 핵은 스스로 다가옴. 숨은 물건 셋(뜰 동쪽 높은 단 등잔·입구 구석 한 모금·성소 한 모금). 옛 기록은 잃는 것 없이 생명·숨은 물건·핵 점수로 |
| 잔불 레벨·쉼터 화로(0104, v0.4.0 G1) | `progression.rs`, `stats.rs`(`stamina_max`·`stamina_regen`·`poise`·`heavy_dmg`·`light_dmg`·`lines_from`), `app/hearth.rs`, 기록 `lv.*`, `tools/test/hearth.js` | 생명·기력·힘 각 5단(값은 총 레벨로 40 + 20 × 레벨, 0109), 쉬면 화로(레벨·가게·떠나기), 레벨 창 미리보기 `지금 → 다음 ▲`, 올리기는 묻는 창, HUD 기력 막대가 길어짐, 체력 상한 12. 고쳐 쥐는 동작은 아직 |
| 플레이어 틱 나누기(0103, v0.4.0 T0) | `world/player_tick.rs` `Ctx`·`tick_*`·`try_dodge`·`try_parry` | 925줄 한 함수를 단계·상태마다 메서드로, 동작 그대로(흔적 비교 120/120). 나무 칸은 그 동사의 메서드에 한 줄씩 |
| 소개 페이지(0093) | `site/`(한국어·`en/`), `.github/workflows/pages.yml`, `tools/site/subset_font.py` | 개발 중이라 한 장만: 첫 화면·두 시점·해 보기·만드는 곳, 스크립트 없음. 주소 knellroot.beardworks.io(도메인·Pages 켜기는 사람이 할 일) |
| 바닥 다듬기(0060 W2) | `scenery.rs` `MORTAR_Y`·`SLAB_*`·`tuft`·`WALL_AO` | 걷는 면 아래로 판 석판(높이·긴 판·들림·빠짐 + 옆면), 풀 다발, 벽 밑 그늘 |
| 화면 공간 선(0061) | `render.rs` `EdgeFx`·`normal_pass`·`edge_fx`, `scenery::with_normal`, `Batch::record` | 법선 패스(뷰 법선 + 선형 깊이) → 이웃 넷: 깊이가 끊기는 앞쪽은 어두운 선, 볼록 모서리는 밝은 선. 랩 `render.edge*` 다섯(덧붙임). GL만 |
| 길 안내(0060 W1) | `stage::StageRun::goal`, `render.rs` `beam`·`GUIDE_*`, `hud::draw_goal_arrow`, `camera::world_to_ndc`·`edge_marker` | 문이 열리면(그리고 핵이 남으면) 그 자리에 호박색 빛기둥 + 빛, 화면 밖 1초면 가장자리 화살, 목표 글 "열린 문으로 성소에 들어가라" |
| 굴리는 목록(0059) | `ui.rs` `ListView`·`Ui::menu`, `touch.rs` 끌기, `input.rs` 휠 | 모든 메뉴·조작 설정이 넘치면 굴린다: 보이는 줄 + ▲▼(누름), 키 커서는 창이 따라가고 가장자리 한 줄 여유, 손가락 끌기(탭 범위 넘으면, 뗄 때 탭 아님)·휠·▲▼는 창을 옮긴다. 지금 메뉴는 넘치지 않아(UI가 화면 높이 비례) 줄이 늘 때 켜진다 |
| 키보드+마우스(0058) | `input.rs`, `app/play.rs` `cursor_aim`, `camera.rs` `screen_ray`, `hud.rs` `draw_reticle` | 기본 배치: 왼쪽 클릭 공격·오른쪽 클릭 마법·휠 클릭 락온·L-Shift 패링, 왼손 그대로(땅 돌리기 Q·E 남김). 마우스 = 오른쪽 스틱: 쿼터뷰에서 커서를 비춘 땅 위 점 쪽으로 겨눔(수류탄은 그 자리), 만진 지 1초(설정 0.5·1·2·4) 지나면 자동 조준, 락온 중엔 안 씀. 싸움 중 OS 커서 대신 조준 표시(작게·보통·크게). 조작 설정에 "키보드+마우스 배치로·키보드만 배치로", 메뉴는 왼쪽 클릭을 뗄 때 탭. 웹 오른쪽 클릭 메뉴·휠 클릭 자동 스크롤 막음. 감도 칸 없음(OS 커서). 시험: 쉼 규칙·슬롯 번호·커서 조준·락온 무시·수류탄 자리·프리셋·prefs, 헤드리스 `tools/test/mouse.js` |
| 장면 | `app/mod.rs`·`app/stack.rs`(0066), `app/flow.rs` | 흐름은 순수 함수 + 시험 셋. 메뉴는 위·아래 + 확인, 터치는 뗄 때 탭, 언어 줄은 좌우로도 바뀐다. 멈춤: Esc·P / Start / 터치 오른쪽 위 |
| 스테이지 | `stage/mod.rs`, `stage/sanctuary.rs` | 글자 지도 37×25칸(칸 2m). 벽은 줄마다 상자 하나로 합쳐 부딪힘. 진행은 `Phase`(입구 → 물결 n → 문 열림 → 깨어남 → 결투 → 핵 → 끝). 시험: 지도 읽기·입구에서 클리어까지·쓰러지면 끝 |
| 종류 표 | `species.rs`, `enemy.rs` | 잔재 체력 3·2.6m/s·원판 0.55초 / 가시짐승 체력 2·3.8m/s·7m 안 돌진(띠 0.65초, 13m/s × 0.5초) / 문지기 체력 40·부채꼴 110° 0.9초·돌진 11m·세 번째마다 파동(1초 예고, 9m까지 9m/s), 반 아래 예고 ×0.75 |
| 글 | `i18n/`, `ui.rs`, `hud.rs` | 한국어·영어. HUD: 체력 마름모·스태미나·목표·구역 카드·시작 이야기·보스 띠 |
| 모델 | `assets/models/*.glb`, `build.rs`, `models.rs`, `tools/art/blender/` | Blender 4.0 bpy 스크립트가 원본(상자·둥근 덩어리·깎은 돌·돌린 꼴·꺾인 관 + 뼈에 붙인 강체 + 128×128 아틀라스, 캐릭터 몸은 Meshy 메시). 미리보기 `docs/art/previews/`, 검사 `check_glb.py`, 리뷰 표 `review_glb.py`. 없으면 상자 |
| 이펙트(0141) | `fx.rs`(`Fx::on_events`·`watch`·`tick`, `brazier_ember`), `backdrop.rs` `Backdrop::fx`, `render.rs` `gem`·`spark4`·`crescent`·`slash_arcs`, `app/play.rs` `step_world`, 랩 `render.fx`, `tools/test/fx.js` + 개발 키 F6 | 사건·상태 변화 → 입자(별·빛줄·먼지·네모 화소), 휘두름의 초승달, 화로 불티. 꾸밈만 상한, 빨강 없음, 히트스톱 동안 1/4 빠르기 |
| 남은 상자 걷기·바람(0142) | `tools/art/blender/props.py` `work_fire`·`work_jar`·`work_tent`, `render.rs` `stage_props`(생활 물건·줍는 것)·`loot_body`·`loot_front`·`shaded_tris`·`octa_faces`·`ball_faces`·`spike_faces`·`glow_disc`·`wind_material`·GLSL `sway`·`flame_sway`, `scenery::sway`(법선 칸 w = 바람 몫), `flora.rs` `LEAF_SWAY`, `backdrop.rs` `LEAVES`, 랩 `render.wind` | 0142 |
| 맞음 피드백(0143) | `world.rs` 맞은 틱의 히트스톱(`HURT_HITSTOP`·`HURT_HEAVY`)·`Player::since_hurt`·`hurt_blow`, `hud.rs` `hurt_edge_k`·`low_hp`·`draw_edge`·체력 칸, `render.rs` `HURT_WHITE`, `app/play.rs` 맞음 킥(`KICK_HURT_DEG`), 랩 `hurt.edge` | 0143 |
| 타격 줌·큰 휘두름(0145) | `camera.rs` `punch_in`·`zoom`·`freeze`, `app/play.rs`(사건 → 줌, `ZOOM_*`), `render.rs` `crescent`(두 겹)·`slash_inner`·`SLASH_GLOW`·`SLASH_EDGE`·`HEAVY_SLASH_FADE`, 랩 `feel.zoom`·`cam.quarter.dist` 15 | 0145 |
| 터치 픽토그램(0146·0147) | `touch.rs` `icon_mask`(13×13)·`icon_cell`·`icon_px`·`icon`·`draw`(잉크 바탕·호박빛 공격 테)·`attack_icon`(든 무기 → 공격 그림) | 0146·0147 |
| 화면 다듬기(0140) | `scenery.rs` `MORTAR_CALM`·`patch`·`SHADE_RES`·`PROP_AO`, `render.rs` `AMBIENT_KEY`·`MOON_KEY`·`actor_moon`·`floor_shade`·`post`·`Grade`, 랩 `render.floor calm`·`key light`·`grade`·`bloom`·`vignette` | 조용한 바닥, 빛 : 그늘 2.5배·그림자 칸 0.125m·소품 밑 그늘·그늘에 선 배우, 톤 곡선·빛 번짐·가장자리 그늘(선 둘과 함께 내부 해상도에서 한 번) |
| 모델 리뷰·소품 v2(0139) | `tools/art/blender/common.py` `Geo.stone`·`lathe`·`limb`·`k_*` 칸, `props.py`, `review_glb.py`, `render.rs` `stage_props` | 소품 여덟(벽 조각·기둥·뿌리·화로·문·핵·종·돌무더기)을 0061 기준으로 다시 빚음 — 돌 쌓기 무늬 상자 대신 큰 덩어리 기하 + 단색, 둥근 것은 매끈하게·뿌리와 테만 각지게. 상자로 그리던 장신구 상자·새긴 돌·문 장치를 모델로(`chest`·`stone_tablet`·`lever`, 없으면 예전 상자) |
| 지형·빛 | `scenery.rs`, `render.rs` `lit` | 한 번 굽기. 바탕 + 달빛 + 화로(7m, 거리 제곱). 배우는 1.35배 밝게. 가리는 벽 칸·소품은 옅게 |
| 오래 켜기(0020) | `tools/test/leak.js` | macroquad 텔레메트리가 텍스처를 흘리던 것을 걷어 냄. 탐침: 살아 있는 텍스처·버퍼·wasm 메모리를 10초마다 — 스테이지 120초·잡몹 80 90초 평평 |
| 손맛 손질(0020·0023·0028·0029) | `camera.rs`, `world.rs`, `render.rs` | 흔들림은 각(거리 무관), 맞으면 몸이 젖혀짐, 회피는 적을 뚫음. 땅 돌리기 = 캐릭터 기준 90°: 몸이 제 오른쪽(왼쪽)으로 90°, 화면도 늘 그쪽으로 90°, 쥔 스틱 그대로 새 정면으로 직진(`world::plan_turn`, 0028 — 0023은 화면이 0°·90°·180°로 갈렸다). 쏟기는 짧은 휘청만(도는 동안 붙잡지 않음, 밀림 0.25m·휘청 0.3초·벽 쿵 피해 없음, 0029) |
| 글꼴 | `assets/fonts/GowunBatang-UI-*.ttf`, `tools/fonts/subset_ui.py` | 0021: 후보 열하나(`docs/art/font-candidates-{pixel,serif}.png`) 중 사장이 고운바탕·함렛으로 좁힘 → 호환성으로 고운바탕(한글 11,172자·굵기 따로 된 파일). 표 글자만 서브셋 두 파일 190KB(Galmuri 330KB보다 작음) |
| v0.1.0·APK(0022) | `.github/workflows/build-android.yml`·`release.yml`, `third_party/miniquad`, `android/res`, `tools/icons/make_icons.py` | nail CI를 가져옴(비공개 에셋 단계 뺌). Android 패드는 nail의 패치 miniquad. 갈래: main = 운영(개발 기능 없음), `release/v0.3` = 개발판(v0.2.0 배포 뒤, 웹 별칭 `dev`·APK 미리 보기 `dev`) |
| 키 배치·다시 붙이기(0024) | `input.rs` `Bindings`·`KEYS`·`Pad`, `app/controls.rs`, `prefs.rs`, web/index.html | 소울라이크식(회피 B·Space, 패링 LB·K, 돌리기 LT·RT·Q·E, 락온 R3·L), 공격만 X·J. 타이틀·멈춤의 "조작 설정"에서 바꾸고 언어와 함께 저장 |
| 패링(0025) | `world.rs` `PState::Parry`·`tick_enemies`, `enemy.rs` `exposed`·`parryable`, `render.rs` | 창 0.18초·앞쪽 근접만·헛치면 0.5초 묶임·스태미나 15. 튕기면 휘청 + 드러남(다음 한 대 치명타 +2). 파동은 못 받음(예고 가운데 X). 터치 방패 버튼 |
| 점프·내려찍기(0027) | `world.rs` `PState::Jump`·`Land`·`plunge`, `tuning.rs` `Jump*`·`Plunge*`, `render.rs`, `wanderer.py` | A / F. 높이 0.9m·체공 0.55초(탄도 정확 적분)·스태미나 12, 뛰기 전 속도를 잇고 조종 ×0.3. 0.35m 위면 뿌리 파동을 넘음(무적은 없음). 공중 공격 = 내려찍기(×2.5로 떨어져 둘레 1.6m에 2, 착지 빈틈 0.35초). 그림자가 높이로 작아짐. 소모품 퀵슬롯은 뺌 |
| 강공격(0030) | `world.rs` `PState::Charging`·`Heavy`·`Strike`, `input.rs` `attack_held`, `touch.rs`, `tuning.rs` `Heavy*`, `wanderer.py` charge·heavy | 평타는 누르는 순간, 한 번 누른 채 0.25초 쥐면 모으기(연타는 새지 않음), 떼면 늘 강타. 다 모으면(0.5초) 피해 3·잡몹 휘청 0.8초·보스 0.25초, 덜 모으면 2·0.4초. 범위 2.2·160°·넉백 ×2. 발밑 고리가 차오르고 다 차면 금빛 깜빡. 등잔 마시기는 Y / R(0048) |
| 등잔(0031) | `world.rs` `PState::Drink`, `input.rs` `Bind::Flask`, `prefs.rs`, `hud.rs`, `touch.rs`, `tuning.rs` `Flask*`·`FLASK_CHARGES`, `wanderer.py` drink | R / Y 한 번 = 한 모금(0048)(0.8초, 0.35초에 삼켜 체력 +2), 셋. 삼키기 전 회피·피격이면 등잔 그대로, 연타·가득·0은 안전하게(누름 버림·깜빡만). 쉼터(0032)에서 쉬면·스테이지를 새로 시작하면 다시 참 |
| 쉼터(0032) | `stage/mod.rs` `rest`·`respawn`·`StageOut::Respawned`, 지도 `R`, `input.rs` `Bind::Interact`, `hud.rs`, `touch.rs` | 입구·문 앞 둘. 곁에서 점프 버튼(0048) = 체력·등잔 가득 + 다시 일어설 곳(깨어 있는 적 10m 안이면 못 쉼). 쉬었으면 쓰러져도 그 쉼터에서 — 진행은 쉬던 때로(물결 처음부터 / 문 앞·문지기 가득). 쉰 쉼터는 기록에 남는다(0064 M2, 슬롯 0066) |
| 모델 다듬기 1(0033) | `render.rs` `render_target_ex(depth)`·`Look`·`lit`·`rim`·`Batch::hull`·`outline_material` | **3D 렌더 타깃에 깊이 버퍼가 없었다**(그린 순서로만 덮임 — 부위 상자·벽이 엉켜 보이던 원인) → 붙임. 툰 빛(달빛 2단·화로 3단)·가장자리 빛 0.35·배우 1px 외곽선(키운 상자 뒷면), 랩 `render.toon`·`render.rim`·`render.outline px`. 외곽선은 배우 정점 ×2 |
| 방랑자 동작(0034) | `wanderer.py`, `render.rs` `move_anim`, `world.rs` `turn_is_right` | 락온 게걸음·뒷걸음, 모으기(두 손·낮게, 다 모이면 떨림)·강타(내디디며 낮게), 땅 돌리기 휙(낮췄다 윗몸이 먼저, 몸 돌기 천천히-빠르게-천천히), 등잔을 왼손에 들고 마시기 |
| 배경 높낮이 1단계(0035) | `stage` `StageDef::heights`·`Level::with_heights`, `scenery.rs` 바위 단·꺼진 땅, 성소 `HEIGHTS` | 그림만(월드는 평평). 성소 둘레 +2~+7m 바위 단, 입구 다리 양옆 −2m 낭떠러지. 외곽선 껍데기를 카메라에서 0.18m 밀어 작은 배우의 안쪽 선을 줄임 |
| 귀여운 방랑자(0036) | `common.py` `Geo.blob`·파스텔 재질·아틀라스 128, `wanderer.py`, `main.rs` 인덱스 용량 | 2.3등신·둥근 라벤더 후드(곰 귀)·얼굴(눈·볼·앞머리)·민트 망토, 매끈한 음영(정점 공유), 디테일(후드 테·꽃 핀·단추·허리의 작은 종·끝동·장화 접단). 그리기 인덱스 용량 5,000 → 15,000(넘친 몸·다리가 잘렸다) |
| 그림 방향 "색 = 생명"(0037) | `docs/art/direction.md`, `wanderer.py`(B 작은 동물), `common.py` `PAL`, `scenery.rs`·`render.rs` 색 | 방랑자 = 두건 쓴 너구리(2등신·눈 크고 낮게·입 없음·꼬리 흔들림). 세계는 해 질 녘 남보라, 침식은 잿빛 + 보라 뿌리, 외곽선 자두빛. 모델 열둘 다시 내보냄. 적 다시 빚기는 2단계 |
| Meshy 방랑자(0038) | `tools/art/sources/meshy/`, `meshy_convert.py`, `common.py` `external`·`merge_textures`, `wanderer.py` | Gemini 그림 → Meshy(CC BY 4.0) → 줄이기·색 굽기 → 부위로 잘라 지금 뼈·동작 17에 붙임(정점 2,235). 걷기 뒤뚱·통통, 목은 두건 가장자리에서. 락온은 0040 |
| 실감나게 귀여운 동작(0039) | `secondary.rs`, `model.rs` `pose_with`, `render.rs`, `wanderer.py` `sq`, `motion_strips.py` | 원칙(KB `believable-cute-character-motion`): 판정 시각은 그대로, 끝맺음·부피 지키는 찌그러짐·늦게 따라오는 꼬리·머리(감쇠 스프링 층)·가속 쪽 기울기·착지 튐. 랩 `anim.*` |
| 락온 구도(0040) | `tuning.rs` `cam.lock.*`, `camera.rs` `occlusion`, `render.rs` `lock_fade` | 어깨 1.6·주시점 밀기 2.3·높이 3.0. 그래도 방랑자가 과녁을 가리면 0.45까지 옅게(랩 `render.lock fade`). 0009 전환 기준 1.8 → 2.1배 |
| 해상도 640×360(0041) | `tuning.rs` `INTERNAL_*`·`UI_REF_H`, `render.rs` `Frame::ui`, `ui.rs` `hints` | 720p ×2·1080p ×3에 꽉 참, 방랑자 20 → 27px. UI는 480×270 기준 배율 그대로. 조작 안내는 넘치면 두 줄 |
| UI 규칙(0042) | `ui.rs` `key`·`fit`·`wrap`·`check`, `app/mod.rs` 안내, `hud.rs` `under_menu` | nail 방식: 메뉴 안내만 + 키캡, 넘치면 "…", 긴 글은 줄바꿈, 개발 F4 넘침 검사(두 언어 0) |
| 걸을 수 있는 높낮이(0043) | `ground.rs`, `world.rs` `ground`·`collide`·`way`, `stage` `with_heights`·`ground`·`settle`, `scenery.rs` 옹벽·테, `sanctuary.rs` `HEIGHTS` | 경사로 + 0.5m 턱 점프(1m는 벽), 어디서든 뛰어내림, 적은 경사로로만(거리 지도), 높이 차 0.75m 넘으면 안 닿음. 성소 전체 높낮이 |
| 구운 달빛 그림자(0044) | `shade.rs`, `render.rs` `lit`·`set_bake_shade`·`LIGHT_DIR`, `scenery.rs` 빛 지도 | 0.25m 가림 높이 지도 + 광선, 바닥은 빛 지도 텍스처(픽셀 그림자), 벽·소품은 정점. 달은 왼쪽 앞 위 45° |
| 캐릭터 실시간 그림자(0045) | `render.rs` `shadow_pass`·`FLOOR_FRAGMENT`·`Batch::casters`, 랩 `render.actor shadow` | 달 쪽 정사영 가림판 256²(0.125m 픽셀)을 바닥 재질이 읽는다, 원판(접촉)은 그대로 |
| 전체 화면(0046) | `main.rs` `fullscreen`, `third_party/miniquad/java/MainActivity.java`, `render.rs` `fit_internal`, `screen.rs`, `web/index.html` `knell_fs_*` | Android 11+ 몰입 모드(바 숨김, 쓸면 잠깐). 내부 해상도를 화면에 맞춤(정수 배율 그대로, 테두리 < 배율, 4:3~21:9). 타이틀 "전체 화면" 줄(웹·데스크톱), 터치 웹은 "시작"에 전체 화면 + 가로 |
| 모으며 걷기·공중 돌기(0047) | `world.rs` 이동·바라보기, 랩 `heavy.charge walk x` | 모으는 동안 걷기 ×0.4, 점프 중에도 스틱·조준·락온 쪽으로 몸이 돈다(내려찍기 시작 뒤 고정) |
| 버튼 재배치·마법(0048 1단계) | `input.rs` `Bind::Cast`, `prefs.rs` MAGIC 2, `world.rs` `PState::Cast`·`Bolt`·`tick_bolts`·`interact_here`, `stage` 쉼터, `hud.rs` 마법 칸, `touch.rs` 별 버튼, 랩 `cast.*` | RB = 마법(조준 스틱·자동 조준, 칸 3, 근접 한 대에 0.25칸, 쉼터 가득), 등잔 Y, A = 점프 + 상호작용(쉼터 곁·안전할 때). RB 직후 0.25초 돌리기 누름 버림 |
| 글자 아틀라스(0049·0054) | `ui.rs` `BAKE`·`bake_size`·`begin` | 화면과 상관없이 한 크기(물리 ≈48px)로만 굽고 `font_scale`, 본문만 미리 굽기 — 창을 바꿔도 1024²(Android 검은 글자 대책) |
| 락온 구도(0050) | 랩 `cam.lock.*` | 어깨 0.8·주시점 밀기 1.0·높이 4.5 — 내려다보는 각 19°, 방랑자는 가운데에서 조금 왼쪽 |
| 무기 넷·콤보·자세 막대(0051) | `weapons.rs`, `world.rs` `PState::Attack{hit}`·`Throw`·`Crack`·`Bomb`·`add_poise`·`tick_bolts`, `enemy.rs` `poise`, `render.rs`, `hud.rs`, `wanderer.py` 무기 부위·동작 | 검 3연타 · 망치(2타·뛰어 내려찍기 + 여진·깨지기 직전 깨기) · 지팡이(4연타·딱 맞춘 떼기·꿰뚫는 탄) · 쇠뇌(RB 탄창·장전·회피 장전, X 수류탄). 멈춤에서 바꿈 |
| 폭딜 창·수류탄 겨누기(0055) | `world.rs` `hurt_by`·`Enemy::broken`·`poise_frac`, `hud.rs` 보스 자세 막대, `PState::Throw` 겨누기 | 깨진 동안 모든 타격 ×1.5, 보스 띠 아래 자세 막대(깨지면 금빛), 쇠뇌 X 꾹 = 겨눠 멀리 |
| 원거리 강공격(0056) | `world.rs` `PState::CastCharge`, `weapons.rs` `charged`·`Bolt::scale`, `input.rs` `cast_held`·`carry`, `touch.rs` 마법 손가락, `wanderer.py` cast_charge·xbow_charge | RB 톡 = 한 발, 쥐면 모으기 → 떼면 모은 탄(칸·볼트 하나, 다 모으면 크고 꿰뚫음) |
| 캔슬(0057) | `world.rs` `cancel_or_buffer`·`Player::pending`·`Next`, `weapons.rs` 타·강타 `cancel`, 랩 `dodge.cancel x`·`land.cancel x` | 후딜레이를 다음 행동(회피·패링·점프·공격·마법·등잔)으로 끊음, 0.3초 받아 두기, 끊는 시점은 무기 성격(망치 늦게) |
| 설정·개발 메뉴(0052) | `app/flow.rs` `Scene::Settings`·`Dev`·`SettingsRow`, `app/settings.rs`, `prefs.rs` 칸 20·21 | 타이틀 = 시작·설정·DEV. 설정 = 언어·전체 화면·흔들림·조작. 멈춤 = 계속·무기·설정·처음부터·타이틀로 |
| 터치 배치(0053) | `touch.rs` `layout`·`Ctx`, `tools/test/touch.js` | 공격 + 둘레 호 넷(회피·패링·마법·점프/쉬기) + 돌리기 캡슐, 락온·등잔은 쓸 수 있을 때만 |
| 웹 로딩·PWA(0062) | `web/index.html` 덮개·`fetch` 감싸기·`knell_loading`, `src/main.rs` `loaded`, `web/manifest.webmanifest`·`sw.js`·`icons/`(`make_icons.py` `web_icons`), `assemble.sh`, `tools/test/pwa.js` | 종 + 막대(wasm 바이트, 첫 프레임 뒤 걷음, 실패면 붉게), 홈 화면 설치(가로 전체 화면), 두 번째 실행부터 오프라인, 새 판은 다음 실행부터 |
| 주위 환경(0063) | `backdrop.rs`(`fog`·`motes`·`sky`), `flora.rs`, `scenery.rs` `walk_min`·식생 몫, `render.rs` `haze_by_pitch` | 걷는 땅 아래 높이 안개(흐르는 점 무늬), 반딧불·달빛 먼지, 숄더뷰 하늘 띠·산등성이 세 겹·뿌리 아치·달(남서 골짜기), 바위 단의 나무·덤불, 벽을 타는 뿌리, 아지랑이는 쿼터뷰에서만 |
| 기록(0064 M2 → 슬롯 셋 0066) | `save.rs`, `app/progress.rs` `continue_run`·`sync_progress`·`persist`, `app/slots.rs`, `StageRun::resume`·`run_at`, `web/index.html` `knell_rec_*`, `tools/test/save.js` | 타이틀 [이어하기]·시작하기 → 슬롯 창(슬롯 셋, 요약, 덮어쓰기·지우기 두 번), 쉰 쉼터에서 이어함, 쉬면·판을 끝내면·처음부터 하면 바뀐 것만 씀, 든 무기도 슬롯에 |
| 잔불(0064 M1) | `ember.rs`, `world.rs` `tick_embers`·`embers`·`drops`, `species.rs` `embers`, `backdrop.rs` `embers`, `hud.rs` | 쓰러지면 튀어나오고 0.35초 뒤 끌려와 주움, 가진 수는 기록에 — 쓰러져도 그대로 |
| 앱·UI 뼈대(0066) | `app/stack.rs`·`flow.rs` `Go`·`Layer`·`focus`, `app/menu.rs` `Menu`·`Out`·`Hit`, 화면 파일 `pause`·`settings`·`shop`·`ask`·`slots`·`title`·`play`·`progress`·`controls_screen`, `ui.rs` `panel`·`list`·`close`·`buttons`·`title_arrows` | 장면 스택(닫으면 연 줄로), 목록 화면 = 줄 표 하나 + 몰이꾼, nail 창 틀(✕·창 밖 탭·◀ 값 ▶·제목 줄 ▲▼), 묻는 창은 아래 창 위. `tick_scene` 340 → 35줄 |
| 아이템 그림(0068) | `item_view.rs`, `Renderer::draw_icon`·`draw_item`, `tools/art/blender/charms.py` → `charms.glb` | 칸·후보 줄의 아이콘(한 번 찍어 캐시), 고른 것은 크게 흔들리는 3D, 장신구 모델 여섯(첫 시안) |
| 이어 패링 쌓기(0086) | `world::parry_chain_add`·`parry_front_cos`, `Player::parry_chain_left/n`, `tuning::PARRY_CHAIN_DECAY`·`PARRY_CHAIN_WIDEN_DEG`, 렌더 이음 반달 | 이음 중에 또 튕기면 남은 창 + 0.2 × 0.5ⁿ, 받는 반원 반각 +45°씩(둘레 전부까지), 열린 동안 넓이만큼 푸른 반달 |
| 박자 부채꼴(0087) | `world::beat_shrink`·`World::beat_cue`·`BeatCue`, `weapons::Sweep`·`SWEEPS`, `tuning::BEAT_CUE_*`, 렌더 공격 부채꼴 | 휘두른 부채꼴이 다음 박자까지 줄어 박자에 가장 작고(창 안 금빛 + 테두리), 칼이 지나간 쪽부터 사라짐(가로 베기는 시작한 쪽, 찌르기·세로는 양 끝부터), 마지막 타엔 없음, 쇠뇌는 좁은 부채꼴 |
| 기력 막힘 알림·패링 기력(0088) | `Event::StaminaDenied`·`Player::denied_new`, `hud::draw_sweat`·막대 떨림·다 쓴 동안 붉은 테, 랩 `parry.stamina gain` | 막히면 "안 됨" 소리 + 막대 깜빡·떨림 + 머리 위 땀방울(누름마다 한 번), 튕기면 기력 +25(마지막 기력으로 튕겨도 반격이 나감) |
| 모으기 공격의 패링(0089) | `Player::release_guard`·`World::parry_open`, 렌더 패링 반달 | 강타·쇠뇌 모은 사격을 떼는 순간 0.18초 패링 창(앞 반원) — 튕기면 강타는 그대로 나가 드러난 적에게, 반격·근접 무기의 모은 마법엔 없음 |
| 멈춤 창 개발 줄(0090) | `flow::pause_items` | 개발판 멈춤 창에서 연구소를 빼고 그 자리(설정 아래)에 개발 줄 — 연구소는 타이틀·개발 창 도구 탭 |
| 문도 시야 가리면 옅게(0091) | `render::gate_box`·`draw_gate`·`Renderer::gate_fade`, `Level::settle`(문 상자도 땅 높이) | 지형 덩어리와 같은 판정·진하기로 문이 옅어짐(쿼터뷰·락온), 문 상자가 땅 높이(+1m)에 맞게 올라감 |
| 연구소는 개발 창 안에만·소품 가림 상자(0094) | `flow::title_items`, `scenery::vert_bounds` | 타이틀·멈춤에서 연구소 줄을 빼고(개발 창 도구 탭에서만), 소품의 가림 상자를 구운 정점 범위로(회전·큰 모델·돌무더기도) |
| 패링 기력 15 → 5(0097) | `tuning.rs` `parry.stamina cost`, `docs/tuning-frozen.txt` | 빗나간 패링은 타이밍·피해로 이미 치르니 기력은 연타를 막는 만큼만 — 성공 +20·실패 −5 |
| 박자 금빛 옅게(0098) | `render.rs` `BEAT_GOLD`·박자 부채꼴·발밑 고리, `hud.rs` 음표·반짝임 | 노랑을 흰빛 쪽으로 + 진하기 30% 안팎 내림(딱 맞춤·락온 표시는 그대로) |
| 경직 이음 창(0099) | `world::stun_chain_next`, `Player::stun_window`, `tuning::STUN_CHAIN_GRACE` | 경직 + 0.5초 안에 또 맞으면 경직이 그전의 반(맞을 때마다 창 새로) — 몰매 속에서도 맞는 사이 틈이 커진다 |
| 적의 턱 뛰기(0100) | `nav::Step`·`hop`·다익스트라 `flow`, `enemy::Hop`, `tuning::ENEMY_CLIMB`·`ENEMY_DROP`, 거리 지도 캐시 | 걷기·뛰어오르기(1m)·뛰어내리기(3m)를 길로, 변 앞에서 포물선으로 뛴다(오를 때 0.42초·내릴 때 0.32초), 경사로가 가까우면 경사로, 보스는 걷기만 |
| 땅 돌리기 시작 CC·착용 중 표시(0085) | `World::apply_turn` 시작 휘청, 랩 `turn.start stagger s`, `equip::wears_something` | 돌기 시작하면 범위 안 적 0.25초 휘청(보스는 무게만큼 짧게, 예고 끊김), "착용 중"은 낀 물건에만(빈 칸·맨손은 커서만) |
| 바닥 표시 땅 따라·땅 돌리기 휘청(0084) | `render::Drape`(`rise`·`shadow`·`line_up`, 평평한 `ring`·`sector` 없앰), `Level::settle` 문 가운데, `camera` `wobble`·`wobble_at`·`roll`, 랩 `turn.wobble in/out deg` | 그림자·쉼터·상자·갈 곳·플레이어 표시 32곳이 경사로·턱을 따라 접힘(가운데의 땅 위 높이는 지킴), 땅 돌리기 시작엔 도는 쪽·끝엔 반대쪽으로 1.5° 휘청 |
| 웹 가로 기본(0082) | `web/index.html` `#rotate`·`knell_portrait`, `screen::portrait_blocked`, `tick_play` | 세로로 든 터치 기기엔 회전 안내(싸움은 멈춤), 누르면 Android는 전체 화면 + 가로 잠금(안 먹히면 세로로 계속), iPhone은 세로로 계속 |
| 슬롯 요약·장비 줄·이어 패링(0081) | `save::Summary`(장·`play`)·`play_text`, `app/slots.rs`, `app/pause.rs`, `tuning::PARRY_CHAIN`·`World::parry_open` | 이어하기 줄 = 장 · 지역 · 플레이 시간(새 키 `play`), 멈춤 장비 줄은 들어가는 줄만, 튕긴 뒤 0.2초 동안 앞쪽 공격도 튕김(튕길 때마다 다시) |
| 개발 화면 한국어(0080) | `i18n/dev.rs`(`Pair`·`t`·`SPEC_KO`·`GROUP_KO`·`SCENARIO_KO`), `app/lab_screen.rs`, `app/settings.rs` 개발 창 | 연구소·DEV 창 글이 설정 언어를 따른다, 연구소 값은 한국어 이름 + 설명 줄에 원래 이름, F3 줄만 영어 |
| 연구소 줄·개발판 커밋 해시(0079) | `flow::title_items`·`pause_items`(`Lab`), `build.rs` `BUILD_HASH`, `title::version_label` | 연구소(랩)는 타이틀·멈춤 메뉴의 설정 바로 아래 줄(개발판, 설정 창에서는 뺌), 개발판 타이틀 버전 옆에 커밋 앞 7자 |
| 딱 맞춰 떼기는 모든 무기(0078) | `tuning::PERFECT_RELEASE`, `world` 강타·모은 사격 `perfect`, `Player::since_perfect`, 렌더 금빛 고리, `Event::Perfect` → 종 `Beat3` | 다 모인 뒤 0.15초 안에 떼면 피해 +1·자세 ×1.5(강타는 넉백 ×1.5도) — 근접 강타 전부와 쇠뇌 모은 사격, 무기 표 `perfect` 칸 없앰 |
| 박자는 모든 무기·장비 목록 커서(0077) | `equip::candidates`·`has_choice`·`pick_start`, `world` 쇠뇌 사격 박자·`on_beat`·`BEAT_WINDOW_MIN`, `hud` 탄창 반짝임 | 모든 칸 목록에 낀 것(착용 중), 커서는 거기서, 견주는 줄은 가장 긴 것만큼 비움. 쇠뇌는 사격이 박자(한 발 0.24초, 박자 사격 = 볼트 안 씀), 창 바닥 ±60ms, 지팡이 설명에서 박자처럼 읽히던 말 뺌 |
| 길찾기·돌진 띠·박자 피드백·맨손·달리기(0076) | `nav.rs`(`Nav`·`clear`·`waypoint`·`charge_room`), `Enemy::spread`·`charge_len`·`touch`, `Player::beat_chain`·`since_beat`·`swing_beat`·`running`, `hud::draw_beat_note`, `sfx::beat_id`, `weapons::Id::Fists`, `equip::pick_start`, `ListView::tapped`, `tuning::RUN_*` | 적은 상자(기둥·소품·문)를 아는 격자로 몸 폭 끈 당기기 + 옆으로 퍼지기, 돌진 띠 = 맞는 곳(폭·끝·쓸고 간 선분·벽에서 멈춤), 박자 타 = 금빛 휘두름·발밑 고리·음표·스태미나 막대 반짝임·오르는 종소리, 무기 후보에 든 것(착용 중)·맨손, 회피 꾹 = 달리기(×1.45, 18/s, 달리다 치면 대시 공격), 터치 강공격·모으기 되살림 |
| 개발 창(0075 → 0083) | `Scene::Dev { tab }`·`app/dev.rs` `DEV_LIST`·`DEV_CATS`·`App::dev_run`·`flow::pause_items` | nail 구조: 위 묶음 탭(도구·스테이지·전투·적·성능) · 왼쪽 줄 · 오른쪽 설명 · 아래 안내, 개발 판은 슬롯에 안 씀, 멈춤 끝에 개발(개발판), 시나리오 8 husk + thorn |
| 공격 예고 설정(0074) | `SettingsRow::Tells`·prefs 칸 26·`View::tells`·`render::TELLS_BASE` | 설정 → 공격 예고 0~100%(0 = 끔), 기본 50%, 70% = 0072 진하기 |
| 적 준비 동작(0073) | `render::STRIKE_KEYS`·`windup_clock`·`windup_hold`·`WINDUP_*` | 들어 올림(예고 60%, smoothstep) → 정점에서 버팀(떨림·눈/핵 번쩍) → 마지막 0.08초에 내리침 = 맞는 틱, 늘어남·뒤로 젖힘, 예고 시간·판정은 그대로. 동작 원본 과장(움츠림 → 까치발 젖힘·늘어남 → 납작): 잔재 attack·가시짐승 charge_windup·문지기 slam·wave·charge_windup |
| 옅은 예고·표시(0072) | `render::telegraph`·`TELE_A`·`TELE_BASE_A`·`TELE_MARK_A`·`TELE_RING_W` | 적 예고 채움 = 테두리 투명도 0.3(겹치지 않게 — 채움은 테두리 안쪽까지), 파동·평타·강타·패링 반달·패링 성공 고리도 옅게 |
| 박자·무기 숫자(0071) | `world::on_beat`·`Player::since_swing/beat/on_beat`·`BEAT_WINDOW`, `Event::Beat`·`sfx::Id::Beat`, `app/equip.rs` `weapon_numbers`·`weapon_compare` | 무기 박자 = 그 타의 한 스윙 시간, 박자 ±15% 안에 누른 평타 콤보는 스태미나 0(받아 둔 이른 누름·첫 타·대시·반격은 씀), 맞으면 '팅'. 장비 창: 무기 칸이면 피해·공격 속도·거리·강타(쇠뇌는 사격·탄창·장전·수류탄), 무기 후보는 지금 무기와 ▲▼ |
| v0.3.x 손맛·조작(0070) | `world.rs` 패링 감속·`handoff_target`·대시 공격 `weapons::DASH`·패링 반격, `app/equip.rs` `slot_nav`, `tuning.rs` `LOCK_HANDOFF`·`DASH_ATTACK_LATE`·`RIPOSTE_WINDOW` | 패링은 멈출 때처럼 감속(뒤로 출렁이지 않게)·막기 자세 앞으로, 락온 대상이 쓰러지면 8m 안 다음 적으로(없으면 풀림), 장비 창 십자 = 커서·땅 돌리기 = 갈아 끼우기, 회피 끝·직후 공격 = 무기별 대시 공격(콤보 둘째 타로), 패링 직후 공격 = 다 모은 강타, 평타 타격감 층(히트스톱 ×1.6·카메라 킥·맞은 쪽 젖힘·떨림·불똥·공격자 멈춤·둘째 소리 `Thwack`), 돌진 띠는 몸이 닿는 순간 다 참(`Enemy::charge_eta`). Select(Tab) = 장비 창 바로, 후보에서 그 칸에 낀 것은 뺌(0070 덧붙임). 연구소(랩)는 타이틀·멈춤 메뉴의 설정 바로 아래 줄(개발판만, `app/lab_screen.rs`, 묶음 열여섯·A/B·내보내기, 값은 닫아도 계속 — 0070 7·15 → 0079), 개발판 APK 패키지 `io.github.aimenjoe.dev.knellroot`(마지막 조각 = 라이브러리 이름, 0070 16), Android 에뮬레이터 연기 시험 `android-smoke.yml` |
| 장비 창(0067 D) | `app/equip.rs`, `Renderer::draw_doll`, `stats::lines`, `Scene::EquipPick` | 디아블로 콘솔식: 가운데 방랑자 인형(든 무기), 칸 넷 → 후보 목록, 바뀌는 상태 `값 → 값 ▲▼`, 후보는 가진 것만(못 얻은 것은 도감 몫, 0068) |
| 장신구(0064 M3) | `stats.rs`·`charms.rs`, `World::stats`·`apply_stats`, `Player::max_hp`·`flask_max`, `StageRun` 상자(`C`)·`take_opened`, `app/pause.rs` 장비 창·`app/progress.rs` `gain_charm`·알림 | 세 칸, 여섯 가운데 상자 둘·문지기 하나에서 얻음, 멈춤 → 장비에서 갈아 끼움, 능력치는 한 곳에서 합침 |
| 성장(0064 M4) | `growth.rs`, `Scene::Core`·`Scene::Shop`, `StageRun::take_rested`·`shop_ready`, `app/progress.rs` `buy`·`app/shop.rs` `core_menu`·`shop_menu` | 문지기를 이기면 핵 고르기(체력·등잔·회복), 쉬면 화로 가게(장신구 셋·등잔 기름), 살 것이 있으면 쉼터가 빛남 |
| 소리(0064 M5) | `sfx.rs`, `app/play.rs` `world_sounds`·`app/mod.rs` `set_sfx`·메뉴 소리, `prefs.rs` 칸 25, `assets/sfx/` | 효과음 16(CC0)·합성 8(패링·예고 넷·깨어남·쉼터·등잔)·성소 환경음, 설정 소리 크기 |
| 묻는 창·UI 맞추기(0065) | `Scene::Ask`·`flow::answer`·`ask_default`, `ui.menu_dimmed`, `Sfx::played` | 새로 하기·처음부터·타이틀로·사기는 한 번 묻는다(취소에서), 가게는 못 사는 것 흐리게·HUD 없음, 메뉴 소리 겹치지 않게, 저장 실패 알림 |
| 나무와 스킬이 하나·기술 화면(0161) | `skills.rs` `ENTRIES`·`Board::cells`, `progression.rs` `Tree`(칸마다의 비트)·`CELL_ROWS`, `save.rs` `tree.cells.*`(옛 `tree.*` 읽기), `app/skills.rs`(`Scene::Skills { edit }`·`SkillAct`)·`app/frame.rs`, `icons.rs`(패시브 그림 15) | 화폐 둘(잔불 = 몸과 장비 / 기술 점수 = 기술 — 레벨 + 엘리트 + 핵 + 조각), 갈래 셋 × 줄 셋의 판 하나에 스킬 여덟·큰 기술 셋·패시브 열다섯, 화로에서 올리고 되돌리고 멈춤에서는 끼우기만, 옛 나무 창·기술 판·"나무" 줄은 없앰 |
| 전투 스킬 S0(0151) | `skills.rs` 표, `world/skill_tick.rs`, `PState::Skill`·`Ult`, `world::weapon_buttons`, `input.rs`(`Bind::Skill1`·`Skill2`, `Pad::DLeft`·`DRight`), `hud.rs` `draw_skills`, `touch.rs`(스킬 버튼·그림 여섯), `app/skills.rs`(`Scene::Skills`), 기록 `skill.1`·`skill.2`, 랩 `skill.*`, 시나리오 "기술 연습" | 스킬 넷(휘돌이·가로지르기·끌어모으기·땅울림) + 큰 기술 쟁반 뒤집기(게이지 — 평타·패링으로만 찬다, 두 버튼 함께 = 시동 안의 바꿔치기), 기세(쓰러뜨리면 대기가 당겨진다), 땅 돌리기는 십자 ←·→ / Z·C로(뜻 그대로), 쇠뇌는 X 사격·RB 수류탄. 판정 전(개발판) |
| 개발 | 타이틀 "DEV: skip to the sanctum gate"·"DEV: PoC scenarios"(1~7, Esc = 타이틀), 랩(멈춤 끝 줄 LAB·F1, 0070), F3 개발 줄, F4 글자 넘침 검사 | 운영판(`--no-default-features`)에는 없다(F3 줄은 있음) |

## 사람이 할 일
**0161 판정(개발판 — 나무와 스킬이 하나·기술 화면, 체크리스트 22번)**: 쉼터 화로 → 기술(둘째 줄)·멈춤 → 기술. ① 화면이 한눈에 읽히는지(22-1~3) ② 확인 → 작은 목록이 편한지(22-4) ③ 패시브가 스킬과 같은 점수를 다투는 것(22-7) ④ **이 생김새로 나머지 탭(상태·장비·도감·가게·설정)을 가도 되는지(22-10)** — 여기서 사장이 고르면 다음 걸음(창 하나 + 탭)으로 간다. 레벨을 잔불로 올리고 레벨마다 기술 점수가 생기는 것은 그대로 두었다("장비용 / 기술용"과 어긋나면 말해 주세요).

**0159 리팩터링 R1~R3(끝)**: `render/` 폴더·`world/`의 단계 파일·`world/tests/` — 동작은 그대로(시험 이름 478개 같음, 화면 차이는 찍는 도구의 잡음 안). 개발판에서 그림이 달라 보이는 곳이 있으면 R1 커밋(`9694336`)부터 본다. R4(`Player` 필드 묶기)는 0161 뒤.

**0158 스킬 리뉴얼(결정 — 만드는 중)**: 기술 점수(레벨업 + 엘리트·보스 첫 쓰러뜨림)로 판에서 배우고 올린다 · 스킬 하나 다섯 단(3단 = 둘 가운데 하나, 5단 = 맺음) · 쓴 점수 합으로 줄이 열린다 · 화로에서 잔불을 내고 되돌린다 · **체력·피해 ×10, 내 체력은 막대**. 단 수와 나무의 모양은 다른 ARPG 스터디로 정한 것 — 뒤집을 수 있다. 차례: N0 눈금 → G0 판 → G1 고르기·맺음(`docs/decisions/0158-…`). **N0 만듦(2026-11-12)**: 체력·피해·회복 ×10(몇 대에 쓰러지나는 그대로 — 시험·모의로 확인), 내 체력은 막대(잃은 몫이 잠깐 밝게 남는다). 판정: 막대의 길이·읽힘, 눈금(10마다 금)이 필요한가 — 그림 `docs/art/n0-0158/`. **G0·G1 만듦(2026-11-12)**: 화로 → 기술(기술 판 — 점수로 배우고 올린다, 3단 고르기는 좌우로 바꾼다, 되돌리기는 잔불) · 스킬 넷의 다섯 단(고르기 여덟·맺음 넷) · 쟁반 뒤집기 세 단 · 멈춤의 기술 창은 배운 것 끼우기와 단 읽기. 개발 창 도구 탭 "기술 다 올리기"·"고르기 바꾸기"로 바로 해 볼 수 있다. 판정은 체크리스트 20번 — 그림 `docs/art/board-0158/`. 남은 것: 나머지 스킬 넷·큰 기술 둘(G2), 나무 넷째 갈래·장신구·떼의 수(G3).

**0155 판정(개발판 — 무기·장신구·두건)**: 무기 넷이 무기로 읽히나(검의 잎 꼴 날·망치의 통 머리·지팡이의 결정 갈래·쇠뇌의 굽은 활 — 장비 창에서), 장신구 여덟의 새 꼴, **두건은 원본으로 되돌렸다(0157)** — 덧붙인 띠·꼭지·자락이 차례로 넥타이·요정 모자·투구로 읽혔다. 고치는 길은 사장이 고른다: 뒷모습 그림으로 Meshy 다시 받기 / 모델러에게 맡기기 / 그대로(`docs/decisions/0157-…`). 그림 `docs/art/items-0155/`.

**0153 판정(개발판 — 방랑자 모델·서 있는 몸짓)**: 꼬리가 등 가운데로 왔는지·세운 정도, **새로 더한 두건 꼬리(금빛 종)와 허리 주머니가 어울리는지**(빼기 쉽다), 가만히 서 있을 때의 몸짓 셋(둘러보기·털기·종 만지기 — 5초 뒤 첫 몸짓, 적이 가까우면 안 나온다)의 잦기. 그림 `docs/art/wanderer-0153/`.

**0152(개발판, 사장 "도감 메뉴에 들어가면 터치가 안 먹어")**: 고쳤다 — 폰에서 도감의 탭·줄·✕가 눌리는지 한 번 확인. "쇠뇌의 버튼이 반대"는 지금 빌드에서 재현하지 못했다(공격 = 사격, RB = 수류탄 — 키보드·터치로 찍음): 타이틀 오른쪽 위의 해시가 새 판인지 먼저 본다(웹은 탭을 모두 닫았다 다시 열어야 새 판).

**0151 판정(개발판 — 전투 스킬 S0, 체크리스트 16번)**: 개발 창 → 전투·적 → "기술 연습(무리 24)"에서 ① "모으고 → 쓸고"가 되는지(16-1) ② 조작 옮기기(땅 돌리기 = 십자 ←·→ / Z·C, 스킬 = LT·RT / Q·E, 쇠뇌 X 사격·RB 수류탄 — 16-5·16-6) ③ 큰 기술(두 버튼 함께, 16-4) ④ 이대로 v0.7.0으로 묶을지 접을지(16-9). **패드·터치·실기기에서는 아직 눌러 보지 못했다**(헤드리스는 키보드뿐). 조작이 바뀐 일이라 v0.6.0을 main에 내기 전에 이 판정을 닫는다 — 접으면 S0 커밋 묶음을 되돌린다. 일곱 물음의 답과 만든 것은 `docs/decisions/0151-combat-skills-interview-and-s0-scope.md`.

**0148 판정(개발판)**: 락온 카메라의 새 값(15-1~15-3) — 다가선 느낌·거리 재기·돌 때의 어지러움. 랩에서 그전 값과 견준다.

**0146·0147 판정(개발판, 폰에서)**: 터치 픽토그램이 무엇인지 바로 읽히는지(14-2 — 등잔·조준), 어두운 바탕이 과하지 않은지(14-3), 공격 그림이 든 무기로 읽히는지(14-4 — 망치·지팡이·쇠뇌는 게임 화면으로는 아직 못 봤다).

**0145 판정(개발판)**: 쿼터뷰 거리 15가 맞는지(13-5 — 캐릭터 크기·땅 돌리기 어지러움·폰 화면), 타격 줌의 세기(13-7), 큰 초승달(13-8·13-9). 그리고 0143 맞음 피드백(13-1~13-4): 멈춤 길이·붉은 테 진하기.

**0142 판정(개발판)**: 체크리스트 12번 묶음 — 특히 ① 문지기 2단계의 뿌리 가시(싸움 안에서는 못 찍었다) ② 바람이 폰·핸드헬드에서 과하지 않은지·프레임 ③ 락온에서 보스 둘이 거칠어 보이는지(보이면 텍스처 1024×512).

**0138(방향만, 2026-11-10)**: 웹 호스팅은 나중에 Netlify → Cloudflare(Workers 정적 파일)로. Netlify 요금제는 **Legacy**(월 100 GB, 사장 2026-11-10 확인) — 급하지 않다. 사람이 할 일: 때때로 Netlify 대시보드의 이번 달 대역폭(nail과 같은 팀이라 합산) — 100 GB에 가까워지면 옮길 시점.

**0093 소개 페이지**: `release/v0.3`을 main에 넣은 뒤 ① 레포 Settings → Pages → Source = **GitHub Actions**(켜기 전 첫 실행은 실패, 켠 뒤 Actions의 pages를 다시 실행) ② DNS(Cloudflare)에 `knellroot` CNAME → `aimenjoe.github.io` ③ Settings → Pages → Custom domain = `knellroot.beardworks.io`, Enforce HTTPS. 페이지 글을 고치면 `python3 tools/site/subset_font.py <GowunBatang-Bold.ttf>`.

**v0.4.1 배포됨(2026-11-08, PR #7 → `a9f2925`)**: 0109~0113(넓어진 0번째 지역·서 있는 적·문 장치·엘리트·마법 쿨타임·공중 RB·레벨 값·마디·늦은 패링 고침). `release/v0.4`는 main까지 앞으로, `release/v0.5`에 main을 합침.

**v0.4.0 배포됨(2026-11-08, PR #6 → `9564892`)**: 운영 웹 `build.txt` = 그 커밋, Release `v0.4.0`에 `knellroot.apk`, `deploy-web`·`release`·`build-android` 모두 success. `release/v0.4`는 main까지 앞으로 맞춤. 고칠 것은 v0.4.x.

**v0.5.0 마일스톤 정함(0116, 갈래 `release/v0.5`)**: 인터뷰로 열여섯 물음 — 보라 뿌리 = 주인 없는 실(개발자 답), 떠난 이유는 게임 안 MYSTERY, 주민 = 여러 작은 동물("주민"), 포포 고리 걸어 둠, ECO-05 조건은 열고 행동 규칙만, ECO-10 같은 종이 다른 역할, 잔류형 정본(관찰), 마을 등급 2 = 체력 ×1.5·자세 ×1.3·잔불 ×1.6, 둘째 문지기 거리·공간형, 내려가는 길 + 되돌아가는 길, 도감 넣음, 주민의 불 = 아궁이·재방문 = 다음 판·공격이 지나감·말풍선. 이름: 떠난 마을·등딱지짐승·꼬투리·깨진 종지기. **다음: T1 지역 잇기부터**. 둘째 지역 설계는 0115(`docs/lore/regions/abandoned-village.md`·`docs/design/stages/abandoned-village-residents.md`).

**v0.5.0 배포(2026-11-10, PR #15 `release/v0.5` → `main`, 사장 Merge — Squash `082a781`)**: `Cargo.toml` 0.5.0·`version_code` 17·`docs/releases/v0.5.0.md`. Actions `release`·`build-android` success, Release `v0.5.0`에 `knellroot.apk`. Squash라 빨리 감기가 안 돼 `release/v0.5`에는 main을 합치고(트리 같음), `release/v0.6`에는 `-s ours`로 합침(내용은 이미 있음). 고칠 것은 v0.5.x(`release/v0.5`) 또는 v0.6에서.

**0149(개발판, 사장 "보스가 깨어나기 전에 타겟팅이 안 돼서 엉뚱한 적에게 락온")**: 깨어나는 2초 동안 보스가 락온 후보에서 빠져 범위 안의 다른 적이 잡히던 것 — `Enemy::waking` + `lockable()`로 깨어나는 중인 보스도 잡힌다(아직 안 깨기 시작한 잠든 보스는 그대로 안 잡힘). 판정 15-4.

**0148(개발판, 사장 "락온 했을때, 좀더 좋게 보이는 설정 찾아서 세팅해줘")**: 락온 카메라를 더 가까이·좁은 시야로 — 거리 7 → 6, 높이 4.5 → 4.2, 시야 58° → 52°, 앞 보기 3 → 4m(어깨·주시점 밀기는 0050 그대로). 방랑자 68 → 89px(쿼터뷰 56px의 1.6배 — 그전 1.2배), 보스 +20~27%, 내려다보는 각 17.3°. 후보 다섯을 네 장면에서 찍어 고름. 판정 15-1~15-3.

**0147(개발판, 사장 "어차피 종류는 정해졌으니까")**: 터치 공격 버튼의 그림이 든 무기를 따른다 — 검·망치·지팡이·쇠뇌·주먹(`touch::attack_icon`, `_` 없는 match). 판정 14-4.

**0146(개발판, 사장 "터치스크린의 아이콘도 좀 거시기 하네")**: 터치 버튼의 가는 선 그림을 13×13 도트 픽토그램(흰 몸 + 어두운 한 칸 테, 정수 px 칸)으로 — nail 0172 판정 11과 같은 길. 버튼 몸은 어두운 반투명 바탕 + 흰 테, 공격만 굵은 호박빛 테. 자리·크기·판정은 그대로. 판정 14-1~14-3.

**0145(개발판, 사장 "센 타격 줌이랑 더 큰 휘두름 … 거리 15")**: ① 쿼터뷰 거리 17 → 15(방랑자 몸 56px = 화면 높이 15.6%, 보이는 땅은 20 대비 −25%) ② 타격 줌 — 강타·자세 깨기 8.5%, 치명타·패링 6%, 딱 맞춘 떼기 3.5%만큼 카메라가 다가갔다 돌아온다(평타에는 없음, 히트스톱 동안 머묾, 랩 `feel.zoom`) ③ 휘두름의 초승달을 두 겹(넓은 빛 띠 + 밝은 날)으로 키우고 강타에도 — 바깥 변은 여전히 닿는 거리. 판정 13-5·13-7~13-9.

**0144(개발판, 사장 "응 제안대로 할께")**: 쿼터뷰 카메라 거리 20 → 17(얼린 값 `CAM_Q_DIST`) — 방랑자 몸 42 → 50px(화면 높이 11.7 → 13.8%), 보이는 땅 15% 줄어듦. 땅 돌리기의 회전 빠르기는 같고(각도의 일) 화면 위 흐름만 18% 빠르다 — 어지러움은 판정 13-5. 시험 `a_lower_pitch_turns_roll_into_pan`은 시선이 실제로 숙인 각으로 견주게 고침.

**0143(개발판, 사장 "맞았는지 잘 모르겠음 … 캐릭터가 너무 작아서")**: 맞은 틱에 다섯 겹 — 세계 멈춤(0.13초, 센 타격 0.18초, 이어 맞으면 반씩)·화면 가장자리 붉은 테(겹 띠, 랩 `hurt.edge`)·잃은 체력 칸이 희게 부서짐 + 줄 떨림·몸 전체 흰 번쩍임·밀려난 쪽 카메라 킥. 한 칸 남으면 가장자리가 숨 쉰다. 캐릭터 크기는 카메라 거리(0012에서 사장이 14 → 20)라 바꾸지 않고 20·17·14 비교 그림만 — 판정 13-1~13-6.

**0142(개발판, 사장 "그 외에 개발버전 수준인것들 찾아줘 … Environment 도 생동감 있게")**: 상자로 그리던 것 걷기 — 주민의 생활 물건 셋을 모델로(아궁이·항아리·천막, 다시 찾은 뒤의 변화는 뼈로), 줍는 것은 정육면체 대신 결정·둥근 덩이 + 빛무리(빛기둥 앞으로 당기고 어두운 테), 문지기 2단계의 뿌리 기둥은 가늘어지는 가시. 환경: 바람(풀 끝·잎·구운 화로 불꽃이 흔들리고 밟으면 풀이 눕는다 — 정점의 법선 칸 w에 몫, 색·법선 패스가 같은 식, 랩 `render.wind`) + 흩날리는 잎. Meshy 모델은 화면 크기로 재서 지금은 다시 변환하지 않음(락온의 보스 둘만 텍셀 0.64~0.70/화소 — 조건부). 판정 12-1~12-7.

**0141(개발판, 사장 "각종 이펙트들도 다듬어줘")**: 이펙트 다듬기 — 타격의 겹(맞힌 자리의 별 + 두 화소 굵기 불똥 + 먼지, 치명타·강타·패링·튕김·쓰러짐·폭발마다 다른 그림), 휘두름의 초승달, 뛰기·착지·회피·달리기·적 돌진과 내리침의 먼지, 정육면체이던 탄·조각·별을 마름모·별로, 화로 불티. 월드는 이펙트를 모른다(`fx.rs`가 사건을 듣는다). **사람이 할 일: 체크리스트 11 — 개발판에서 F6(한 프레임 = 한 틱)으로 느리게 보거나 그냥 싸워 보고, 과한지·모자란지. 패링 성공·치명타·폭발은 장면으로 확인하지 못했다(시험으로만).**

**0140(개발판, 사장 "제안 순서대로 해줘 … 그림자가 명확해야")**: 화면 다듬기 — ① 조용한 바닥(줄눈·석판 밝기 차를 줄이고 큰 얼룩으로) ② 빛 대비(센 달빛 + 낮고 차가운 바탕빛 → 빛 : 그늘 1.8 → 2.5, 그림자 칸 0.25 → 0.125m, 소품 밑 그늘, 화로 빛 1.3배, 그늘에 선 배우는 달빛을 덜 받음, 문은 건물 밝기로) ③ 마무리 패스(톤 곡선·따뜻한 빛 번짐·가장자리 그늘 — 선 둘과 함께 내부 해상도에서 한 번만). ④ 주민 옷 색 줄이기는 해 보니 나아지지 않아 하지 않음. **사람이 할 일: 연구소 "그림" 묶음에서 새 값 다섯을 켬·끔으로 견주기(체크리스트 10), 핸드헬드에서 마을 첫 굽기·프레임.**

**0139(개발판, 사장 "3d 모델들 리뷰해주고, 고도화 해줘")**: 모델 스물둘을 숫자(`review_glb.py`: 게임 화면 px·매끈한 면·텍셀 밝기)와 게임 화면 스크린숏으로 리뷰 → 그림 기준(0061) 이전 초안으로 남아 있던 **소품 여덟을 다시 빚고**, 렌더가 상자로 그리던 **장신구 상자·새긴 돌·문 장치를 모델로** 옮겼다. 캐릭터·적·주민·장신구 모델은 그대로(기하·동작 바이트 같음). 판정: 체크리스트 9(소품이 지형·캐릭터와 같은 그림인가, 화로·문·상자·새긴 돌) + 2-4에 등딱지짐승 밝기 숫자. 남긴 것: 주민 옷 잔무늬(9-6)·주민 생활 물건 셋·떨어진 아이템은 아직 그대로.

**0137(개발판, 사장 "Android에서 했던것처럼, iOS에도 컨트롤러 지원")**: iOS 게임 컨트롤러 — 패치한 miniquad `ios_pad.rs`가 Apple GameController를 그리기마다 읽어 Android와 같은 F13~F24 키·`joystick_axes`로 넘기고, 게임은 `cfg(pad_keys)` 한 길로 읽는다(`gamepads`는 iOS에서 뺌). Info.plist에 컨트롤러 지원 키 둘. **사람이 할 일: iPhone에 패드(Xbox·DualSense·MFi 등)를 블루투스로 잇고 개발판 .ipa로** — 버튼 열둘(A·B·X·Y·LB·RB·LT·RT·스틱 누름·Menu = 멈춤·Options = 장비), 스틱 세기·오른쪽 스틱 조준, D패드로 메뉴, 패드를 뗐다 붙이기, 안내 글이 패드 글자로 바뀌는지.

**0136(개발판, 사장 "iOS는?" → 1단계만)**: 서명하지 않은 iOS `.ipa`를 CI `build-ios.yml`(macOS 러너)이 만든다 — main = 운영판, `release/**` = 개발판(번들 `.dev`, 운영 앱과 나란히). `ios/Info.plist`(가로만·전체 화면)·아이콘 `ios/icons/`. iOS 앱은 터치로 시작하고 전체 화면 줄이 없다. 패드는 iOS에서 아직 안 된다(gilrs에 iOS 없음). **사람이 할 일: Actions → build-ios 아티팩트를 Sideloadly로 내 Apple ID 서명·설치**(무료 7일) → 설정 → 일반 → VPN 및 기기 관리에서 신뢰 → 가로·터치 버튼·소리·저장·글꼴.

**0135(개발판, v0.6 P1 — PC 빌드)**: Windows·macOS·Linux 묶음이 CI `build-desktop.yml`에서 나온다(`release/**` push = 개발판 아티팩트, 릴리스 때 운영판을 Release에). 모델·소리·글꼴이 모두 바이너리 안이라 nail의 에셋 경로 문제가 없고 묶음은 실행 파일 하나 + 고지 셋. 타이틀 끝 **끝내기**·**F11**·전체 화면 기억(prefs 칸 28)·Windows 콘솔 창 없음·창 아이콘. 제3자 고지 `tools/licenses.py` → `assets/THIRD_PARTY_LICENSES.txt`(Meshy CC BY 4.0 표기 포함). **같이 고침**: 외곽선·그림자 배치가 정점만 세어 색인 15,000을 넘겨 잘리던 것(웹에도 있었다 — 경고 2,350 → 0, `render::fits`). **사람이 할 일: Windows PC·Mac에서 켜 보기**(Actions → build-desktop 아티팩트, 또는 v0.6.0 Release) — 화면·소리·패드·전체 화면·F11·다시 켰을 때 기억·끝내기. Mac은 처음 열 때 "개인정보 보호 및 보안 → 그래도 열기"(공증 없음), Windows는 SmartScreen "추가 정보 → 실행".

**v0.6.0 "다듬기" 마일스톤 정함(0134, 갈래 `release/v0.6`)**: 인터뷰 — 중심 = 다듬기(판정 반영), 함께 = 판정 줄 반영·무기 강화·PC 빌드 먼저, 셋째 지역은 나중에 lore에 맞게, 크기 = v0.5만큼. **사람이 할 일: 판정 체크리스트 `docs/testing/v0-6-judgment-checklist.md`(8묶음 45칸)를 해 보고 칸마다 "좋음 / 고침"**. 차례: J0(체크리스트) → P1 PC 빌드(판정을 기다리는 동안) → J1 판정 반영 → W1 무기 강화(시작할 때 짧은 인터뷰).

**0133(개발판, 사장 "보스전투, phase 2 만들자")**: 두 보스 2단계 — 체력 반에서 2초 전환(무릎 → 터지며 밀쳐냄, 그동안 맞지 않고 튕김), 새 수 하나(문지기 = 뿌리 기둥 줄: 줄지은 원 예고, 옆으로 비킴·뛰어도 맞음 / 종지기 = 종 내던지기: 선 자리 원판 → 떨어진 자리에서 고리 한 겹), 돌진 뒤 곧장 잇기(내려치기·울림), 막대 표시 없음. 개발 전투·적 "문지기 2단계"·"종지기 2단계"(시작하자마자 전환). 모의: 90% 승리 문턱은 그대로(종지기 첫 도착 p≈0.6, 문지기 맨몸 p≈0.5) — 읽을 것이 바뀌는 단계. 판정: 0133 판정 줄. **다음: v0.5.0 배포(사장) → v0.6 마일스톤**.

**0132(개발판, 모델 — 사장이 Gemini 그림 → Meshy GLB)**: 등딱지짐승 모델 — 잿빛 등딱지·보라 결정 뿌리·챙 아래 호박빛 눈(glow), 걷기·쉼·맞음, 공격은 머리를 넣고 앞을 들었다(예고 동안 버팀) 쿵(원판). 꼬투리 모델 — 잿빛 몸·올리브 씨앗 꼬투리(+ 보라 뿌리)·호박빛 눈, 심기 = 꼬투리를 휙 → 부풀어 버팀 → 꽉 오므림(터짐), 물러나기 = 뒷발로 미는 깡충(적이 뛰는 동안 `hop` 동작). 개발 "전투·적 → 등딱지짐승 + 꼬투리 + 청소자". 판정: 게임 거리에서 읽히나·어둡지 않나, 앞 들기·부푸는 꼬투리가 원판보다 먼저 보이나. 깨진 종지기 모델 — 두건·졸린 호박빛 눈·가슴 금의 빛나는 핵·보라 뿌리·왼손의 깨진 종, 울림 = 종을 머리 위로 → 앞 땅에, 휘두르기 = 비틀어 뒤로 → 가로지름, 조각 = 가슴 앞으로 들어 털기, 돌진 = 숙여 종을 끌기, 깨어남 = 꺼진 핵이 켜지며 일어섬(개발 "전투·적 → 깨진 종지기"). 불 지키는 주민(늙은 오소리) 모델 — 멀면 아궁이를 숙여 들여다보며 쑤시고, 5m 안이면 조금 굽은 채 돌아본다, 주민은 제 일 쪽(생활 물건)을 보고 선다. 물 모으는 주민(작은 두더지) 모델 — 멀면 쪼그려 큰 손으로 항아리를 받쳐 들고 물을 기다리고, 가까우면 손을 모아 돌아본다. 꿰매는 주민(어린 양) 모델 — 멀면 천막 천을 들고 실을 당기고, 가까우면 손을 내리고 조용히 돌아본다(개발 "둘째 지역 주민 곁에서"). **떠난 마을의 모델 다 갖춤(적 둘·보스·주민 셋)** — 판정: 0132 판정 줄. 그림(`docs/art/gemini-prompts-0132.md`)을 받는 대로.

**0131(개발판, 사장 "디아블로식 빛기둥")**: 상자를 열면 아이템이 옆 바닥에 떨어지고 주워야 얻는다(기록도 그때). 떨어진 것·숨은 물건에 종류 색의 가는 빛기둥(높이 2.6m, 흰빛·금빛·민트·크림 — 갈 곳 호박빛과 다르게), 10m 안 다 보임·14m에서 사라짐. 개발 "둘째 지역 상자 곁에서(떨어진 아이템)". 판정: 기둥 세기·거리, 열고 한 번 더 줍는 것의 손맛.

**0130(v0.5.0 C5, 개발판)**: 도감 — 멈춤 메뉴 넷째 줄. 적(본 것 = 도는 모델 + 관찰 `·`·추측 `?`, 청소자·엘리트 성질·반 아래 깨어남을 보면 줄이 더 열림 / 못 본 것 = 검은 실루엣 + 나오는 곳)·장신구(가진 것 = 그림·수치·세계 안의 말 / 아니면 나오는 곳). 깨어난 적을 9m 안에서 보면 기록 `codex`(개발 판은 안 씀). 판정: 실루엣이 궁금함을 주나, 추측이 답처럼 읽히지 않나. 다음은 모델·동작(등딱지짐승·꼬투리·깨진 종지기·주민 — 0116 차례 끝).

**0129(v0.5.0 C4, 개발판)**: 떠난 마을의 글 — 새긴 돌 셋(종탑 터: 종 치는 법 · 골목: 닳은 이름과 키 잰 자국 · 거주지: 마을 규칙, 곁에서 "읽기" → 화면 위 글 판 6초), 장신구 아홉에 세계 안의 말 한 줄(장비 창, 옅게), 주민 대사 확정. 개발 "둘째 지역 새긴 돌 곁에서(읽기)". 판정: 돌이 보이나, 글이 답을 주지 않나. 다음은 C5(도감).

**0128(개발판, 사장 "정박에만 때리면 난이도가 내려간다")**: 적 공격 엇박 — 종마다 박자 표(지연 0.35/0.6초 + 더 젖힘·떨림·들이쉬는 소리, 빠름 × 0.6·바닥 0.38초, 연속 공격 고르지 않은 간격, 반 아래 보스 페인트). 잡몹 조금(지연 15~25%), 엘리트·보스 많이. 바닥 예고는 설정 "예고 타이밍"(기본 끔 = 자리가 짙어지다 정해진 농도에서 맞음, 켬 = 예전 채움). 판정: 지연이 몸·소리로 읽히나, 타이밍 끔 예고, 잡몹 지연의 피로, 페인트의 공정함. **다음 일: 2·3단계 보스**(사장 2026-11-09).

**0127(v0.5.0 C6, 개발판)**: 마을 장신구 셋 — 꿰맨 천(체력 +1·기력 회복 −10%, 광장 동쪽 상자)·물 받는 항아리(한 모금 +1·등잔 −1, 물 받는 주민 곁방 상자)·깨진 종 조각(드러나거나 깨진 적에게 +1, 종지기 첫 승리). 무거운 손은 강타(패링 반격 포함)에만 +1. 종지기 체력 46 → 58(첫 도착 몸 90% 승리에 성공률 ≈ 0.6, 봇 47~48초). 판정: 사람 시간, 손을 바꾼 뒤 잡몹 속도, 항아리가 고를 만한가. 다음은 C4(글 — 새긴 돌·주민 대사).

**0126(개발판, 사장 요청 — 싸움 길이 모의)**: `src/balance.rs`가 실제 월드에서 봇을 돌려 "도달 시점의 몸 × 받아내기 성공률"별 싸움 시간·받은 피해를 잰다(`cargo test --release bellkeeper_fight_length -- --ignored --nocapture`). 결과: 종지기 · 첫 도착 몸(Lv3, 귀 + 손)은 봇으로 26~29초, 90% 이기려면 성공률 ≈ 0.5(첫 문지기 · 맨몸과 같다), 손(+1) 하나가 시간을 거의 반으로. 패링 성공률은 시간(−15~20%)보다 생존을 크게 바꾼다. 같이 고침: 울림 고리 간격 2.6 → 6m(고리마다 뛰어넘기가 불가능했다), 종지기 휘두르기 = 앞 부채꼴. **정할 것(사장)**: 둘째 문지기의 목표 어려움·길이, 손의 세기.

**0125(v0.5.0 C3, 개발판)**: 둘째 문지기 깨진 종지기(상자) — 마을 끝의 임시 문지기를 바꿨다. 붙으면 울림 고리 두 겹(뛰어넘기·굴러 사이로)과 종 휘두르기를 번갈아, 떨어지면 종 조각 부채꼴(앞에서 패링 = 되받아쳐 −3)과 종 끌기 돌진, 반 아래 고리 셋·조각 다섯. 핵은 `core.bellkeeper`로 따로 셋까지, 나무 마디는 더한다. 개발 전투·적 "깨진 종지기". 판정(Z3): 첫 문지기와 다르게 거리를 고르는 싸움인가, 고리 간격(0126에서 6m), 되받아치기 3 피해. 다음은 C6(장신구 셋 — "종지기의 귀"가 종지기 첫 승리 보상).

**0123·0124(개발판, 사장 판정 반영)**: 아이템·성장을 덜 후하게(등잔 2회·마시기 1초, 잔불 약 60/판·레벨 50 + 25×L, 숨은 물건 줄임, 장신구·나무·레벨 단 낮춤 — 0123), 마을은 사다리 지름길을 걷고 적 33 + 엘리트 셋·빛줄기는 기둥 빛(0124). 판정: 성장이 "느리지만 있다"인가, 등잔 2번이 빠듯하지만 되나, 마시는 1초, 빛줄기 아래 몸이 비치나, 마을 밀도. PR #11(먼지·반딧불도 바닥 따라)·#12(등급은 종 바탕에서 다시 셈)는 보완 뒤 Merge됨(2026-11-09).

**0122(v0.5.0 C7, 개발판)**: 남은 주민 셋(상자 배우) — 개발 "둘째 지역 주민 곁에서(말 걸기)". 판정(Z6): 고요하고 쓸쓸한 누가 사는 곳으로 읽히나, 싸움을 방해하지 않나, 말풍선 3.5초·다시 누르기, 다음 판에 와서 바뀐 것(불빛·항아리·천막)을 알아차리나. **v0.4.2 배포됨(2026-11-08, PR #9 → `b4ddd5a`, Release `v0.4.2` APK)**. 다음은 C3(둘째 문지기 깨진 종지기 — 상자).

**0121(v0.5.0 C1, 개발판)**: 떠난 마을 지도 — 개발 "둘째 지역(떠난 마을) 어귀부터". 판정: 비탈을 내려간다고 읽히나, 길을 잃지 않나, 광장 서쪽 장치로 사다리를 내리면 거주지 화로 → 광장이 짧아지나, 골목의 꼬투리·등딱지짐승 조합. 다음은 C7(주민 셋 — 상자 배우·말풍선·재방문 기록·공격이 지나감). PR #10 검토(2026-11-08, 사장): 마을 끝은 C3 전까지 싸우는 임시 문지기 — 판 전체를 결과 화면까지(0118 문장 분명히). T1 수용 테스트 문서 `docs/testing/v0-5-t1-region-linking-acceptance.md`(수동 항목).

**0120(v0.5.0 C2, 개발판)**: 떠난 마을의 적 셋(상자 모델) — 개발 "전투·적 → 등딱지짐승 + 꼬투리 + 청소자" 또는 "둘째 지역 어귀부터". 판정(Z2): 등딱지를 글 없이 강타·옆·위로 뚫게 되나·튕김 0.35초, 꼬투리에게 다가갈지 피할지, 청소자를 둘지 잡을지. 판정이 끝난 동작부터 그림(Gemini → Meshy) 프롬프트 초안을 쓴다. 다음은 C1(마을 지도).

**구역 등급(0112·0116, v0.5.0 T3, 개발판)**: 마을의 잡몹은 등급 2 — 잔재 체력 5·가시짐승 3, 잔불 4·5. 판정(0069 Z2와 함께): 마을에서 첫 지역보다 "한 대 더" 쳐야 하는 게 성장한 만큼 맞나, 성소로 올라가면 쉬워진 것이 보람인가. 다음은 C2(새 적 둘 상자 — 등딱지짐승·꼬투리 — + 마을의 잔재 다른 역할).

**0119(v0.5.0 T2, 개발판)**: 떠난 마을이 땅속으로 — 하늘·달 대신 흙빛 동굴, 천장 틈 빛줄기 넷, 떨어지는 흙. 판정: 성소 → 마을로 내려왔을 때 "더 깊다"로 읽히나(Z1), 너무 어두워 싸움이 안 보이지 않나(달빛 0.4), 풀 빌보드가 땅속에서 어색하지 않나. 다음은 T3(구역 등급).

**0118(v0.5.0 T1, 개발판)**: 지역 잇기 — 성소 핵을 거두면 판이 끝나지 않고 성소 바닥이 열린다(어두운 구멍, 가까이 가면 빛기둥이 걷힌다) → 밟으면 지층 단면(깊이 0 → 40m) → 떠난 마을(아직 빈 상자 지도) 어귀. 어귀의 빛 웅덩이(오르는 길)로 성소에 되돌아간다. 판정: 단면이 "내려간다"로 읽히나(Z1), 전환 길이 1.6초·올라가기 1.9초, 오르내려도 체력·등잔·잔불이 그대로인가, 마을 화로에서 쉰 뒤 성소에서 쓰러지면 마을에서 서는 게 납득되나(Z5). 다음은 T2(지역 환경).

**0117(v0.4.2 패치)**: 무기별 기력 배율(검 1.0·망치 1.7·지팡이 0.75·쇠뇌 0.7·맨손 0.6), 원거리도 기력(쏘기·모은 사격·공중 볼트·수류탄 — 기력이 없으면 안 나감), 기력 충전 45 → 28/s·지연 0.6 → 0.8초(얼린 값, 0117이 판정), 무기는 얻어서(맨손·검만 처음부터 — 쇠뇌 성벽길·지팡이 뜰 남동 단·망치 뿌리 굴 상자, 옛 기록은 다 가진 채로), 서 있는 적 24 → 39, 입구의 상자·조각·한 모금을 깊은 곳으로, 단단한 껍질은 가게로, 가게 값 60·90·110·120·150. 판정: 다시 차는 4.4초가 답답하지 않고 고르게 되는가(답답하면 지연부터 줄인다), 쇠뇌 0.7, 39의 밀도와 걷는 시간, 성벽길·뿌리 굴 중 어디를 먼저 가는가.

**0113(개발판)**: 0번째 지역 — 물결 대신 서 있는 적(보면 깨고·멀어지면 제자리로 가득), 지도 약 3배(입구·다리·무너진 길·뿌리 굴·서쪽 성벽길 고리·동쪽 탑·뜰·성소, 화로 셋, 적 24 + 엘리트 3), 문은 성벽길 끝 장치로(기록에 남는 지름길), 화로에서 쉬면 적이 다시 선다(이 판에 쓰러뜨린 엘리트 빼고). 판정: 넓어졌다·길 찾기·밀도·끈·장치 고리·첫 판 10~15분·Android 30fps.

**0112(v0.4.x, 개발판)**: 엘리트 — 성소 물결에 셋((0113부터 서 있는 적 — 뜰 가운데 굳은 잔재, 뜰 남동 단 되풀이 가시짐승, 뿌리 굴 남는 잔재)), 몸이 크고 발밑에 뼈빛 표지(굵은 고리·고리 둘·끊어진 고리), 처음 쓰러뜨리면 잔불 +30. 개발 시나리오 "엘리트"(숫자 9). 판정: 글 없이 다른 놈으로 읽히나·성질마다 싸우는 법이 달라지나·셋째 물결이 너무 세지 않나. 잡몹 레벨(구역 등급)은 v0.5.0 둘째 구역과 함께.

**0111(v0.4.x, 개발판)**: 마법 칸이 시간으로 찬다(한 칸 3초, 근접 명중 충전은 걷음). 공중 RB = 근접 무기는 마법을 비스듬히 내려 쏘고, 쇠뇌는 볼트 세 발 내려 쏘기(탄창 안 씀) — 쏘면 0.2초 떠 있고, 한 점프에 한 번. 판정: 3초가 맞나·마법만 쏘게 되지 않나·체공이 읽히나·쇠뇌 세 발이 세지 않나.

**0110(v0.4.x, 개발판)**: 맞기 전에 나가지 못한 패링(공격 후딜레이에 받아 둔 것·경직 중에 누른 것)은 버린다 — 경직이 풀린 뒤 뒤늦은 패링·헛침 굳음이 없다. 회피는 그대로 받아 둔다. 판정: 맞은 뒤 손이 묶이는 느낌이 사라졌나, 후딜레이 중 받아 둔 패링이 캔슬 시점에 늦게 나가는 것도 불편한가(그렇다면 넓힌다).

**0109(v0.4.x, 개발판)**: 레벨 값이 총 레벨로(40 + 20 × 레벨 — 어느 줄이든 같은 값, 첫 판 1·넷째 판 4·이 구역 15까지 2,700 ≈ 34판), 나무 단위 "점수" → "마디"(영어 Notch). 판정: 첫 판 뒤 레벨 1이 "한 번 올렸다"로 느껴지나, 넷째 판 레벨 4에서 문지기가 아직 실력 싸움인가, 가게(185)와 레벨이 잔불을 두고 다투는 맛이 있나. 이미 올린 레벨은 그대로.

**v0.4.0 "자라는 방랑자" 판정(0096 L1~L5, 개발판 https://dev--knellroot.netlify.app)** — 만들 일(T0·G1~G5)과 칸 표시(0108)가 다 들어갔다(0103~0108):
- L1 잔불이 끝까지 쓸 데가 있나(0109로 고침 — 총 레벨 값, 15까지 2,700), L2 레벨 한 단이 느껴지나(특히 기력 — 막대가 길어진다), L3 나무로 빌드가 갈리나(핵 셋 + 조각 셋 = 6마디 — 두 사람이 다르게 쓰면 싸움 모양이 다른가, 특히 비껴서기·길을 여는 발·모은 맞받기), L4 다 열어도 박자·패링을 못 맞히면 문지기가 어려운가, L5 v0.3 기록으로 이어하면 잃은 것이 없나.
- 칸 표시(0108): 맺음타 마지막 타 부채꼴 · 끝맺음 금빛 두 겹 · 비껴서기 민트 고리 · 맞서는 자 호박 반달이 읽히는지(헤드리스로는 그 상태까지 못 가 못 봤다).

**0107(v0.4.0 G5, 개발판)**: 칸마다 써 보기 — 값은 `progression::cell` 한 곳. 모은 맞받기(문지기 자세 반)가 너무 센지, 길을 여는 발이 벽 곁에서 어색하지 않은지, 맞서는 자가 파동을 "막았다"로 읽히는지(지금은 패링 번쩍·소리).

**0106(v0.4.0 G2·G4, 개발판)**: 나무 창이 한눈에 읽히는지(칸 밝기 네 단·목표 핀·"거기까지 n점"), 종소리 조각 셋을 글 없이 찾는지(입구 무너진 벽 조각 뒤·뜰 서쪽 높은 단 뿌리 뒤·성소 북쪽 턱 위). 칸 동작은 G5 전이라 열어도 싸움은 그대로다.

**0105(v0.4.0 G3, 개발판)**: 숨은 물건 셋을 글 안내 없이 찾을 수 있는지(입구 남동 구석·뜰 동쪽 높은 단 무너진 벽 뒤·성소 안 북동) — 너무 숨었으면 테를 밝게. 처음 이길 때 핵이 다가오는 것이 "몸이 반응한다"로 읽히는지, 알림 "핵 +1 — 화로에서 나무를 연다"(나무는 G2에서 — 지금은 쌓이기만). 옛 v0.3 기록으로 이어하면 체력·등잔·한 모금이 그대로인지(L5).

**0104(v0.4.0 G1, 개발판)**: 쉬면 화로 → 레벨. 판정 L1(잔불이 끝까지 쓸 데가 있나 — 줄당 350·셋 1,050이 한 판 70~100에 지루하지 않은지)·L2(한 단이 싸움에서 느껴지나 — 특히 기력: 막대가 길어지고 3·5단에 차는 빠르기). 힘 5단 평타 +1이 너무 세지 않은지(피해 1~3이라 두 배 가까이). 고쳐 쥐는 동작(`regrip`, `tools/art/blender/wanderer.py`)은 아직 — 지금은 소리 + 알림만이라 밋밋하면 말해 달라.

**0086**: 이어 패링이 쌓여 둘러싸였을 때 빠져나가는 느낌인지·너무 쉬워지지 않는지.
**0087**: 줄어드는 부채꼴로 박자를 맞추기 쉬운지, 가장 작은 몫·금빛이 "지금"으로 읽히는지, 동작마다 사라지는 쪽이 칼과 맞는지.
**0088**: 땀방울·막대 떨림·소리가 "기력 없음"으로 읽히는지, 패링 기력 25가 반격엔 넉넉하고 기력 걱정을 없앨 만큼 많지는 않은지.
**0089**: 모았다 떼며 맞받는 것이 노려 볼 만한 보상인지, 패링 버튼을 따로 쓸 이유를 없애지는 않는지, 덜 모은 강타에도 줄지.

**0085**: 땅 돌리기 시작 휘청 0.25초가 작은지·탈출기로 너무 세지 않은지.

**0084**: 땅 돌리기 휘청 1.5°가 작은지·멀미 없는지(연구소에서 바로), 경사로·턱에서 바닥 표시가 묻히거나 뜨는 곳이 남았는지.

**0081**: 이어 패링 0.2초가 "비슷한 타이밍"인지·앞쪽만 받는 것이 떼 속에서 억울하지 않은지, 슬롯의 "시간"이 플레이 시간이 맞는지.

**0078**: 딱 맞춰 떼기 0.15초가 무기마다 맞는지·타이밍 규칙 셋(박자·패링·딱 맞춤)이 많지 않은지, 안내 화면은 아직 없음.

**0077**: 쇠뇌 박자 0.24초를 탈 수 있는지·볼트 안 쓰는 덤이 너무 센지.

**0076(개발판, 2026-11-07)**: 판정 — 적이 기둥을 돌아오는지·떼가 둘러싸는지, 돌진 띠 끝·옆과 맞는 곳이 같은지(문지기 먼 돌진이 짧게 끝나는 것은 둠 — 늘 닿아야 하면 `charge_range` 8), 박자 고리·음표·종소리로 박자를 타게 되는지(안 되면 메트로놈), 맨손(검 동작 빈손 — 어색하면 주먹 동작), 달리기 ×1.45·18/s. 터치 강공격·원거리 모으기가 0070 7 뒤로 안 됐던 것 고침.

**v0.3.x 패치(0070)**: 1~5·9·10 개발판에 들어감 — 손맛 판정(대시 공격·패링 반격·락온 넘기기·패링 멈춤·장비 창 십자·평타 타격감·돌진 예고가 닿는 때). Select = 장비 창·후보에서 낀 것 빼기는 들어감. 랩을 멈춤 메뉴 안의 창으로 옮김(0070 7).

**다음 마일스톤**: v0.4.0 "자라는 방랑자" `docs/decisions/0096`(채택 2026-11-07 — 잔불 레벨 생명·기력·힘, 기술 나무 박자·맞받기·발걸음, 핵 고르기를 나무로, 0095). 순서 T0(`tick_player` 나누기) → G1 레벨 → G3 → G2·G4 나무·화면 → G5 칸 동작. 구체 설계(레벨 값·나무 15칸·핵 점수 셋까지·종소리 조각 셋 자리·등잔 옮기기)는 `docs/decisions/0102`(채택 2026-11-07, lore 0101 위에서). v0.5.0은 "확장" — 다른 지역(둘째 구역 "내려가는 길" 0069, 사장 2026-11-07).

46줄로 쌓였던 것을 플레이 순서로 묶었다(2026-11-05). 줄마다 괄호의 결정 번호에 자세한 것·랩 값이 있다. 이상하면 "어디서·무엇이" 한 줄이면 된다.

### A. 한 판(개발판 https://dev--knellroot.netlify.app — v0.2.0 판정 S1~S4)
1. [ ] **켜기**: 로딩 종·막대(0062), 타이틀. 폰이면 "홈 화면에 추가" → 아이콘·가로 전체 화면·비행기 모드로 다시 열기(0062·0046).
2. [ ] **설정·조작**: 설정이 찾기 쉬운지·흔들림 줄(0052). 키보드+마우스(0058 — 옛 저장이면 조작 설정 → "키보드+마우스 배치로"), 패드(0024·0048), 터치(0053). 키를 바꾸고 다시 켜도 남는지(0024).
3. [ ] **입구**: 방랑자 몸·동작이 귀여운지(0038·0039 — 걷기 뒤뚱·멈추기·착지), 바닥이 시험용 같지 않은지(0060 W2), 낭떠러지 아래 안개·반딧불(0063), 턱·경사로·0.5m 턱 점프(0043).
4. [ ] **뜰(S2)**: 잔재·가시짐승이 구분되고 귀여운지(0060 W3), 가시짐승 돌진 띠를 보고 피하는지, 떼싸움이 읽히는지. 그 김에 — 무기 넷 바꿔 가며(0051), 강공격·모으며 걷기(0030·0047), 원거리 강공격(0056), 캔슬(0057), 패링 0.18초(0025), 점프·내려찍기(0027), 땅 돌리기(0028·0029), 등잔(0031), 마법 칸(0048).
5. [ ] **문(S1)**: 문이 열린 뒤 헤매지 않는지 — 호박색 빛기둥·화면 밖 화살·목표 글(0060 W1). 쉼터 자리·쓰러진 뒤 되돌아가기가 억울하지 않은지(0032).
6. [ ] **보스(S3)**: 문지기가 귀여운지·위압감이 남는지·팔 이음매가 벌어져 보이는지(0060 W3). 예고 셋(부채꼴·띠·고리)을 읽고 피하는지, 2~4분에 이기거나 질 이유를 아는지. 락온 구도(0050·0040)·게걸음(0034)·자세 깨기와 폭딜(0051·0055).
7. [ ] **분위기(S4)**: 어둡고 조용한 성소 + 호박색 불빛, 캐릭터가 바닥에 묻히지 않는지. 빛·그림자(0044·0045), 화면 선(0061), 하늘·뿌리 아치·나무·벽 뿌리(0063), 이야기 글(시작 두 줄·구역 이름·결과 — `src/i18n/ko.rs`).

### A2. v0.3.0(0064) — 들어간 것부터
- [ ] **잔불(M1)**: 튀어나오고 끌려오는 손맛(너무 빠르거나 멀리서 끌리지 않는지), HUD 수가 보이는지. 쓰러져도 그대로인지.
- [ ] **소리(M5)**: 귀로 — 크기가 맞는지(설정 → 소리 크기), 예고 소리만 듣고 부채꼴·돌진·파동·잔재를 가를 수 있는지(R4), 환경음(바람 + 먼 종)이 성소에 맞는지, 거슬리는 소리.
- [ ] **성장(M4)**: 쉼터에서 쉬면 가게가 열리는 게 귀찮지 않은지(쉴 때마다), 값(25·35·45·80)이 한 판에 하나쯤인지, 문지기를 이기고 고르는 핵이 기쁜지.
- [ ] **장신구(M3)**: 입구 상자(시작 자리 북쪽)·뜰 동쪽 끝 상자를 열어 보기, 멈춤 → 장비에서 갈아 끼우기. 장신구마다 느낌이 달라지는지(특히 가벼운 발·무거운 손의 맞바꿈), 설명 글이 알아듣기 쉬운지.
- [ ] **기록(M2 + 슬롯 셋 0066)**: 쉼터에서 쉬고 새로고침(웹)·다시 켜기(APK) → 타이틀 "이어하기" → 슬롯 창 → 그 쉼터에서. "시작하기"로 다른 슬롯에서 새로 해도 옛 슬롯이 그대로인지, 지우기·덮어쓰기가 두 번 눌러야 되는지, 무기가 슬롯마다 따로인지.
- [ ] **아이템 그림(0068)**: 장신구 여섯 모델 시안(`docs/art/previews/charms.png`·장비 창) — 껍질·짚신·종·뿌리 이빨·돌 장갑·주머니가 알아보이는지, 고칠 것. 무기 아이콘 각도.
- [ ] **장비 창(0067 D)**: 멈춤 → 장비. 인형이 귀엽고 무기가 잘 보이는지, 칸 → 후보 → 끼우기가 번거롭지 않은지(좌우로 바로 돌리기도 됨), 바뀌는 값 미리보기가 읽히는지, 폰 화면에서 글이 작지 않은지.
- [ ] **창 틀·뼈대(0066)**: 멈춤·장비·설정·가게·핵·결과·DEV가 nail식 창(제목·✕·이름 … 값)으로 — 읽기 쉬운지. 터치로 ✕·창 밖 탭·◀▶ 탭, 설정에서 닫으면 연 줄로 돌아오는지(멈춤 → 설정 → 닫기 = "설정"에 커서), 묻는 창이 아래 창 위에 뜨는지.
- [ ] **결정 검토 질문 여섯**(2026-11-05 대화, 세계관 D2는 (a)로 닫음 — 0004): 그림 상한(0001)·얼린 값 절차(0013)·P5/S2/S3 판정 시점·상표(0005)·카메라 픽셀 흔들림(0061 ②)·락온 시험 기준(0040).

### B. 핸드헬드(Android APK — P5)
8. [ ] 패드 전부(두 스틱·D패드·Start, 0022), 상태·내비게이션 바가 숨는지(0046), 전체 화면을 여러 번 켜고 꺼도 글자가 검게 안 되는지(0049·0054), 고운바탕이 읽히는지(0021), 10분 넘게 켜 두기(0020).
9. [ ] **fps(P5: 데스크톱 60 / Android 30)**: 뜰 셋째 물결과 개발 시나리오 7(잡몹 80)의 개발 줄(F3) 사진. 낮으면 랩(F1)에서 하나씩 끄고 다시 — `render.edge`(0061)·`render.actor shadow`(0045)·`render.fog`·`render.motes`·`render.sky`·`render.flora`(0063). 해상도 640×360이 꽉 차는지(0041).

### C. 문서(시간 날 때)
10. [ ] 아이템 설계 초안 `docs/design/items.md`(0026·0027) — v0.3.0(0064)에서 장신구·성장·잔불로 들어갔다.
11. [ ] 굴리는 목록(0059)은 넘치는 메뉴가 생기면 폰에서 끄는 빠르기·탭 범위.
12. [x] **세계관 바이블**(`docs/lore/`, 0101): 2026-11-07 인터뷰로 정함 — 법칙 A~E·불변 규칙 여덟·성장 원칙·핵 반응 단계는 CANON, 다섯 막은 제안 유지, ECO-05는 다음 새 적 전·ECO-10은 v0.5.0 지역 설계 전. 나무 칸 이름 후보(`growth.md` 4)는 G2 때.
