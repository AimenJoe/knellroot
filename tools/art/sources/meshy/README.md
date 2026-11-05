# Meshy 원본 — Knellroot

**AI 생성**: Meshy(meshy.ai) Image to 3D, 무료 플랜 → **CC BY 4.0**(출처 표기 필요, 상업 사용 가능 — https://www.meshy.ai/pricing, 2026-11-04 확인).
입력 그림은 Gemini(나노바나나) 원본 `../ai/knellroot/wanderer_v1_*.png`.

| 파일 | 받은 날 | 무엇 |
|---|---|---|
| `raccoon_v1.glb` | 2026-11-04 | 방랑자 v1(Meshy 이름 "Cozy Bell Raccoon"), 삼각형 101,304 · 2048 텍스처 셋 |
| `raccoon_v1_low.glb`(+ `_tex.png`) | 2026-11-04 | 위를 `meshy_convert.py ... 1.4 1900 256`으로 줄인 것 — `wanderer.py`가 읽는다(0038) |
| `warden_v2.glb` | 2026-11-05 | 문지기 v2(Meshy 이름 "Sleepy Stone Guardian", 사장이 Image to 3D로 받음 — 입력 `../ai/knellroot/warden_v2_front.png`), 삼각형 318,304 · 텍스처 셋(base color 포함). 플랜(무료면 CC BY 4.0)은 사장에게 확인 중 |

게임에 넣을 때: `blender -b -P tools/art/blender/meshy_convert.py -- <원본> <출력> 1.4 2300 256`(줄이기 + 새 UV + 색 굽기).
배포에 쓰면 `assets/LICENSES.md`에 "Meshy로 만든 모델(AI), CC BY 4.0"을 적는다.
