# 그림 · 모델 · 동작

`CLAUDE.md`에서 옮긴 규칙이다(0278) — 문장은 그대로이고 같은 힘을 가진다. 고치는 곳: `tools/art/` · `assets/models/` · `src/anim.rs` · `src/gait.rs` · `src/idle.rs` · `src/model.rs`.
이 영역의 파일을 고치기 전에 읽고, 구조를 바꿨으면 여기를 고친다. 굵은 머리말에서 문단만 갈랐다.

## 그림 기준

**그림 기준은 t3ssel8r 풍 3D 도트(0061, 사장: 늘)**: 모델링할 때마다 0061 체크리스트 — **귀여움 문법이 먼저**(0036: 큰 머리·큰 눈 낮게·입 없음·둥근 실루엣, 보스도), 큰 면 몇 개의 둥근 덩어리(상자 + 면 법선은 마인크래프트 인상), 각진 것은 뿌리만(0037), 실루엣 + 큰 색 덩어리, 밝은 선이 붙을 볼록 모서리는 뿌리·핵·테에, 게임 거리의 px 크기 먼저, 판정은 게임 안 스크린숏.

## 모델 파이프라인 · 방랑자의 스킨 몸

그림·모델: 손으로 PNG·GLB를 고치지 않는다. 모델 원본은 Blender bpy 스크립트 `tools/art/blender/<이름>.py`(공용 `common.py`: 상자 부위·뼈에 붙인 강체·128×128 아틀라스·동작. 소품은 0139 꼴 — 깎은 돌 `stone`·돌린 꼴 `lathe`·꺾인 관 `limb` + 무늬 없는 `k_*` 칸: 돌 쌓기 무늬 대신 큰 덩어리를 실제로 쌓고, 둥근 것은 매끈하게·뿌리와 테만 각지게. 고치기 전·후에 `review_glb.py`(게임 화면 px·텍셀 밝기)와 게임 스크린숏. 적·문지기의 2단계는 Meshy 메시를 줄이고 색을 굽고 부위로 자른다 — KB `guides/ai-mesh-to-rigid-part-model.md`, 0038·0060.

**그림·메시는 API로 받는다(0234)**: 참조 그림 `tools/art/nanobanana.py`(Meshy Image to Image의 나노바나나 — 첨부 그림을 data URI로, Gemini 키는 무료 등급이라 이미지 한도 0), 메시 `tools/art/meshy_image3d.py`(Image to 3D — 빛 지우기·3만 삼각형·네 방향 미리보기) — 쓴 크레딧을 알린다. 보스의 생김새는 그림 → 사장 확인 → 메시(0157). 새 몸에는 그 전 모델의 뼈 이름·동작 표를 이어받는다(`root_husk.acts`·`thorn_beast.acts(m, leg_k, rear_k)`) — 게임 코드는 종 표의 `model`과 `STRIKE_KEYS` 한 줄. 등짐꾼의 짐은 모델 부위 `load`(깨지면 렌더가 숨긴다, `Enemy::load_lost`).

**방랑자는 스킨 몸(0193)**: Meshy 리깅·동작 GLB → `meshy_rig.py`(줄인 몸 + 뼈 + 쓸 동작 → `sources/meshy/raccoon_v2_rig.glb` · 0247부터 몸은 v3(그림 A → Meshy 7.1 → 자동 리깅): `--retarget raccoon_v2_rig.glb`로 v2의 동작 49개를 뼈 이름으로 옮겨 담은 `raccoon_v3_rig_anim.glb` — 새 몸에 동작을 다시 사지 않는다, 엉덩이 뼈의 z = 몸 앞으로 돌림을 맞춘다) → `wanderer_skin.py`(표 `CLIPS`: 원본 구간 → 게임 이름, 길이는 옛 동작 그대로·닿는 순간을 맞춤, 엉덩이 수평 이동 뺌, 무기·등잔은 손뼈의 단단한 조각, 평타·강타는 정면에서 시작·끝(강타는 모으기 끝의 쪽에서 — `CHAIN`), 돌아서기는 몸이 스스로 돌지 않고(`FACE_STILL`), 점프는 엉덩이를 띄우지 않고(`FLAT`, 게임이 `air`로 띄운다), 쇠뇌는 활쏘기 원본을 겨누는 줄로(`AIM`), 회피는 원본이 간 쪽을 몸의 앞·뒤로(`FACE_TRAVEL` — 뒤로 피하면 뒤로 뛰기 `dodge_back`, 0197) — 시험 `wanderer_clips_start_and_end_facing_front_and_do_not_spin`·`the_crossbow_points_where_the_bolt_goes`, 0196) — 게임은 `model::skin_into`로 CPU에서 휜다(그리기·그림자·외곽선 모두 `batch::parts` 한 곳)) — 자동 리깅이 망토 자락까지 팔 뼈에 묶어 두어 위팔·아래팔 가중치는 팔 뼈대에서의 거리로 가둔다(`confine_arms`, 0256) — 망토 정점(텍스처 보라 `cloak_verts`)의 팔 사슬 무게와 팔 사슬 밑 덤 뼈(`stray_arm_bones` — 오른손 밑 `Bone_061`이 망토 판을 쥐고 있었다)의 무게는 모두 가슴으로, 걷기·달리기는 오른팔을 서기 쪽으로 붙든다(`ARM_HOLD`), 구르기 끝은 정면 쪽 `ROLL_FACE`(0260), 끝 자세가 서기로 이어지는 동작은 끝에서 몸·고개를 세운다(`END_UPRIGHT`·`HEAD_SETTLE`, 0256) → `blender -b -P` → `assets/models/<이름>.glb`(+ 미리보기 `docs/art/previews/`, 검사 `check_glb.py`). GLB는 `build.rs`가 바이너리에 넣고 `models.rs`가 이름으로 찾는다 — 없으면 상자. 부위 이름 `glow_*` = 스스로 빛남. 상태 → 동작 이름은 `render::enemy_anim`·`player`. 방랑자가 서 있는 동안의 몸짓(0153)은 `idle.rs`(렌더가 들고 있는 순수 규칙 — 숨 고리 `idle`의 끝에서만, 조용할 때만, 표 `FIDGETS` 순서로) — 새 몸짓 = 숨 고리의 0초 자세에서 시작해 거기로 돌아오는 동작 하나 + 표 한 줄 + `check_glb`의 길이. Meshy 부위를 옮길 때 잘린 자리는 면으로 메우지 않고 한 점으로 오므린다(`wanderer.py` `_pinch`). **주인공·보스의 생김새에 코드로 새 꼴을 덧붙이지 않는다(0157)** — 천·살 같은 흐르는 꼴은 도형을 붙이면 그 도형으로 읽힌다(두건에 띠·꼭지·자락 → 넥타이·요정 모자·투구). 생김새를 바꾸려면 그림 먼저(앞·뒤·옆) → 사장 확인 → 메시를 다시 받는다. 단단한 소품·무기는 코드 도형으로. 지형은 `scenery.rs`가 한 번 굽는다(빛은 정점 색, `render::lit`). 에셋 출처는 `assets/LICENSES.md`. AI를 쓴 곳은 숨기지 않는다.

## 걸음 시계

방랑자의 걸음 시계는 **쌓는다**(0175 `gait.rs` — `월드 시간 × 빠르기`는 가속하는 동안 걸음 주기를 건너뛰었다), 조용한 동작(서기·걷기·달리기·몸짓)으로 들어갈 때만 지난 자세에서 짧게 섞는다(`model::locals`·`blend_locals`) — 누르는 순간 보여야 하는 동작은 섞지 않는다. 동작 시각을 빠르기로 바꾸는 새 동작도 시각을 쌓는다.

## 다리가 짧은 몸의 걸음

다리가 짧은 몸의 걸음은 다리가 아니라 **온몸으로**(0173 — root 굴림·솟기·몸통 찌그러짐·팔): 다리만 흔들면 게임 거리에서 서 있는 것과 같다. 새 적의 `walk`는 게임 화면에서 틱마다 찍어 실루엣이 갈리는지 본다.
