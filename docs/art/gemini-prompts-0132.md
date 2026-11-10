# 나노바나나(Gemini) → Meshy 참조 그림 프롬프트 — 0132(v0.5.0 모델·동작: 등딱지짐승·꼬투리·깨진 종지기·주민 셋)

> 사장 2026-11-09: "Meshy에 넣을 이미지 작성을 위한 프롬프트 및 참조 이미지 줘".
> 흐름은 0038과 같다(`docs/art/gemini-prompts-0038.md`, KB `guides/ai-mesh-to-rigid-part-model.md`): **나노바나나 = 참조 그림 한 장** → `tools/art/tiny_preview.py`로 작은 크기 확인 → **Meshy Image to 3D**(무료 = 정면 한 장 + Meshy 6 Lite, 텍스처 없음 / 유료 7.1이면 정면·옆·뒤 Multi-view) → GLB를 Claude에게 → 줄이기·색 굽기·부위 자르기·뼈·동작.
> 받은 그림은 `tools/art/sources/ai/knellroot/<이름>_v1_front.png`(✦ 로고는 지운다). AI로 그린 것은 숨기지 않는다.

## 규칙(앞에서 배운 것)
- **캐릭터마다 새 대화**. 매번 [첨부 설명] + [PURPOSE] + [WORLD RULES] + [CHARACTER]를 다 붙인다(이름만 주면 흔한 그림으로 채운다).
- **귀여움 문법이 먼저**(0036·0061): 큰 머리, 큰 눈 낮게, 입 없음, 둥근 실루엣. 각지고 뾰족한 것은 **뿌리만**(0037). 문지기 v1이 "마인크래프트 골렘"이 된 까닭 = 상자 모델을 참조로 붙이고 귀여움 문법을 꺼서(0061) → **지금 상자 그림은 붙이지 않는다**.
- 네발짐승은 A-pose 대신 **3/4 앞(약 45°) 네 발로 선 자세** — 가시짐승 v2가 이 한 장으로 Meshy 6 Lite에서 네 발·갈기·꼬리까지 나왔다.
- 빛은 그림에 굽지 않는다(우리 렌더가 툰 빛·외곽선을 얹는다): 고르고 평평한 빛, 넓은 평면색, 재질마다 2~3톤.
- 화면 크기(쿼터뷰 약 20~50px, 보스 60~150px)에서 읽히게 — 받은 그림을 그 크기로 줄여 보고, 특징이 안 남으면 "make X bigger"로 다시.

## 첨부할 참조 그림(`docs/art/refs-0132/`)
| 파일 | 무엇 | 붙이는 곳 |
|---|---|---|
| `01_wanderer_front.png` | 방랑자(플레이어) — 세계의 귀여움 문법·비율·크기 기준 | **모두** |
| `02_husk_front.png` | 잔재 — 뿌리에 먹힌 것의 색(잿빛 라벤더 + 보라 뿌리 + 호박빛 눈) | 등딱지짐승·꼬투리·종지기 |
| `03_warden_v2_front.png` | 첫 문지기 — 보스의 귀여운 무게감, 몸의 호박빛 핵 | 종지기 |
| `04_thorn_v2_meshy.png` | 가시짐승 Meshy 결과(정면·옆·뒤) — 네발짐승 꼴·완성도 기준 | 등딱지짐승·꼬투리 |
| `05_scale_lineup.png` | 게임 안 키 비교(사람 확인용 — Gemini에 붙이지 않아도 된다) | — |

> 주민은 **산 것**이라 잔재·문지기 그림을 붙이지 않는다(잿빛·뿌리가 옮아온다). 방랑자 그림만.

## 공통 — 적(뿌리에 먹힌 것) [PURPOSE] + [WORLD RULES]
```
[PURPOSE] One reference image for an image-to-3D tool. The 3D model will be shown very small in a retro
low-resolution game (this creature is only 20 to 50 pixels tall on screen, rendered with pixel-art style toon
lighting and 1-pixel outlines added by the game). So design it to READ AT TINY SIZE.
Render ONE creature only, full body, centered. Nothing else in the image.
Style: a stylized low-poly 3D game model with a hand-painted texture. Few big simple forms, chunky and
exaggerated proportions, a strong clear silhouette. Details are PAINTED as large flat color areas, not sculpted.
Each material uses only 2 or 3 flat tones. No fur strands, no texture noise, no small patterns, no tiny accessories.
Lighting: flat, even and shadowless, like an unlit model; at most a very faint ambient occlusion. No highlights
except painted eye highlights. No outlines. NOT pixel art, NOT voxel, NOT Minecraft-like, NOT made of boxes.
No floor, no pedestal, no text, no logo. Pure plain white background. Square image, the creature fills about 80%.

[WORLD RULES] "Color is life." Living things are soft pastel and warm (lavender, mint, peach, cream, a little gold),
with smooth rounded forms. Things eaten by the purple roots have lost their color: faded ash-lavender grey, and the
dark purple roots growing out of them are the only sharp, angular, faceted shapes. Danger is the only red.
Cute proportions: big head, big eyes set LOW on the face and close together, NO mouth, short chubby limbs.
```

## 1. 등딱지짐승(Shellback, 키 1.2m — 방랑자의 2/3) — 새 대화
첨부: `01_wanderer_front.png` + `02_husk_front.png` + `04_thorn_v2_meshy.png`
> 게임에서: 앞을 등딱지로 막아 앞에서 든 가벼운 공격을 튕긴다(0120). 옆·뒤·강타는 들어간다 → **등딱지의 앞 가장자리가 얼굴을 덮는 차양**이 실루엣에서 읽혀야 한다.
```
Attached: our player character (a raccoon cub), a "root husk" enemy and a "thorn beast" enemy from the same game.
Do NOT draw any of them. Draw a DIFFERENT creature in the same cute style, proportion grammar and color rules:
use the husk and the thorn beast ONLY for the drained ash-grey color, the purple roots and the amber ember eyes.

[PURPOSE] ... (paste the common enemy PURPOSE block)
Pose for this one: seen from a three-quarter front view (about 45 degrees), standing still on four short legs,
legs slightly apart and clearly separated, head low and forward.

[WORLD RULES] ... (paste the common enemy WORLD RULES block)

[CHARACTER] The "shellback": a chubby little armadillo-turtle creature, a bit smaller than the raccoon cub, eaten
by the roots. Its whole back is ONE big, thick, dome-shaped shell made of 3 or 4 broad, gently faceted plates —
much bigger than its body, like a heavy round lid. The FRONT rim of the shell juts forward over its face like a
visor or a shield, so from the front you see mostly shell, with the face peeking out underneath.
Under the visor: a small round head with two LARGE bright amber ember eyes set low and close together, no mouth.
Four very short stubby legs with round feet, a tiny stub tail.
All its color drained: body and shell in faded ash-lavender grey (2 flat tones: the shell slightly darker), matte.
Two or three big painted cracks run across the shell, and from the widest crack a small cluster of angular
dark-purple roots sticks up (the only sharp shapes). It looks stubborn and a little sad, hiding behind its shell —
never scary.
```

## 2. 꼬투리(Pod, 키 0.9m — 방랑자의 절반) — 새 대화
첨부: `01_wanderer_front.png` + `02_husk_front.png` + `04_thorn_v2_meshy.png`
> 게임에서: 멀리서 바닥에 무언가를 심고(잠시 뒤 터짐), 가까이 오면 깡충 물러난다(0120). 이름은 "뿌리에서 났다"고 정하지 않는다(0116 ECO-02 열린 채) → **꼬투리는 보라 뿌리 색이 아니라** 마른 씨앗 꼬투리 색. 뛰는 몸이라 **긴 뒷발**이 읽혀야 한다.
```
Attached: our player character (a raccoon cub), a "root husk" enemy and a "thorn beast" enemy from the same game.
Do NOT draw any of them. Draw a DIFFERENT creature in the same cute style, proportion grammar and color rules:
use the husk and the thorn beast ONLY for the drained ash-grey color, the purple roots and the amber ember eyes.

[PURPOSE] ... (paste the common enemy PURPOSE block)
Pose for this one: seen from a three-quarter front view (about 45 degrees), crouching still and ready to hop,
all four feet on the ground and clearly separated.

[WORLD RULES] ... (paste the common enemy WORLD RULES block)

[CHARACTER] The "pod": a tiny round hopping creature, about half the raccoon cub's height, like a cross between a
jerboa and a little frog, eaten by the roots. A big round head merged into a small round body, two LARGE bright amber
ember eyes set low, no mouth, two short drooping ears. Small front paws and two LONG, strong hind feet folded under
it, made for hopping (clearly readable in silhouette). On its back it carries one big closed seed pod, like a
bean pod or a dry seed husk, as big as its body, strapped to it by nothing — it simply grows there. The seed pod is a
dry, dusty olive-brown (2 flat tones) with a split seam along its top, NOT purple. The creature itself is drained
ash-lavender grey (2 flat tones) with one painted crack; one thin angular dark-purple root curls out of the pod's
seam (the only sharp shape). Skittish and shy, never scary.
```

## 3. 깨진 종지기(The Broken Bellkeeper, 보스, 키 3.0m — 문지기와 비슷) — 새 대화
첨부: `01_wanderer_front.png` + `02_husk_front.png` + `03_warden_v2_front.png`
> 게임에서(0125·0128): 붙으면 땅을 쳐 고리를 울리고(울림 고리) 종을 앞으로 휘두르고, 멀면 종 조각을 부채꼴로 던지고, 종을 끌며 돌진한다. 이기면 핵이 남는다.
> 세계관(0116): **"깨진 것이 종인지 지기인지는 열어 둔다"** — 몸에도 금이 가 있게. 마을 종은 이미 깨져 있다. **울리는 종은 보여 주지 않는다** — 종은 깨지고 일부가 떨어져 나간 것. 종소리의 근원과 잇지 않는다.
```
Attached: our player character (a raccoon cub), a "root husk" enemy and the first boss, the "sleepless warden".
Do NOT draw any of them. Draw a DIFFERENT boss in the same cute style and color rules, with the same cute-but-heavy
boss proportions as the warden. Use the husk and the warden ONLY for the drained ash-grey color, the purple roots,
the amber eyes and the amber core.

[PURPOSE] One reference image for an image-to-3D tool. The 3D model will be shown small in a retro
low-resolution game (this boss is about 60 to 150 pixels tall on screen, rendered with pixel-art style toon
lighting and 1-pixel outlines added by the game). So design it to READ AT SMALL SIZE.
Render ONE character only, full body, centered, straight front view, standing, legs slightly apart. The left arm
hangs down and slightly out (A-pose). The right arm hangs down holding the bell described below.
Style: (same as the common enemy PURPOSE block: low-poly, hand-painted, 2-3 flat tones, flat shadowless light,
NOT pixel art, NOT voxel, NOT Minecraft-like, no floor, no text, pure white background, fills about 80%.)

[WORLD RULES] ... (paste the common enemy WORLD RULES block)

[CHARACTER] The "broken bellkeeper": a tall, round, hunched old bell-ringer creature, about three times the raccoon
cub's height. Cute boss proportions: a big round head on a wide, bell-shaped body that flares out at the bottom like
a heavy long robe or a bell's skirt, short stubby feet peeking out, short thick arms with big round mittens.
On its head, a small round hood that droops forward a little. Face: two big round glowing amber eyes set LOW and
close together, heavy sleepy eyelids, no mouth.
Its own body is cracked: one long painted crack runs from its shoulder down its front, and through that crack a warm
amber glow shows from inside (its core) — the brightest thing on it after the eyes. Thick angular dark-purple roots
grow out of the crack and over one shoulder (the only sharp shapes).
In its right mitten it holds, by the top loop, a big old bronze bell almost as tall as the raccoon cub, hanging down
so that its rim rests on the ground beside its right foot, as if it has been dragging it. The bell is BROKEN: a big
crack runs up its side and one chunk of its rim is missing. The bell is dull, darkened bronze (2 flat tones), with no
clapper visible and no decorations. The creature itself is drained ash-lavender grey (2 flat tones).
Lonely and heavy, like it has been dragging this bell for a very long time. Cute enough to feel sorry for.
```
> Meshy에서 종이 손과 붙어 한 덩어리로 나오면 Claude가 부위 자르기에서 종을 오른손 뼈에 붙인 따로 부위로 나눈다(무기 부위와 같은 꼴). 종이 너무 작게 나오면 "make the bell bigger"로 다시.

## 공통 — 주민(산 것) [PURPOSE] + [WORLD RULES]
> 0115·0116: 주민 = **여러 작은 동물**(화면 글 "주민"), 평범하고, 방랑자를 **무심하게 경계**한다, 해설자·현자가 아니다. 남아 있는 까닭은 말하지 않는다 → 뿌리·핵·종 표식을 몸에 두지 않는다. nail 이웃 종의 후손일 수 있으나 말하지 않는다 → 이름 있는 이웃(포포·수수·보리·모모·도토·콩이·토끼풀)의 생김새를 따르지 않는다.
> 종(種)은 **Claude의 제안**(바꾸셔도 된다): 불 지키는 주민 = 늙은 오소리 · 물 모으는 주민 = 작은 두더지 · 꿰매는 주민 = 어린 양. 게임 안 몸빛(`residents.rs`)과 맞췄다 — 따뜻한 갈색 · 회청색 · 베이지.
```
[PURPOSE] One reference image for an image-to-3D tool. The 3D model will be shown very small in a retro
low-resolution game (this character is only 20 to 40 pixels tall on screen, rendered with pixel-art style toon
lighting and 1-pixel outlines added by the game). So design it to READ AT TINY SIZE.
Render ONE character only, full body, centered, straight front view, neutral A-pose: standing straight, legs
slightly apart, arms down and out about 30 degrees so they don't touch the body. Nothing in the hands.
Style: a stylized low-poly 3D game model with a hand-painted texture. Few big simple forms, chunky and
exaggerated proportions, a strong clear silhouette. Details are PAINTED as large flat color areas, not sculpted.
Each material uses only 2 or 3 flat tones. No fur strands, no texture noise, no small patterns, no tiny accessories.
Lighting: flat, even and shadowless, like an unlit model; at most a very faint ambient occlusion. No highlights
except painted eye highlights. No outlines. NOT pixel art, NOT voxel, NOT Minecraft-like. No floor, no pedestal,
no text, no logo. Pure plain white background. Square image, the character fills about 80% of the height.

[WORLD RULES] "Color is life." This character is ALIVE: soft, warm, slightly faded pastel colors (it lives in a
quiet, lonely, half-empty underground village at dusk), smooth rounded forms, no roots, no cracks, no glowing parts.
Cute proportions: about 2 heads tall, big head, big eyes set LOW on the face and close together, NO mouth,
short chubby arms and legs, round mitten paws. It is ordinary and a little wary: calm, half-lidded eyes, not smiling,
not waving, simply minding its own work.
```

## 4. 불 지키는 주민(V01, 늙은 오소리) — 새 대화
첨부: `01_wanderer_front.png`
```
Attached: our player character, a raccoon cub. Do NOT draw the raccoon. Draw a DIFFERENT character in the same cute
style and proportion grammar.

[PURPOSE] ... (paste the common resident PURPOSE block)
[WORLD RULES] ... (paste the common resident WORLD RULES block)

[CHARACTER] The "hearth keeper": an old badger, a little shorter than the raccoon cub. Round head with a soft
cream face stripe between two warm brown cheek patches, small round ears, big calm dark eyes set low with heavy
sleepy lids, no mouth. Slightly hunched old shoulders. Warm brown-orange fur in 2 flat tones. A long soot-grey
apron with one big patch, and a short brown cloth tied around its head like a kerchief. Big round mitten paws
darkened with a little soot. Quiet, tired, still warm.
```

## 5. 물 모으는 주민(V02, 작은 두더지) — 새 대화
첨부: `01_wanderer_front.png`
```
Attached: our player character, a raccoon cub. Do NOT draw the raccoon. Draw a DIFFERENT character in the same cute
style and proportion grammar.

[PURPOSE] ... (paste the common resident PURPOSE block)
[WORLD RULES] ... (paste the common resident WORLD RULES block)

[CHARACTER] The "water collector": a small mole, about as tall as the raccoon cub. Round body, soft blue-grey fur
in 2 flat tones, a small pink nose, tiny round eyes set low (larger than a real mole's so they read at small size),
no mouth. BIG round pink-palmed digging mitten paws. A simple short sky-blue tunic with a rolled-up hem and one
wide belt. Practical and careful, a little guarded.
```

## 6. 꿰매는 주민(V03, 어린 양) — 새 대화
첨부: `01_wanderer_front.png`
```
Attached: our player character, a raccoon cub. Do NOT draw the raccoon. Draw a DIFFERENT character in the same cute
style and proportion grammar.

[PURPOSE] ... (paste the common resident PURPOSE block)
[WORLD RULES] ... (paste the common resident WORLD RULES block)

[CHARACTER] The "cloth mender": a young lamb, about as tall as the raccoon cub. A big round fluffy head of cream
wool shaped as 3 or 4 big soft lumps (NOT many small curls), a beige face with two small droopy ears, big calm eyes
set low, no mouth. A long beige patchwork shawl over its shoulders made of 3 big patches in faded colors
(dusty rose, sage, sand), with a few big painted cross-stitches along the seams. Small round hooves for feet,
round mitten hands. Patient and quiet.
```

## 뒤·옆(유료 Multi-view일 때만 — 같은 대화에서 정면을 받은 뒤)
```
Now render exactly the same character as a separate image, same scale, same flat lighting, same white background,
same pose, seen from directly behind (back view). Then another separate image seen from its left side
(side view, facing left). Do not change any colors, proportions or details.
```

## 받은 뒤
1. `tools/art/sources/ai/knellroot/<이름>_v1_front.png`로 저장(✦ 로고 지우기).
2. `python3 tools/art/tiny_preview.py <그림>` — 20·50px(보스는 60·150px)에서 눈·실루엣 특징(등딱지의 차양, 꼬투리의 긴 뒷발과 등 꼬투리, 종지기의 깨진 종)이 남는지.
3. Meshy Image to 3D: Texture 켬(유료) / 무료 6 Lite, Symmetry 켬(종지기는 끔 — 종이 한쪽), Triangle 3~5k, GLB.
4. GLB를 `tools/art/sources/meshy/<이름>_v1.glb`에 두고 Claude에게 — 줄이기·색 굽기·부위 자르기(등딱지·꼬투리·종은 따로 부위)·뼈·동작(0073 준비 동작 표, 0128 지연의 더 젖힘).
