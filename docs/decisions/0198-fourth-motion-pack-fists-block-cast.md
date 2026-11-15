# 0198 — 넷째 동작 묶음: 맨손(잽·어퍼컷·앞차기) · 패링 막기 · 마법 시전 · 싸움 자세 뒷걸음

- 날짜: 2026-11-15
- 상태: 채택(개발판, 판정 전)
- 사장: (Meshy `…_All_animation_3.glb`, 동작 19) "모션들 더 긁어왔어"

## 묶음에 든 것과 쓴 곳
같은 뼈대(역 바인드 행렬 md5가 앞 묶음들과 같다). 손 속도·발 속도·엉덩이를 프레임마다 재고 칸 그림으로 봤다.

| 원본 | 무엇 | 쓴 곳 |
|---|---|---|
| Left_Jab_from_Guard | 왼손이 9프레임에 닿는 잽(원본은 옆으로 선 권투 자세 — 엉덩이 −80°) | `fist1` 맨손 첫 타(판정 때 왼손이 정면 — `HIT_FACE` "lhand") |
| Right_Jab_from_Guard | 오른손 12프레임(손 속도 4.9) | `fist2` 둘째 타 |
| Right_Uppercut_from_Guard | 오른손을 내렸다(11) 올려침(13~16) | `fist3` 셋째 타 · 맨손 대시 공격 |
| Spartan_Kick | 무릎을 들고(0~9) 오른발을 앞으로 내지름(12~17, 발 속도 3.4) | `fist_charge`(무릎 든 채 모으기) · `fist_heavy`(모아 차기, 몸 정면 — "hips") |
| Block10 | 두 손을 앞으로 모으며 무릎을 굽혀 버팀(0.5초) | `parry` — 0193·0196에서 빌리던 M15를 바꿈 |
| mage_soell_cast_8 | 두 손을 가슴 앞에 모았다가(0~7) 앞으로 밀어냄(7~25) | `cast_charge`·`cast` — M06(한 손만 까딱)을 바꿈 |
| Walk_Fight_Back | 싸움 자세로 물러나는 걸음(제자리, 40프레임 한 주기) | `walk_back` — 보폭이 작아 걸음 시계 2.8배(`WALK_BACK_RATE`, 시험이 발 빠르기를 잰다) |
| Charged_Axe_Chop | 웅크려 모았다 일어나 내려침 — 내려치는 손 속도 2.0으로 약하다 | 안 씀(망치 강타 후보에서 뺌) |
| Charged_Slash · Charged_Upward_Slash · Left_Slash | 모아 베기·올려 베기·왼쪽 베기 | 아직 안 씀 — 검 강타·지팡이 강타 대안 후보 |
| Punch_Combo_1 · Boxing_Practice | 짧은 잽 셋(0.3초 안)·권투 연습 6.8초 | 안 씀(잽 셋을 따로 쓰는 쪽이 타가 또렷하다) |
| Shield_Push_Left | 방패로 밀기 | 안 씀(방패 없음) |
| Running·Walking·Charged_Ground_Slam·Weapon_Combo·Sword_Parry_Backward_2 | 앞 묶음과 같은 이름 | 앞 묶음 것을 그대로 |

## 바꾼 코드
- 맨손 표(`weapons::FISTS_COMBO`)의 동작 이름을 `fist1`~`fist3`, 강타 `fist_charge`·`fist_heavy`로. 동작은 맨손 판정 구간으로 구워
  `clip_rate` = 1(대시 공격만 셋째 타를 빌려 1.14배), `SWEEPS`·Blender `HIT_WINDOWS`에 넷(모두 가운데 — 앞으로 내지른다).
- `wanderer_skin.py`: `HIT_FACE`(판정 순간 정면에 둘 것 — 칼끝·왼손·몸), 차기도 `CHAIN`(모은 쪽에서 시작).
- 시험: 맨손의 왼 잽은 왼손, 앞차기는 오른발로 앞을 지나는지 잰다. 모아 차기는 모으기 끝의 쪽에서 시작.
