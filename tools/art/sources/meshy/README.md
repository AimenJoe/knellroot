# Meshy 원본 — Knellroot

**AI 생성**: Meshy(meshy.ai) Image to 3D, 무료 플랜 → **CC BY 4.0**(출처 표기 필요, 상업 사용 가능 — https://www.meshy.ai/pricing, 2026-11-04 확인).
입력 그림은 Gemini(나노바나나) 원본 `../ai/knellroot/wanderer_v1_*.png`.

| 파일 | 받은 날 | 무엇 |
|---|---|---|
| `raccoon_v1.glb` | 2026-11-04 | 방랑자 v1(Meshy 이름 "Cozy Bell Raccoon"), 삼각형 101,304 · 2048 텍스처 셋 |
| `raccoon_v1_low.glb`(+ `_tex.png`) | 2026-11-04 | 위를 `meshy_convert.py ... 1.4 1900 256`으로 줄인 것 — `wanderer.py`가 읽는다(0038) |
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

게임에 넣을 때: `blender -b -P tools/art/blender/meshy_convert.py -- <원본> <출력> 1.4 2300 256`(줄이기 + 새 UV + 색 굽기).
배포에 쓰면 `assets/LICENSES.md`에 "Meshy로 만든 모델(AI), CC BY 4.0"을 적는다.
