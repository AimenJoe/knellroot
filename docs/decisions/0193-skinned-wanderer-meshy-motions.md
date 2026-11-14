# 0193 — 방랑자 몸을 스킨으로, 동작을 Meshy 동작으로

- 날짜: 2026-11-14
- 상태: 채택(개발판, 판정 전)
- 사장: "각 모션들을 실제랑 비슷하게 최대한 현실감 있게 바꾸는 작업이야. 걷는 거라든지 공격 및 기술 모션들, 지금은 너무 인형 같아."
  → Meshy Pro로 리깅·동작 GLB 셋을 받음 → "자 쓸 수 있는 것 써."

## 왜
옛 방랑자(0037·0038·0189)는 부위마다 뼈 하나에 통째로 붙은 **단단한 조각**이었다 — 팔꿈치·무릎·허리가 휘지 않으니 아무리 키프레임을
다듬어도 인형처럼 보인다. 사람 같은 동작은 정점이 여러 뼈에 걸쳐 휘어야(스킨) 나온다. Meshy의 자동 리깅(Mixamo 뼈 이름)과
Text to Motion·동작 라이브러리가 그 동작을 준다.

## 바꾼 것
### 게임(스킨 몸)
- `model.rs`: glTF 스킨을 읽는다 — `Skin { joints, ibm }`, 메시의 `JOINTS_0`·`WEIGHTS_0`(정점마다 넷), `Model::skin_into(i, pose, pos, nrm)`
  = Σ 가중치 × (뼈 월드 × 역 바인드) × 정점(법선도). 스킨 메시는 제 마디의 행렬을 쓰지 않는다(glTF 규칙).
- `render/batch.rs`: 그리기·그림자·외곽선이 `parts()` 한 곳으로 메시를 받는다 — 스킨이면 휜 정점(모델 행렬 = 인스턴스), 아니면 예전대로.
  외곽선 부풀리기는 스킨이면 법선 방향으로. 크기 0으로 숨긴 부위는 여기서 거른다(스킨 몸은 제 마디 행렬을 안 써서 크기 0이 안 먹는다).
- 무기는 손뼈에 붙은 단단한 조각 그대로(든 것만 보이는 규칙 0051 그대로). 아이콘(`render/items.rs`)과 던진 무기(`blades`)는 손에 쥔 방향을
  되돌려(`Model::rest_rotation`) 옛 각도 표를 그대로 쓴다.
- 2차 동작(`secondary.rs`)은 뼈대 마디 이름 `root`(몸 전체 찌그러짐)만 남는다 — 꼬리·머리 늦게 따라오기는 단단한 조각 전용이라 빠진다
  (Meshy 동작이 꼬리·망토 뼈를 움직이지 않는다 — 판정 뒤에 볼 것).
- CPU 스킨: 몸 정점 2,963 × 패스 셋(색·그림자·외곽선). 헤드리스 쿼터뷰 cpu 11ms 남짓(소프트웨어 GL) — 기기에서 볼 것.

### 모델 파이프라인(두 단계)
1. `tools/art/blender/meshy_rig.py`: 사장의 Meshy GLB 셋(각 14~16 MB, 정점 96,786 · 뼈 58 · 동작 13 + 20 + 20) →
   `tools/art/sources/meshy/raccoon_v2_rig.glb`(6.3 MB) — 몸은 `meshy_convert.py`와 같은 길(합치기 → 줄이기 → 새 UV → 색 굽기, 가중치는 남는다),
   동작은 쓸 것 34개만(이름 없는 셋째 묶음은 만든 차례 `M01`~`M18` = `docs/art/meshy-motion-prompts.md`의 번호). 원본 셋은 레포에 넣지 않는다.
   세 GLB의 뼈 이름·역 바인드 행렬이 같음을 확인했다(md5) — 동작을 섞어 쓸 수 있다.
2. `tools/art/blender/wanderer_skin.py` → `assets/models/wanderer.glb`(1.52 MB, 옛 0.65 MB):
   - 키 1.4 m(옛 몸과 같게), 엉덩이를 원점 위로. 텍스처 256(옛 몸과 같게) + 무기 아틀라스를 한 장으로.
   - 무기 넷·등잔은 `wanderer.py`의 코드 꼴 그대로 옮긴다: 날 = 손뼈 z(엄지 쪽), 넓은 면 = 손뼈 x(손등) — 쥔 주먹의 마디 쪽으로 날이 선다.
     등잔은 왼손 밑 `lamp` 뼈(마실 때만 크기 1).
   - 동작 표 `CLIPS`: 게임 이름 = 원본 클립의 한 구간. **길이는 옛 동작 그대로**(게임 상태 시간·`Clip::at` 구간이 거기에 맞춰져 있다 —
     얼린 값은 건드리지 않는다), 구간 안의 "닿는 순간"(오른손 속도 봉우리)을 옛 동작의 그 순간에 맞춰 구간마다 늘이고 줄인다.
     서기·몸짓은 원본 길이(되풀이·몸짓 시계가 모델의 길이를 읽는다), 뒷걸음은 한 주기(1.2초).
   - 엉덩이의 수평 이동은 뺀다(게임이 몸을 옮긴다): 되풀이 = 평균, 한 번 = 처음 → 끝 직선을 빼서 흔들림만 남긴다.
   - 100fps로 읽고 키를 줄인다(RDP, 회전 0.003·위치 2 mm) — 가중치 없는 끝 뼈(손끝·발끝·머리 끝)는 채널을 두지 않는다.
   - 줄이며 접힌 면은 각지게(가져온 사용자 법선·각진 모서리 표시·자동 매끈을 먼저 지운다 — 안 그러면 정점이 갈려 5,000으로 는다).

### 동작 대응(요약 — 전체는 `wanderer_skin.py` `CLIPS`)
| 게임 | 원본 |
|---|---|
| `idle`·`walk`·`run`(새로) | Combat_Stance · Walking · Running |
| `attack`·`attack2`·`attack3` | Triple_Combo_Attack 봉우리 18·36·54 |
| `charge`·`heavy` | Attack(봉우리 26) |
| `parry` · `dodge` · `hurt`·`fall` | Sword_Parry_Backward_2 · Roll_Dodge_3 · Dead |
| `wake` · `walk_back` · `turn_l`·`turn_r` · `idle_look`·`idle_shake` · `drink` | M01 · M02 · M03 · M04 · M05 |
| `cast_charge`·`cast` · `throw` · 지팡이 넷 · 지팡이 모으기·떼기 | M06 · M07 · M08 · M09 |
| `hammer_charge` · `hammer1`·`hammer2` · `hammer_slam` | M10 · M11 · Charged_Ground_Slam |
| `xbow_charge`·`xbow_shoot` | M12 |
| `sk_gather`·`sk_brace`·`sk_echo`·`sk_throw`·`sk_embers` · `ult_upend` | M13~M17 · M18 |
| `sk_whirl` · `sk_cut` · `sk_quake`·`plunge` · `ult_line` · `ult_flurry` | Skill_03 · Thrust_Slash · Charged_Ground_Slam · Sword_Judgment · Weapon_Combo |
| `jump`·`land` | Basic_Jump(꼭대기 · 닿는 순간) |

빠진 것: `strafe_l`·`strafe_r`(락온 옆걸음 — 걷기로 대신), `idle_bell`(몸짓 표에서 건너뛴다).

## 아직 모르는 것(판정)
- 회피 0.25초에 1.6초짜리 구르기를 넣었다 — 6배 빠르다. 회피 시간은 얼린 값(0013)이라 동작 쪽을 잘라 맞췄다. 어색하면: 구르기 앞부분만 쓰거나,
  "짧은 옆 스텝" 동작을 새로 받는다.
- 쓰러짐(`Dead`)은 등으로 눕고 깨어남(M01)은 그 자세에서 일어난다 — 두 장면 사이는 어두워져서 이어질 필요는 없다.
- 공격 둘째 타(Triple_Combo 27~45)는 몸을 57°까지 숙인다 — 인형 같지는 않지만 과하면 다른 구간을 고른다.
- 파일이 0.87 MB 늘었다(동작 JSON이 절반) — 웹 첫 로딩.

## 라이선스
몸 메시는 v2 그대로(Meshy 무료 플랜, CC BY 4.0 — 0189). 리깅·동작은 사장이 Meshy Pro로 받았다(2026-11-14). Pro 결과물의 조건은
`assets/LICENSES.md`에 사장 확인으로 적는다(내가 약관을 직접 확인하지 못했다).
