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

## 구현 상태(2026-11-06 — 첫 판 2026-11-04, v0.3.0 배포 2026-11-06·0066까지)
| 항목 | 위치 | 비고 |
|---|---|---|
| 바닥 다듬기(0060 W2) | `scenery.rs` `MORTAR_Y`·`SLAB_*`·`tuft`·`WALL_AO` | 걷는 면 아래로 판 석판(높이·긴 판·들림·빠짐 + 옆면), 풀 다발, 벽 밑 그늘 |
| 화면 공간 선(0061) | `render.rs` `EdgeFx`·`normal_pass`·`edge_fx`, `scenery::with_normal`, `Batch::record` | 법선 패스(뷰 법선 + 선형 깊이) → 이웃 넷: 깊이가 끊기는 앞쪽은 어두운 선, 볼록 모서리는 밝은 선. 랩 `render.edge*` 다섯(덧붙임). GL만 |
| 길 안내(0060 W1) | `stage::StageRun::goal`, `render.rs` `beam`·`GUIDE_*`, `hud::draw_goal_arrow`, `camera::world_to_ndc`·`edge_marker` | 문이 열리면(그리고 핵이 남으면) 그 자리에 호박색 빛기둥 + 빛, 화면 밖 1초면 가장자리 화살, 목표 글 "열린 문으로 성소에 들어가라" |
| 굴리는 목록(0059) | `ui.rs` `ListView`·`Ui::menu`, `touch.rs` 끌기, `input.rs` 휠 | 모든 메뉴·조작 설정이 넘치면 굴린다: 보이는 줄 + ▲▼(누름), 키 커서는 창이 따라가고 가장자리 한 줄 여유, 손가락 끌기(탭 범위 넘으면, 뗄 때 탭 아님)·휠·▲▼는 창을 옮긴다. 지금 메뉴는 넘치지 않아(UI가 화면 높이 비례) 줄이 늘 때 켜진다 |
| 키보드+마우스(0058) | `input.rs`, `app/play.rs` `cursor_aim`, `camera.rs` `screen_ray`, `hud.rs` `draw_reticle` | 기본 배치: 왼쪽 클릭 공격·오른쪽 클릭 마법·휠 클릭 락온·L-Shift 패링, 왼손 그대로(땅 돌리기 Q·E 남김). 마우스 = 오른쪽 스틱: 쿼터뷰에서 커서를 비춘 땅 위 점 쪽으로 겨눔(수류탄은 그 자리), 만진 지 1초(설정 0.5·1·2·4) 지나면 자동 조준, 락온 중엔 안 씀. 싸움 중 OS 커서 대신 조준 표시(작게·보통·크게). 조작 설정에 "키보드+마우스 배치로·키보드만 배치로", 메뉴는 왼쪽 클릭을 뗄 때 탭. 웹 오른쪽 클릭 메뉴·휠 클릭 자동 스크롤 막음. 감도 칸 없음(OS 커서). 시험: 쉼 규칙·슬롯 번호·커서 조준·락온 무시·수류탄 자리·프리셋·prefs, 헤드리스 `tools/test/mouse.js` |
| 장면 | `app/mod.rs`·`app/stack.rs`(0066), `app/flow.rs` | 흐름은 순수 함수 + 시험 셋. 메뉴는 위·아래 + 확인, 터치는 뗄 때 탭, 언어 줄은 좌우로도 바뀐다. 멈춤: Esc·P / Start / 터치 오른쪽 위 |
| 스테이지 | `stage/mod.rs`, `stage/sanctuary.rs` | 글자 지도 37×25칸(칸 2m). 벽은 줄마다 상자 하나로 합쳐 부딪힘. 진행은 `Phase`(입구 → 물결 n → 문 열림 → 깨어남 → 결투 → 핵 → 끝). 시험: 지도 읽기·입구에서 클리어까지·쓰러지면 끝 |
| 종류 표 | `species.rs`, `enemy.rs` | 잔재 체력 3·2.6m/s·원판 0.55초 / 가시짐승 체력 2·3.8m/s·7m 안 돌진(띠 0.65초, 13m/s × 0.5초) / 문지기 체력 40·부채꼴 110° 0.9초·돌진 11m·세 번째마다 파동(1초 예고, 9m까지 9m/s), 반 아래 예고 ×0.75 |
| 글 | `i18n/`, `ui.rs`, `hud.rs` | 한국어·영어. HUD: 체력 마름모·스태미나·목표·구역 카드·시작 이야기·보스 띠 |
| 모델 | `assets/models/*.glb`, `build.rs`, `models.rs`, `tools/art/blender/` | Blender 4.0 bpy 스크립트가 원본(상자 부위 + 뼈에 붙인 강체 + 64×64 아틀라스). 미리보기 `docs/art/previews/`. 없으면 상자 |
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
| 딱 맞춰 떼기는 모든 무기(0078) | `tuning::PERFECT_RELEASE`, `world` 강타·모은 사격 `perfect`, `Player::since_perfect`, 렌더 금빛 고리, `Event::Perfect` → 종 `Beat3` | 다 모인 뒤 0.15초 안에 떼면 피해 +1·자세 ×1.5(강타는 넉백 ×1.5도) — 근접 강타 전부와 쇠뇌 모은 사격, 무기 표 `perfect` 칸 없앰 |
| 박자는 모든 무기·장비 목록 커서(0077) | `equip::candidates`·`has_choice`·`pick_start`, `world` 쇠뇌 사격 박자·`on_beat`·`BEAT_WINDOW_MIN`, `hud` 탄창 반짝임 | 모든 칸 목록에 낀 것(착용 중), 커서는 거기서, 견주는 줄은 가장 긴 것만큼 비움. 쇠뇌는 사격이 박자(한 발 0.24초, 박자 사격 = 볼트 안 씀), 창 바닥 ±60ms, 지팡이 설명에서 박자처럼 읽히던 말 뺌 |
| 길찾기·돌진 띠·박자 피드백·맨손·달리기(0076) | `nav.rs`(`Nav`·`clear`·`waypoint`·`charge_room`), `Enemy::spread`·`charge_len`·`touch`, `Player::beat_chain`·`since_beat`·`swing_beat`·`running`, `hud::draw_beat_note`, `sfx::beat_id`, `weapons::Id::Fists`, `equip::pick_start`, `ListView::tapped`, `tuning::RUN_*` | 적은 상자(기둥·소품·문)를 아는 격자로 몸 폭 끈 당기기 + 옆으로 퍼지기, 돌진 띠 = 맞는 곳(폭·끝·쓸고 간 선분·벽에서 멈춤), 박자 타 = 금빛 휘두름·발밑 고리·음표·스태미나 막대 반짝임·오르는 종소리, 무기 후보에 든 것(착용 중)·맨손, 회피 꾹 = 달리기(×1.45, 18/s, 달리다 치면 대시 공격), 터치 강공격·모으기 되살림 |
| 개발 메뉴 개편(0075) | `Scene::Dev { tab }`·`settings::DEV_TABS`·`App::dev_run`·`flow::pause_items` | 묶음(combat·stage·perf & tools) + 설명, 개발 판은 슬롯에 안 씀, 멈춤 끝에 DEV(개발판), 시나리오 8 husk + thorn |
| 공격 예고 설정(0074) | `SettingsRow::Tells`·prefs 칸 26·`View::tells`·`render::TELLS_BASE` | 설정 → 공격 예고 0~100%(0 = 끔), 기본 50%, 70% = 0072 진하기 |
| 적 준비 동작(0073) | `render::STRIKE_KEYS`·`windup_clock`·`windup_hold`·`WINDUP_*` | 들어 올림(예고 60%, smoothstep) → 정점에서 버팀(떨림·눈/핵 번쩍) → 마지막 0.08초에 내리침 = 맞는 틱, 늘어남·뒤로 젖힘, 예고 시간·판정은 그대로. 동작 원본 과장(움츠림 → 까치발 젖힘·늘어남 → 납작): 잔재 attack·가시짐승 charge_windup·문지기 slam·wave·charge_windup |
| 옅은 예고·표시(0072) | `render::telegraph`·`TELE_A`·`TELE_BASE_A`·`TELE_MARK_A`·`TELE_RING_W` | 적 예고 채움 = 테두리 투명도 0.3(겹치지 않게 — 채움은 테두리 안쪽까지), 파동·평타·강타·패링 반달·패링 성공 고리도 옅게 |
| 박자·무기 숫자(0071) | `world::on_beat`·`Player::since_swing/beat/on_beat`·`BEAT_WINDOW`, `Event::Beat`·`sfx::Id::Beat`, `app/equip.rs` `weapon_numbers`·`weapon_compare` | 무기 박자 = 그 타의 한 스윙 시간, 박자 ±15% 안에 누른 평타 콤보는 스태미나 0(받아 둔 이른 누름·첫 타·대시·반격은 씀), 맞으면 '팅'. 장비 창: 무기 칸이면 피해·공격 속도·거리·강타(쇠뇌는 사격·탄창·장전·수류탄), 무기 후보는 지금 무기와 ▲▼ |
| v0.3.x 손맛·조작(0070) | `world.rs` 패링 감속·`handoff_target`·대시 공격 `weapons::DASH`·패링 반격, `app/equip.rs` `slot_nav`, `tuning.rs` `LOCK_HANDOFF`·`DASH_ATTACK_LATE`·`RIPOSTE_WINDOW` | 패링은 멈출 때처럼 감속(뒤로 출렁이지 않게)·막기 자세 앞으로, 락온 대상이 쓰러지면 8m 안 다음 적으로(없으면 풀림), 장비 창 십자 = 커서·땅 돌리기 = 갈아 끼우기, 회피 끝·직후 공격 = 무기별 대시 공격(콤보 둘째 타로), 패링 직후 공격 = 다 모은 강타, 평타 타격감 층(히트스톱 ×1.6·카메라 킥·맞은 쪽 젖힘·떨림·불똥·공격자 멈춤·둘째 소리 `Thwack`), 돌진 띠는 몸이 닿는 순간 다 참(`Enemy::charge_eta`). Select(Tab) = 장비 창 바로, 후보에서 그 칸에 낀 것은 뺌(0070 덧붙임). 연구실(랩)은 설정 안 조작 설정 위 줄(개발판만, `app/lab_screen.rs`, 묶음 열여섯·A/B·내보내기, 값은 닫아도 계속 — 0070 7·15), 개발판 APK 패키지 `io.github.aimenjoe.dev.knellroot`(마지막 조각 = 라이브러리 이름, 0070 16), Android 에뮬레이터 연기 시험 `android-smoke.yml` |
| 장비 창(0067 D) | `app/equip.rs`, `Renderer::draw_doll`, `stats::lines`, `Scene::EquipPick` | 디아블로 콘솔식: 가운데 방랑자 인형(든 무기), 칸 넷 → 후보 목록, 바뀌는 상태 `값 → 값 ▲▼`, 후보는 가진 것만(못 얻은 것은 도감 몫, 0068) |
| 장신구(0064 M3) | `stats.rs`·`charms.rs`, `World::stats`·`apply_stats`, `Player::max_hp`·`flask_max`, `StageRun` 상자(`C`)·`take_opened`, `app/pause.rs` 장비 창·`app/progress.rs` `gain_charm`·알림 | 세 칸, 여섯 가운데 상자 둘·문지기 하나에서 얻음, 멈춤 → 장비에서 갈아 끼움, 능력치는 한 곳에서 합침 |
| 성장(0064 M4) | `growth.rs`, `Scene::Core`·`Scene::Shop`, `StageRun::take_rested`·`shop_ready`, `app/progress.rs` `buy`·`app/shop.rs` `core_menu`·`shop_menu` | 문지기를 이기면 핵 고르기(체력·등잔·회복), 쉬면 화로 가게(장신구 셋·등잔 기름), 살 것이 있으면 쉼터가 빛남 |
| 소리(0064 M5) | `sfx.rs`, `app/play.rs` `world_sounds`·`app/mod.rs` `set_sfx`·메뉴 소리, `prefs.rs` 칸 25, `assets/sfx/` | 효과음 16(CC0)·합성 8(패링·예고 넷·깨어남·쉼터·등잔)·성소 환경음, 설정 소리 크기 |
| 묻는 창·UI 맞추기(0065) | `Scene::Ask`·`flow::answer`·`ask_default`, `ui.menu_dimmed`, `Sfx::played` | 새로 하기·처음부터·타이틀로·사기는 한 번 묻는다(취소에서), 가게는 못 사는 것 흐리게·HUD 없음, 메뉴 소리 겹치지 않게, 저장 실패 알림 |
| 개발 | 타이틀 "DEV: skip to the sanctum gate"·"DEV: PoC scenarios"(1~7, Esc = 타이틀), 랩(멈춤 끝 줄 LAB·F1, 0070), F3 개발 줄, F4 글자 넘침 검사 | 운영판(`--no-default-features`)에는 없다(F3 줄은 있음) |

## 사람이 할 일
**0078**: 딱 맞춰 떼기 0.15초가 무기마다 맞는지·타이밍 규칙 셋(박자·패링·딱 맞춤)이 많지 않은지, 안내 화면은 아직 없음.

**0077**: 쇠뇌 박자 0.24초를 탈 수 있는지·볼트 안 쓰는 덤이 너무 센지.

**0076(개발판, 2026-11-07)**: 판정 — 적이 기둥을 돌아오는지·떼가 둘러싸는지, 돌진 띠 끝·옆과 맞는 곳이 같은지(문지기 먼 돌진이 짧게 끝나는 것은 둠 — 늘 닿아야 하면 `charge_range` 8), 박자 고리·음표·종소리로 박자를 타게 되는지(안 되면 메트로놈), 맨손(검 동작 빈손 — 어색하면 주먹 동작), 달리기 ×1.45·18/s. 터치 강공격·원거리 모으기가 0070 7 뒤로 안 됐던 것 고침.

**v0.3.x 패치(0070)**: 1~5·9·10 개발판에 들어감 — 손맛 판정(대시 공격·패링 반격·락온 넘기기·패링 멈춤·장비 창 십자·평타 타격감·돌진 예고가 닿는 때). Select = 장비 창·후보에서 낀 것 빼기는 들어감. 랩을 멈춤 메뉴 안의 창으로 옮김(0070 7).

**다음 마일스톤**: v0.4.0 "내려가는 길" 제안 `docs/decisions/0069` — 끝의 "사장이 정할 것" 다섯(보라 뿌리의 뜻·둘째 구역·방랑자와 포포·도감·판정 기준). D2는 (a)로 닫음(0004).

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
