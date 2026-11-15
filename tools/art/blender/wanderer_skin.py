"""방랑자 v3 — 스킨 몸 + Meshy 동작(0193).

    blender -b -P tools/art/blender/wanderer_skin.py -- [모델 폴더] [미리보기 폴더]

- 몸·뼈·원본 동작: `tools/art/sources/meshy/raccoon_v2_rig.glb`(`meshy_rig.py`가 사장의 Meshy 리깅·동작 GLB 셋에서 만든 것).
  뼈는 Mixamo 이름(`mixamorig:*`) + 꼬리·망토·귀 뼈. 게임은 뼈 가중치로 CPU에서 몸을 휜다(`model::skin_into`).
- 무기 넷·등잔: `wanderer.py`의 코드 꼴 그대로(0157 — 생김새에 코드 꼴을 새로 붙이지 않는다; 든 것은 소품). 옛 손뼈의
  자리·방향에서 새 오른손뼈로 옮긴다: 날 = 손의 엄지 쪽(뼈의 z), 넓은 면 = 손등(뼈의 x) — 쥔 주먹의 마디 쪽으로 날이 선다.
  등잔은 왼손 밑 `lamp` 뼈(마실 때만 크기 1).
- 동작: 표 `CLIPS` — 게임 이름 = 원본 클립의 한 구간. 길이는 옛 동작 그대로(게임의 상태 시간·`Clip::at` 구간이 옛 동작에 맞춰져
  있다), 원본 구간 안의 "닿는 순간"(오른손 속도 봉우리)을 옛 동작의 그 순간에 맞춘다(구간마다 늘이고 줄이기).
  엉덩이의 수평 이동은 뺀다(게임이 몸을 옮긴다) — 되풀이 동작은 평균을, 한 번 하는 동작은 처음 → 끝 직선을 빼서 흔들림만 남긴다.
"""
import math
import os
import sys

import bpy
from mathutils import Matrix, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import wanderer as W  # noqa: E402

RIG = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "raccoon_v2_rig.glb")
NAME = "wanderer"
HEIGHT = 1.4  # 옛 방랑자와 같은 키(귀 끝까지)
OUT_FPS = 100  # 옛 동작처럼 — 길이(0.43·0.25 …)가 프레임에 딱 떨어진다. 키는 줄이므로 크기는 늘지 않는다
SRC_FPS = 24  # 원본 프레임 단위(`meshy_rig.py`가 24로 가져온다 — 아래 표의 수)
HAND_R, HAND_L = "mixamorig:RightHand", "mixamorig:LeftHand"
TEX = 256
GRIP_IN = 0.55  # 손목 → 가운데 손가락 뿌리의 이 몫이 주먹 가운데
# 쥔 방향(사장 2026-11-14 "무기가 손에 붙은 위치와 방향"): Meshy 동작의 손은 칼을 쥔 주먹이 아니라 펼친 손이고, 손의 돌림이
# 동작마다 제각각이다 — 원본 검 동작 열 곳의 닿는 순간을 손뼈 좌표로 재니 날 방향이 평균에서 22~72° 흩어졌다(손에 못 박은 한 방향은
# 어느 동작에선가 하늘·땅을 본다). 그래서 무기는 손뼈 밑 `weapon` 뼈에 달고 그 돌림을 프레임마다 몸에서 계산해 굽는다(`weapon_pass`):
# 날 = 어깨 → 주먹(위아래는 `BLADE_RISE`배 — 팔이 짧아 늘 아래로 처진다), 날이 앞서는 쪽 = 주먹이 가는 쪽(느리면 몸 앞).
BLADE_RISE = 0.5
EDGE_SPEED = 1.0  # m/s(뼈대 단위) — 이보다 빠르면 날이 손이 가는 쪽을 본다
EDGE_SMOOTH = 6  # 앞뒤 프레임(100fps) — 손 속도를 고르게
# 막기 꼴(사장 2026-11-15 "패링이 공격 1타라서 이상한데"): 원본 Sword_Parry_Backward_2는 칼을 아래로 늘어뜨린 채 거의 움직이지 않아
# 공격 준비처럼 보인다 — 막는 동작을 새로 받기 전까지 칼만 "몸 앞쪽에 세워 든" 막기 꼴로(날이 오는 쪽 = 앞).
# (이름, 들어가는 몫 — 동작 길이의 비): 그 몫 동안 규칙의 방향에서 막기 꼴로 옮겨 가고 그 뒤는 머문다.
GUARD = {}  # 0202: 패링은 방패 밀기(Shield_Push_Left — 왼팔로 민다)라 칼을 세우지 않는다. 옛 Block10 막기는 {"parry": 0.25}
GUARD_BLADE = Vector((0.25, -0.45, 0.85))  # 뼈대 공간: 왼쪽(+X) · 앞(−Y) · 위(+Z) — 앞쪽으로 세워 든 칼(가로로 들면 주먹이 허리께라 몸을 지난다)
# 숙임 바로잡기(앞, 옆 도 — 엉덩이 → 머리가 수직에서 기운 각): 클립 평균을 등뼈 셋에 나눠 돌려 그 각으로 세운다(`solve_upright`).
# 지금은 비어 있다 — 숙였던 Combat_Stance 대신 곧게 선 Idle_02를 쓴다(그때는 {"idle": (8.0, 5.0)}, 앞 23°·옆 18° → 8°·5°).
UPRIGHT = {}
SPINE = ("mixamorig:Spine", "mixamorig:Spine1", "mixamorig:Spine2")
# 판정 구간(초 — 시작, 길이): 게임의 `world::hit_times`·`heavy_times`(랩 평타·강타 시간 × 무기 표의 `time`)와 같은 값.
# 콤보·돌기 동작의 가운데를 잘라 와서 판정 순간에 몸이 옆·뒤를 보던 것(지팡이 둘째·셋째 타는 엉덩이가 뒤)을, 이 구간 동안 몸 전체를 돌려
# 주먹(= 칼)이 정면을 지나게 맞춘다(`face_hit` — 시작·끝은 원본 방향 그대로, 사이를 매끄럽게). 판정·초승달은 늘 정면 부채꼴이다.
# 값이 게임과 어긋나면 `weapons::tests`의 칼끝 시험이 잡는다.
# 기술·큰 기술(구간 = `skills.rs`의 `Clip::seg` — 시동 · 이어짐 · 후딜, 시동 끝 = 판정의 순간)의 몸 방향(`face_skill`):
# 시작·끝은 정면(누르는 순간·서기로 돌아갈 때 홱 돌지 않게), 판정의 순간은 "tip" = 칼끝이 정면 · "hips" = 몸이 정면 ·
# "hold" = 이어짐 내내 몸이 정면 · "spin" = 이어짐 구간에 몸이 정확히 한 바퀴(`SPIN`쪽 — 이 구간이 바퀴마다 되풀이된다).
SKILL_FACE = {
    "sk_whirl": ((0.15, 0.22, 0.30), "spin"), "sk_cut": ((0.12, 0.20, 0.25), "tip"),
    "sk_gather": ((0.20, 0.0, 0.30), "hips"), "sk_quake": ((0.30, 0.0, 0.40), "tip"),
    "sk_brace": ((0.10, 1.20, 0.35), "hold"), "sk_echo": ((0.25, 0.0, 0.20), "hips"),
    "sk_throw": ((0.18, 0.10, 0.25), "tip"), "sk_embers": ((0.25, 0.0, 0.30), "hips"),
    "ult_upend": ((0.30, 0.20, 0.50), "hips"), "ult_flurry": ((0.30, 0.0, 0.20), "hips"),
    "ult_line": ((0.30, 0.15, 0.45), "tip"),
    "throw": ((0.2, 0.0, 0.3), "hips"),  # 0204: 수류탄이 손을 떠나는 순간 몸이 정면(원본 M16은 몸 왼쪽으로 휘둘러 던진다)
    "parry": ((0.08, 0.22, 0.0), "hold"),  # 기술은 아니지만 같은 장치 — 막는 동안 몸이 정면(원본 M15는 들며 옆으로 비튼다)
}
# 몸이 스스로 돌지 않는 동작(0196): 게임이 몸의 쪽(`facing`)을 돌리는 동안의 발 디딤 — 원본은 몸이 150° 넘게 돌아 게임의 90°와
# 겹쳐 두 번 돌고, 끝에 서기로 홱 돌아왔다(돌아서기). 프레임마다 엉덩이를 정면으로.
FACE_STILL = {"turn_l", "turn_r"}
# 몸을 움직여 가는 동작(0197): 원본에서 엉덩이가 간 쪽을 몸의 앞("front")·뒤("back")에 맞춘다 — 게임이 그 쪽으로 몸을 옮긴다.
FACE_TRAVEL = {"dodge": "front", "dodge_back": "back"}
# 모아 친 강타는 모으기 동작이 끝난 쪽에서 시작한다(0196 — 망치 모으기는 몸을 −60° 감아 쥔다: 정면에서 시작하면 떼는 순간 홱 돈다).
CHAIN = {"heavy": "charge", "hammer_slam": "hammer_charge", "staff_release": "staff_charge", "fist_heavy": "fist_charge"}
# 떠 있는 동안 엉덩이를 올리지 않는 동작(0196): 게임이 몸을 `air`만큼 올린다 — 원본의 뛰어오름(엉덩이 +0.2)을 그대로 두면 두 번 뜬다.
# 엉덩이 높이를 서기의 높이 위로 올리지 않는다(웅크림은 남는다).
FLAT = {"jump", "land"}
# 겨누기(0196, 쇠뇌 — 원본은 활쏘기 Archery_Shot): 활을 든 왼팔이 몸 왼쪽을 가리킨다 → 몸을 돌려 오른손 → 왼손(겨누는 줄)이 정면을 보게,
# 쇠뇌는 그 줄을 따라 눕힌다(날 = 오른손 → 왼손, 위 = 하늘). (이름, 들어가는 몫): 그 몫 동안 서기 방향에서 겨누는 방향으로.
AIM = {}  # 0204: 쇠뇌는 활쏘기 대신 두 손을 앞으로 내민 시전 자세(`XBOW_FWD`) — 활쏘기는 몸이 옆으로 섰다(사장)
# 앞으로 겨누기(0204, 쇠뇌): 원본은 두 손을 어깨 높이로 앞에 내민 시전(mage_soell_cast_8) — 몸이 정면을 본다. 쇠뇌는 몸 앞(−Y)을
# 가리키고 날개는 가로(쉬는 자세의 쇠뇌 그대로 — 0196의 겨누기는 날개 축을 하늘로 세워 "쇠뇌를 세로로 든다").
XBOW_FWD = {"xbow_charge", "xbow_shoot"}
# 겨눈 자세에서 나갔다 돌아오는 동작(0205): [(초, 몫)] — 몫 1 = `XBOW_FWD`의 방향, 0 = 팔을 따라(어깨 → 주먹). 토스는 처음·끝이 겨눈 자세다.
XBOW_BLEND = {"throw": [(0.0, 1.0), (0.06, 0.0), (0.38, 0.0), (0.5, 1.0)]}
# 두 손 쥠(0202, 지팡이 — 원본 M08·M09는 두 손으로 쥔 지팡이 동작): 무기는 오른손 뼈에만 달려 있어 어깨 → 주먹 규칙으로 두면
# 왼손과 따로 놀며 엉뚱한 쪽을 가리켰다(사장 "지팡이는 다 이상하고"). 두 손이 가까이(`TWO_HAND_NEAR` 안) 있으면 지팡이를
# 오른손 → 왼손 줄로 눕히고, 한 손을 떼어 휘두르면(멀어지면 `TWO_HAND_FAR`까지) 옛 규칙으로 돌아간다.
# 0203: 평타 1~3타는 한 손으로 크게 휘두르는 Weapon_Combo라 옛 규칙(어깨 → 주먹), 넷째(Attack)는 두 손으로 들어 내려친다.
TWO_HAND = {"staff4", "staff_charge", "staff_release"}
TWO_HAND_NEAR, TWO_HAND_FAR = 0.30, 0.55
SPIN = -1.0  # 휘돌이가 도는 쪽(원본 Skill_03은 오른쪽으로 돈다 — 요가 줄어드는 쪽). 이펙트 `render::overlays::WHIRL_CLIP_TURN`과 같아야 한다

HIT_WINDOWS = {
    "attack": (0.08, 0.10), "attack2": (0.08, 0.10), "attack3": (0.10, 0.12), "heavy": (0.06, 0.12),
    "hammer1": (0.14, 0.17), "hammer2": (0.16, 0.20), "hammer_slam": (0.08, 0.16),
    "staff1": (0.06, 0.07), "staff2": (0.06, 0.07), "staff3": (0.06, 0.08), "staff4": (0.10, 0.13),
    "staff_release": (0.05, 0.11),
    "fist1": (0.048, 0.06), "fist2": (0.048, 0.06), "fist3": (0.064, 0.08), "fist_heavy": (0.048, 0.096),
    "fist_dash": (0.056, 0.07), "hammer_dash": (0.128, 0.16),
}
# 판정 순간에 무엇을 정면에 두나(0198): 기본은 칼끝(오른손), 왼 잽은 왼손(오른손은 막는 자리 — 원본은 옆으로 선 권투 자세라 엉덩이가
# −80°를 본다), 발차기는 몸(엉덩이).
HIT_FACE = {"fist1": "lhand", "fist_heavy": "hips", "fist_dash": "hips"}

# (게임 이름, 원본, 앵커[(게임 초, 원본 프레임)] 또는 ("loop", 시작, 끝, 길이) 또는 ("hit", 시작, 봉우리 찾기 (a, b), 끝, 길이, 게임의 순간)).
# 옛 동작 길이·순간은 옛 wanderer.glb에서 잰 값(오른손 속도 봉우리).
CLIPS = [
    # 서기·걷기·달리기 — 되풀이(길이: 서기·몸짓은 원본 그대로, 걷기는 옛 길이 — 걸음 시계가 옛 걷기 0.8초로 맞춰져 있다)
    ("idle", "Idle_02", ("loop", 0, 44.8, None)),  # 사장이 다시 준 대기(곧게 선 숨 고리, 1.87초) — 옛 Combat_Stance는 23° 숙였다
    ("walk", "Walking", ("loop", 0, 24.8, 0.8)),
    ("run", "Running", ("loop", 0, 15.2, None)),
    # 뒷걸음(0198): 싸움 자세로 물러나는 Walk_Fight_Back(제자리, 한 주기 40프레임) — 옛 M02는 보폭이 작았다.
    ("walk_back", "Walk_Fight_Back", ("cycle", 0, 40, None)),
    ("idle_look", "M04", ("span", 0, 44, None)),
    ("idle_shake", "M04", ("span", 44, 71, None)),
    # 검 세 타(Triple_Combo_Attack 봉우리 18·36·54)
    # 검 3연타(0201 — 사장 "우에서 좌가 1타인 모션으로"): Double_Combo_Attack — 오른쪽 위로 들어(0~8) 왼쪽 아래로 크게 내려 베고(봉우리 15),
    # 왼쪽 아래에서 오른쪽 위로 올려 베고(29), 오른쪽에서 내려친다(41). 한 원본이라 타 사이의 자세가 이어진다. 옛 Triple_Combo_Attack은 왼→오가 1타였다.
    ("attack", "Double_Combo_Attack", ("anchors", [(0.0, 4), (0.08, 13), (0.13, 15), (0.18, 17), (0.43, 23)])),
    ("attack2", "Double_Combo_Attack", ("hit", 23, (26, 31), 36, 0.43, 0.13)),
    ("attack3", "Thrust_Slash", ("anchors", [(0.0, 6), (0.10, 17), (0.16, 19), (0.22, 21), (0.52, 25)])),  # 0202: 내디디며 찌르기(처음 뜻)
    # 검 모아 베기(0199): Charged_Slash — 칼을 왼쪽 앞으로 감아 쥐고(13 → 22, 모으는 동안 버팀) 오른쪽으로 크게 벤다(27~29, 손 속도 5.0).
    # 옛 판은 Attack(내려치기)이었다.
    ("charge", "Charged_Slash", ("span", 13, 22, 0.3)),
    ("heavy", "Charged_Slash", ("anchors", [(0.0, 22), (0.06, 26), (0.12, 29), (0.58, 51)])),
    # 패링(2026-11-15): 라이브러리 Sword_Parry_Backward_2는 오른손이 몸 뒤(엉덩이보다 0.2~0.28 m 뒤)라 칼이 가려지고 공격 준비처럼 보였다 —
    # 되울림 원본(M15 "칼을 얼굴 앞에 세워 든다")에서 손이 몸 앞 가슴 높이로 올라오는 구간(12~21프레임)을 쓴다. 0.08초에 들고 버틴다.
    # 패링(0198): 막기 원본 Block10(두 손을 앞으로 모으며 무릎을 굽혀 버틴다, 0 → 5 → 12) — 임시로 빌리던 M15 대신.
    ("parry", "Shield_Push_Left", ("anchors", [(0.0, 16), (0.05, 24), (0.10, 28), (0.2, 31), (0.5, 45)])),  # 0202: 누르자마자 왼팔로 민다
    # 회피(0197): 구르기의 몸통 구간만(3 뜀 → 14 말림 → 26 웅크려 닿음 — 옛 2~30은 0.25초에 4.7배로 뭉갰다). 원본은 왼쪽 앞 37°로 굴러
    # 몸을 돌려 구르는 쪽 = 앞(`FACE_TRAVEL`). 뒤로 피할 때(입력 없이·락온 뒷걸음)는 Back_Jump(2 웅크림 → 4 뜀 → 14 닿음 → 17 받음).
    ("dodge", "Roll_Dodge_3", ("anchors", [(0.0, 3), (0.12, 14), (0.25, 26)])),
    ("dodge_back", "Back_Jump", ("anchors", [(0.0, 2), (0.04, 4), (0.19, 14), (0.25, 17)])),
    ("hurt", "Dead", ("hit", 0, (2, 6), 12, 0.35, 0.058)),
    ("fall", "Dead", ("span", 4, 71, 1.3)),
    ("wake", "M01", ("span", 0, 71, 4.0)),
    ("turn_l", "M03", ("hit", 0, (2, 10), 24, 0.25, 0.1)),
    ("turn_r", "M03", ("hit", 40, (44, 64), 68, 0.25, 0.075)),
    # 점프(0196): 원본 Jump_Over_Obstacle_2(24프레임 — 2에 발이 뜨고 8~9가 꼭대기, 14에 닿는다). 게임은 위 속도로 이 동작의 자리를
    # 고른다(`render::actors` — 오르는 동안 앞 반, 내려오는 동안 뒤 반, 오래 떨어지면 다리를 내린 끝). 옛 Basic_Jump는 꼭대기 한 장을 세웠다.
    ("jump", "Jump_Over_Obstacle_2", ("anchors", [(0.0, 2), (0.5, 14)])),
    ("land", "Jump_Over_Obstacle_2", ("anchors", [(0.0, 14), (0.15, 18)])),
    # 줍기·상자 열기(0196): 원본 Collect_Object(144프레임 — 11~29 웅크림, 59~80 오른손을 앞 아래로 뻗어 집음, 83~104 일어섬)를 0.9초로.
    # 장치 당기기(0197): 끌어모으기 원본 M13(7 손을 뻗음 → 19 잡음 → 43 몸으로 당김 → 59 놓음)을 0.65초로.
    # 맨손(0198): 검 동작을 빌리던 것을 주먹으로 — 왼 잽(9에 닿음) → 오른 잽(11~14) → 오른 어퍼컷(12~16), 모아 치기 = 앞차기
    # (Spartan_Kick: 0~9 무릎을 들어 버팀 = 모으기, 12~17 발을 내지름). 길이·판정 구간은 맨손 표(`weapons::FISTS_COMBO`)의 셈 그대로.
    ("fist1", "Left_Jab_from_Guard", ("anchors", [(0.0, 5), (0.048, 9), (0.108, 12), (0.258, 26)])),
    ("fist2", "Right_Jab_from_Guard", ("anchors", [(0.0, 7), (0.048, 11), (0.108, 14), (0.258, 26)])),
    ("fist3", "Right_Uppercut_from_Guard", ("anchors", [(0.0, 4), (0.064, 12), (0.144, 16), (0.344, 24)])),
    ("fist_charge", "Spartan_Kick", ("span", 0, 9, 0.3)),
    # 맨손 달리기 공격(0199, 사장 "달리기 공격이 스파르탄 킥이 더 어울릴 것"): 같은 앞차기를 대시 공격 구간에.
    ("fist_dash", "Spartan_Kick", ("anchors", [(0.0, 5), (0.056, 12), (0.126, 17), (0.301, 28)])),
    # 망치 달리기 공격(0200): 같은 앞차기를 망치의 대시 구간(무거워 1.6배 — 무릎을 들어 0.128초에 닿는다)에. 망치는 든 채.
    ("hammer_dash", "Charged_Ground_Slam", ("anchors", [(0.0, 4), (0.06, 11), (0.128, 37), (0.288, 42), (0.688, 58)])),  # 0203: 몸 전체로 내려치기(앞차기는 발이 작아 안 보였다)
    # 패링 성공(0199): 막은 자리에서 왼팔로 밀쳐낸다 — Shield_Push_Left(22 → 27 밀침 → 45). 렌더가 튕긴 뒤 서 있는 동안 튼다.
    ("parry_push", "Shield_Push_Left", ("anchors", [(0.0, 28), (0.1, 31), (0.4, 45)])),  # 0202: 튕긴 뒤 = 민 자리에서 거두기
    ("fist_heavy", "Spartan_Kick", ("anchors", [(0.0, 9), (0.048, 12), (0.144, 17), (0.464, 30)])),
    ("pull", "M13", ("anchors", [(0.0, 7), (0.15, 19), (0.45, 43), (0.65, 59)])),
    ("pickup", "Collect_Object", ("anchors", [(0.0, 11), (0.22, 29), (0.30, 59), (0.55, 80), (0.62, 83), (0.9, 104)])),
    ("plunge", "Charged_Ground_Slam", ("anchors", [(0.0, 4), (0.10, 11), (0.22, 34), (0.29, 39), (0.3, 40)])),  # 0202: 공중에서 든다
    ("plunge_land", "Charged_Ground_Slam", ("anchors", [(0.0, 40), (0.05, 42), (0.2, 52), (0.35, 62)])),  # 0202: 내려찍고 웅크렸다 일어남
    ("drink", "M05", ("span", 0, 60, 0.8)),
    # 마법(0198): 시전 원본 mage_soell_cast_8 — 두 손을 가슴 앞에 모으고(0 → 7, 모으기는 여기서 버틴다) 앞으로 밀어낸다(7 → 25).
    ("cast_charge", "mage_soell_cast_8", ("span", 0, 7, 0.3)),
    # 마법(0197): 왼손을 앞으로 내밀어(28 → 35) 내민 채 버티고(46) 거둔다(58) — 옛 판은 내미는 순간 바로 거둬 손만 까딱였다.
    ("cast", "mage_soell_cast_8", ("anchors", [(0.0, 7), (0.075, 14), (0.25, 25), (0.4, 31)])),
    ("throw", "Toss", ("anchors", [(0.0, 0), (0.5, 12)])),  # 0205: 지은 언더핸드 토스(`CRAFT`) — 0204의 M16은 머리 위로 휘둘러 던졌다
    # 지팡이(M08 봉우리 11·24·29·42 — 찌르기·가로 치기·올려 치기·돌아 내려치기)
    # 0203(사장 "지팡이는 아직 휘두르는 느낌이 없어"): M08은 손이 가장 빠를 때도 3.8 m/s — Weapon_Combo 앞쪽의 휘두르기 셋(4.7~5.3 m/s,
    # 올려 베기 → 내려 베기 → 머리 위에서 내려치기)과 Attack의 두 손 내려치기(9 m/s)로.
    ("staff1", "Weapon_Combo", ("anchors", [(0.0, 7), (0.06, 12), (0.13, 17), (0.3, 21)])),
    ("staff2", "Weapon_Combo", ("anchors", [(0.0, 24), (0.06, 28), (0.13, 32), (0.3, 35)])),
    ("staff3", "Weapon_Combo", ("anchors", [(0.0, 44), (0.06, 50), (0.14, 55), (0.32, 60)])),
    ("staff4", "Attack", ("anchors", [(0.0, 8), (0.10, 21), (0.23, 26), (0.56, 40)])),
    ("staff_charge", "M09", ("span", 0, 22, 0.3)),
    ("staff_release", "M09", ("hit", 22, (24, 34), 56, 0.52, 0.05)),
    # 망치(M10 모으기·M11 두 타)
    # 망치 모으기·강타(0199): Heavy_Hammer_Swing — 머리 위로 들어(7 → 27, 모으는 동안 버팀) 내려친다(33~37). 옛 강타는 Charged_Ground_Slam이라
    # 점프 내려찍기·땅울림과 같은 동작이었다. 모으기도 같은 원본이라야 떼는 순간 자세가 이어진다(옛 M10은 들지 않았다).
    ("hammer_charge", "Heavy_Hammer_Swing", ("span", 7, 27, 0.3)),
    ("hammer1", "M11", ("hit", 10, (18, 30), 36, 0.73, 0.225)),
    ("hammer2", "M11", ("hit", 36, (42, 54), 72, 0.86, 0.233)),
    ("hammer_slam", "Heavy_Hammer_Swing", ("anchors", [(0.0, 27), (0.08, 34), (0.24, 39), (0.75, 44)])),
    # 쇠뇌(0196): 원본 Archery_Shot(120프레임 — 45~57 두 팔을 들어 겨눔, 57~91 겨눈 채 버팀, 92 놓음, 95~ 내림). 옛 M12는 가슴 앞에
    # 쥔 채 거의 움직이지 않았다. 몸은 겨누는 줄이 정면을 보게(`AIM`).
    # 0204: 두 손을 앞으로 내밀어 겨눈 채(시전 20~31), 쏘면 반동으로 가슴께까지 당겼다(→ 8) 다시 내민다.
    ("xbow_charge", "mage_soell_cast_8", ("anchors", [(0.0, 20), (0.3, 28)])),
    ("xbow_shoot", "mage_soell_cast_8", ("anchors", [(0.0, 28), (0.058, 26), (0.13, 8), (0.24, 22)])),
    # 기술(구간 = `skills.rs`의 `Clip::seg` — 시동 끝 = 닿는 순간)
    ("sk_whirl", "Skill_03", ("anchors", [(0.0, 16), (0.15, 24), (0.37, 33), (0.67, 39)])),
    ("sk_cut", "Thrust_Slash", ("anchors", [(0.0, 8), (0.12, 18), (0.32, 33), (0.57, 50)])),
    ("sk_gather", "M13", ("hitseg", 24, (40, 56), 76, [0.20, 0.0, 0.30])),
    ("sk_quake", "Charged_Ground_Slam", ("hitseg", 8, (34, 44), 66, [0.30, 0.0, 0.40])),
    ("sk_brace", "M14", ("anchors", [(0.0, 0), (0.10, 6), (1.30, 58), (1.65, 80)])),
    ("sk_echo", "M15", ("hitseg", 30, (42, 52), 64, [0.25, 0.0, 0.20])),
    ("sk_throw", "M16", ("anchors", [(0.0, 14), (0.18, 26), (0.28, 32), (0.53, 50)])),
    ("sk_embers", "M17", ("hitseg", 14, (26, 36), 56, [0.25, 0.0, 0.30])),
    ("ult_upend", "M18", ("anchors", [(0.0, 34), (0.30, 57), (0.50, 64), (1.0, 82)])),
    ("ult_flurry", "Weapon_Combo", ("anchors", [(0.0, 40), (0.30, 54), (0.50, 70)])),
    ("ult_line", "Sword_Judgment", ("anchors", [(0.0, 10), (0.30, 24), (0.45, 32), (0.90, 62)])),
]
LOOPS = {"idle", "walk", "run", "walk_back"}


def world_pt(arm, bone, which="head"):
    b = arm.data.bones[bone]
    return arm.matrix_world @ (b.head_local if which == "head" else b.tail_local)


def basis(a, b):
    a = a.normalized()
    b = (b - a * a.dot(b)).normalized()
    return Matrix((a, b, a.cross(b))).transposed()  # 열 = a, b, a×b


def load_old():
    """옛 빌더에서 무기·등잔과 손 자리만(동작·텍스처 합치기는 건너뛴다)."""
    W.acts = lambda m: None
    merge = C.merge_textures
    C.merge_textures = lambda *a, **k: None
    m = W.make()
    C.merge_textures = merge
    rig = m.rig
    bpy.context.view_layer.update()
    old = {
        "hand_r": rig.matrix_world @ rig.data.bones["hand_r"].head_local,
        "hand_l": rig.matrix_world @ rig.data.bones["hand_l"].head_local,
        "atlas": m.mat.node_tree.nodes["Image Texture"].image,
        "weapons": [m.objects[n] for n in ("sword", "w_hammer", "w_staff", "w_crossbow")],
        "lamp": [m.objects[n] for n in ("lamp_geo", "glow_lamp")],
    }
    keep = set(old["weapons"] + old["lamp"])
    for o in list(bpy.data.objects):
        if o not in keep:
            if o.type == "MESH" and o.parent and o.parent.type == "ARMATURE" and o not in keep:
                pass
    for o in keep:
        mw = o.matrix_world.copy()
        o.parent = None
        o.matrix_world = mw
    for o in list(bpy.data.objects):
        if o not in keep:
            bpy.data.objects.remove(o)
    for a in list(bpy.data.actions):
        bpy.data.actions.remove(a)
    return old


def load_rig():
    sc = bpy.context.scene
    sc.render.fps = SRC_FPS
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=RIG)
    new = [o for o in bpy.data.objects if o not in before]
    arm = next(o for o in new if o.type == "ARMATURE")
    body = next(o for o in new if o.type == "MESH" and o.vertex_groups)
    for o in new:
        if o not in (arm, body):
            bpy.data.objects.remove(o)
    arm.animation_data.action = None
    for a in bpy.data.actions:  # 가져오면 "<이름>_<뼈대 이름>"
        if a.name.endswith("_" + arm.name):
            a.name = a.name[: -len(arm.name) - 1]
    for pb in arm.pose.bones:
        pb.rotation_mode = "QUATERNION"
    return arm, body


def place(arm, body):
    """키 1.4 m, 엉덩이(쉬는 자세)가 원점 위."""
    bpy.context.view_layer.update()
    zs = [(arm.matrix_world @ v.co).z for v in body.data.vertices]  # 몸은 뼈대의 자식 — 쉬는 자세의 정점
    k = HEIGHT / (max(zs) - min(zs))
    hip = arm.data.bones["mixamorig:Hips"].head_local
    arm.scale = (k, k, k)
    arm.location = (-hip.x * k, -hip.y * k, -min(zs) * k)
    bpy.context.view_layer.update()
    return k


def add_bone(arm, name, parent, at_world):
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.select_all(action="DESELECT")
    arm.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    eb = arm.data.edit_bones
    b = eb.new(name)
    p = arm.matrix_world.inverted() @ at_world
    b.head = p
    b.tail = p + Vector((0, 0, 0.1))
    b.parent = eb[parent]
    bpy.ops.object.mode_set(mode="OBJECT")
    arm.pose.bones[name].rotation_mode = "QUATERNION"


def attach(arm, old):
    """무기 넷은 옛 방향 그대로 주먹 자리의 `weapon` 뼈(오른손 자식)에 — 돌림은 `weapon_pass`가 동작마다 굽는다.
    등잔은 자리만 왼손 밑 `lamp` 뼈로."""
    # 쉬는 자세에서 붙인다 — 가져온 뼈대는 마지막 동작의 자세를 들고 있어서, 그대로 붙이면 그 자세 기준으로 어긋나 붙는다
    # (0193 첫 판의 "무기가 손에서 떨어진" 까닭).
    arm.animation_data.action = None
    for pb in arm.pose.bones:
        pb.matrix_basis.identity()
    bpy.context.view_layer.update()
    grip = world_pt(arm, HAND_R).lerp(world_pt(arm, "mixamorig:RightHandMiddle1"), GRIP_IN)
    add_bone(arm, "weapon", HAND_R, grip)
    xf = Matrix.Translation(grip - old["hand_r"])
    for o in old["weapons"]:
        mw = xf @ o.matrix_world
        o.parent = arm
        o.parent_type = "BONE"
        o.parent_bone = "weapon"
        o.matrix_world = mw
    grip_l = world_pt(arm, HAND_L).lerp(world_pt(arm, "mixamorig:LeftHandMiddle1"), GRIP_IN)
    add_bone(arm, "lamp", HAND_L, grip_l)
    lf = Matrix.Translation(grip_l - old["hand_l"])
    for o in old["lamp"]:
        mw = lf @ o.matrix_world
        o.parent = arm
        o.parent_type = "BONE"
        o.parent_bone = "lamp"
        o.matrix_world = mw


# ---------------------------------------------------------------- 동작
def hand_speed(arm, src, f0, f1):
    """원본 [f0, f1] 프레임마다 오른손(엉덩이 기준) 속도 — 봉우리 찾기."""
    sc = bpy.context.scene
    arm.animation_data.action = bpy.data.actions[src]
    out, prev = {}, None
    for f in range(int(math.floor(f0)), int(math.ceil(f1)) + 1):
        sc.frame_set(f)
        p = arm.pose.bones[HAND_R].head - arm.pose.bones["mixamorig:Hips"].head
        out[f] = 0.0 if prev is None else (p - prev).length
        prev = p
    return out


def peak(arm, src, a, b):
    sp = hand_speed(arm, src, a - 1, b)
    return max(range(int(a), int(b) + 1), key=lambda f: sp.get(f, 0.0))


def hips_series(arm, src, frames):
    sc = bpy.context.scene
    arm.animation_data.action = bpy.data.actions[src]
    out = []
    for f in frames:
        sc.frame_set(int(f))
        out.append(arm.pose.bones["mixamorig:Hips"].head.z)
    return out


def anchors_of(arm, name, src, spec):
    kind = spec[0]
    act = bpy.data.actions[src]
    end = act.frame_range[1]
    if kind in ("loop", "span", "cycle"):
        _, s0, s1, dur = spec
        dur = dur if dur else (s1 - s0) / SRC_FPS
        return [(0.0, s0), (dur, s1)]
    if kind == "hit":
        _, s0, (a, b), s1, dur, tp = spec
        sp = peak(arm, src, a, b)
        print(f"[skin] {name}: {src} {s0}-{sp}-{s1} -> {dur}s (hit {tp})")
        return [(0.0, s0), (tp, sp), (dur, s1)]
    if kind == "hitseg":
        _, s0, (a, b), s1, seg = spec
        sp = peak(arm, src, a, b)
        print(f"[skin] {name}: {src} {s0}-{sp}-{s1} seg {seg}")
        return [(0.0, s0), (seg[0], sp), (sum(seg), s1)]
    if kind == "anchors":
        return spec[1]
    if kind in ("apex", "touch"):
        frames = list(range(0, int(end) + 1))
        zs = hips_series(arm, src, frames)
        top = max(frames, key=lambda f: zs[f])
        if kind == "apex":
            return [(0.0, top - 3), (spec[1], top + 3)]
        low = min(range(top, len(frames)), key=lambda f: zs[f])
        print(f"[skin] {name}: apex {top} touch {low}")
        return [(0.0, max(top, low - 2)), (spec[1], low + 4)]
    raise ValueError(kind)


def src_at(anchors, t):
    for (t0, f0), (t1, f1) in zip(anchors, anchors[1:]):
        if t <= t1 or (t1, f1) == anchors[-1]:
            u = 0.0 if t1 <= t0 else min(max((t - t0) / (t1 - t0), 0.0), 1.0)
            return f0 + (f1 - f0) * u
    return anchors[-1][1]


def lean_of(v):
    """엉덩이 → 머리 벡터의 (앞, 옆) 기울기(라디안, 뼈대 공간 — 앞 = −Y, 옆 = +X)."""
    return math.atan2(-v.y, v.z), math.atan2(v.x, v.z)


def spine_rot(fwd, side):
    """등뼈 하나가 맡는 몫(셋으로 나눔): 옆축(X)으로 `fwd`, 앞축(Y)으로 `side`."""
    k = len(SPINE)
    return Matrix.Rotation(side / k, 3, "Y") @ Matrix.Rotation(fwd / k, 3, "X")


def lean_after(snap, fwd, side):
    """등뼈를 돌린 뒤 엉덩이 → 머리의 기울기: 머리 자리를 위 등뼈부터 제 머리를 축으로 차례로 돌린다(부모가 돌면 아래 축도 따라 돈다)."""
    hips, pivots, head = snap
    r = spine_rot(fwd, side)
    x = head
    for p in reversed(pivots):
        x = p + r @ (x - p)
    return lean_of(x - hips)


def solve_upright(snaps, want):
    """클립 평균 기울기가 `want`(앞, 옆 도)가 되는 등뼈 돌림(앞, 옆 라디안)."""
    wf, ws = (math.radians(x) for x in want)
    fwd = side = 0.0
    for _ in range(8):
        got = [lean_after(s, fwd, side) for s in snaps]
        mf = sum(g[0] for g in got) / len(got)
        ms = sum(g[1] for g in got) / len(got)
        fwd -= mf - wf
        side -= ms - ws
    return fwd, side, (math.degrees(mf), math.degrees(ms))


def uprighten(arm, fwd, side):
    """등뼈 셋을 돌린다 — 바뀌기 전 자세에서 읽은 틀로 바탕 회전을 고친다(자식은 따라온다)."""
    r = spine_rot(fwd, side)
    frames = {n: arm.pose.bones[n].matrix.to_3x3() @ arm.pose.bones[n].matrix_basis.to_3x3().inverted() for n in SPINE}
    for n in SPINE:
        pb = arm.pose.bones[n]
        loc, q, sc = pb.matrix_basis.decompose()
        a = frames[n]
        rot = a.inverted() @ r @ a @ q.to_matrix()
        pb.matrix_basis = Matrix.LocRotScale(loc, rot.to_quaternion(), sc)


def sample(arm, src, anchors, upright=None):
    """게임 프레임(OUT_FPS)마다 원본을 읽는다 → {뼈: [(loc, quat, scale)]}, 엉덩이 머리(뼈대 공간).
    `upright`(도)면 클립 평균 숙임을 그 각까지 등뼈로 세운다(한 클립에 한 각 — 숨쉬는 흔들림은 남는다)."""
    sc = bpy.context.scene
    arm.animation_data.action = bpy.data.actions[src]
    dur = anchors[-1][0]
    n = max(1, int(round(dur * OUT_FPS)))
    times = [src_at(anchors, k / OUT_FPS) for k in range(n + 1)]

    def go(f):
        for pb in arm.pose.bones:  # 채널이 없는 뼈는 쉬는 자세
            pb.location = (0, 0, 0)
            pb.rotation_quaternion = (1, 0, 0, 0)
            pb.scale = (1, 1, 1)
        sc.frame_set(int(math.floor(f)), subframe=f - math.floor(f))

    corr = None
    if upright is not None:
        snaps = []
        for f in times:
            go(f)
            pb = arm.pose.bones
            snaps.append((pb["mixamorig:Hips"].head.copy(), [pb[n].head.copy() for n in SPINE], pb["mixamorig:Head"].head.copy()))
        before = [lean_of(h - p) for p, _, h in snaps]
        fwd, side, got = solve_upright(snaps, upright)
        corr = (fwd, side)
        print(f"[skin] {src}: lean fwd/side {math.degrees(sum(b[0] for b in before) / len(before)):.0f}/"
              f"{math.degrees(sum(b[1] for b in before) / len(before)):.0f} -> {got[0]:.0f}/{got[1]:.0f}")
    poses, hips, hands = [], [], []
    for f in times:
        go(f)
        if corr is not None:
            uprighten(arm, *corr)
        poses.append({pb.name: pb.matrix_basis.decompose() for pb in arm.pose.bones if pb.name not in ("lamp", "weapon")})
        hips.append(arm.pose.bones["mixamorig:Hips"].head.copy())
        hands.append((arm.pose.bones[HAND_R].head.copy(), arm.pose.bones["mixamorig:RightArm"].head.copy(),
                      arm.pose.bones[HAND_L].head.copy()))
    return n, poses, hips, hands


def strip_travel(arm, poses, hips, loop):
    """엉덩이의 수평 이동을 뺀다(뼈대 공간 x·y). 되풀이 = 평균, 한 번 = 처음 → 끝 직선. 쉬는 자세의 엉덩이 자리가 기준."""
    b = arm.data.bones["mixamorig:Hips"]
    rest = b.head_local
    r_inv = b.matrix_local.to_3x3().inverted()
    n = len(hips)
    if loop:
        mx = sum(h.x for h in hips) / n
        my = sum(h.y for h in hips) / n
        line = [(mx, my)] * n
    else:
        a, z = hips[0], hips[-1]
        line = [(a.x + (z.x - a.x) * k / max(n - 1, 1), a.y + (z.y - a.y) * k / max(n - 1, 1)) for k in range(n)]
    for k in range(n):
        h = hips[k]
        want = Vector((rest.x + h.x - line[k][0], rest.y + h.y - line[k][1], h.z))
        loc, q, s = poses[k]["mixamorig:Hips"]
        poses[k]["mixamorig:Hips"] = (r_inv @ (want - rest), q, s)


ROT_TOL = 0.003  # 쿼터니언 성분 오차(≈ 0.35°) — 이 안이면 키를 줄인다
LOC_TOL = 0.002  # m(뼈대 공간, 줄이기 전 단위)


def reduce(rows, tol):
    """키 줄이기(RDP): rows[k] = 값들. 처음·끝 + 직선 보간 오차가 tol을 넘는 자리만 남긴 프레임 번호."""
    n = len(rows)
    keep = {0, n - 1}
    stack = [(0, n - 1)]
    while stack:
        a, b = stack.pop()
        if b - a < 2:
            continue
        worst, at = 0.0, None
        for k in range(a + 1, b):
            u = (k - a) / (b - a)
            err = max(abs(rows[a][i] + (rows[b][i] - rows[a][i]) * u - rows[k][i]) for i in range(len(rows[k])))
            if err > worst:
                worst, at = err, k
        if worst > tol:
            keep.add(at)
            stack += [(a, at), (at, b)]
    return sorted(keep)


def put(act, bn, path, rows, frames):
    for i in range(len(rows[0])):
        fc = act.fcurves.new(f'pose.bones["{bn}"].{path}', index=i, action_group=bn)
        fc.keyframe_points.add(len(frames))
        fc.keyframe_points.foreach_set("co", [v for k in frames for v in (k, rows[k][i])])
        for kp in fc.keyframe_points:
            kp.interpolation = "LINEAR"
        fc.update()


def moving_bones(arm, body):
    """채널을 둘 뼈: 몸 정점을 움직이는 뼈와 그 조상 + 손(무기·등잔이 붙는다). 가중치 없는 끝 뼈(손끝·발끝·머리 끝)는 뺀다 —
    채널 하나마다 glTF JSON이 수백 바이트라 파일이 커진다."""
    weighted = set()
    for v in body.data.vertices:
        for g in v.groups:
            if g.weight > 1e-3:
                weighted.add(body.vertex_groups[g.group].name)
    keep = set()
    for b in arm.data.bones:
        if b.name in weighted or b.name in (HAND_R, HAND_L):
            p = b
            while p:
                keep.add(p.name)
                p = p.parent
    return keep


BLADE_REACH = 0.5  # 칼끝 어림: 주먹에서 `weapon_pass`의 날 방향으로 이만큼(뼈대 단위)


def two_hand_mix(fist, left):
    """두 손 쥠(`TWO_HAND`)의 몫: 두 손이 가까우면 1(무기 = 오른손 → 왼손 줄), 멀면 0(어깨 → 주먹)."""
    span = (left - fist).length
    return smooth01((TWO_HAND_FAR - span) / (TWO_HAND_FAR - TWO_HAND_NEAR)) if span > 1e-3 else 0.0


def hand_yaw(hip, hand, two=False):
    """칼끝이 몸 둘레 어느 쪽인가(라디안, 정면 0 · 왼쪽 +, 뼈대 공간 — 앞 = −Y, 왼쪽 = +X).
    `hand` = (주먹, 어깨, 왼손) — 칼끝 = 주먹 + 날 방향(`weapon_pass`와 같은 셈) × `BLADE_REACH`. `two` = 두 손 쥠(0202)."""
    fist, shoulder = hand[0], hand[1]
    d = fist - shoulder
    d.z *= BLADE_RISE
    d.normalize()
    if two:
        g = two_hand_mix(fist, hand[2])
        d = (d * (1 - g) + (hand[2] - fist).normalized() * g).normalized()
    v = fist + d * BLADE_REACH - hip
    return math.atan2(v.x, -v.y)


def smooth01(x):
    x = min(max(x, 0.0), 1.0)
    return x * x * (3 - 2 * x)


def own_yaw(arm, pose):
    """한 프레임의 엉덩이가 보는 쪽(라디안, 정면 0 · 왼쪽 +) — 엉덩이 뼈의 z(쉬는 자세의 앞)."""
    rr = arm.data.bones["mixamorig:Hips"].matrix_local.to_3x3()
    f = rr @ pose["mixamorig:Hips"][1].to_matrix() @ Vector((0, 0, 1))
    return math.atan2(f.x, -f.y)


def near(a, ref):
    """`a`를 `ref`에 가장 가까운 같은 각으로(±2π)."""
    return a + 2 * math.pi * round((ref - a) / (2 * math.pi))


def turn_body(arm, poses, turns):
    """프레임마다 몸 전체를 세로축으로 `turns[k]`(라디안)만큼 돌린다(엉덩이 머리를 축으로)."""
    rr = arm.data.bones["mixamorig:Hips"].matrix_local.to_3x3()
    for k, c in enumerate(turns):
        loc, q, sc = poses[k]["mixamorig:Hips"]
        r = Matrix.Rotation(c, 3, "Z")
        poses[k]["mixamorig:Hips"] = (loc, (rr.inverted() @ r @ rr @ q.to_matrix()).to_quaternion(), sc)


def keyed_turns(n, keys, wrap=True):
    """(초, 돌림) 열쇠 사이를 매끄럽게 이은 프레임별 돌림(`wrap`이면 돌림은 앞 열쇠에 가까운 각으로 풀어 잇는다 — 아니면 준 그대로)."""
    ks = []
    for t, c in sorted(keys):
        ks.append((t, near(c, ks[-1][1]) if ks and wrap else c))
    out = []
    for k in range(n + 1):
        t = k / OUT_FPS
        if t <= ks[0][0]:
            out.append(ks[0][1])
            continue
        for (t0, c0), (t1, c1) in zip(ks, ks[1:]):
            if t <= t1:
                out.append(c0 + (c1 - c0) * smooth01((t - t0) / max(t1 - t0, 1e-4)))
                break
        else:
            out.append(ks[-1][1])
    return out


def at_frame(n, t):
    return min(n, max(0, round(t * OUT_FPS)))


def face_hit(arm, name, poses, hips, hands, window, start=0.0):
    """판정 구간 동안 몸을 세로축으로 돌려 구간 가운데에 칼끝이 정면에 오게(`HIT_WINDOWS`). 시작·끝은 몸이 정면(0196 — 원본 콤보는
    타마다 몸이 돌아 끝에 옆·뒤를 보았다: 검 둘째 타 +80°, 망치 첫 타 −150°, 지팡이 둘째·셋째 타 내내 뒤 — 콤보를 거기서 멈추면 서기로
    홱 돌았다. 다음 타도 정면에서 시작하니 이어 쳐도 같은 자리에서 잇는다)."""
    s0, a0 = window
    n = len(poses) - 1
    dur = n / OUT_FPS
    own = [own_yaw(arm, p) for p in poses]
    for k in range(1, n + 1):
        own[k] = near(own[k], own[k - 1])
    # 몸의 쪽(= 원본 쪽 + 돌림)이 열쇠 사이에서 가장 덜 돌게 각을 고른다(돌림만 풀어 이으면 원본이 도는 만큼 한 바퀴를 더 돌았다 —
    # 검 셋째 타·망치 두 타). 시작은 `start`(모으기 동작 끝의 쪽 — `CHAIN`, 없으면 정면).
    k0, k1 = at_frame(n, s0), at_frame(n, s0 + a0)
    km = at_frame(n, s0 + a0 / 2)
    mode = HIT_FACE.get(name)
    if mode == "hips":
        aim = own[km]
    elif mode == "lhand":
        v = hands[km][2] - hips[km]
        aim = math.atan2(v.x, -v.y)
    else:
        aim = hand_yaw(hips[km], hands[km], name in TWO_HAND)
    turn = near(-aim, start - own[k0])
    end = near(-own[n], own[k1] + turn - own[n])
    turns = keyed_turns(n, [(0.0, start - own[0]), (s0, turn), (s0 + a0, turn), (dur, end)], wrap=False)
    turn_body(arm, poses, turns)
    two = name in TWO_HAND
    a = hand_yaw(hips[at_frame(n, s0)], hands[at_frame(n, s0)], two) + turns[at_frame(n, s0)]
    z = hand_yaw(hips[at_frame(n, s0 + a0)], hands[at_frame(n, s0 + a0)], two) + turns[at_frame(n, s0 + a0)]
    span = math.degrees(math.atan2(math.sin(z - a), math.cos(z - a)))
    print(f"[skin] face {name}: turn {math.degrees(turn):+.0f}°, tip {math.degrees(a):+.0f}° -> {math.degrees(z):+.0f}° (sweep {span:+.0f}°)")


def face_skill(arm, name, poses, hips, hands, spec):
    """기술 동작의 몸 방향(`SKILL_FACE`): 시작·끝 정면 + 판정의 순간(또는 이어짐 내내) 정면, 휘돌이는 이어짐에 한 바퀴."""
    (s0, a0, r0), mode = spec
    n = len(poses) - 1
    end = n / OUT_FPS
    own = [own_yaw(arm, p) for p in poses]
    for k in range(1, n + 1):  # 풀어 잇기
        own[k] = near(own[k], own[k - 1])
    if mode == "spin":
        # 바라는 엉덩이 쪽: 시동 동안 정면, 이어짐 동안 정면에서 한 바퀴, 그 뒤 정면(= 한 바퀴 뒤). 돌림 = 바라는 쪽 − 원본 쪽.
        want = [0.0 if t < s0 else SPIN * 2 * math.pi * min((t - s0) / max(a0, 1e-4), 1.0)
                for t in (k / OUT_FPS for k in range(n + 1))]
        turns = [w - o for w, o in zip(want, own)]
    else:
        keys = [(0.0, -own[0]), (end, -own[n])]
        hit = at_frame(n, s0)
        if mode == "tip":
            keys.append((s0, -hand_yaw(hips[hit], hands[hit])))
        else:
            keys.append((s0, -own[hit]))
        if mode == "hold":
            keys.append((s0 + a0, -own[at_frame(n, s0 + a0)]))
        turns = keyed_turns(n, keys)
    turn_body(arm, poses, turns)
    print(f"[skin] face {name}: {mode} — start {math.degrees(own[0] + turns[0]):+.0f}°, "
          f"hit {math.degrees(own[at_frame(n, s0)] + turns[at_frame(n, s0)]):+.0f}°, end {math.degrees(own[n] + turns[n]):+.0f}° "
          f"(turned {math.degrees(min(turns)):+.0f}..{math.degrees(max(turns)):+.0f}°)")


def face_still(arm, name, poses):
    """`FACE_STILL`: 프레임마다 엉덩이를 정면으로."""
    turns = [-own_yaw(arm, p) for p in poses]
    turn_body(arm, poses, turns)
    print(f"[skin] face {name}: still ({math.degrees(min(turns)):+.0f}..{math.degrees(max(turns)):+.0f}°)")


def face_travel(arm, name, poses, hips, way):
    """`FACE_TRAVEL`: 처음 → 끝 엉덩이 이동이 몸의 앞(또는 뒤)이 되게 몸 전체를 한 각만큼 돌린다."""
    d = hips[-1] - hips[0]
    if way == "back":
        d = -d
    turn = -math.atan2(d.x, -d.y)
    turn_body(arm, poses, [turn] * len(poses))
    print(f"[skin] face {name}: travel {way} — turned {math.degrees(turn):+.0f}° ({d.length:.2f} m)")


def aim_yaw(hand):
    """겨누는 줄(오른손 → 왼손)의 쪽(라디안, 정면 0 · 왼쪽 +, 뼈대 공간)."""
    v = hand[2] - hand[0]
    return math.atan2(v.x, -v.y)


def face_aim(arm, name, poses, hands, ramp):
    """`AIM`: 겨누는 줄이 정면을 보게 몸을 돌린다 — `ramp` 몫 동안 서기(엉덩이 정면)에서 옮겨 간다."""
    n = len(poses) - 1
    end = n / OUT_FPS
    own0 = own_yaw(arm, poses[0])
    want = -aim_yaw(hands[n])
    keys = [(0.0, -own0 if ramp > 0 else want), (end * max(ramp, 1e-3), want), (end, want)]
    turns = keyed_turns(n, keys)
    turn_body(arm, poses, turns)
    print(f"[skin] face {name}: aim {math.degrees(want):+.0f}° (start {math.degrees(turns[0]):+.0f}°)")


def flatten(arm, poses, hips, top):
    """`FLAT`: 엉덩이를 `top`(뼈대 공간 z) 위로 올리지 않는다."""
    r_inv = arm.data.bones["mixamorig:Hips"].matrix_local.to_3x3().inverted()
    for k, h in enumerate(hips):
        if h.z > top:
            loc, q, sc = poses[k]["mixamorig:Hips"]
            poses[k]["mixamorig:Hips"] = (loc + r_inv @ Vector((0, 0, top - h.z)), q, sc)


def write_action(arm, name, n, poses, lamp, keep):
    """뼈마다 회전(엉덩이는 위치도) — 줄인 키, 직선 보간. 위치·크기가 쉬는 자세 그대로인 뼈는 채널을 두지 않는다."""
    act = bpy.data.actions.new(name)
    act.use_fake_user = True
    for bn in poses[0].keys():
        if bn not in keep:
            continue
        qs, prev = [], None
        for p in poses:
            q = p[bn][1].copy()
            if prev is not None and prev.dot(q) < 0:
                q.negate()
            qs.append(tuple(q))
            prev = q
        if any(max(abs(a - b) for a, b in zip(q, (1, 0, 0, 0))) > 1e-4 for q in qs):
            put(act, bn, "rotation_quaternion", qs, reduce(qs, ROT_TOL))
        if bn == "mixamorig:Hips":
            ls = [tuple(p[bn][0]) for p in poses]
            put(act, bn, "location", ls, reduce(ls, LOC_TOL))
    put(act, "lamp", "scale", [(lamp,) * 3, (lamp,) * 3], [0, 1] if n >= 1 else [0])
    act.fcurves.find('pose.bones["lamp"].scale', index=0).keyframe_points[-1].co.x = n
    for i in (1, 2):
        act.fcurves.find('pose.bones["lamp"].scale', index=i).keyframe_points[-1].co.x = n
    act.use_frame_range = True
    act.frame_start, act.frame_end = 0, n
    arm.animation_data.action = None
    tr = arm.animation_data.nla_tracks.new()
    tr.name = name
    tr.strips.new(name, 0, act)
    tr.mute = True
    return act


def weapon_pass(arm, act, loop):
    """굽힌 동작을 다시 돌며 `weapon` 뼈의 돌림을 프레임마다 정해 키로(머리 주석의 규칙)."""
    sc = bpy.context.scene
    ad = arm.animation_data
    ad.action = act
    n = int(act.frame_range[1])
    hand, sh, hips, lh = (arm.pose.bones[k] for k in (HAND_R, "mixamorig:RightArm", "mixamorig:Hips", HAND_L))
    hr = arm.data.bones[HAND_R].matrix_local
    br = arm.data.bones["weapon"].matrix_local
    rb = br.to_3x3().normalized()
    old_frame = basis(Vector((0, -1, 0)), Vector((1, 0, 0)))
    snaps = []
    for k in range(n + 1):
        sc.frame_set(k)
        q = (hand.matrix @ hr.inverted() @ br).to_3x3().normalized()
        snaps.append((hand.head.copy(), sh.head.copy(), hips.head.copy(), q, lh.head.copy()))
    rel = [h - p for h, _, p, _, _ in snaps]

    def at(i):
        return rel[i % (n + 1)] if loop else rel[min(max(i, 0), n)]

    qs, prev_e, prev_q = [], None, None
    for k, (h, s_, _, q, l_) in enumerate(snaps):
        d = h - s_
        d.z *= BLADE_RISE
        d.normalize()
        v = Vector()
        for j in range(-EDGE_SMOOTH, EDGE_SMOOTH + 1):
            v += (at(k + j + 1) - at(k + j - 1)) * (OUT_FPS / 2)
        v /= 2 * EDGE_SMOOTH + 1
        e0 = Vector((0, -1, 0)) - d * (-d.y)
        if e0.length < 1e-3:
            e0 = Vector((0, 0, 1)) - d * d.z
        e0.normalize()
        ev = v - d * v.dot(d)
        w = min(ev.length / EDGE_SPEED, 1.0)
        e = e0 * (1 - w) + (ev.normalized() if ev.length > 1e-6 else e0) * w
        if act.name in GUARD:
            g = smooth01(k / max(n * GUARD[act.name], 1))
            d = (d * (1 - g) + GUARD_BLADE.normalized() * g).normalized()
            e = e * (1 - g) + Vector((0, -1, 0)) * g
        if act.name in AIM:
            g = smooth01(k / max(n * AIM[act.name], 1)) if AIM[act.name] > 0 else 1.0
            d = (d * (1 - g) + (l_ - h).normalized() * g).normalized()
            e = e * (1 - g) + Vector((0, 0, 1)) * g
        if act.name in XBOW_FWD:
            d, e = Vector((0, -1, 0)), Vector((1, 0, 0))
        if act.name in XBOW_BLEND:
            g = keyed(XBOW_BLEND[act.name], k / OUT_FPS)
            d = (d * (1 - g) + Vector((0, -1, 0)) * g).normalized()
            e = e * (1 - g) + Vector((1, 0, 0)) * g
        if act.name in TWO_HAND:
            g = two_hand_mix(h, l_)
            if g > 0.0:
                d = (d * (1 - g) + (l_ - h).normalized() * g).normalized()
        e = (e - d * e.dot(d)).normalized()
        if prev_e is not None and e.dot(prev_e) < 0:  # 손이 되돌아도 날을 뒤집지 않는다(망치 머리가 홱 돌지 않게)
            e.negate()
        prev_e = e
        rot = (q.inverted() @ basis(d, e) @ old_frame.transposed() @ rb).to_quaternion()
        if prev_q is not None and prev_q.dot(rot) < 0:
            rot.negate()
        prev_q = rot
        qs.append(tuple(rot))
    put(act, "weapon", "rotation_quaternion", qs, reduce(qs, ROT_TOL))
    ad.action = None


# 지은 동작(0205): 원본에 없는 동작을 원본의 한 자세 위에 관절을 돌려 짓는다 — 뼈대 공간의 축(앞 −Y · 왼쪽 +X · 위 +Z)으로
# 뼈의 머리를 돌림 중심으로. 각(도)의 열쇠 [(초, 각)]는 사이를 매끄럽게(코사인) 잇는다. 원본 프레임 = 1/24초.
# 언더핸드 토스(사장 "쇠뇌 공격 모션에서 오른손으로 언더핸드로 툭 던지는 — 조준은 좀 흐트러지고, 오른발을 약간 내밀면서"):
# 쇠뇌 겨눈 자세(시전 24)에서 오른팔을 엉덩이 옆 뒤로 크게 뺐다가(0.1초) 앞 아래로 툭(0.2초 = `throw.startup`, 놓는 순간),
# 오른발은 반 걸음 나갔다 돌아오고 몸은 살짝 숙인다. 쇠뇌는 오른손에 든 채 팔을 따라 돈다(조준이 흐트러진다).
CRAFT = {
    "Toss": {
        "base": ("mage_soell_cast_8", 24),
        "len": 0.5,
        "turns": [  # (뼈, 축, [(초, 도)]) — 축 X: + = 앞이 아래로(팔·등뼈), 다리는 − = 앞으로 든다
            ("mixamorig:Spine1", "X", [(0.0, 0), (0.1, -4), (0.2, 9), (0.3, 7), (0.5, 0)]),
            ("mixamorig:RightArm", "X", [(0.0, 0), (0.1, 118), (0.2, 48), (0.27, 30), (0.5, 0)]),
            ("mixamorig:RightForeArm", "X", [(0.0, 0), (0.1, -10), (0.2, 12), (0.5, 0)]),
            ("mixamorig:LeftArm", "X", [(0.0, 0), (0.1, 18), (0.2, 30), (0.5, 0)]),
            ("mixamorig:RightUpLeg", "X", [(0.0, 0), (0.08, -14), (0.16, -18), (0.34, -18), (0.5, -4)]),
            ("mixamorig:RightLeg", "X", [(0.0, 0), (0.08, 28), (0.16, 12), (0.34, 12), (0.5, 2)]),
        ],
    },
}


def keyed(keys, t):
    for (t0, a0), (t1, a1) in zip(keys, keys[1:]):
        if t <= t1:
            u = 0.0 if t1 <= t0 else (t - t0) / (t1 - t0)
            return a0 + (a1 - a0) * (0.5 - 0.5 * math.cos(math.pi * min(max(u, 0.0), 1.0)))
    return keys[-1][1]


def craft_actions(arm):
    """`CRAFT`의 동작을 짓는다(원본 동작처럼 이름으로 `CLIPS`가 읽는다)."""
    sc = bpy.context.scene
    if arm.animation_data is None:
        arm.animation_data_create()
    for name, c in CRAFT.items():
        src, f = c["base"]
        arm.animation_data.action = bpy.data.actions[src]
        for pb in arm.pose.bones:
            pb.location, pb.rotation_quaternion, pb.scale = (0, 0, 0), (1, 0, 0, 0), (1, 1, 1)
        sc.frame_set(f)
        base = {pb.name: pb.matrix_basis.copy() for pb in arm.pose.bones}
        act = bpy.data.actions.new(name)
        arm.animation_data.action = act
        n = int(round(c["len"] * 24))
        for k in range(n + 1):
            t = k / 24
            for pb in arm.pose.bones:
                pb.matrix_basis = base[pb.name]
            bpy.context.view_layer.update()
            for bone, axis, keys in c["turns"]:
                pb = arm.pose.bones[bone]
                a = math.radians(keyed(keys, t))
                if abs(a) < 1e-6:
                    continue
                head = pb.head.copy()
                rot = Matrix.Translation(head) @ Matrix.Rotation(a, 4, axis) @ Matrix.Translation(-head)
                pb.matrix = rot @ pb.matrix
                bpy.context.view_layer.update()
            for pb in arm.pose.bones:
                for path in ("location", "rotation_quaternion", "scale"):
                    pb.keyframe_insert(path, frame=k)
        print(f"[skin] craft {name}: {n} frames from {src} {f}")


def make_actions(arm, body):
    if arm.animation_data is None:
        arm.animation_data_create()
    keep = moving_bones(arm, body)
    print("[skin] bones with channels", len(keep), "of", len(arm.data.bones))
    made = []
    stand = None  # 서기의 엉덩이 높이(첫 줄 `idle`에서 잰다 — `FLAT`이 쓴다)
    ends = {}  # 동작마다 끝의 몸 쪽(`CHAIN`)
    for name, src, spec in CLIPS:
        anchors = anchors_of(arm, name, src, spec)
        n, poses, hips, hands = sample(arm, src, anchors, UPRIGHT.get(name))
        if name in FACE_TRAVEL:
            face_travel(arm, name, poses, hips, FACE_TRAVEL[name])
        strip_travel(arm, poses, hips, name in LOOPS)
        if name == "idle":
            stand = sum(h.z for h in hips) / len(hips)
        if name in FLAT:
            flatten(arm, poses, hips, stand)
        if name in FACE_STILL:
            face_still(arm, name, poses)
        if name in AIM:
            face_aim(arm, name, poses, hands, AIM[name])
        if name in HIT_WINDOWS:
            start = ends.get(CHAIN.get(name), 0.0)
            face_hit(arm, name, poses, hips, hands, HIT_WINDOWS[name], math.atan2(math.sin(start), math.cos(start)))
        if name in SKILL_FACE:
            face_skill(arm, name, poses, hips, hands, SKILL_FACE[name])
        ends[name] = own_yaw(arm, poses[-1])
        if spec[0] == "cycle":  # 되풀이 이음매: 끝 = 처음
            poses[-1] = poses[0]
        act = write_action(arm, name, n, poses, 1.0 if name == "drink" else 0.0, keep)
        weapon_pass(arm, act, name in LOOPS)
        made.append(act)
    for a in list(bpy.data.actions):
        if a not in made:
            bpy.data.actions.remove(a)
    return made


def texture(body, old):
    img = next(n.image for n in body.data.materials[0].node_tree.nodes if n.type == "TEX_IMAGE")
    img.scale(TEX, TEX)  # 옛 방랑자와 같은 256 — 게임 거리에서 몸은 100px 남짓
    W.lift_purples(img)
    C.merge_textures([body], img, old["weapons"] + old["lamp"], old["atlas"], NAME)


def export(arm, objs, out_dir):
    sc = bpy.context.scene
    sc.render.fps = OUT_FPS
    arm.name = "root"  # 2차 동작(`secondary.rs`)이 몸 전체를 찌그러뜨리는 마디
    bpy.ops.object.select_all(action="DESELECT")
    for o in [arm] + objs:
        o.select_set(True)
    path = os.path.join(out_dir, NAME + ".glb")
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_yup=True, export_texcoords=True,
                              export_normals=True, export_materials="EXPORT", export_image_format="AUTO",
                              export_cameras=False, export_lights=False, export_extras=False, export_skins=True,
                              export_all_influences=False, export_animations=True, export_animation_mode="ACTIONS",
                              export_force_sampling=False, export_optimize_animation_size=True,
                              export_anim_slide_to_zero=False, export_bake_animation=False, use_selection=True)
    print("[skin] wrote", path)
    return path


def preview(arm, objs, path, act=None, t=0.0, yaw=-35.0, elev=20.0, frame=None):
    """`common.Model.preview`와 같은 Workbench 직교 한 장."""
    m = C.Model.__new__(C.Model)
    m.scene = bpy.context.scene
    m.rig = arm
    m.objects = {o.name: o for o in objs}
    sc = bpy.context.scene
    sc.render.fps = OUT_FPS
    ad = arm.animation_data
    for tr in ad.nla_tracks:
        tr.mute = True
    ad.action = bpy.data.actions[act] if act else None
    for pb in arm.pose.bones:
        if not act:
            pb.location = (0, 0, 0)
            pb.rotation_quaternion = (1, 0, 0, 0)
            pb.scale = (1, 1, 1)
    sc.frame_set(int(round(t * OUT_FPS)))
    _preview_keep_pose(m, path, yaw, elev, frame)


def _preview_keep_pose(m, path, yaw, elev, frame=None):
    keep = C.Model.pose_at
    C.Model.pose_at = lambda self, action=None, t=0.0: None
    try:
        C.Model.preview(m, path, None, 0.0, yaw=yaw, elev=elev, frame=frame, size=200)
    finally:
        C.Model.pose_at = keep


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, sheet_dir=None):
    old = load_old()
    arm, body = load_rig()
    place(arm, body)
    bpy.context.view_layer.objects.active = body  # 가져온 몸의 사용자 법선(glTF NORMAL)을 지워야 매끈/각진 표시가 법선을 정한다
    bpy.ops.mesh.customdata_custom_splitnormals_clear()
    for e in body.data.edges:  # 가져올 때 법선이 갈린 모서리에 붙은 "각진 모서리" 표시도 — 정점이 갈려 늘어난다
        e.use_edge_sharp = False
    if hasattr(body.data, "use_auto_smooth"):  # Blender 4.0: 켜져 있으면 각도로 법선을 갈라 정점이 는다
        body.data.use_auto_smooth = False
    W._flat_where_folded(body.data)  # 줄이며 접힌 작은 면(면 법선 ↔ 매끈한 법선이 반대)은 각지게 — `check_glb` 뒤집힌 면
    attach(arm, old)
    texture(body, old)
    craft_actions(arm)
    make_actions(arm, body)
    objs = [body] + old["weapons"] + old["lamp"]
    for o in old["weapons"][1:]:  # 미리보기에서는 칼만
        o.hide_render = True
    path = export(arm, objs, models_dir)
    preview(arm, objs, os.path.join(preview_dir, f"{NAME}.png"), "idle", 0.0)
    preview(arm, objs, os.path.join(preview_dir, f"{NAME}_front.png"), "idle", 0.0, yaw=-8.0, elev=14.0)
    if sheet_dir:
        for name, _, _ in CLIPS:
            act = bpy.data.actions[name]
            n = int(act.frame_range[1])
            for i in range(6):
                preview(arm, objs, os.path.join(sheet_dir, f"{name}_{i}.png"), name, n * i / 5 / OUT_FPS,
                        yaw=-60.0, elev=25.0, frame=(Vector((0, 0, 0.55)), 2.4))
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_SHEET_DIR"))
