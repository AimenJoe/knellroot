# Meshy 참고 그림·프롬프트 (0038 제안 단계)

**AI 사용 표시**: 이 폴더의 그림은 이 레포의 Blender 스크립트(`tools/art/blender/meshy_refs.py`, Claude가 씀)로 찍었다. Meshy로 만든 결과를 쓰면 출처에 "Meshy(AI)"라고 적는다.

## 그림
| 파일 | 무엇 | 넣는 곳 |
|---|---|---|
| `wanderer_front/side/back.png` | 지금 방랑자(B 너구리) — 정면·옆(왼쪽을 봄)·뒤 | Multi-Image to 3D (최대 4장) |
| `wanderer_threeq.png` | 3/4 | Image to 3D 한 장만 쓸 때 |
| `husk_*.png` | 잔재 시안(물이 빠진 너구리 껍데기 + 보라 뿌리) | 같음 |

주의: 지금 모델은 상자가 둥근 수준이라, 그림을 그대로 넣으면 Meshy도 각진 모양을 따라 한다. **비율·색의 참고**로 보고, 결과가 각지면 아래 Text to 3D를 쓰거나 Gemini 시트(`../gemini-prompts-0038.md`)를 먼저 받아 그 그림을 넣는다.

## Text to 3D — 방랑자
```
A cute chibi raccoon cub wanderer, 2 heads tall, big round head, standing in a relaxed A-pose.
Soft lavender hood with round raccoon ears poking through, dark eye-mask ring around large glossy black eyes set low on the face, small cream muzzle and tiny nose, pink blush, no mouth.
Short mint-green cloak with a small brass bell at the chest, peach knitted scarf, cream belt, small brown boots.
Fluffy tail with brown and cream rings.
Holds a short simple sword in the right hand.
Smooth rounded stylized shapes, pastel colors, hand-painted flat colors, game-ready low poly, clean silhouette.
```
- 피할 것(입력 칸이 있으면): `realistic fur, photorealistic, mouth, teeth, sharp spikes, armor plates, extra fingers, text, base, pedestal`
- 칼이 몸과 붙어 나오면 칼을 빼고 만든다(칼은 우리 쪽 따로 된 부위).

## Text to 3D — 잔재(husk)
```
A small hollow husk shaped like a chibi raccoon cub, 2 heads tall, standing still with slumped shoulders.
Its color has been drained: faded ash-grey body, cracked like dry clay, empty dark eye sockets with one tiny dim amber glint.
Thin purple roots grow out of the cracks and from the top of the head, sharp and thorny.
Smooth rounded stylized body, sharp angular roots, muted pastel grey with purple accents, game-ready low poly.
```
- 피할 것: `blood, gore, realistic, scary teeth, mouth, pedestal, base`

## 설정
- 스타일: 실사 말고 스타일라이즈(카툰·로우폴리 계열).
- Topology Triangle, 목표 폴리곤 3,000~5,000(게임 예산 2,400보다 넉넉히 받고 우리가 줄인다).
- Symmetry 켬. 포즈 옵션이 있으면 A-pose.
- 텍스처: PBR 끄거나 base color만. 우리 렌더는 조명 없이 정점·아틀라스 색만 쓴다.
- 받을 형식: GLB.

## 받은 뒤
Meshy 결과(고폴리·한 덩어리 메시)는 그대로 게임에 넣지 않는다. 지금 파이프라인은 뼈마다 붙은 강체 부위(`common.py`)라, 결과를 **형태 참고**로 스크립트 비율·곡률을 다시 잡는다. 결과 GLB를 `assets/vendor/`(커밋 안 함)에 두고 알려 주면 비교 그림을 찍는다.

## 라이선스 — 쓰기 전에 확인
무료 플랜 결과물의 권리는 출처마다 다르게 적혀 있다(도움말: CC BY 4.0 / 블로그: 개인·학습용). 공개 레포·배포 게임에 넣기 전에 Meshy 약관(Terms)에서 지금 플랜의 조건을 확인한다.
