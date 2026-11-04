# Gemini v1 → Meshy 입력 (2026-11-04)

원본: `tools/art/sources/ai/knellroot/`. 시트를 방향별로 잘랐다(가운데 빈 줄 기준, 1024 정사각 흰 바탕, 세로 자리·크기는 원본 그대로).
`*_front_tiny.png` = 게임 크기 미리보기(`tools/art/tiny_preview.py`: 원본 · 쿼터뷰 20px · 락온 50px).

## 방랑자 — 쓸 수 있다
- 20px: 라벤더 두건·민트 몸·꼬리·종이 색 덩어리로 남는다. 50px: 눈·주둥이·볼까지 읽힌다. 지금 상자 모델보다 확실히 낫다.
- 어긋남 하나: **꼬리** — 정면은 옆(캐릭터 왼쪽)으로 뻗었는데 뒤는 가운데로 늘어졌다. Meshy가 둘을 섞으면 꼬리가 갈라지거나 뭉갤 수 있다 → 먼저 정면+옆으로, 뒤를 넣고 싶으면 아래 "다시 받기" 1.
- 키: 정면 842px · 옆·뒤 824px(2% 차이, 괜찮다).

## 잔재 — 정면만 쓴다
- 정면은 좋다: 20px에서도 호박색 눈 한 점·보라 뿌리 둘이 남는다(지금 모델에서는 사라졌던 것).
- 뒤·옆은 정면과 **다른 그림**: 외곽선 있는 셀 화풍, 귀가 네모, 팔이 내려가 있고 몸 비율이 다르다. 그리고 옆(왼쪽을 봄 = 캐릭터의 **왼쪽** 면)에 호박색 눈이 보이는데, 정면에서 호박색 눈은 캐릭터의 **오른쪽**(보는 사람 왼쪽)이다 — 반대.
- → Meshy에는 **정면 한 장**(Image to 3D)으로 먼저. 여러 방향이 필요하면 "다시 받기" 2.

## Meshy 설정 바꿈
- **Symmetry: 끔(또는 Auto)** — 둘 다 좌우가 다르다(꼬리·목도리 끝, 눈 하나·뿌리 자리). 앞에서 "켬"이라 한 것을 고친다.
- 나머지 그대로: Texture 켬(base color), PBR 맵 끔, Triangle 3~5k, A-pose, GLB.

## 다시 받기(같은 대화에 이어서)
1. 방랑자 뒤:
```
Redo only the back view. The tail must match the front view: it comes out of the lower back and curves out to
the character's LEFT side, so seen from behind it appears on the viewer's LEFT. Everything else stays the same.
```
2. 잔재 뒤·옆:
```
The back and side views do not match the front image. Redo both as separate images matching the FRONT image exactly:
same faceted low-poly style with flat lighting and NO outlines, same round ears, same arms spread out in the A-pose,
same body shape and size. The glowing amber eye is on the character's RIGHT side and the empty dark hollow on its
LEFT side. The side view is seen from the character's LEFT side (facing the viewer's left), so it shows the empty
dark hollow, not the amber eye.
```
