# Blender 모델 스크립트(초안 3D)

침묵의 성소·떠난 마을의 모델. 모든 모델은 **bpy 스크립트가 원본**이고 `.glb`·미리보기 PNG는 결과물이다 — 손으로 고치지 않는다.
AI 사용: 모델 모양·색·동작은 Claude가 이 스크립트로 코드로 찍었다(그림 생성 AI·외부 에셋 없음).

## 다시 만들기

```sh
blender --background --python tools/art/blender/build_all.py          # 전부 + 검사 (= blender -b -P …)
blender --background --python tools/art/blender/wanderer.py           # 하나만
blender --background --python tools/art/blender/props.py -- gate bell # 소품 몇 개만
python3 tools/art/blender/check_glb.py                                 # 검사만(블렌더 없이)
python3 tools/art/blender/review_glb.py                                # 리뷰 표: 게임 화면 px·매끈한 면·텍셀 밝기(블렌더 없이, 0139)
```

Blender 4.0.2로 확인(glTF 내보내기에 python3-numpy 필요). EGL 경고는 무시해도 된다(Workbench 렌더는 된다).
결과: `assets/models/<이름>.glb`, `docs/art/previews/<이름>.png`(쉬는 자세) + 캐릭터는 `<이름>_<동작>.png` 한 장, 움직이는 소품은 `_open`·`_pulse`·`_toll`.

## 파이프라인

1. `common.py`의 `Model`: 상자(`Geo.box` — 직육면체, 윗면 좁히기 `taper`·밀기 `shift`, 회전 `rot`, 안 보이는 면 빼기 `drop`)로 부위를 만든다. 부위 = 메시 오브젝트 하나 = glTF 노드 하나.
   - **다듬은 꼴(0139, 소품)**: `Geo.stone`(모서리를 깎은 돌 — `box`와 같은 치수, 깎을 모서리 무리 `t` 윗면 둘레·`v` 세로·`b` 아랫면 둘레), `Geo.lathe`(돌려 만든 꼴 — 기둥 북·그릇·종·결정·불꽃, `smooth`면 둥글게·아니면 면마다 각지게, 띠마다 칸을 달리), `Geo.limb`(꺾인 관 — 뿌리·버팀대·불꽃 혀, 끝 반지름 0 = 뾰족). 면은 칸 가운데 한 색(`cell_mid` — 무늬·칸 테두리를 쓰지 않는다)이라 `k_*` 단색 칸과 쓴다. **건물의 돌은 깎고, 둥근 것은 매끈하게, 뿌리·결정·테만 각지게.**
2. 아마추어 뼈에 부위를 `parent_type='BONE'`으로 붙인다(강체). **정점 가중치·스키닝·모프 없음.** 뼈는 모두 +Z를 향하게 만든다 — 자세 각도를 뼈와 상관없이 같은 축으로 적으려고.
3. 텍스처: 파일마다 128×128 아틀라스 하나(같은 그림). 위 넷째(y 0~31) 8px 칸(재질마다 옆면·윗면), 그 아래(y 32~63) 16px 큰 칸(벽 돌 쌓기·밝은 돌·뿌리 결·문 널빤지 — v1 소품용, 지금은 쓰는 모델이 없다) + 파스텔 칸, 아래 절반(y 64~)은 무늬 없는 소품 칸 `k_*`(0139). 칸마다 은은한 디더·결정적 잡음·1px 어두운 가장자리, 윗면 칸은 조금 밝게. 상자의 각 면이 자기 재질 칸 하나를 통째로 쓴다. 재질은 파일마다 하나, 표본은 NEAREST.
4. 동작: `Model.action(이름, 길이, 키, loop)`. 키의 각도는 월드 축 오일러(도, `common.py` 머리말의 부호 규칙). 100fps로 베지어 곡선을 만든 뒤 **3프레임마다 + 키 시각 + 끝 프레임**에서 읽어 직선 키로 다시 박고 그대로 내보낸다(블렌더의 표본 찍기는 끝 프레임을 빠뜨려 0.43초가 0.42초가 된다). 내내 쉬는 자세인 채널은 뺀다 — 게임 로더는 채널이 없는 노드를 쉬는 자세로 둔다. 동작마다 NLA 트랙에 넣고 `export_animation_mode='ACTIONS'`로 동작 하나 = glTF 애니메이션 하나(이름 그대로).
5. 미리보기: Workbench(텍스처 색, 스튜디오 조명, 외곽선) 직교 카메라, 앞(-Y)에서 오른쪽으로 35°·위 32°, 경계 상자에 맞춤. 바닥 판은 미리보기에만 있다(내보낸 뒤 더함). 게임에는 조명이 없으니 실제보다 입체감이 조금 더 보인다.
6. `check_glb.py`: 이미지 정확히 1·PNG·NEAREST·재질 1, 메시에 JOINTS_0/WEIGHTS_0·모프 없음, 동작 이름·길이가 명세대로, 정점 예산, 꼭 있어야 할 노드(glow_*·sword·door_*), 삼각형 감김이 법선과 맞는지(뒤집힌 면).

## 규칙

- 단위 m, 블렌더 Z 위, 발·바닥 = z 0. 캐릭터 앞 = 블렌더 **-Y**(내보내면 glTF +Z = 게임의 앞). 캐릭터 오른쪽 = -X(`arm_r`·`leg_r`·검).
- 빛나는 부위는 오브젝트 이름 `glow_` 접두(게임이 밝게 할 수 있다). 메시 노드 이름은 `<뼈>_geo`, 뼈 노드는 뼈 이름, 아마추어 노드는 `<모델>_rig`.
- 팔레트(`common.PAL`): 숯빛·짙은 청회·돌 회색·밝은 돌·바랜 갈색·뼈·이끼·뿌리 보라갈색·짙은 뿌리·호박빛·엷은 불씨·청록·강철 + 어두운 청동·그늘·나무. **채도 높은 빨강은 쓰지 않는다**(위험 예고 전용).
- 정점 예산: 캐릭터 700, 보스 1600, 소품 400(glTF 정점 = 면마다 4) — 다듬은 소품(0139)은 모델마다 따로(`check_glb.py` `BUDGET`: 깎은 돌 하나가 64).
- 사람이 Blockbench 등으로 바꿀 때도 같은 노드·동작 이름과 축을 지키면 게임 쪽은 그대로다.

## 모델

| 모델 | 뼈 | 동작(초, ↻ = 고리) | 정점 | glow_ |
|---|---|---|---|---|
| `wanderer` 방랑자 1.8m | root, body, head, arm_l, arm_r, hand_r(검 `sword`), leg_l, leg_r, cloak | idle 2.4↻(+ 몸짓 idle_look 2.6·idle_shake 1.3·idle_bell 2.2, 0153 — 뼈 `hood_tip`·`tail`. 무기 넷·등잔은 0155 `weapons`. 두건은 원본 + 면 고르기뿐(0157 — 덧붙인 꼴은 걷었다), `hood_tip` 뼈는 붙은 부위 없이 남는다), walk 0.8↻, attack 0.43(0–0.08 당김·0.08–0.18 베기·~0.43 회복), dodge 0.25, hurt 0.35, turn 0.25 | 636 | glow_eyes |
| `root_husk` 뿌리의 잔재 1.5m | root, body, head, arm_l, arm_r, leg_l, leg_r | idle 1.6↻, walk 1.0↻, attack 1.3(0–0.5 들기·0.5–0.7 내려찍기·~1.3 회복), hurt 0.3 | 580 | glow_core |
| `thorn_beast` 가시짐승 1.1×1.7m | root, body, head, leg_fl, leg_fr, leg_bl, leg_br, tail | idle 1.4↻, run 0.5↻, charge_windup 0.6, charge 0.4↻, recover 0.7, hurt 0.3 | 648 | glow_eyes |
| `shellback` 등딱지짐승 1.0×1.25×1.2m | root, body, head, leg_fl, leg_fr, leg_bl, leg_br | idle 1.6↻, walk 1.0↻, attack 1.3(0–0.6 머리 넣고 앞 들기·0.72 쿵·~1.3), hurt 0.3 | 717 | glow_eyes |
| `pod` 꼬투리 0.6×0.9×0.9m | root, body(피벗 바닥 — 앞발 포함), head, seed(등의 씨앗 꼬투리 + 보라 뿌리), leg_bl, leg_br | idle 1.4↻, walk 0.5↻(작은 깡충), hop 0.4(뛰기 진행으로), plant 1.0(0.18 꼬투리 휙 · 0.45 부풀어 버팀 · 0.6 꽉 오므림), hurt 0.3 | 785 | glow_eyes |
| `bellkeeper` 깨진 종지기 3.0m | root, body(옷자락 밑동 피벗), head, core(glow_core), arm_l(종을 든 팔, +X), bell, arm_r, leg_l, leg_r | idle 2.0↻, walk 1.4↻, toll 1.6(1.0 머리 위로 들기·1.12 앞 땅 내리침), slam 1.5(0.9 비틀어 뒤로·1.0 앞 가로지름), throw 1.2(0.6 들어 젖힘·0.7 털기), charge_windup 0.8, charge 0.5↻, recover 0.8, hurt 0.4, wake 2.0 | 1608 | glow_eyes, glow_core |
| `res_hearth` 불 지키는 주민(늙은 오소리) 1.3m | root, body(엉덩이 피벗), head, arm_l, arm_r, leg_l, leg_r — 주민 셋 공용 `resident_parts.py`(자르는 치수 `Cut` + 뼈) | work 3.2↻(숙여 아궁이를 두 번 쑤심 → 허리 폄), look 2.4↻(조금 굽어 돌아봄) | 612 | – |
| `res_water` 물 모으는 주민(작은 두더지) 1.35m | 같은 꼴(`resident_parts.py`) | work 3.6↻(쪼그려 큰 두 손으로 항아리를 받쳐 들고 올려다봄 → 고쳐 받음 → 들여다봄), look 2.2↻(손을 배 앞에 모음) | 785 | – |
| `res_cloth` 꿰매는 주민(어린 양) 1.35m | 같은 꼴(`resident_parts.py`) | work 2.8↻(왼손으로 천을 받쳐 들고 오른손으로 바늘을 꽂아 실을 당김 두 번), look 2.6↻(손을 내리고 조용히) | 659 | – |
| `gatekeeper` 잠들지 않는 문지기 3.2m | root, body, core, head, arm_l, arm_r, club, leg_l, leg_r, shin_l, shin_r | idle 2.0↻, walk 1.4↻, slam 1.5(0–0.9 들기·0.9–1.1 찍기·~1.5), charge_windup 0.8, charge 0.5↻, wave 1.6(0–1.0 들기·1.0–1.2 땅 치기·~1.6), hurt 0.4, wake 2.0(무릎 꿇음 → 섬) | 1200 | glow_core(뼈 `core`, 숨·피격에 크기), glow_eyes |
| `wall_segment` 벽 조각 2.0×0.8×3.05m | – | – | 904 | – |
| `pillar_broken` 부서진 기둥 1.2×2.9m | – | – | 522 | – |
| `root_clump` 뿌리 덤불 1.8×1.1m | – | – | 448 | – |
| `brazier` 화로(그릇 테 0.86m, 불꽃 끝 1.42m) | – | – | 367 | glow_flame |
| `gate` 성소 문 6×4.6m | door_l(+X, 경첩 x 2.25), door_r(-X, 경첩 x -2.25) | open 1.5(앞 -Y로 100°, 0초 = 닫힘) | 1788 | – |
| `core_crystal` 핵 결정 0.6m | crystal | pulse 1.0↻ | 328 | glow_core |
| `bell` 종 2.1m | bell, clapper | toll 2.0 | 742 | – |
| `rubble` 돌무더기 1m | – | – | 452 | – |
| `chest` 장신구 상자 0.92×0.62×0.74m | lid(뒤 모서리 경첩) | open 0.4(끝 = 열린 자세) | 324 | glow_lock |
| `stone_tablet` 새긴 돌 0.78×1.15m | – | – | 268 | glow_carve |
| `lever` 문 장치 0.62×1.56m | handle(경첩 z 0.8) | pull 0.4(0초 = 당기기 전, 끝 = 당긴 자세) | 328 | glow_grip |

- `wall_segment`: 바닥 가운데가 원점, 끝 면이 x ±1.0이라 x로 나란히 이어진다(앞 = -Y에 뿌리).
- `bell`: 종 몸의 입술 높이는 z 1.05 — 내던지는 종(0133, `render.rs`)이 이 높이로 맞춘다. 종 틀 노드 `bell_frame`은 그때 감춘다.
- `chest`·`stone_tablet`·`lever`: 앞 = -Y(게임의 +z, 화면 아래쪽). 스스로 빛나는 부위의 세기는 게임이 준다(숨 쉬기·곁에 섰을 때).
- `gate`: `door_l`은 모델 자신의 왼쪽(+X, 앞에서 보는 사람에게는 오른쪽). 문짝 메시 노드는 `door_l_leaf`·`door_r_leaf`.
- `gatekeeper`: 명세 뼈에 무릎(`shin_*`)과 손목(`club`)을 더했다 — 무릎 꿇기(`wake`)와 걸음의 무릎 굽힘, 몽둥이 각도.
- 깊이 굽히는 자세(문지기 `wake`·`wave`, 강체 발)에서는 발 끝이 바닥 아래로 조금 들어간다 — 게임의 바닥이 가린다.
