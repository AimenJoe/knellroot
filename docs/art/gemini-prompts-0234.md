# 나노바나나(Gemini) → Meshy 참조 그림 프롬프트 — 0234(넷째 지역 멈추지 않는 갱도: 되풀이꾼·등짐꾼·사슬에 묶인 권양지기)

> 사장 2026-11-18: "새 캐릭터들 제대로 모델링 되어있니?" → 아니다(셋째·넷째 지역의 다섯이 코드 모델·빌린 모델) → "1, 2 진행해줘".
> 흐름은 0132·0185와 같다(`docs/art/gemini-prompts-0185.md`, KB `guides/ai-mesh-to-rigid-part-model.md`): **나노바나나 = 참조 그림 한 장** →
> 작은 크기로 평가(`tools/art/tiny_preview.py`) → **Meshy Image to 3D** → 줄이기·색 굽기·부위·뼈·동작.
> 이번부터는 Claude가 그림을 직접 받는다 — `tools/art/nanobanana.py`(기본 = Meshy Image to Image API의 나노바나나 — 세션의 Gemini 키는 무료 등급이라 이미지 한도 0). 사장이 웹에서 같은 프롬프트로 받아도 된다.
> 받은 그림은 `tools/art/sources/ai/knellroot/<이름>_v1_front.png`. AI로 그린 것은 숨기지 않는다.
> **보스(권양지기)의 생김새는 그림 → 사장 확인 → 메시**(0157). 잡몹 둘은 평가가 좋으면 바로 Meshy로(크레딧은 알린다).

## 규칙(0132·0185에서 그대로)
- **캐릭터마다 새 대화**. 매번 [첨부 설명] + [PURPOSE] + [WORLD RULES] + [CHARACTER]를 다 붙인다.
- **귀여움 문법이 먼저**(0036·0061): 큰 머리, 큰 눈 낮게, 입 없음, 둥근 실루엣. 각지고 뾰족한 것은 **뿌리만**(0037).
- **지금 게임의 코드 모델·빌린 모델 그림은 붙이지 않는다**(0061 — 문지기 v1이 "마인크래프트 골렘"이 된 까닭).
- **들고 쓰는 단단한 소품은 그리지 않는다**(0208 10절 — 곡괭이·사슬·갈고리는 코드 도형, 무기처럼 손뼈에 붙인다): 손은 비우고 쥘 수 있게 둥글게.
- 두 발로 서는 몸은 **정면 A-pose**(잔재·종지기), 네 발로 서는 몸은 **3/4 앞(약 45°)**(가시짐승·등딱지짐승).
- 빛은 그림에 굽지 않는다: 고르고 평평한 빛, 넓은 평면색, 재질마다 2~3톤(그늘이 텍스처에 구워지면 게임 빛이 한 번 더 얹힌다 — KB 가이드 함정).

## 종을 고른 까닭(Claude의 제안 — 바꾸셔도 된다)
| 게임 이름 | 몸의 바탕 | 까닭 |
|---|---|---|
| 되풀이꾼(곡괭이 재활동체, 키 1.6m) | **두더지 광부** — 두 발, 큰 삽 같은 앞발 | 갱도를 판 것은 땅을 파는 짐승. 마을의 물 모으는 주민도 두더지 — "같은 종이 다른 지역에서 다른 역할"(ECO-10). 생전의 일(파기)을 되풀이한다(ECO-05) |
| 등짐꾼(짐 진 재활동체, 키 1.5m·폭 넓음) | **카피바라 짐꾼** — 네 발, 등에 돌짐 | 지금 동작 뼈대가 네발(등딱지짐승의 것) — 짐 나르는 네발짐승. 둥글고 순한 몸이라 "짐에 눌린 슬픔"이 귀엽게 읽힌다. 짐은 따로 깨지는 부위(0216 W2) |
| 사슬에 묶인 권양지기(보스, 키 2.8m) | **늙은 오소리 십장** — 두 발, 굵은 팔 | 사슬을 휘두르고 갈고리를 던지려면 팔이 있어야 한다(지금은 네발 문지기를 빌려 씀). 얼굴의 흰 줄은 작은 화면에서도 읽힌다. 마을의 불 지키는 주민(늙은 오소리)과 같은 종 — 이야기에서 잇지는 않는다 |

## 첨부할 참조 그림(`docs/art/refs-0132/`)
| 파일 | 붙이는 곳 |
|---|---|
| `01_wanderer_front.png` 방랑자 — 귀여움 문법·비율·크기 기준 | **셋 다** |
| `02_husk_front.png` 잔재 — 뿌리에 먹힌 것의 색(잿빛 라벤더 + 보라 뿌리 + 호박빛 눈) | **셋 다** |
| `03_warden_v2_front.png` 첫 문지기 — 보스의 귀여운 무게감, 몸의 호박빛 핵 | 권양지기 |

## 공통 — [PURPOSE]의 둘째 줄부터 + [WORLD RULES] (0132 적 공통과 같다)
```
Style: a stylized low-poly 3D game model with a hand-painted texture. Few big simple forms, chunky and
exaggerated proportions, a strong clear silhouette. Details are PAINTED as large flat color areas, not sculpted.
Each material uses only 2 or 3 flat tones. No fur strands, no texture noise, no small patterns, no tiny accessories.
Lighting: flat, even and shadowless, like an unlit model; at most a very faint ambient occlusion. No highlights
except painted eye highlights. No outlines. NOT pixel art, NOT voxel, NOT Minecraft-like, NOT made of boxes.
No floor, no pedestal, no text, no logo. Pure plain white background. Square image, the creature fills about 80%.

[WORLD RULES] "Color is life." Living things are soft pastel and warm, with smooth rounded forms. Things eaten by
the purple roots have lost their color: faded ash-lavender grey, and the dark purple roots growing out of them are
the only sharp, angular, faceted shapes. Danger is the only red (do not use red).
Cute proportions: big head, big eyes set LOW on the face and close together, NO mouth, short chubby limbs.
```

## 1. 되풀이꾼(Repeater, 키 1.6m — 방랑자와 비슷) — 새 대화
첨부: `01_wanderer_front.png` + `02_husk_front.png`
> 게임에서(0208 5절): 벽을 같은 박자로 판다. 싸우면 **세 번 내려찍기를 같은 박자로** 되풀이하고, 패링하면 박자가 끊겨 휘청. 일부는 끝까지 일만 한다.
> 실루엣에서 읽혀야 할 것: ① 두 발로 선 둥근 몸 ② **삽 같은 큰 앞발**(곡괭이를 쥐는 손 — 곡괭이는 코드로 쥐어 준다) ③ 찌그러진 둥근 **안전모**(꺼진 등).
```
Attached: our player character (a raccoon cub) and a "root husk" enemy from the same game.
Do NOT draw either of them. Draw a DIFFERENT creature in the same cute style, proportion grammar and color rules:
use the husk ONLY for the drained ash-grey color, the dark purple roots and the amber ember eyes.

[PURPOSE] One reference image for an image-to-3D tool. The 3D model will be shown very small in a retro
low-resolution game (this creature is only 20 to 50 pixels tall on screen, rendered with pixel-art style toon
lighting and 1-pixel outlines added by the game). So design it to READ AT TINY SIZE.
Render ONE creature only, full body, centered, straight front view, standing upright on two short legs, legs
slightly apart and clearly separated. Both arms hang down and slightly out from the body (A-pose), hands EMPTY
and slightly open as if they could grip a tool handle. Do NOT draw any tool.
(paste the common Style / [WORLD RULES] block)

[CHARACTER] The "repeater": a mole miner, about as tall as the raccoon cub, that died down in the mine shafts and
now keeps doing its old work. Cute proportions: a big round head merged into a round pear-shaped body, short legs.
A short rounded snout, NO mouth. Two big round glowing amber ember eyes set LOW and close together, half-lidded
and empty, as if it is not really there. The arms are short but end in BIG, WIDE, ROUNDED digging paws like
shovels (mole paws) — the paws are the biggest thing after the head, each with three thick rounded claws.
On its head sits a round, dented old miner's helmet in dull dark bronze (2 flat tones) with a small DEAD headlamp
(a dark, unlit round lens) — the helmet sits a little crooked. A plain work apron of darker grey covers the belly.
Body and head in faded ash-lavender grey (2 flat tones, the belly slightly lighter).
Three or four short, angular dark-purple ROOTS grow out of its back and one shoulder (the only sharp shapes,
short and few — NOT wrapped around the body). A thin painted crack on the helmet.
Tired, empty and endlessly patient — cute and a little sad, never scary.
```

## 2. 등짐꾼(Packbearer, 키 1.5m·폭 1.3m — 방랑자보다 조금 낮고 훨씬 넓다) — 새 대화
첨부: `01_wanderer_front.png` + `02_husk_front.png`
> 게임에서(0208·0216 W2): 정해진 길로 짐을 나르고, 싸우면 **짐을 앞세워 곧게 밀고 온다**(띠 예고 — 레일 수레와 같은 꼴). 자세가 처음 깨지면
> **등의 짐이 깨져 떨어지고** 느려진다 → 짐은 몸과 갈리는 색·덩어리여야 한다(부위로 잘라 따로 숨긴다).
> 실루엣에서 읽혀야 할 것: ① 네 발로 선 낮고 넓은 둥근 몸 ② **등에 진 큰 돌짐**(몸만 한 덩어리 — 묶은 끈) ③ 앞으로 숙인 무거운 머리.
```
Attached: our player character (a raccoon cub) and a "root husk" enemy from the same game.
Do NOT draw either of them. Draw a DIFFERENT creature in the same cute style, proportion grammar and color rules:
use the husk ONLY for the drained ash-grey color, the dark purple roots and the amber ember eyes.

[PURPOSE] One reference image for an image-to-3D tool. The 3D model will be shown very small in a retro
low-resolution game (this creature is only 20 to 50 pixels tall on screen, rendered with pixel-art style toon
lighting and 1-pixel outlines added by the game). So design it to READ AT TINY SIZE.
Render ONE creature only, full body, centered. Nothing else in the image.
Pose: seen from a three-quarter front view (about 45 degrees), standing still on four short thick legs, legs
slightly apart and clearly separated, head low and forward as if pushing ahead.
(paste the common Style / [WORLD RULES] block)

[CHARACTER] The "packbearer": a stocky pack-carrying creature like a round chubby capybara, a little shorter
than the raccoon cub but twice as wide, that died hauling stone in the mine and still carries its load.
Cute proportions: a big round head with a blunt rounded snout and tiny round ears, NO mouth, two big round glowing
amber ember eyes set LOW and close together, heavy tired eyelids. A barrel-shaped round body on four short,
thick, round legs.
On its back it carries ONE HUGE LOAD: a big rounded heap of 3 or 4 large smooth river stones stacked together,
almost as big as its own body, in a warm sandy-brown stone color (2 flat tones) — clearly a different color and
shape from the body so the load reads as a separate object. Two thick dull-brown straps go around its belly to hold
the load (simple bands, no buckles).
Body and head in faded ash-lavender grey (2 flat tones, the belly slightly lighter).
Two or three short, angular dark-purple ROOTS grow up between the stones of the load and one from its shoulder
(the only sharp shapes, short and few).
Bent under the weight, slow and stubborn — cute and sad, never scary.
```

## 3. 사슬에 묶인 권양지기(The Chained Winchkeeper, 보스, 키 2.8m — 방랑자의 1.5배) — 새 대화
첨부: `01_wanderer_front.png` + `02_husk_front.png` + `03_warden_v2_front.png`
> 게임에서(0208 6절·0212): 권양기(사슬을 감는 큰 기계)에 사슬로 묶여 갱도 한가운데를 지킨다. 1단계 **사슬 휘두르기·갈고리 던져 끌기·추 떨어뜨리기**,
> 2단계 레일 수레를 부르고, **3단계 사슬이 끊겨** 처음으로 걸어 다니며 쫓는다(돌진·내려치기). "멈추지 못한 것이 멈춘 순간"이 마지막 싸움.
> 실루엣에서 읽혀야 할 것: ① 두 발로 선 크고 둥근 몸 ② **굵고 긴 두 팔**(사슬·갈고리를 쥐는 손 — 사슬·갈고리는 코드로) ③ 한 손목의 **쇠고랑**(사슬이
> 여기서 권양기로 이어진다 — 고랑은 몸의 일부로 그린다) ④ 오소리 얼굴의 **흰 줄**(작은 화면에서 얼굴이 읽히게) ⑤ 가슴 금 사이의 **호박빛 핵**.
> 과거(이 갱도의 십장이었다는 가설)는 그림에서 말하지 않는다 — 옷·도구로 직업을 설명하지 않는다.
```
Attached: our player character (a raccoon cub), a "root husk" enemy and the first boss "warden" from the same game.
Do NOT draw any of them. Draw a DIFFERENT boss in the same cute style and color rules, with the same cute-but-heavy
boss proportions as the warden. Use the husk and the warden ONLY for the drained ash-grey color, the dark purple
roots, the amber ember eyes and the glowing amber core.

[PURPOSE] One reference image for an image-to-3D tool. The 3D model will be shown small in a retro
low-resolution game (this boss is 60 to 150 pixels tall on screen, rendered with pixel-art style toon lighting
and 1-pixel outlines added by the game). So design it to READ AT SMALL SIZE.
Render ONE character only, full body, centered, straight front view, standing upright, legs slightly apart and
clearly separated. Both long arms hang down and slightly out from the body (A-pose), big hands EMPTY and loosely
open as if they could grip a heavy chain. Do NOT draw any chain, hook or tool.
(paste the common Style / [WORLD RULES] block)

[CHARACTER] The "chained winchkeeper": a huge, old, heavy badger, one and a half times as tall as the raccoon cub
and very broad, that has been chained to a great winch deep in the mine for a very long time. Cute boss proportions:
a big round head low between massive rounded shoulders, a wide round pear-shaped body, short thick legs, and LONG,
THICK, heavy arms that reach down to its knees, ending in big round hands with short blunt claws.
Face: the badger's broad white stripe runs from the nose up over the top of its head, between two darker grey side
bands — painted as big simple flat shapes. Two big round glowing amber ember eyes set LOW and close together in the
dark bands, heavy tired eyelids, NO mouth.
On its LEFT wrist: one thick, heavy round iron SHACKLE cuff in dark iron grey (2 flat tones) with a single big
round ring on its side where a chain would attach — the ring hangs empty.
Its chest is cracked: one big painted crack across the chest, and through it a warm glowing amber core shows from
inside (the brightest thing on it after the eyes). Four or five short, thick, angular dark-purple ROOTS grow out of
the crack and over both shoulders (the only sharp shapes — NOT tentacles, NOT wrapped around the body).
A plain wide leather belt in dull brown around its round belly — nothing else worn.
Body and head in faded ash-lavender grey with the darker grey badger bands (2 flat tones each).
It looks immensely tired and heavy, like something that could never stop working and never be allowed to rest —
cute and sad rather than scary.
```

## 실제로 받은 그림(2026-11-18) — 처음 장에서 어긋난 것을 프롬프트 끝에 덧붙여 다시
나노바나나 Pro(`tools/art/nanobanana.py --model nano-banana-pro`, Meshy Image to Image API로 — 한 장 9 크레딧). 이끼먹이·이끼 덮인 문지기는
`gemini-prompts-0185.md`의 블록 그대로 첫 장이 맞았다. 셋은 첫 장 뒤 아래 문단을 블록 끝에 붙였다(같은 대화에서 고쳐 달라는 것과 같다).

- 되풀이꾼(첫 장: 팔이 무릎까지 — 유인원·나무늘보 같음, 뿌리가 막대):
```
IMPORTANT FIXES: the head must be BIG — about as wide as the whole body, the head is the biggest part. The arms are SHORT and stubby: the paws hang only to the hips, NOT to the knees (not an ape, not a sloth). The paws are wide flat round shovels. The eyes sit LOW on the face, near the snout, big and round. Overall height-to-width is chubby like a toddler. ROOTS: every purple root is a single straight, faceted, pointed crystal-like shard (like the purple shards on the attached husk), NOT a branching twig, NOT antlers, never forked. Only three shards.
```
- 등짐꾼(첫 장: 하마 얼굴, 뿌리가 사슴뿔처럼 갈라짐):
```
IMPORTANT FIXES: the snout is short and rounded (cute capybara or guinea pig, NOT a hippo). ROOTS: every purple root is a single straight, faceted, pointed crystal-like shard (like the purple shards on the attached husk), NOT a branching twig, NOT antlers, never forked. Only three short shards in total, two between the stones and one on the shoulder.
```
- 권양지기(첫 장·둘째 장: 근육질 몸·작은 머리 — 귀여움 문법에서 벗어남, 셋째 장이 채택):
```
IMPORTANT FIXES: CHIBI boss proportions exactly like the attached warden: the round head is HUGE — as wide as the shoulders, about 40% of the total height. The body is a soft round sack with NO muscles, NO pecs, NO abs, NO shoulder bulges. The arms are thick soft rounded tubes, long enough to reach the knees. Short stumpy legs. Cute first, heavy second. ROOTS: every purple root is a single straight, faceted, pointed crystal-like shard (like the purple shards on the attached husk), NOT a branching twig, never forked.
```

## 뒤·옆(유료 Multi-view일 때만 — 같은 대화에서 정면을 받은 뒤)
```
Now render exactly the same creature as a separate image, same scale, same flat lighting, same white background,
same pose, seen from directly behind (back view). Then another separate image seen from its left side
(side view, facing left). Do not change any colors, proportions or details.
```

## 받은 뒤
1. 그림 → `tools/art/sources/ai/knellroot/<이름>_v1_front.png`(되풀이꾼 `repeater` · 등짐꾼 `packbearer` · 권양지기 `winchkeeper`) —
   `tiny_preview.py`로 실제 크기: 되풀이꾼(20·50px) 삽 앞발·안전모·낮은 눈 / 등짐꾼(20·50px) 몸과 갈리는 짐·네 발 /
   권양지기(60·150px) 흰 줄 얼굴·고랑·핵의 빛·긴 팔, 촉수·골렘으로 읽히지 않나.
2. Meshy Image to 3D: Symmetry 켬(권양지기는 고랑이 한쪽이라 끔), GLB.
3. 부위: 되풀이꾼 = 머리(+안전모)·몸·팔 둘(오른손에 곡괭이 뼈)·다리 둘 / 등짐꾼 = 머리·몸·**짐**(색으로 — 깨지면 숨긴다)·다리 넷 /
   권양지기 = 머리·몸·핵·팔 둘(왼손목 고랑에 사슬 뼈)·다리 둘. 동작은 지금 빌린 뼈대(잔재·등딱지짐승)의 이름을 그대로 이어받고, 권양지기는 새 뼈대.
