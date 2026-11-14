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
- 끝 자세는 늘 "칼을 든 준비 자세"로 — 다음 타·서기와 이어 붙이기 쉽다.

## 1순위 — 검(기본 무기)
| 이름 | 프롬프트 |
|---|---|
| `attack` | A sword fighter holding a one-handed sword in the right hand does one quick horizontal slash from right to left at chest height. Short wind-up drawing the sword back past the right shoulder, fast snappy swing driven by the hips, brief follow-through, then settles back into a combat-ready stance. Feet stay planted. |
| `attack2` | A sword fighter holding a one-handed sword in the right hand does one fast backhand slash from left to right at waist height, stepping half a step forward with the left foot as the torso uncoils, short follow-through, then returns to a combat-ready stance. |
| `attack3` | A sword fighter holding a one-handed sword in the right hand does a heavy finishing overhead chop: raises the sword high above the head, steps forward and cuts straight down with full body weight, leans into the strike, holds the low follow-through for a beat, then recovers to a combat-ready stance. |
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
