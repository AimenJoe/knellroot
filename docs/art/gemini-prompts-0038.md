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

## Meshy에 넣을 그림 (Gemini → Meshy Image to 3D)

위의 시트는 **Claude가 보고 스크립트로 옮기는** 용도다. Meshy에 넣을 그림은 조건이 다르다:
- **한 장에 하나, 정면, A-pose, 민짜 흰 바탕, 고른 빛**(Meshy 문서: 앞을 보는 중립 자세, 한 대상, 1040px 넘게, 바탕을 지움. Multi-view는 최대 4장 — **방향마다 따로 된 그림**, 한 장짜리 시트가 아니다).
- **외곽선·평면색 대신 입체 렌더처럼**: 그림자 단계로 부피가 읽혀야 깊이를 잡는다. 굵은 외곽선은 면에 그려진 선이나 이상한 홈으로 나온다(추정 — 직접 확인 전).
- **칼·등잔은 빼고**: 가는 것은 몸에 붙거나 녹는다. 칼은 우리 쪽 따로 된 부위.
- 순서: ① 정면 한 장(가장 중요, 한 장만으로도 됨) → ② 같은 대화에서 뒤·옆 → Meshy Multi-view에 ①②. 시트로 받았으면 Claude에게 주면 방향별로 잘라 준다.
- 참고 그림 `meshy-0038/wanderer_front.png`는 **색·비율만** — 모양은 상자라 그대로 따라 하면 안 된다고 적는다.

### 공통 덩어리(Meshy용, 매번 맨 앞)
```
[PURPOSE] A single reference image for an image-to-3D tool. Render ONE character only, full body, centered,
facing the camera straight on (front view), neutral A-pose: standing straight, legs slightly apart,
arms held down and out about 30 degrees away from the body so they don't touch it. Nothing in the hands.
Style: a cute stylized 3D figure like a soft vinyl toy or clay figurine, smooth rounded chunky shapes,
matte pastel materials, soft even studio lighting from the front, gentle shading so the volume is readable,
no outlines, no hard cast shadow, no floor, no pedestal, no text, no logo. Pure plain white background.
Square image, the character fills about 80% of the height.

[WORLD RULES] "Color is life." Living things are soft pastel and warm (lavender, mint, peach, cream, a little gold).
Things eaten by the purple roots have lost their color: faded ash-lavender grey with dark purple roots growing
out of cracks. Cute proportions: about 2 heads tall, big head, big eyes set LOW on the face and close together,
NO mouth, small blush under the eyes, short chubby arms and legs, round mitten paws.
```

### 방랑자 — 새 대화
```
[CHARACTER] The wanderer: a small young raccoon cub. Round caramel-brown head, cream muzzle, small dark nose,
a soft brown mask ring around each big glossy dark eye (one white highlight each), no mouth, pink inside the ears.
A small lavender hood on the back of the head with the round ears poking out of it. Short mint-green bell-shaped
cloak with a cream hem that does NOT cover the arms, a peach knitted scarf, a tiny gold bell at the chest,
small warm-brown boots. A fluffy striped tail (caramel and brown rings, cream tip) sticking out to one side
so it is visible from the front. Use the attached image ONLY for colors and proportions; its blocky shapes are
a placeholder — make everything round and soft.
```

### 잔재 — 새 대화
```
[CHARACTER] A "root husk": a small round woodland creature the size of a raccoon cub, eaten by the roots.
ALL color drained: faded ash-lavender grey, matte like dry clay with a few shallow cracks. One eye is an empty
dark hollow, the other a tiny amber ember dot. Two thin dark-purple roots sprout from the top of the head and
one purple root vein runs across the body. Droopy ears, slumped shoulders but still in the A-pose. No mouth,
no base under the feet. Sad and a little eerie, never gory.
```

### 뒤·옆 (같은 대화에서 정면을 받은 뒤)
```
Now render exactly the same character, same scale, same lighting, same white background, same A-pose,
seen from directly behind (back view). Then a separate image seen from its left side (side view, facing left).
Do not change any colors, proportions or details.
```
