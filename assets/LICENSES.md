# 에셋 출처

넣을 때 여기에 한 줄씩: 파일, 출처 URL, 라이선스, 바꾼 것, AI를 썼으면 어디에 어떻게.

| 파일 | 출처 | 라이선스 | 비고 |
|---|---|---|---|
| `models/test_critter.gltf` | 이 레포 `tools/art/make_test_model.py`가 코드로 찍음(상자·텍스처 16×16·애니메이션 모두) | 이 레포와 같음 | P4 스파이크 시험 모델(0013). AI 그림·외부 에셋 없음 |
| `fonts/GowunBatang-UI-{Regular,Bold}.ttf` | 고운바탕 https://github.com/yangheeryu/Gowun-Batang (Google Fonts 판 https://github.com/google/fonts/tree/main/ofl/gowunbatang) | SIL OFL 1.1(`fonts/GowunBatang-OFL.txt`, 예약 이름 없음) | `tools/fonts/subset_ui.py`로 i18n 표 글자 + 라틴만 남긴 서브셋(0021) |
| `models/*.glb` | 이 레포 `tools/art/blender/<이름>.py`(Blender 4.0 bpy)가 만듦 — 상자 부위·텍스처 아틀라스(코드로 찍은 무늬)·동작 모두 스크립트 | 이 레포와 같음 | 1단계 초안 모델(0019). 스크립트는 Claude(AI)가 사장 지시로 썼다. 외부 에셋·AI 이미지 생성 없음(아래 `wanderer.glb` 빼고) |
| `models/wanderer.glb`(몸) | **AI 생성**: Gemini(나노바나나) 그림 `tools/art/sources/ai/knellroot/wanderer_v1_*.png` → Meshy(https://www.meshy.ai) Image to 3D 무료 플랜 `tools/art/sources/meshy/raccoon_v1.glb` | **CC BY 4.0** — "Cozy Bell Raccoon", Meshy로 만듦(https://www.meshy.ai/pricing: 무료 플랜 결과물은 CC BY 4.0) | `meshy_convert.py`로 줄이고 색을 다시 구움, `wanderer.py`가 부위로 잘라 뼈에 붙임(0038). 칼·등잔·동작은 스크립트 |
| `models/gatekeeper.glb`(몸) | **AI 생성**: Gemini(나노바나나) 그림 `tools/art/sources/ai/knellroot/warden_v2_*.png` → Meshy(https://www.meshy.ai) 7.1 Image to 3D 무료(iOS 앱) `tools/art/sources/meshy/warden_v2.glb` | **CC BY 4.0** — "Sleepy Stone Guardian", Meshy로 만듦(무료 플랜 결과물은 CC BY 4.0) | `meshy_convert.py`로 줄이고 색을 다시 구움, `gatekeeper.py`가 부위로 잘라 뼈에 붙이고 돌 색을 어둡게(0060 W3). 동작은 스크립트 |

유료·제3자 팩 원본은 `assets/vendor/`(gitignore)에만.
