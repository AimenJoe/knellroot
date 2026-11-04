# 나노바나나(Gemini) 그림 시안 프롬프트 — 0038(제안)

> 사장: "니가 디자인 시안 제대로 못 만들겠으면 제미나이한테 줄까? Under the roots 개발 하면서 했던것 처럼?"
> 나누기: **나노바나나 = 2D 디자인 시트(정면·옆·뒤)**, Claude = 프롬프트 · 시트를 보고 Blender 스크립트(`Geo.blob`)로 옮기기 · 게임 화면 확인.
> 규칙(KB `guides/image-model-for-game-illustrations.md`, nail에서 직접 겪음): **캐릭터마다 새 대화**, 참고 그림 + 생김새 글(이름만 주면 흔한 그림으로 채운다), [목적·그림체] + [세계 규칙] + [캐릭터] 덩어리를 매번 붙인다, 시트 먼저, 한 장에 넷 넘게 세우지 않는다.
> 받은 그림은 `tools/art/sources/ai/knellroot/<이름>_v1.png`(✦ 로고는 nail `strip_logo.py`처럼 지운다). AI로 그린 것은 숨기지 않는다(nail 0232와 같은 원칙).

## 붙일 참고 그림
- 방랑자: `docs/art/b-0037/wanderer.png`(지금 3D 모델) 또는 `docs/art/concept-0037/b_little_beast_f.png`
- 잔재: `docs/art/concept-0037/husk_f.png`

## 공통 덩어리(매번 맨 앞에 붙인다)

```
[PURPOSE] Character design sheet for a small 3D action game. The sheet will be used as a reference to build a simple
low-poly 3D model out of rounded blocks (soft, toy-like, chunky shapes — no fine detail). Draw it as clean 2D
concept art: flat colors with one soft shade step, a soft dark plum outline, no texture noise, no text, no logos.
Plain solid light grey background. Front view, side view (facing left) and back view in one row, same size,
same height, cleanly separated. Full body, standing straight, arms slightly away from the body.

[WORLD RULES] "Color is life." Living things are soft pastel and warm (lavender, mint, peach, cream, a little gold).
Things eaten by the purple roots have lost their own color: faded ash-lavender grey, with dark purple roots
growing out of cracks. Danger is the only red. The world is a quiet dusk underground, deep indigo, never black.
Cute proportions: about 2 heads tall, big head, big eyes set LOW on the face and close together, NO mouth,
small blush under the eyes, short chubby arms and legs, round mitten paws.
```

## 1. 방랑자(작은 너구리) — 새 대화

```
[CHARACTER] The wanderer: a small young raccoon cub, about 2 heads tall. Round caramel-brown head, cream muzzle
and a small dark nose, a soft brown mask ring around each big dark eye (eyes clearly visible, one white
highlight each), no mouth, pink inside the ears. A small lavender hood sits on the back of the head; the ears
poke out of it. Mint-green bell-shaped cloak with a cream hem, a peach scarf with one tail hanging in front,
a tiny gold bell on the left hip, small warm-brown boots peeking under the cloak. A striped tail curls up
behind (caramel and brown rings, cream tip). Holds a short sword in the right paw (silver blade, gold guard).
Keep the attached image's colors and proportions, but make it more charming and readable.
```

## 2. 뿌리의 잔재(색을 잃은 작은 것) — 새 대화

```
[CHARACTER] A "root husk": a small creature that was once a cute round woodland animal, now eaten by the roots.
Same cute round body and ears as a living one, but ALL its color is gone: faded ash-lavender grey. One eye is
an empty dark hollow, the other is a tiny amber ember dot. Two thin dark-purple roots sprout from the top of its
head and one purple root vein runs across its body like a crack. Its feet are rooted into a dark purple root
base. Droopy ears, slumped arms. Sad and a little eerie, never gory. No mouth.
```

## 3. 가시짐승 — 새 대화

```
[CHARACTER] A "thorn beast": a round, stocky little boar the size of the wanderer's whole body, eaten by the roots.
Faded ash-lavender grey body, a mane of sharp dark-purple thorn-roots along its back (the only sharp shapes),
a blunt snout, one amber ember eye, short stubby legs. It charges in straight lines. Cute silhouette, eerie color.
```

## 4. 잠들지 않는 문지기(보스) — 새 대화

```
[CHARACTER] The "sleepless warden": a big, heavy guardian about three times the wanderer's height, made of round
old stone blocks (soft indigo-grey stone, rounded corners), slow and calm. A crown of dark-purple roots grows from
its head and down its shoulders. In the middle of its chest glows a warm amber core (the stolen color, pulsing).
Long heavy arms, one ending in a big stone club. Two small amber eye slits. Imposing but with a gentle, sleepy face.
```

## 5. 분위기 한 장(키 아트) — 새 대화, 1번·2번 결과를 참고로

```
[SCENE] 16:9 key art. The small raccoon wanderer (attached, keep exactly) walks with a warm lamp through a quiet
underground sanctum at dusk: deep indigo stone, broken pillars, purple roots creeping over the floor.
Around the lamp, colors come back faintly (moss turns green, a flower turns pink). In the distance, a faded
ash-grey husk watches. Far below, a bell glows amber. Calm, lonely, tender. Soft dark plum outlines.
```

---

## Meshy에 넣을 그림 — t3ssel8r 쪽 (Gemini → Meshy Image to 3D → 우리 렌더)

> 사장: "3d 모델링을 다른 툴에 맡긴다면, 다시 우리 t3ssel8r 스타일로 만들 수 있는 거 아니여? 디테일 살려서?" → "제미나이 프롬프트도 t3ssel8r 쪽으로 고쳐줘."
> 배경: `docs/research/2026-11-03-t3ssel8r-style.md`. 그 룩의 픽셀·1px 외곽선·띠 빛·팔레트는 **우리 렌더가** 만든다. 그래서 Gemini 그림은 **픽셀 아트가 아니라**, 낮은 해상도로 찍었을 때 잘 읽히는 **로우폴리 모델 + 손으로 칠한 텍스처**처럼 그린다.

### 왜 이렇게 쓰나
- **크기**: 지금 카메라로 방랑자는 쿼터뷰 약 20px·락온 약 50px(계산, `tools/art/tiny_preview.py`로 본 것: 20px에서 눈은 검은 점 둘, 두건은 한 줄, 칼·뿌리·호박색 눈은 사라진다). → **디테일 = 큰 색 덩어리**: 귀·두건·꼬리·종·뿌리를 과장하고 작은 무늬는 뺀다.
- **빛은 그림에 굽지 않는다**: 우리 렌더가 툰 빛(띠 2~3단)과 림을 얹는다. 그림에 짙은 음영이 있으면 Meshy가 텍스처에 구워 음영이 두 번 든다. 그래서 **고르고 평평한 빛**, 아주 옅은 AO만. (Meshy가 깊이를 잡을 단서는 실루엣·여러 방향 그림으로 준다.)
- **텍스처는 넓은 평면색**: 재질마다 2~3톤, 털 결·잡음·작은 무늬 없음 — 128px로 줄여도 뭉개지지 않게.
- **둥근 = 살아 있음, 각진 = 뿌리**(0037): 살아 있는 몸은 적은 면이지만 매끈하게, 뿌리는 각지고 면이 보이게.
- Meshy 입력 조건(문서): 한 장에 하나, 정면, A-pose, 민짜 바탕, 1040px 넘게, 방향마다 따로 된 그림(최대 4장).

### 순서
1. 새 대화 + 참고 그림 `docs/art/meshy-0038/<이름>_front.png`(색·비율만) + **공통 + 캐릭터** → 정면.
2. 받은 그림을 `python3 tools/art/tiny_preview.py <그림>`으로 20px·50px에서 본다. 얼굴·특징이 안 남으면 "make X bigger"로 다시.
3. 같은 대화에서 **뒤·옆** → Meshy Multi-view(정면·옆·뒤). Meshy는 Texture 켬(base color), PBR 맵 끔, Triangle 3~5k, Symmetry 켬, A-pose, GLB.
4. GLB를 Claude에게 → 부위 자르기·줄이기·텍스처 128 → 게임 화면 480×270에서 지금 모델과 나란히.

### 공통 덩어리(Meshy용, 매번 맨 앞)
```
[PURPOSE] One reference image for an image-to-3D tool. The 3D model will be shown very small in a retro
low-resolution game (the character is only 20 to 50 pixels tall on screen, rendered with pixel-art style toon
lighting and 1-pixel outlines added by the game). So design it to READ AT TINY SIZE.
Render ONE character only, full body, centered, straight front view, neutral A-pose: standing straight, legs
slightly apart, arms down and out about 30 degrees so they don't touch the body. Nothing in the hands.
Style: a stylized low-poly 3D game model with a hand-painted texture. Few big simple forms, chunky and
exaggerated proportions, a strong clear silhouette. Details are PAINTED as large flat color areas, not sculpted.
Each material uses only 2 or 3 flat tones. No fur strands, no texture noise, no small patterns, no tiny accessories.
Lighting: flat, even and shadowless, like an unlit model; at most a very faint ambient occlusion. No highlights
except painted eye highlights. No outlines. NOT pixel art, NOT voxel. No floor, no pedestal, no text, no logo.
Pure plain white background. Square image, the character fills about 80% of the height.

[WORLD RULES] "Color is life." Living things are soft pastel and warm (lavender, mint, peach, cream, a little gold),
with smooth rounded forms. Things eaten by the purple roots have lost their color: faded ash-lavender grey, and the
dark purple roots growing out of them are the only sharp, angular, faceted shapes. Danger is the only red.
Cute proportions: about 2 heads tall, big head, big eyes set LOW on the face and close together, NO mouth,
short chubby arms and legs, round mitten paws.
```

### 방랑자 — 새 대화
```
[CHARACTER] The wanderer: a small young raccoon cub. Big round caramel-brown head. The face must read at 20 pixels:
a wide dark-brown mask band across both eyes, two LARGE glossy black eyes inside it with one big white highlight
each, a cream muzzle patch below, a dark nose dot, no mouth, a soft pink blush dot under each eye.
Big round ears with pink insides poking out of a chunky lavender hood that frames the head as one clear color block.
A short mint-green bell-shaped cloak with a cream hem (arms stay free), a thick peach scarf, an OVERSIZED gold bell
at the chest, chunky warm-brown boots. A big fluffy tail with 3 wide rings (caramel, dark brown, cream tip),
sticking out to one side so it is visible from the front.
Use the attached image ONLY for colors and proportions; its blocky shapes are a placeholder — make every living
form round and soft.
```

### 잔재 — 새 대화
```
[CHARACTER] A "root husk": a small round woodland creature the size of a raccoon cub, eaten by the roots.
All its color drained: one flat faded ash-lavender grey, matte like dry clay, with 2 or 3 big painted cracks.
One eye is an empty dark hollow; the other is a LARGE bright amber ember with a glow, the brightest spot on the body.
Two thick, tall, angular dark-purple roots grow out of the top of its head like a broken crown (clearly visible in
silhouette), and one thick purple root vein runs diagonally across its chest. Droopy ears, slumped shoulders but
still in the A-pose. No mouth, no base under the feet. Sad and a little eerie, never gory.
```

### 가시짐승 — 새 대화(시안 그림 없음 — 참고 그림 없이, 또는 잔재 결과를 붙여 같은 세계로)
```
[CHARACTER] A "thorn beast": a round, stocky little boar about as big as the raccoon cub's whole body, standing on
four short stubby legs (for this one the pose is a neutral four-legged stance, side by side legs, head forward).
Faded ash-lavender grey body with a blunt rounded snout. A tall mane of big angular dark-purple thorn-roots runs
along its back — the only sharp shapes, clearly readable in silhouette. One LARGE bright amber ember eye on each side.
```

### 잠들지 않는 문지기(보스) — 새 대화
```
[CHARACTER] The "sleepless warden": a big heavy guardian about three times the raccoon cub's height, made of a few
big rounded old stone blocks (soft indigo-grey stone, simple painted stone seams, no fine cracks). A crown of thick
angular dark-purple roots grows from its head and down its shoulders. In the middle of its chest, a LARGE glowing
warm amber core — the brightest thing on it. Long heavy arms in the A-pose, the right one ending in a big round
stone club. Two small amber eye slits. Imposing but with a gentle, sleepy face.
```

### 뒤·옆 (같은 대화에서 정면을 받은 뒤)
```
Now render exactly the same character as a separate image, same scale, same flat lighting, same white background,
same pose, seen from directly behind (back view). Then another separate image seen from its left side
(side view, facing left). Do not change any colors, proportions or details.
```
