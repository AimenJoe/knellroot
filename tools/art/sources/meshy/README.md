# Meshy 원본 — Knellroot

**AI 생성**: Meshy(meshy.ai) Image to 3D, 무료 플랜 → **CC BY 4.0**(출처 표기 필요, 상업 사용 가능 — https://www.meshy.ai/pricing, 2026-11-04 확인).
입력 그림은 Gemini(나노바나나) 원본 `../ai/knellroot/wanderer_v1_*.png`.

| 파일 | 받은 날 | 무엇 |
|---|---|---|
| `raccoon_v1.glb` | 2026-11-04 | 방랑자 v1(Meshy 이름 "Cozy Bell Raccoon"), 삼각형 101,304 · 2048 텍스처 셋 |
| `raccoon_v1_low.glb`(+ `_tex.png`) | 2026-11-04 | 위를 `meshy_convert.py ... 1.4 2800 512`로 줄인 것(0153 — 그전 1900·256: 꼬리 줄무늬·눈이 뭉개졌다) — `wanderer.py`가 읽는다(0038) |
| `warden_v2.glb` | 2026-11-05 | 문지기 v2(Meshy 이름 "Sleepy Stone Guardian", 사장이 Image to 3D로 받음 — 입력 `../ai/knellroot/warden_v2_front.png`), 삼각형 318,304 · 텍스처 셋(base color 포함). 무료 플랜(iOS 앱, Meshy 7.1, 텍스처 포함으로 받음) → CC BY 4.0 |
| `warden_v2_low.glb`(+ `_tex.png`) | 2026-11-05 | 위를 `meshy_convert.py ... 3.2 2200 256`으로 줄인 것(정점 2,192·삼각형 1,987) — `gatekeeper.py`가 읽는다. 31만 삼각형이라 변환에 30분 걸렸다 |
| `husk_v2.glb` | 2026-11-05 | 잔재 v2(Meshy 이름 "Crystal Gloom Bear", 사장이 웹에서 Meshy 6 Lite Image to 3D — 입력 `../ai/knellroot/husk_v1_front.png`), 삼각형 91,586 · 텍스처 셋. 무료 → CC BY 4.0 |
| `husk_v2_low.glb`(+ `_tex.png`) | 2026-11-05 | 위를 `meshy_convert.py ... 1.5 700 128`로 줄인 것(정점 656·삼각형 554, 3분) — `root_husk.py`가 읽는다 |
| `thorn_v2.glb` | 2026-11-05 | 가시짐승 v2(Meshy 이름 "Crystalback Piglet", 사장이 웹 Meshy 6 Lite — 입력 Gemini v2 그림), 삼각형 98,462 · 텍스처 셋. 무료 → CC BY 4.0 |
| `thorn_v2_low.glb`(+ `_tex.png`) | 2026-11-05 | 위를 `meshy_convert.py ... 1.3 700 128`로 줄인 것(정점 682·삼각형 614) — `thorn_beast.py`가 읽는다 |
| `shellback_v1.glb` | 2026-11-09 | 등딱지짐승 v1(Meshy 이름 "Crystalback Turtle", 사장이 Image to 3D — 입력 Gemini 그림 `../ai/knellroot/shellback_v2_*.png`, 프롬프트 `docs/art/gemini-prompts-0132.md`), 삼각형 196,104 · 텍스처 셋. 무료 → CC BY 4.0 |
| `shellback_v1_low.glb`(+ `_tex.png`) | 2026-11-09 | 위를 `meshy_convert.py ... 1.2 700 128`로 줄인 것(정점 642·삼각형 614, 15분) — `shellback.py`가 읽는다 |
| `pod_v1.glb` | 2026-11-09 | 꼬투리 v1(Meshy 이름 "Sproutback Frog", 사장이 Image to 3D — 입력 `../ai/knellroot/pod_v1_front.png`), 삼각형 169,354 · 텍스처 셋. 무료 → CC BY 4.0 |
| `pod_v1_low.glb`(+ `_tex.png`) | 2026-11-09 | 위를 `meshy_convert.py ... 0.9 700 128`로 줄인 것(정점 655·삼각형 564, 8분) — `pod.py`가 읽는다(꼬투리 부위는 텍스처 색으로 가른다) |
| `bellkeeper_v1.glb` | 2026-11-09 | 깨진 종지기 v1(Meshy 이름 "Hollow Lantern Bellringer", 사장이 Image to 3D — 입력 `../ai/knellroot/bellkeeper_v1_front.png`), 삼각형 164,932 · 텍스처 셋. 무료 → CC BY 4.0. 종은 왼손 앞에 놓였다 |
| `bellkeeper_v1_low.glb`(+ `_tex.png`) | 2026-11-09 | 위를 `meshy_convert.py ... 3.0 1500 256`으로 줄인 것(정점 1,498·삼각형 1,318, 7분) — `bellkeeper.py`가 읽는다(종·핵·눈은 텍스처 색으로 가른다) |
| `hearth_v1.glb` | 2026-11-09 | 불 지키는 주민 v1(Meshy 이름 "Little Apron Bandit", 사장이 Image to 3D — 입력 `../ai/knellroot/hearth_v1_front.png`), 텍스처 셋. 무료 → CC BY 4.0. Meshy가 짧은 꼬리를 더했다 |
| `hearth_v1_low.glb`(+ `_tex.png`) | 2026-11-09 | 위를 `meshy_convert.py ... 1.3 700 128`로 줄인 것(정점 612·삼각형 582, 5분) — `res_hearth.py`가 읽는다 |
| `water_v1.glb` | 2026-11-09 | 물 모으는 주민 v1(Meshy 이름 "Sleepy Blue Mole", 사장이 Image to 3D — 입력 `../ai/knellroot/water_v1_front.png`), 텍스처 셋. 무료 → CC BY 4.0 |
| `water_v1_low.glb`(+ `_tex.png`) | 2026-11-09 | 위를 `meshy_convert.py ... 1.35 700 128`로 줄인 것(정점 700·삼각형 622, 5분) — `res_water.py`가 읽는다 |
| `cloth_v1.glb` | 2026-11-09 | 꿰매는 주민 v1(Meshy 이름 "Patchwork Lamb", 사장이 Image to 3D — 입력 `../ai/knellroot/cloth_v2_front.png`), 텍스처 셋. 무료 → CC BY 4.0 |
| `cloth_v1_low.glb`(+ `_tex.png`) | 2026-11-09 | 위를 `meshy_convert.py ... 1.35 700 128`로 줄인 것(정점 659·삼각형 523, 5분) — `res_cloth.py`가 읽는다 |
| `raccoon_v2_rig.glb` | 2026-11-14 | 방랑자 v2의 **리깅·동작**: 사장이 Meshy Pro로 받은 GLB 넷(`…_animated_mixamo_All_animation.glb` 둘 + `…_2.glb`, 각 14~16 MB, 뼈 58 · 동작 13·20·20 + 2026-11-15 대기 `…_Idle_02.glb`)을 `meshy_rig.py`로 합친 것 — 줄인 스킨 몸(정점 2,814 · 텍스처 512) + 뼈 + 쓸 동작(2026-11-15 0196: 줍기 `Collect_Object`·활쏘기 `Archery_Shot`, 0197: 뒤로 뛰기 `Back_Jump`, 0198: 넷째 파일 `…_All_animation_3.glb`의 잽 둘·어퍼컷·앞차기·막기·마법 시전·싸움 뒷걸음 일곱, 0199: 모아 베기 `Charged_Slash`·방패 밀기 `Shield_Push_Left`, 0206: 도끼 내려찍기 `Charged_Axe_Chop`·올려 베기 `Charged_Upward_Slash`를 더함 — 동작 49 — 셋째 묶음 이름 없는 것은 `M01`~`M18`). 원본 셋은 레포에 넣지 않는다(사장의 Meshy 계정). `wanderer_skin.py`가 읽는다(0193) |

게임에 넣을 때: `blender -b -P tools/art/blender/meshy_convert.py -- <원본> <출력> 1.4 2300 256`(줄이기 + 새 UV + 색 굽기).
배포에 쓰면 `assets/LICENSES.md`에 "Meshy로 만든 모델(AI), CC BY 4.0"을 적는다.
| `raccoon_v3.glb`(+ `_low.glb`·`_low_tex.png`) | 2026-11-19 | 방랑자 v3(0246): 그림 `../ai/knellroot/wanderer_v3a_front.png`(나노바나나 pro) → Meshy Image to 3D **7.1**(`latest`, 30 크레딧, 리메시 3만 삼각형·빛 지움). `_low`는 `meshy_convert.py … 1.4 2800 512` |
| `raccoon_v3_rig.glb` | 2026-11-19 | 위에 Meshy Rigging API(`tools/art/meshy_rigging.py`, 템플릿 mixamo + 손가락, 5 크레딧) — 뼈 65(`mixamorig:*` 몸통 + 두건·망토·꼬리 `Bone_0xx`). 동작은 v2의 49개를 옮겨 쓴다(0246) |
| `raccoon_v3_rig_anim.glb` | 2026-11-19 | 게임이 쓰는 v3 몸(0246): `blender -b -P tools/art/blender/meshy_rig.py -- raccoon_v3_rig_anim.glb raccoon_v3_rig.glb --retarget raccoon_v2_rig.glb --target 2850` — 줄인 몸(glTF 정점 2,825·텍스처 512) + v2 동작 49개를 뼈 이름(`mixamorig:*` 29개)으로 옮겨 담음. 0 크레딧 |

