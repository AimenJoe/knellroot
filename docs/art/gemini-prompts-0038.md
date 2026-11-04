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
