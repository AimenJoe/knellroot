# Meshy Text to Motion 프롬프트 — 방랑자 공격·기술 동작

사장 2026-11-14: "각 모션들을 실제랑 비슷하게 최대한 현실감 있게 — 지금은 너무 인형 같아." Meshy Pro로 리깅(Mixamo 템플릿)한 방랑자 v2에
Text to Motion으로 붙인다. 이름 = `tools/art/blender/wanderer.py`의 동작 이름(게임이 이 이름으로 찾는다).

## 쓰는 법
- 한 프롬프트 = 한 동작(Meshy·외부 안내 모두 "단일·단순 동작"이 잘 나온다고 한다). 콤보는 타마다 따로.
- 길이: **2초**(최소값 — 2~10초, 0.5초 단위). 게임의 평타는 0.3~0.5초라 들일 때 휘두름 구간만 잘라 게임의 시동·판정·후딜 구간에 맞춘다
  (스킬은 표의 `Clip`이 이미 구간 길이를 들고 있다). 더 긴 동작(몰아치기·버팀)만 3초.
- 모드: 품질 쪽(prime). 프롬프트는 400자 이하(아래 모두 그 안).
- 모든 프롬프트는 **같은 리깅 결과**에 붙이고, 내려받을 때 GLB · Mixamo 템플릿 · All Added.
- 앞으로 나아가는 이동이 있어도 된다 — 들일 때 몸통의 수평 이동은 빼고(게임이 몸을 옮긴다) 높이만 남긴다.
- 회피: 지금 게임의 회피는 0.25초의 낮은 앞 돌진이다. 첫 줄이 그 꼴, 둘째 줄(구르기)은 회피 시간을 늘릴 때의 안 — 둘 다 받아 보고 고른다.
- **반복 막기**(사장 2026-11-14 "2타는 두 번 때리는데"): 길이가 최소 2초라 짧은 동작은 남은 시간을 한 번 더 휘둘러 채운다. "performs exactly one …"로 시작하고 끝에 "holds completely still until the end. No second swing."을 붙인다. 두 번 휘두른 클립도 첫 휘두르기만 잘라 쓰면 되므로 버리지 않아도 된다.
- 3타는 게임에서 **내디디며 찌르기**(`weapons.rs` `SWORD_COMBO` — 좁고 길게)다. 처음 적은 내려치기 프롬프트는 틀렸다.
- 끝 자세는 늘 "칼을 든 준비 자세"로 — 다음 타·서기와 이어 붙이기 쉽다.

## 연속 동작으로 만들고 자르기(사장 2026-11-14 — "프롬프트로는 두 번째 동작을 잘 만들 수 없었어. 연속 동작을 만들고 자르는 식이 더 좋을 듯")
콤보는 한 클립에 이어서 만들고, 들일 때 **오른손 속도의 봉우리**로 타를 나눈다(봉우리 앞의 느린 구간 = 시동, 봉우리 = 판정, 뒤 = 후딜).
타 사이에 "짧고 또렷한 멈춤"을 넣으면 자르는 자리가 분명해진다. 길이 4초(지팡이 4.5초).

| 이름(자를 것) | 프롬프트 |
|---|---|
| `sword_combo` → `attack`·`attack2`·`attack3` | A sword fighter holding a one-handed sword in the right hand performs a three-hit combo with a short clear pause between hits: first a horizontal slash right to left at chest height, then a backhand slash left to right at waist height with a half step forward, then a deep lunging thrust straight ahead. Ends holding a combat-ready stance. |
| `staff_combo` → `staff1`~`staff4` | A fighter holding a light wooden staff with both hands performs a fast four-hit combo with a brief pause between hits: a straight thrust, a horizontal swing left to right, a rising strike with the lower end, then one full spin finishing with a strong downward strike. Ends in a ready stance. |
| `hammer_combo` → `hammer1`·`hammer2` | A fighter holding a heavy two-handed war hammer performs a slow two-hit combo with a clear pause between hits: first a wide horizontal swing from the right with the weight pulling the body around, then a step forward and an overhead slam onto the ground. Ends lifting the hammer back to a ready stance. |

### 받은 첫 묶음(2026-11-14, `Meshy_AI_Hooded_Raccoon_Rogue_…_animated_mixamo_All_animation.glb`)
- 몸: 정점 96,786 · 뼈 58(Mixamo 이름 + 꼬리 `Bone_014~017`·망토 `Bone_028~035`·귀 `Bone_044~047`·`headfront`) · 30fps · 손에 칼 없음(칼은 게임의 부위를 `RightHand`에 붙인다).
- 동작 13: Walking(1.03s)·walking_2_inplace·Running(0.63s)·Combat_Stance·Attack·Weapon_Combo_2·Thrust_Slash·Sword_Judgment·Heavy_Hammer_Swing·Archery_Shot·Basic_Jump·Back_Jump·Jump_Run.
- 오른손 속도 봉우리(프레임, m/s): Weapon_Combo_2 = 26(4.1)·48(6.9)·72(5.6) — 세 타가 또렷하다(자를 자리 ≈ 12–36 · 36–58 · 58–84). Attack = 26(11.3, 크게 한 번 — 강타 후보).
  Thrust_Slash = 18(5.8, 찌르기)·33(4.0). Sword_Judgment = 24(10.0)·80(3.4). Heavy_Hammer_Swing = 37(4.4).
- 앞으로 가는 이동: Weapon_Combo_2·Sword_Judgment·Basic_Jump·Back_Jump는 엉덩이가 0.5~1.1m 움직인다 — 들일 때 수평 이동을 뺀다.

## 빠진 것(첫 묶음 13개 뒤, 2026-11-14) — 이어 붙인 한 클립에서 자른다
첫 묶음이 채우는 것: `idle`(Combat_Stance)·`walk`(walking_2_inplace)·달리기(Running)·`attack`·`attack2`(Weapon_Combo_2)·`attack3`(Thrust_Slash 찌르기)·`heavy`(Attack)·`hammer1`(Heavy_Hammer_Swing)·`jump`·`land`(Basic_Jump·Jump_Run). 나머지 39개 이름을 아래 열셋 클립 + 콤보 셋(지팡이·망치 위) + 기술·큰 기술 열하나(2·3순위)로 채운다.

(2026-11-14 사장 "Parry dodge 이상하게 나왔어": 성질이 다른 둘을 한 클립에 이으면 뒤섞인다 — 패링·회피는 따로. 패링은 "칼을 가로로 들어 내려치는 것을 막는다"처럼 꼴이 분명하게, 회피는 Text to Motion이 잘 아는 앞구르기로. 라이브러리(Fighting)에 Block·Roll·Dodge가 있으면 그쪽이 먼저.)

| 클립 → 자를 이름 | 길이 | 프롬프트 |
|---|---|---|
| react → `hurt`·`fall`·`wake` | 5초 | A sword fighter in a combat-ready stance is hit hard in the chest, staggers back two steps, then is hit again and collapses forward face down onto the ground, lies still for a moment, then slowly pushes up with the hands, gets to the knees and stands back up into a combat-ready stance. |
| `parry`(따로) | 2초 | A sword fighter holding a one-handed sword in the right hand stands in a guard stance, then quickly raises the sword horizontally in front of the face with the left hand bracing the blade to block a heavy overhead strike, the knees bend from the impact, then pushes the blade away and returns to the guard stance. Only one block. |
| `dodge`(따로) | 2초 | A sword fighter holding a one-handed sword in the right hand does one quick forward dive roll: crouches, rolls forward over one shoulder along the ground, and comes back up onto the feet into a crouched guard stance. Only one roll, then holds still. |
| charge_heavy → `charge`·`heavy` | 4초 | A sword fighter crouches low and draws a one-handed sword far back behind the right hip, holds the coiled tense pose for two seconds while trembling with gathered power, then releases a wide explosive sweeping slash across the front at waist height and rises into a combat-ready stance. |
| `walk_back` | 2초 | A sword fighter holding a one-handed sword in the right hand walks slowly backwards while still facing forward, guard raised, small careful steps, keeping the eyes on an enemy in front. In place, looping. |
| turn → `turn_l`·`turn_r` | 3초 | A fighter stomps the right foot down and twists the whole body sharply to the left as if turning the ground underfoot, pauses, then stomps again and twists sharply to the right, and returns to a combat-ready stance. |
| fidgets → `idle_look`·`idle_shake` | 4초 | A small hooded fighter standing relaxed looks slowly to the left, then to the right, returns to the front, pauses, then gives the whole body a quick shake like shaking off water and settles back into a relaxed standing pose. |
| `plunge` | 3초 | A sword fighter holding a one-handed sword jumps high, raises the sword over the head at the top of the jump, then drops straight down driving the sword point into the ground on landing, crouched from the impact, then pulls the sword out and stands in a combat-ready stance. |
| `drink` | 2초 | A fighter holding a sword in the right hand lifts a small lantern-shaped flask with the left hand, tilts the head back and drinks from it, lowers it with a relieved breath, and returns to a combat-ready stance. |
| cast → `cast_charge`·`cast` | 3초 | A fighter holding a sword in the right hand draws the open left hand back to the chest and gathers energy there for a moment, body tense, then thrusts the left palm forward sharply to fire a magic bolt with a slight recoil, and returns to a combat-ready stance. |
| `throw` | 2초 | A fighter holding a weapon in the right hand takes a small round bomb with the left hand, tosses it underhand onto the ground a few steps ahead, and hops back half a step into a ready stance. |
| staff → `staff_charge`·`staff_release` | 4초 | A fighter holding a light wooden staff with both hands crouches and holds it low across the body gathering strength for two seconds, then swings it in one full circle around the body at waist height, turning the whole body once, and returns to a ready stance. |
| hammer → `hammer_charge`·`hammer_slam` | 4초 | A fighter holding a heavy two-handed war hammer cocks it back over the right shoulder and holds the crouched tense pose for two seconds, then leaps up and slams the hammer down onto the ground in front with both hands, landing in a deep crouch, then rises to a ready stance. |
| crossbow → `xbow_charge`·`xbow_shoot` | 3초 | A fighter raises a crossbow to the right shoulder with both hands, aims steadily forward and holds still for a moment, then fires it, the shoulders jerk back from the recoil, and steadies the crossbow again in an aiming pose. |

## 1순위 — 검(기본 무기)
| 이름 | 프롬프트 |
|---|---|
| `attack` | A sword fighter holding a one-handed sword in the right hand does one quick horizontal slash from right to left at chest height. Short wind-up drawing the sword back past the right shoulder, fast snappy swing driven by the hips, brief follow-through, then settles back into a combat-ready stance. Feet stay planted. |
| `attack2` | A sword fighter holding a one-handed sword in the right hand performs exactly one backhand slash from left to right at waist height, with a half step forward as the torso uncoils. After this single strike, the fighter returns to a combat-ready stance and holds completely still until the end. No second swing. |
| `attack3` | A sword fighter holding a one-handed sword in the right hand performs exactly one strong lunging thrust: steps forward deeply and drives the sword point straight ahead at chest height with the arm fully extended, holds the extended lunge for a beat, then pulls back to a combat-ready stance and holds still until the end. No second strike. |
| `charge` | A sword fighter crouches low and draws a one-handed sword far back behind the right hip, gathering power, body coiled and tense, trembling slightly while holding the pose. Looping hold, no release. |
| `heavy` | A sword fighter crouched low with a one-handed sword drawn back behind the right hip releases a powerful wide sweeping slash: explosive hip turn, the blade sweeps a full half circle in front at waist height, body follows through low and wide, then rises back into a combat-ready stance. |
| `parry` | A sword fighter does a sharp defensive parry: snaps a one-handed sword up diagonally in front of the body to deflect an incoming blow, short recoil back from the impact, then quickly returns to a combat-ready stance. |
| `dodge` | A sword fighter holding a one-handed sword does a very quick low evasive dash forward: ducks the head and shoulders, lunges low and fast with the body leaning forward, then pops back up into a combat-ready stance. |
| `dodge`(구르기 안) | A sword fighter holding a one-handed sword does a quick forward dodge roll: ducks, tucks and rolls forward over the shoulder, and springs back up into a combat-ready stance. |

## 2순위 — 기술(스킬 여덟)
| 이름 | 프롬프트 |
|---|---|
| `sk_whirl` | A sword fighter holding a one-handed sword spins around twice in place with the sword arm extended, slashing all around the body in a full circle, then stops and settles into a combat-ready stance. |
| `sk_cut` | A sword fighter holding a one-handed sword dashes forward in a low lunge and cuts through with one long horizontal slash while passing, skids to a stop and turns back into a combat-ready stance. |
| `sk_gather` | A fighter holding a sword in the right hand reaches the open left hand forward, grips the air, and pulls back hard toward the body as if dragging something heavy closer, then returns to a combat-ready stance. |
| `sk_quake` | A sword fighter holding a one-handed sword jumps up, raises the sword overhead with both hands and smashes it down into the ground in front, landing in a deep crouch from the impact, then rises into a combat-ready stance. |
| `sk_brace` | A sword fighter plants the feet wide and crouches, holding a one-handed sword horizontally in front with both hands to brace against blows, holds firm under pressure, then bursts outward with a strong shove of the arms. |
| `sk_echo` | A sword fighter holds a one-handed sword upright in front of the face in a calm focused pose, breathes once, then flicks the blade outward sharply as if ringing it, and returns to a combat-ready stance. |
| `sk_throw` | A sword fighter throws a one-handed sword forward overhand with a strong spinning throw, follows through, waits with the right hand open, then catches the sword as it returns and settles into a combat-ready stance. |
| `sk_embers` | A fighter holding a sword in the right hand sweeps the open left hand in a wide arc around the body as if scattering sparks from the palm, turning the torso fully, then returns to a combat-ready stance. |

## 3순위 — 큰 기술(셋)
| 이름 | 프롬프트 |
|---|---|
| `ult_upend` | A fighter crouches, plants both hands on the ground, then heaves upward with the whole body, twisting and lifting as if flipping an enormous tray, ending standing tall with arms raised, then lowers into a combat-ready stance. |
| `ult_flurry` | A sword fighter holding a one-handed sword unleashes a rapid flurry of many fast slashes in front of the body, alternating forehand and backhand cuts with quick small steps, ending with one strong final slash and a short pause. (3초) |
| `ult_line` | A sword fighter lowers into a still iaido stance with a one-handed sword held low, pauses, then performs one lightning-fast long forward slash, ending frozen in an extended follow-through before slowly standing up. |

## 4순위 — 다른 무기
| 이름 | 프롬프트 |
|---|---|
| `hammer1` | A fighter swings a heavy two-handed war hammer in a slow wide horizontal swing from the right side, the weight pulling the body around, then recovers the hammer back to a ready stance. |
| `hammer2` | A fighter lifts a heavy two-handed war hammer high overhead and brings it straight down in a crushing slam onto the ground in front, knees bending from the weight, then lifts it back into a ready stance. |
| `hammer_charge` | A fighter holds a heavy two-handed war hammer cocked back over the right shoulder, crouched and gathering strength, body tense and trembling. Looping hold, no release. |
| `hammer_slam` | A fighter with a heavy two-handed war hammer leaps up and slams the hammer down onto the ground in front with both hands, landing in a deep crouch from the impact, then rises to a ready stance. |
| `staff1` | A fighter holding a light wooden staff with both hands does a fast straight thrust forward with the tip, then pulls back to a ready stance. |
| `staff2` | A fighter holding a light wooden staff with both hands swings it in a quick horizontal strike from left to right, then returns to a ready stance. |
| `staff3` | A fighter holding a light wooden staff with both hands does a quick rising upward strike with the lower end, then returns to a ready stance. |
| `staff4` | A fighter holding a light wooden staff with both hands spins it once and finishes with a strong downward strike in front, then returns to a ready stance. |
| `staff_charge` | A fighter holds a light wooden staff low across the body with both hands, crouched and gathering strength. Looping hold, no release. |
| `staff_release` | A fighter holding a light wooden staff swings it in a full circle around the body at waist height, turning the whole body once, then returns to a ready stance. |
| `cast` | A fighter holding a sword in the right hand thrusts the open left palm forward sharply to fire a magic bolt, slight recoil of the arm, then returns to a combat-ready stance. |
| `cast_charge` | A fighter holding a sword in the right hand draws the left hand back to the chest, gathering energy in the palm, body tense. Looping hold, no release. |
| `throw` | A fighter holding a weapon in the right hand tosses a small round bomb underhand with the left hand onto the ground a few steps ahead, then steps back into a ready stance. |
| `xbow_charge` | A fighter raises a crossbow to the right shoulder with both hands and aims steadily forward, holding still. Looping hold. |
| `xbow_shoot` | A fighter aiming a crossbow from the right shoulder fires it, the arms and shoulders jerk back from the recoil, then steady the crossbow again. |

## 그다음(공격 밖 — 몸의 반응)
`hurt`(맞고 휘청), `fall`(쓰러짐), `wake`(엎드렸다 일어남), `jump`·`land`·`plunge`(공중에서 내려찍기), `drink`(등잔 마시기)는 공격을 들인 뒤에.
