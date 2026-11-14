# 나노바나나(Gemini) → Meshy 참조 그림 프롬프트 — 0185(셋째 지역: 이끼먹이·이끼 덮인 문지기)

> 사장 2026-11-14: "캐릭터에 대해서는 이미지 생성할 참조 이미지랑 프롬프트 주면 생성해서 줄게, 평가해 줘. 그런 후 Meshy로 모델 만들겠어."
> 흐름은 0132와 같다(`docs/art/gemini-prompts-0132.md`, KB `guides/ai-mesh-to-rigid-part-model.md`): **나노바나나 = 참조 그림 한 장** →
> Claude가 작은 크기로 평가(`tools/art/tiny_preview.py`) → **Meshy Image to 3D** → GLB를 Claude에게 → 줄이기·색 굽기·부위·뼈·동작.
> 받은 그림은 `tools/art/sources/ai/knellroot/<이름>_v1_front.png`(✦ 로고는 지운다). AI로 그린 것은 숨기지 않는다.

## 규칙(0132에서 그대로)
- **캐릭터마다 새 대화**. 매번 [첨부 설명] + [PURPOSE] + [WORLD RULES] + [CHARACTER]를 다 붙인다.
- **귀여움 문법이 먼저**(0036·0061): 큰 머리, 큰 눈 낮게, 입 없음, 둥근 실루엣. 각지고 뾰족한 것은 **뿌리만**(0037).
- **지금 게임의 코드 모델(상자 덩어리) 그림은 Gemini에 붙이지 않는다** — 문지기 v1이 그렇게 해서 "마인크래프트 골렘"이 됐다(0061).
  `06_scale_lineup.png`는 사람이 크기를 보는 용도만.
- 네발짐승은 **3/4 앞(약 45°) 네 발로 선 자세** 한 장(가시짐승 v2·등딱지짐승이 이 한 장으로 Meshy에서 잘 나왔다).
- 빛은 그림에 굽지 않는다: 고르고 평평한 빛, 넓은 평면색, 재질마다 2~3톤.

## 첨부할 참조 그림(01~03은 0132의 것을 그대로 — `docs/art/refs-0132/`, 04~06은 `docs/art/refs-0185/`)
| 파일 | 무엇 | 붙이는 곳 |
|---|---|---|
| `refs-0132/01_wanderer_front.png` | 방랑자(플레이어) — 귀여움 문법·비율·크기 기준 | **둘 다** |
| `refs-0132/02_husk_front.png` | 잔재 — 뿌리에 먹힌 것의 색(잿빛 라벤더 + 보라 뿌리 + 호박빛 눈) | 문지기 |
| `refs-0132/03_warden_v2_front.png` | 첫 문지기 — 보스의 귀여운 무게감, 몸의 호박빛 핵 | 문지기 |
| `refs-0185/04_resident_water_front.png` | 물 모으는 주민 — **산 것**의 파스텔·둥근 몸 | 이끼먹이 |
| `refs-0185/05_garden_mood.png` | 잔뿌리의 정원 게임 화면(물·이끼·달빛) | 사람 확인용(붙여도 되지만 배경은 그리지 말라고 했다) |
| `refs-0185/06_scale_lineup.png` | 게임 안 크기(방랑자 + 코드 모델 문지기 · 방랑자 + 이끼먹이) | 사람 확인용 — **붙이지 않는다** |

> 이끼먹이는 **산 것**이라 잔재·문지기 그림을 붙이지 않는다(잿빛·뿌리가 옮아온다). 방랑자 + 주민 그림만.

## 1. 이끼먹이(Mossgrazer, 키 0.55m — 방랑자의 1/3) — 새 대화
첨부: `01_wanderer_front.png` + `04_resident_water_front.png`
> 게임에서(0185): 공격하지 않는 **정상 피식자**. 정원 물가에서 이끼를 뜯고, 방랑자가 다가오면 깡충깡충 달아났다가 멀어지면 돌아온다.
> 락온되지 않고 잿빛 보정도 받지 않는다 — **화면에서 적과 한눈에 갈려야** 한다(파스텔·작고 둥근 몸·푸른 기운 없는 따뜻한 색).
> 종(種)은 **Claude의 제안**(바꾸셔도 된다): 작은 토끼·기니피그 사이의 둥근 초식동물 + **등에 자라는 민트 이끼 방석**(먹는 이끼와 같은 색 —
> "작은 정원을 업고 다닌다"). 뿌리·핵·금 같은 침식 표식은 몸에 두지 않는다.
```
Attached: our player character (a raccoon cub) and a villager (a small mole) from the same game.
Do NOT draw either of them. Draw a DIFFERENT creature in the same cute style, proportion grammar and color rules.

[PURPOSE] One reference image for an image-to-3D tool. The 3D model will be shown very small in a retro
low-resolution game (this creature is only 12 to 25 pixels tall on screen, rendered with pixel-art style toon
lighting and 1-pixel outlines added by the game). So design it to READ AT TINY SIZE.
Render ONE creature only, full body, centered. Nothing else in the image.
Pose: seen from a three-quarter front view (about 45 degrees), standing still on four short legs, legs slightly
apart and clearly separated, head a little lowered as if about to nibble.
Style: a stylized low-poly 3D game model with a hand-painted texture. Few big simple forms, chunky and
exaggerated proportions, a strong clear silhouette. Details are PAINTED as large flat color areas, not sculpted.
Each material uses only 2 or 3 flat tones. No fur strands, no texture noise, no small patterns, no tiny accessories.
Lighting: flat, even and shadowless, like an unlit model; at most a very faint ambient occlusion. No highlights
except painted eye highlights. No outlines. NOT pixel art, NOT voxel, NOT Minecraft-like, NOT made of boxes.
No floor, no pedestal, no text, no logo. Pure plain white background. Square image, the creature fills about 80%.

[WORLD RULES] "Color is life." This creature is ALIVE and harmless: soft, warm pastel colors, smooth rounded
forms, no roots, no cracks, no glowing parts, nothing sharp. Cute proportions: a very big round head merged into
a round little body, big dark eyes set LOW on the face and close together, NO mouth, tiny stubby legs.

[CHARACTER] The "mossgrazer": a tiny, timid, round plant-eater, about one third as tall as the raccoon cub —
somewhere between a baby rabbit and a guinea pig, but rounder, like a soft bun on four stubby feet.
Body and head in warm cream (2 flat tones: the belly and face slightly lighter). Two short rounded ears in soft
sage green. On its back grows ONE thick, soft cushion of mint-green moss, like a little garden it carries,
made of 3 big round lumps (NOT many small bits), with two or three tiny round sprouts on top.
A small round tail tuft of the same mint moss. Big gentle eyes, slightly worried, always ready to run away.
Peaceful and a little shy — it must look clearly friendly, never like a monster.
```

## 2. 이끼 덮인 문지기(The Moss-Clad Warden, 키 ≈ 2.3m, 폭 2.2m, 길이 2.6m) — 새 대화
첨부: `01_wanderer_front.png` + `02_husk_front.png` + `03_warden_v2_front.png`
> 게임에서(0185, 설계 PR 21 "문턱지기" — 내부 작업명은 화면에 쓰지 않는다): 네 발로 낮게 다가와 **앞을 쓸고**, 붙어 있으면 **머리를 껍질
> 속으로 묻고 웅크려 앞을 막는다**(앞 평타는 튕긴다), 떨어지면 **곧게 돌진**하고 늦게 돈다. 체력 반 아래에서 무릎 꿇었다 일어서는 2단계.
> 그래서 실루엣에서 읽혀야 할 것: ① 둥근 생명체의 **원래 실루엣**(둥근 몸·짧은 네 다리) ② 등·옆구리를 덮은 **굳은 껍질 판**(돌가루·
> 이끼가 얹힌 큰 덩어리 몇 장) ③ 머리 위로 내민 **껍질 챙**(웅크리면 얼굴을 덮는다) ④ 껍질 판 사이로 비치는 **호박빛 핵**.
> 설계: "뿌리와 한 덩어리가 된 촉수 괴물로 만들지 않는다". 과거(물길의 경계를 지키던 몸이었다는 가설)는 그림에서 말하지 않는다.
> 생김새의 바탕은 **Claude의 제안**(바꾸셔도 된다): 굴을 파는 큰 웜뱃 — 실제 웜뱃은 쫓기면 굴로 들어가 단단한 엉덩이(연골에 덮인 뼈판)로
> 입구를 막는다(교차 확인: 연구자 인터뷰·박물관 자료 — "머리를 부순다"는 이야기는 확인되지 않은 속설이라 쓰지 않는다). 우리 문지기는
> 그 "몸으로 틈을 막는" 습성을 앞쪽 껍질로 옮겼다. 대안: 천산갑(몸을 말아 비늘로 막는다) — 이 경우 [CHARACTER]의 첫 줄만 바꾸면 된다.
```
Attached: our player character (a raccoon cub), a "root husk" enemy and the first boss "warden" from the same game.
Do NOT draw any of them. Draw a DIFFERENT creature in the same cute style, proportion grammar and color rules:
use the husk and the warden ONLY for the drained ash-grey color, the dark purple roots, the amber ember eyes
and the glowing amber core, and the warden for the "cute but heavy" boss weight.

[PURPOSE] One reference image for an image-to-3D tool. The 3D model will be shown small in a retro
low-resolution game (this boss is 60 to 150 pixels tall on screen, rendered with pixel-art style toon lighting
and 1-pixel outlines added by the game). So design it to READ AT SMALL SIZE.
Render ONE creature only, full body, centered. Nothing else in the image.
Pose: seen from a three-quarter front view (about 45 degrees), standing still on four short thick legs, legs
slightly apart and clearly separated, body low and wide, head low and forward.
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

[CHARACTER] The "moss-clad warden": a huge, heavy, round burrowing creature — like a giant wombat, about one and a
half times as tall as the raccoon cub and three times as wide — that once squeezed through narrow water channels
and now has been eaten by the roots. Its ORIGINAL round body is still clearly visible: a big rounded bean-shaped
body on four short, thick, round legs, and a big round head held low in front.
Its back and both flanks are covered by 4 or 5 BROAD, HARDENED SHELL PLATES — thick, smooth, gently rounded slabs
like river stones grown together, slightly darker grey than the body, with pale stone-dust patches painted on them.
The FRONT plate juts forward over the head like a heavy brim or visor, so the face peeks out from underneath
(when it curls up, the brim covers the whole face).
Big flat patches of dark teal-green MOSS sit on top of the plates (3 or 4 big soft cushions, not small bits).
In the gap between the two biggest back plates, a glowing amber core shows through like a warm crack of light.
A few thin dark-purple ROOTLETS hang from the edges of the plates along its sides (the only angular shapes,
short and few — NOT tentacles, NOT wrapped around the body).
Under the brim: two LARGE bright amber ember eyes set low and close together, half-lidded, no mouth.
Body and head in faded ash-lavender grey (2 flat tones: the belly slightly lighter).
It looks heavy, stubborn and tired, like something that has been blocking a passage for a very long time —
cute and sad rather than scary.
```

## 뒤·옆(유료 Multi-view일 때만 — 같은 대화에서 정면을 받은 뒤)
```
Now render exactly the same creature as a separate image, same scale, same flat lighting, same white background,
same pose, seen from directly behind (back view). Then another separate image seen from its left side
(side view, facing left). Do not change any colors, proportions or details.
```

## 받은 뒤
1. 그림을 Claude에게 — `tools/art/sources/ai/knellroot/mossgrazer_v1_front.png`·`mossward_v1_front.png`로 저장(✦ 로고 지우기)하고
   `tiny_preview.py`로 실제 크기에서 평가한다: 이끼먹이(12·25px) — 둥근 몸·이끼 방석·낮은 큰 눈이 남나, 적과 갈리나 /
   문지기(60·150px) — 껍질 챙·등 껍질 판·핵의 빛·짧은 네 다리가 남나, 촉수·골렘으로 읽히지 않나.
2. 좋으면 Meshy Image to 3D: Symmetry 켬, Triangle 3~5k(이끼먹이) · 5~8k(문지기), GLB.
3. GLB를 `tools/art/sources/meshy/<이름>_v1.glb`에 두고 Claude에게 — 줄이기·색 굽기·부위 자르기(문지기: 머리·몸·껍질·핵·다리 넷 /
   이끼먹이: 머리·몸·이끼·귀·다리)·뼈를 지금 코드 모델의 뼈 이름 그대로 붙여 동작(쓸기·껍질·돌진·깨어남 / 뜯기·깡충)을 옮긴다.
