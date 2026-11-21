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

# 0247: v3 몸(Meshy 7.1 — 그림 A) + v2 동작을 뼈 이름으로 옮겨 담은 것(`meshy_rig.py --retarget`)
RIG = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "raccoon_v3_rig_anim.glb")
NAME = "wanderer"
HEIGHT = 1.4  # 옛 방랑자와 같은 키(귀 끝까지)
OUT_FPS = 100  # 옛 동작처럼 — 길이(0.43·0.25 …)가 프레임에 딱 떨어진다. 키는 줄이므로 크기는 늘지 않는다
SRC_FPS = 24  # 원본 프레임 단위(`meshy_rig.py`가 24로 가져온다 — 아래 표의 수)
HAND_R, HAND_L = "mixamorig:RightHand", "mixamorig:LeftHand"
FORE_L = "mixamorig:LeftForeArm"
SHIELD_AT, SHIELD_OFF = 0.55, 0.055  # 방패(0223): 팔뚝의 어디(팔꿈치 0 → 손목 1) · 팔뚝에서 손등 쪽으로 띄운 몫(m)
TEX = 512  # 0245(사장 "고사양으로" → 텍스처만): 256 → 512 — 숄더뷰·락온·인형에서 몸이 크게 잡힌다
GRIP_IN = 0.55  # 손목 → 가운데 손가락 뿌리의 이 몫이 주먹 가운데(손 메시를 못 찾을 때만 — `fist`)
FIST_R = 0.13  # 쉬는 자세 몸(키 1.4 m)에서 손목으로부터 손 메시를 찾는 반지름(m)
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
# 숙임 상한(사장 2026-11-18 "검 1타에서 머리를 너무 앞으로 숙여서 어색해"): (몸, 머리) 도 — 프레임마다 엉덩이 → 머리의 기울기가 몸 상한을
# 넘으면 등뼈 셋으로 그 각까지 세우고(`solve_upright`를 한 장에), 가슴 → 머리의 꺾임이 머리 상한을 넘으면 목·머리로 반씩 되돌린다
# (`cap_lean`). Double_Combo_Attack의 1타 끝(원본 19~23)은 몸 52°에 머리가 가슴에서 55° 더 꺾여 머리가 바닥을 봤다 — 서기의 머리 꺾임은
# 10~13°, 걷기 30°. 원본 프레임마다 같은 규칙이라 1타 → 2타의 이음매(23)가 그대로 이어진다.
LEAN_CAP = {"attack": (30.0, 30.0), "attack2": (30.0, 30.0),
            # 0238: 모아 베기도(Charged_Slash — 모으기·강타가 같은 원본, 머리 꺾임 58~68°).
            "charge": (30.0, 30.0), "heavy": (30.0, 30.0),
            # 0256(사장 2026-11-19 "뒤로 뛰기, 점프는 너무 숙여져 있고"): 뛰어넘기(Jump_Over_Obstacle)·뒤로 뛰기(Back_Jump)는
            # 몸이 앞으로 엎어질 듯 숙였다 — 공중에서도 몸은 거의 서 있게.
            "jump": (18.0, 20.0), "dodge_back": (18.0, 20.0)}
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
    # 0214: 제비걸음(이어짐 = 걸음 하나 — 걸음마다 되풀이된다)·막아치기·뿌리 솟기.
    "sk_swallow": ((0.12, 0.45, 0.25), "tip"), "sk_bash": ((0.15, 0.0, 0.35), "hips"),
    "sk_roots": ((0.25, 0.30, 0.35), "tip"),
    "throw": ((0.2, 0.0, 0.3), "hips"),  # 0204: 수류탄이 손을 떠나는 순간 몸이 정면(원본 M16은 몸 왼쪽으로 휘둘러 던진다)
    "parry": ((0.08, 0.22, 0.0), "hold"),  # 기술은 아니지만 같은 장치 — 막는 동안 몸이 정면(원본 M15는 들며 옆으로 비튼다)
    "block": ((0.0, 0.2, 0.0), "hold"),  # 방패 막기(0223) — 내내 정면
}
# 방패의 앞면(0223): 방패는 왼팔뚝의 손등 쪽에 붙어 있어 서 있을 때는 팔 바깥을 본다. 방패를 내미는 동작에서는 팔뚝이 앞으로 뻗고
# 손등이 위라 판이 쟁반처럼 눕는다 — 이 동작들에서는 `shield` 뼈를 돌려 앞면이 앞(조금 위)을 보게 한다. 값 = [(초, 몫 0..1)].
SHIELD_FRONT = {
    "parry": [(0.0, 0.0), (0.05, 1.0), (0.3, 1.0), (0.5, 0.0)],
    "block": [(0.0, 1.0), (0.2, 1.0)],
    "parry_push": [(0.0, 1.0), (0.15, 1.0), (0.4, 0.0)],
    "sk_bash": [(0.0, 0.0), (0.1, 1.0), (0.3, 1.0), (0.5, 0.0)],
}
SHIELD_FACE = Vector((0.0, -1.0, 0.3)).normalized()  # 앞(−Y) + 조금 위 — 위에서 내려다보는 카메라에 판이 보이게
SHIELD_OUT = [Vector((0.0, 0.0, 1.0))]  # 쉬는 자세에서 판의 앞면이 보는 쪽(뼈대 공간) — `attach`가 채운다
# 몸이 스스로 돌지 않는 동작(0196): 게임이 몸의 쪽(`facing`)을 돌리는 동안의 발 디딤 — 원본은 몸이 150° 넘게 돌아 게임의 90°와
# 겹쳐 두 번 돌고, 끝에 서기로 홱 돌아왔다(돌아서기). 프레임마다 엉덩이를 정면으로.
FACE_STILL = {"turn_l", "turn_r"}
# 몸을 움직여 가는 동작(0197): 원본에서 엉덩이가 간 쪽을 몸의 앞("front")·뒤("back")에 맞춘다 — 게임이 그 쪽으로 몸을 옮긴다.
# 다른 동작의 이름이면 그 동작이 돈 각을 그대로(0238 — 구르기에서 이어지는 일어나기는 구르기와 같은 각이라야 이음매가 맞는다).
FACE_TRAVEL = {"dodge": "front", "dodge_back": "back", "dodge_rise": "dodge"}
# 모아 친 강타는 모으기 동작이 끝난 쪽에서 시작한다(0196 — 망치 모으기는 몸을 −60° 감아 쥔다: 정면에서 시작하면 떼는 순간 홱 돈다).
CHAIN = {"heavy": "charge", "hammer_slam": "hammer_charge", "staff_release": "staff_charge", "fist_heavy": "fist_charge"}
# 떠 있는 동안 엉덩이를 올리지 않는 동작(0196): 게임이 몸을 `air`만큼 올린다 — 원본의 뛰어오름(엉덩이 +0.2)을 그대로 두면 두 번 뜬다.
# 엉덩이 높이를 서기의 높이 위로 올리지 않는다(웅크림은 남는다).
FLAT = {"jump", "land"}
# 겨누기(0196, 쇠뇌 — 원본은 활쏘기 Archery_Shot): 활을 든 왼팔이 몸 왼쪽을 가리킨다 → 몸을 돌려 오른손 → 왼손(겨누는 줄)이 정면을 보게,
# 쇠뇌는 그 줄을 따라 눕힌다(날 = 오른손 → 왼손, 위 = 하늘). (이름, 들어가는 몫): 그 몫 동안 서기 방향에서 겨누는 방향으로.
AIM = {"bow_draw": 0.7, "bow_shoot": 0.0}  # 0204: 쇠뇌는 활쏘기 대신 두 손을 앞으로 내민 시전 자세(`XBOW_FWD`) — 활쏘기는 몸이 옆으로 섰다(사장)
# 0231: 활은 활쏘기 원본 그대로 — 옆으로 서서 왼손을 과녁 쪽으로 뻗는 것이 활이다. 활은 왼손의 `bow` 뼈에(`bow_pass`).
# 활대는 늘 하늘로 서고, 화살(활의 −y)은 겨누는 동작(`BOW_AIM`)에서는 겨누는 줄(오른손 → 왼손), 그 밖에서는 몸 앞.
BOW_AIM = {"bow_draw", "bow_shoot"}
# 앞으로 겨누기(0204, 쇠뇌): 원본은 두 손을 어깨 높이로 앞에 내민 시전(mage_soell_cast_8) — 몸이 정면을 본다. 쇠뇌는 몸 앞(−Y)을
# 가리키고 날개는 가로(쉬는 자세의 쇠뇌 그대로 — 0196의 겨누기는 날개 축을 하늘로 세워 "쇠뇌를 세로로 든다").
XBOW_FWD = {"xbow_charge", "xbow_shoot"}
# 겨눈 자세에서 나갔다 돌아오는 동작(0205): [(초, 몫)] — 몫 1 = `XBOW_FWD`의 방향, 0 = 팔을 따라(어깨 → 주먹). 토스는 처음·끝이 겨눈 자세다.
XBOW_BLEND = {"throw": [(0.0, 1.0), (0.06, 0.0), (0.38, 0.0), (0.5, 1.0)]}
# 두 손 쥠(0202, 지팡이 — 원본 M08·M09는 두 손으로 쥔 지팡이 동작): 무기는 오른손 뼈에만 달려 있어 어깨 → 주먹 규칙으로 두면
# 왼손과 따로 놀며 엉뚱한 쪽을 가리켰다(사장 "지팡이는 다 이상하고"). 두 손이 가까이(`TWO_HAND_NEAR` 안) 있으면 지팡이를
# 오른손 → 왼손 줄로 눕히고, 한 손을 떼어 휘두르면(멀어지면 `TWO_HAND_FAR`까지) 옛 규칙으로 돌아간다.
# 0203: 평타 1~3타는 한 손으로 크게 휘두르는 Weapon_Combo라 옛 규칙(어깨 → 주먹), 넷째(Attack)는 두 손으로 들어 내려친다.
TWO_HAND = {"staff4"}  # 0206: 모으기·모아 치기는 한 손 내려찍기(Charged_Axe_Chop)
TWO_HAND_NEAR, TWO_HAND_FAR = 0.30, 0.55
SPIN = -1.0  # 휘돌이가 도는 쪽(원본 Skill_03은 오른쪽으로 돈다 — 요가 줄어드는 쪽). 이펙트 `render::overlays::WHIRL_CLIP_TURN`과 같아야 한다

HIT_WINDOWS = {
    "attack": (0.08, 0.10), "attack2": (0.08, 0.10), "attack3": (0.10, 0.12), "heavy": (0.06, 0.12),
    "hammer1": (0.14, 0.17), "hammer2": (0.16, 0.20), "hammer_slam": (0.08, 0.16),
    "staff1": (0.06, 0.07), "staff2": (0.06, 0.07), "staff3": (0.06, 0.08), "staff4": (0.10, 0.13),
    "staff_release": (0.05, 0.11), "sword_dash": (0.08, 0.10),
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
    # 방패 막기(0223): 패링 버튼을 쥔 동안 — 패링(방패 밀기)의 민 자리(패링 창이 끝나는 0.18초 ≈ 원본 30)에서 조금 거둬(28) 머문다.
    # 막기 원본 Block10은 칼을 가로로 세우고 몸을 옆으로 돌려 선 자세라 방패 막기로는 안 읽혔다.
    ("block", "Shield_Push_Left", ("anchors", [(0.0, 30), (0.2, 28)])),
    # 회피(0197): 구르기의 몸통 구간만(3 뜀 → 14 말림 → 26 웅크려 닿음 — 옛 2~30은 0.25초에 4.7배로 뭉갰다). 원본은 왼쪽 앞 37°로 굴러
    # 몸을 돌려 구르는 쪽 = 앞(`FACE_TRAVEL`). 뒤로 피할 때(입력 없이·락온 뒷걸음)는 Back_Jump(2 웅크림 → 4 뜀 → 14 닿음 → 17 받음).
    # 0237(사장 "앞구르기 할때 좀 어색한 모션"): 3에서 시작하면 첫 장이 이미 38° 숙여 서기에서 튀었고, 26(엉덩이 0.19 — 주저앉음)에서
    # 끝나 서기·걷기로 섞이는 0.14초(`gait::BLEND`) 동안 용수철처럼 튀어 올랐다 → 1(살짝 숙임)에서 들어가 굴러(5 → 21) 일어나는 33
    # (엉덩이 0.40 — 서기 0.45에 가깝다)까지. 구르는 몫은 그대로 0.12초 남짓.
    # 0238(사장 "지적한 사항 구현" — 0.25초에 한 바퀴라 빠르다): 회피 시간(얼린 값)은 그대로 두고 일어나기를 떼어 냈다 — 회피 0.25초는
    # 구르기만(1 → 5 들어감 → 21 한 바퀴 → 23 닿음, 구르는 몫 0.12 → 0.185초), 일어나기(23 → 38, 0.2초 `dodge_rise`)는 회피가 끝나
    # 서기·걷기·달리기로 갈 때 렌더가 이어 튼다(`render::actors` `ROLL_RISE` — 공격·다시 회피면 바로 그 동작).
    # 0256(사장 "회피는 너무 속도가 빨라"): 회피 0.25 → 0.32초(`tuning::DODGE_TIME`) — 구르기·뒤로 뛰기의 시간표를 같은 배(×1.28)로.
    ("dodge", "Roll_Dodge_3", ("anchors", [(0.0, 1), (0.045, 5), (0.282, 21), (0.32, 23)])),
    ("dodge_rise", "Roll_Dodge_3", ("anchors", [(0.0, 31), (0.2, 38)])),
    ("dodge_back", "Back_Jump", ("anchors", [(0.0, 2), (0.051, 4), (0.243, 14), (0.32, 17)])),
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
    # 0206(사장 "charged 시리즈 — 모으기 모션이 겹치거나 없는 거에"): 느린 M09 한 바퀴 대신 Charged_Axe_Chop — 모으며 일어나 머리
    # 위로 들어(74 → 93) 쥐고, 떼면 내려찍는다(103~107, 5 m/s).
    ("staff_charge", "Charged_Axe_Chop", ("anchors", [(0.0, 74), (0.3, 93)])),
    ("staff_release", "Charged_Axe_Chop", ("anchors", [(0.0, 93), (0.05, 103), (0.16, 108), (0.52, 124)])),
    # 0206: 검 달리기 공격 = Charged_Upward_Slash 달려들며 아래에서 머리 위까지 올려 베기(21~27) — 예전엔 1타를 빌렸다.
    ("sword_dash", "Charged_Upward_Slash", ("anchors", [(0.0, 6), (0.08, 21), (0.18, 27), (0.43, 36)])),
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
    # 활(0231): 같은 Archery_Shot — 당기기 = 두 팔을 들어 시위를 당겨 겨눈 자리까지(40 → 60, 끝에서 버틴다), 쏘기 = 겨눈 채(70)
    # → 놓음(92, 쇠뇌와 같은 0.058초) → 손이 뒤로 빠짐(97) → 다시 당긴 자리(80). 길이는 쇠뇌와 같다(렌더의 놓는 순간·겨눈 채 머묾).
    ("bow_draw", "Archery_Shot", ("anchors", [(0.0, 40), (0.3, 60)])),
    ("bow_shoot", "Archery_Shot", ("anchors", [(0.0, 70), (0.058, 92), (0.13, 97), (0.24, 80)])),
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
    # 0214 넷째 줄: 제비걸음 = 내디디며 찌르기(Thrust_Slash)의 찌름을 걸음마다(이어짐 구간 = 한 걸음: 18 찌름 → 13 당김 → 18 다시
    # 찌름 → 머묾, 게임이 걸음마다 되풀이), 막아치기 = 왼팔로 밀치기(패링과 같은 원본, 민 자리까지), 뿌리 솟기 = 땅 내리찍기(땅울림
    # 원본 — 이어짐 동안 내리찍은 자세로 뿌리가 솟는다).
    ("sk_swallow", "Thrust_Slash", ("anchors", [(0.0, 8), (0.12, 18), (0.255, 13), (0.39, 18), (0.57, 18), (0.82, 33)])),
    ("sk_bash", "Shield_Push_Left", ("anchors", [(0.0, 16), (0.15, 28), (0.5, 45)])),
    ("sk_roots", "Charged_Ground_Slam", ("hitseg", 8, (34, 44), 66, [0.25, 0.30, 0.35])),
]
LOOPS = {"idle", "walk", "run", "walk_back"}


def world_pt(arm, bone, which="head"):
    b = arm.data.bones[bone]
    return arm.matrix_world @ (b.head_local if which == "head" else b.tail_local)


def fist(arm, body, side):
    """주먹 가운데(무기·등잔·활을 쥐는 자리, 쉬는 자세의 월드 좌표): 손목 너머(아래팔 축으로)·손목에서 `FIST_R` 안의 몸 정점 가운데,
    그 팔 사슬(아래팔·손·손가락·손에 달린 뼈)에 묶인 것만. 0247 — v3 리깅은 오른손 뼈가 손목에 몰려(길이 6 mm, 가운데 손가락 뿌리 14 mm)
    뼈로 셈한 쥔 자리가 손목에 머물고 실제 손(메시)은 7 cm 아래였다. 왼손은 손가락 뼈도 없고 손 메시가 아래팔에 묶였다 — 그래서 뼈가
    아니라 메시로 잰다. 정점이 없으면 옛 셈(손목 → 가운데 손가락 뿌리의 `GRIP_IN`)."""
    hand = HAND_R if side == "R" else HAND_L
    fore = "mixamorig:RightForeArm" if side == "R" else FORE_L
    wrist, elbow = world_pt(arm, hand), world_pt(arm, fore)
    ax = (wrist - elbow).normalized()
    chain = {fore, hand} | {b.name for b in arm.data.bones[hand].children_recursive}
    groups = {g.index: g.name for g in body.vertex_groups}
    pts = []
    for v in body.data.vertices:
        if not v.groups:
            continue
        g = max(v.groups, key=lambda g: g.weight)
        if groups.get(g.group) not in chain:
            continue
        p = body.matrix_world @ v.co
        if (p - wrist).dot(ax) > -0.005 and (p - wrist).length < FIST_R:
            pts.append(p)
    if len(pts) >= 6:
        c = sum(pts, Vector()) / len(pts)
        print(f"[skin] fist {side}: {len(pts)} verts, {(c - wrist).length * 100:.1f} cm past the wrist")
        return c
    mid = "mixamorig:RightHandMiddle1" if side == "R" else "mixamorig:LeftHandMiddle1"
    print(f"[skin] fist {side}: no hand verts — bone fallback")
    return wrist.lerp(world_pt(arm, mid) if mid in arm.data.bones else world_pt(arm, hand, "tail"), GRIP_IN)


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
        "bow": m.objects["w_bow"],
        "lamp": [m.objects[n] for n in ("lamp_geo", "glow_lamp")],
        "shield": m.objects["shield"],
    }
    keep = set(old["weapons"] + old["lamp"] + [old["shield"], old["bow"]])
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


# 팔의 가중치 가두기(0256, 사장 "손이 움직이는 게 아니라 옷이 움직이는 게 있음(맨손으로 확인)"): Meshy 자동 리깅은 망토 자락까지
# 팔 뼈에 묶었다 — 쉬는 자세에서 오른 위팔 뼈에 묶인 정점 254개의 중앙값이 팔 뼈대에서 0.14 m, 가장 먼 것 0.3 m(키 1.4 m 몸).
# 그래서 주먹을 뻗으면 손 대신 망토 앞자락이 판자처럼 앞으로 나갔다. 위팔·아래팔 무리의 가중치를 팔 뼈대(위팔 머리 → 손 끝)에서의
# 거리로 줄이고(`ARM_KEEP` 안은 그대로, `ARM_DROP` 밖은 0, 사이는 매끈하게) 줄인 몫은 가슴(Spine2)으로 — 몸의 합은 그대로.
# 손·손가락 무리는 그대로(주먹 자체).
ARM_KEEP, ARM_DROP = 0.07, 0.12
ARM_GROUPS = ("Arm", "ForeArm")
# 0260(사장 2026-11-20 "망토가 손이랑 같이 움직이는 경우가 많음"): 0256은 위팔·아래팔을 거리로만 가뒀다 — 재 보니 쇄골(Shoulder)에 정점
# 314개가 뼈에서 중앙값 0.24 m(팔을 들면 쇄골이 돌며 망토가 통째로 끌려간다), 위팔 뼈에서 0.07 m 안쪽(거리 규칙이 남기는 자리)에 위팔을 감싼
# 망토가 그대로 남아 주먹을 뻗으면 보라 판이 팔을 따라 늘어났다. 거리로는 팔 살과 망토가 갈리지 않는다 — **색**으로 가른다: 텍스처에서 보라인
# 정점(`is_cloak` — 위팔 100개 중 76개, 쇄골 314개 중 191개, 손은 0개)은 팔 사슬의 무게를 모두 가슴으로. 보라가 아닌 쇄골 정점도 멀면(목·가슴의
# 옷) 거리로 가둔다: 뼈마다 (재는 마디, 그대로 둘 거리, 다 뺄 거리)(m).
# 자동 리깅의 덤 뼈도: 오른손 밑 `Bone_061`(왼손 밑 `Bone_060`은 빈 뼈)에 망토 자락 정점 30개(무게 18.7)가 100% 묶여 주먹을 뻗으면 0.9 m를
# 따라 나갔다 — 팔 사슬 밑의 mixamo가 아닌 뼈(`stray_arm_bones`)는 색과 상관없이 모두 가슴으로.
# 구르기 튐(사장 2026-11-20 "구르는게 이제 좀 방방뛰는 느낌이네"): 덤 뼈의 몫만은 가슴이 아니라 엉덩이로(`STRAY_TO`) — 가슴에 묶인 판은 구르기의
# 공 자세(등뼈 셋 75° 굽힘)에서 지느러미처럼 펴져, 바닥에 닿는 가장 낮은 점이 되어 몸을 0.12 m 들어 올렸다(구르는 0.13~0.16초의 무게중심
# 0.74 → 0.88 m — 큰 튐 하나가 같은 높이의 튐 둘로). 엉덩이도 팔을 따라가지 않으니 0260의 뜻(망토가 주먹을 따라 나가지 않음)은 그대로.
STRAY_TO = "mixamorig:Hips"
CONFINE_MORE = {"Shoulder": (("Shoulder",), 0.10, 0.18)}
CLOAK_HUE = (0.70, 0.92)  # 보라의 색상(0..1 — 252°~331°)
CLOAK_SAT = 0.25
ARM_CHAIN = ("Shoulder", "Arm", "ForeArm", "Hand")  # 이 이름으로 시작하는 무리(손가락 포함)


def stray_arm_bones(arm):
    """팔 사슬(어깨·위팔·아래팔·손) 밑에 매달린 mixamo가 아닌 뼈 이름 — 자동 리깅이 망토 자락에 붙인 덤 뼈."""
    out = set()
    for b in arm.data.bones:
        if b.name.startswith("mixamorig:"):
            continue
        p = b.parent
        while p is not None:
            if any(p.name.startswith(f"mixamorig:{side}{c}") for side in ("Left", "Right") for c in ARM_CHAIN):
                out.add(b.name)
                break
            p = p.parent
    return out


def cloak_verts(body):
    """텍스처에서 보라인 정점(망토·두건)의 번호 — 정점의 UV들에서 읽은 색의 평균."""
    import colorsys
    import numpy as np
    img = next(n.image for n in body.data.materials[0].node_tree.nodes if n.type == "TEX_IMAGE")
    w, h = img.size
    px = np.empty(w * h * 4, dtype=np.float32)
    img.pixels.foreach_get(px)
    px = px.reshape(h, w, 4)
    me = body.data
    uvl = me.uv_layers.active.data
    sums = {}
    for poly in me.polygons:
        for li in poly.loop_indices:
            uv = uvl[li].uv
            x = min(w - 1, max(0, int(uv.x * w)))
            y = min(h - 1, max(0, int(uv.y * h)))
            c, k = sums.get(me.loops[li].vertex_index, (np.zeros(3), 0))
            sums[me.loops[li].vertex_index] = (c + px[y, x, :3], k + 1)
    out = set()
    for vi, (c, k) in sums.items():
        hh, sat, _ = colorsys.rgb_to_hsv(*(c / k))
        if CLOAK_HUE[0] <= hh <= CLOAK_HUE[1] and sat > CLOAK_SAT:
            out.add(vi)
    return out


def confine_arms(arm, body):
    from mathutils.geometry import intersect_point_line
    names = {g.index: g.name for g in body.vertex_groups}
    chest = body.vertex_groups.get("mixamorig:Spine2") or body.vertex_groups.new(name="mixamorig:Spine2")
    mw = body.matrix_world

    def seg(side, b):
        bone = arm.data.bones[f"mixamorig:{side}{b}"]
        return arm.matrix_world @ bone.head_local, arm.matrix_world @ bone.tail_local

    segs = {side: [seg(side, b) for b in ("Arm", "ForeArm", "Hand")] for side in ("Left", "Right")}
    more = {(side, g): ([seg(side, b) for b in on], k, d) for side in ("Left", "Right") for g, (on, k, d) in CONFINE_MORE.items()}

    def dist(p, side, lines=None):
        best = 1e9
        for a, b in lines or segs[side]:
            q, t = intersect_point_line(p, a, b)
            t = max(0.0, min(1.0, t))
            best = min(best, (p - (a + (b - a) * t)).length)
        return best

    cloak = cloak_verts(body)
    stray = stray_arm_bones(arm)
    low = body.vertex_groups.get(STRAY_TO) or body.vertex_groups.new(name=STRAY_TO)
    moved, verts, to_low = 0.0, 0, 0.0
    for v in body.data.vertices:
        p = mw @ v.co
        give = 0.0
        for g in list(v.groups):
            n = names[g.group]
            # 무게는 0으로 쓰기 전에 읽는다 — `g`는 살아 있는 칸이라 쓴 뒤에 읽으면 0이다(0260은 그래서 덤 뼈·망토 몫을 아무 데도
            # 주지 않았다: `Bone_061`에만 묶인 정점은 무게 없이 내보내져 `neutral_bone`에 붙어 쉬는 자세로 굳었다).
            w = g.weight
            if n in stray:
                if w > 1e-4:
                    body.vertex_groups[g.group].add([v.index], 0.0, "REPLACE")
                    low.add([v.index], w, "ADD")
                    to_low += w
                continue
            if v.index in cloak and any(n.startswith(f"mixamorig:{side}{b}") for side in ("Left", "Right") for b in ARM_CHAIN):
                if w > 1e-4:
                    body.vertex_groups[g.group].add([v.index], 0.0, "REPLACE")
                    give += w
                continue
            for side in ("Left", "Right"):
                rule = None
                if n in (f"mixamorig:{side}{b}" for b in ARM_GROUPS):
                    rule = (None, ARM_KEEP, ARM_DROP)
                for g_name in CONFINE_MORE:
                    if n == f"mixamorig:{side}{g_name}":
                        rule = more[(side, g_name)]
                if rule is not None:
                    lines, k0, k1 = rule
                    d = dist(p, side, lines)
                    keep = 1.0 - smooth01((d - k0) / (k1 - k0))
                    cut = g.weight * (1.0 - keep)
                    if cut > 1e-4:
                        body.vertex_groups[g.group].add([v.index], g.weight - cut, "REPLACE")
                        give += cut
        if give > 0.0:
            chest.add([v.index], give, "ADD")
            moved += give
            verts += 1
    print(f"[skin] confine arms: {verts} verts ({len(cloak)} cloak), weight {moved:.1f} -> Spine2;"
          f" stray {sorted(stray)} weight {to_low:.1f} -> {STRAY_TO}")


def attach(arm, body, old):
    """무기 넷은 옛 방향 그대로 주먹 자리의 `weapon` 뼈(오른손 자식)에 — 돌림은 `weapon_pass`가 동작마다 굽는다.
    등잔은 자리만 왼손 밑 `lamp` 뼈로."""
    # 쉬는 자세에서 붙인다 — 가져온 뼈대는 마지막 동작의 자세를 들고 있어서, 그대로 붙이면 그 자세 기준으로 어긋나 붙는다
    # (0193 첫 판의 "무기가 손에서 떨어진" 까닭).
    arm.animation_data.action = None
    for pb in arm.pose.bones:
        pb.matrix_basis.identity()
    bpy.context.view_layer.update()
    grip = fist(arm, body, "R")
    add_bone(arm, "weapon", HAND_R, grip)
    xf = Matrix.Translation(grip - old["hand_r"])
    for o in old["weapons"]:
        mw = xf @ o.matrix_world
        o.parent = arm
        o.parent_type = "BONE"
        o.parent_bone = "weapon"
        o.matrix_world = mw
    grip_l = fist(arm, body, "L")
    add_bone(arm, "lamp", HAND_L, grip_l)
    lf = Matrix.Translation(grip_l - old["hand_l"])
    # 활(0231): 왼손 주먹의 `bow` 뼈 — 옛 빌더는 오른손 자리에 지었으니 그 자리에서 왼손 주먹으로 옮긴다. 돌림은 `bow_pass`.
    add_bone(arm, "bow", HAND_L, grip_l)
    o = old["bow"]
    mw = Matrix.Translation(grip_l - old["hand_r"]) @ o.matrix_world
    o.parent = arm
    o.parent_type = "BONE"
    o.parent_bone = "bow"
    o.matrix_world = mw
    for o in old["lamp"]:
        mw = lf @ o.matrix_world
        o.parent = arm
        o.parent_type = "BONE"
        o.parent_bone = "lamp"
        o.matrix_world = mw
    # 방패(0223): 왼팔뚝 바깥(손등 쪽)에 판을 붙인다 — 팔뚝의 `shield` 뼈(팔꿈치와 손목 사이). 판의 축(+z로 지은 것)을 손등 쪽으로,
    # 손등 쪽 = 팔뚝 방향 × 뒤(+y) — 쉬는 자세의 팔(손바닥이 아래)에서 위·바깥을 본다. 막는 자세에서 팔뚝을 세우면 앞을 본다.
    elbow, wrist = world_pt(arm, FORE_L), world_pt(arm, HAND_L)
    along = (wrist - elbow).normalized()
    out = along.cross(Vector((0.0, 1.0, 0.0))).normalized()
    side = along.cross(out).normalized()
    mid = elbow.lerp(wrist, SHIELD_AT) + out * SHIELD_OFF
    add_bone(arm, "shield", FORE_L, mid)
    SHIELD_OUT[0] = (arm.matrix_world.inverted().to_3x3() @ out).normalized()
    o = old["shield"]
    rot = Matrix((side, along, out)).transposed().to_4x4()  # 열 = x·y·z가 가는 곳(z = 판의 축 → 손등 쪽)
    mw = Matrix.Translation(mid) @ rot @ Matrix.Translation(-old["hand_l"]) @ o.matrix_world
    o.parent = arm
    o.parent_type = "BONE"
    o.parent_bone = "shield"
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


def turn_bone(arm, name, axis, angle):
    """뼈 하나를 뼈대 공간의 축으로 제 머리를 중심 삼아 돌린다(자식은 따라온다)."""
    pb = arm.pose.bones[name]
    p = pb.head.copy()
    pb.matrix = Matrix.Translation(p) @ Matrix.Rotation(angle, 4, axis) @ Matrix.Translation(-p) @ pb.matrix
    bpy.context.view_layer.update()


def cap_lean(arm, body_max, head_max):
    """한 장의 숙임을 상한까지(`LEAN_CAP`)."""
    bpy.context.view_layer.update()
    pb = arm.pose.bones
    snap = (pb["mixamorig:Hips"].head.copy(), [pb[n].head.copy() for n in SPINE], pb["mixamorig:Head"].head.copy())
    f, s = lean_of(snap[2] - snap[0])
    m = math.hypot(f, s)
    if m > math.radians(body_max):
        k = math.radians(body_max) / m
        fwd, side, _ = solve_upright([snap], (math.degrees(f * k), math.degrees(s * k)))
        uprighten(arm, fwd, side)
        bpy.context.view_layer.update()
    c = (pb["mixamorig:Neck"].head - pb["mixamorig:Spine2"].head).normalized()
    h = (pb["mixamorig:Head"].tail - pb["mixamorig:Head"].head).normalized()
    over = c.angle(h) - math.radians(head_max)
    if over > 0.0:
        axis = h.cross(c).normalized()  # 머리 축을 가슴 축 쪽으로
        turn_bone(arm, "mixamorig:Neck", axis, over * 0.5)
        turn_bone(arm, "mixamorig:Head", axis, over * 0.5)


# 몸 말기 — 원본 프레임마다 몸을 더 만다(빈 표 — 0239 덧에서 구르기에 썼다가 0240 공 굴리기로 옮김).
# (원본 → ([(원본 프레임, 몫 0..1)] 들어가고 나오는 시간표, {뼈: 굽힐 각(도)}))
# 각은 그 뼈가 지금 지닌 몸의 옆축(쉬는 자세의 +X를 뼈와 함께 돌린 축)으로: + = 앞으로 굽힘(등·목·머리·정강이), 넓적다리는 −(무릎이 앞으로).
TUCK = {}

# 공처럼 구르기(0240 — 사장 "미리보기 보면 구르지 않는데?"): 어른 몸의 구르기 원본을 바닥 위로 띄우니(0239) 엉덩이를 높이 들고 거꾸로 서서 넘는
# 물구나무로 읽혔다(엉덩이 0.78 m에서 거꾸로 세 장, 등으로 눕고, 앉는다 — 바닥을 따라 구르지 않았다). 큰 머리의 몸은 공으로 말아 그 공의
# 가운데를 축으로 한 바퀴 돌리고 가장 낮은 점을 늘 바닥에 붙인다(제자리에서 돌고, 게임이 몸을 앞으로 옮긴다 = 바닥을 따라 구른다).
# 처음·끝 자세는 원본에서(`start` 살짝 숙임 · `end` 웅크려 일어나는 중 — 일어나기 `dodge_rise`가 거기서 잇는다), 공 = `end` + `BALL_BEND`.
ROLL = {
    "name": "dodge", "src": "Roll_Dodge_3", "start": 1.0, "end": 31.0,
    # 초 — 말기 들어감(시작 → 공) · 공 · 풀림(공 → 끝), 한 바퀴의 시간표(몫 0..1, 코사인으로 이음)
    "curl_in": 0.064, "curl_out": 0.243,  # 0256: 회피 0.32초에 맞춰 ×1.28
    "spin": [(0.026, 0.0), (0.269, 1.0)],
}
BALL_BEND = {"mixamorig:Spine": 25.0, "mixamorig:Spine1": 25.0, "mixamorig:Spine2": 25.0,
             "mixamorig:Neck": 30.0, "mixamorig:Head": 25.0,
             "mixamorig:LeftUpLeg": -80.0, "mixamorig:RightUpLeg": -80.0,
             "mixamorig:LeftLeg": 100.0, "mixamorig:RightLeg": 100.0,
             "mixamorig:LeftArm": -35.0, "mixamorig:RightArm": -35.0}
# 공에 꼬리도 만다(사장 2026-11-20 "구르는게 이제 좀 방방뛰는 느낌이네"): 꼬리가 곧게 뻗은 채 돌면 공이 길쭉해(무게중심에서 바깥까지
# 0.46~0.91 m) 꼬리 끝이 바닥에 닿는 순간 가장 낮은 점 붙이기가 몸을 장대처럼 들어 올렸다(0.20초 무게중심 0.93 m · 꼭대기 1.62 m — 서기 1.42).
# 엉덩이 밑의 mixamo가 아닌 뼈 사슬(`tail_bones` — 자동 리깅의 꼬리)을 마디마다 이 각만큼 앞으로(`BALL_BEND`와 같은 옆축·같은 부호).
TAIL_TUCK = 60.0
ROLL_MARGIN = 0.02  # m — 공의 가장 낮은 점을 바닥에서 이만큼(일어나기의 `CLEAR_MARGIN`과 같게 — 이음매)
# 구르기의 처음·끝 몸 방향(도, 엉덩이 앞 — 정면 0 · 왼쪽 +, 0260 사장 "회피 구른 후 너무 옆이야, 정면과 가깝게"): 원본의 처음·끝 자세는
# 간 쪽을 앞으로 돌린 뒤에도 −72°·−70°(서기 −24°)라 옆을 보고 일어났다(일어나기 −74°). 말기 들어감·풀림 동안 이 각으로 옮겨 간다 —
# 일어나기(`dodge_rise`)도 같은 각을 쓴다(`TRAVEL_TURNS`).
ROLL_FACE = -15.0


def tail_bones(arm):
    """엉덩이 밑에 매달린 mixamo가 아닌 뼈(자동 리깅의 꼬리 사슬) — 부모 → 자식 차례(`stray_arm_bones`와 같은 꼴)."""
    def depth(b):
        k = 0
        while b.parent is not None:
            b, k = b.parent, k + 1
        return k
    out = []
    for b in arm.data.bones:
        if b.name.startswith("mixamorig:"):
            continue
        p = b.parent
        while p is not None and not p.name.startswith("mixamorig:"):
            p = p.parent
        if p is not None and p.name == "mixamorig:Hips":
            out.append(b)
    return [b.name for b in sorted(out, key=depth)]


def hips_yaw(arm):
    """지금 자세의 엉덩이가 보는 쪽(라디안 — `own_yaw`와 같은 셈)."""
    f = arm.pose.bones["mixamorig:Hips"].matrix.to_3x3() @ Vector((0, 0, 1))
    return math.atan2(f.x, -f.y)


def tuck_pose(arm, src, f):
    """`TUCK`: 원본 `src`의 프레임 `f`에 그 몫만큼 몸을 만다(부모부터 — 자식은 따라온다)."""
    if src not in TUCK:
        return
    keys, bends = TUCK[src]
    w = keyed(keys, f) if keys else 0.0
    if w <= 1e-4:
        return
    bpy.context.view_layer.update()
    x = Vector((1.0, 0.0, 0.0))
    for name, deg in bends.items():  # 표의 차례 = 부모 → 자식
        pb = arm.pose.bones[name]
        lat = pb.matrix.to_3x3() @ arm.data.bones[name].matrix_local.to_3x3().inverted() @ x
        turn_bone(arm, name, lat.normalized(), math.radians(deg) * w)


def set_src(arm, src, f):
    """원본 `src`의 프레임 `f` 자세로(채널 없는 뼈는 쉬는 자세)."""
    sc = bpy.context.scene
    arm.animation_data.action = bpy.data.actions[src]
    for pb in arm.pose.bones:
        pb.location = (0, 0, 0)
        pb.rotation_quaternion = (1, 0, 0, 0)
        pb.scale = (1, 1, 1)
    sc.frame_set(int(math.floor(f)), subframe=f - math.floor(f))
    bpy.context.view_layer.update()


def grab(arm):
    return {pb.name: pb.matrix_basis.decompose() for pb in arm.pose.bones}


def put_pose(arm, a, b, u):
    """두 자세(`grab`) 사이 `u`(0..1)를 뼈마다 섞어 놓는다."""
    arm.animation_data.action = None
    for pb in arm.pose.bones:
        (la, qa, sa), (lb, qb, sb) = a[pb.name], b[pb.name]
        if qa.dot(qb) < 0:
            qb = -qb
        pb.matrix_basis = Matrix.LocRotScale(la.lerp(lb, u), qa.slerp(qb, u), sa.lerp(sb, u))
    bpy.context.view_layer.update()


def rotate_about(arm, name, axis, angle, c):
    """뼈 하나를 뼈대 공간의 점 `c`를 지나는 축으로 돌린다(자식은 따라온다)."""
    pb = arm.pose.bones[name]
    pb.matrix = Matrix.Translation(c) @ Matrix.Rotation(angle, 4, axis) @ Matrix.Translation(-c) @ pb.matrix
    bpy.context.view_layer.update()


def shift_bone(arm, name, d):
    pb = arm.pose.bones[name]
    pb.matrix = Matrix.Translation(d) @ pb.matrix
    bpy.context.view_layer.update()


def mesh_box(body):
    """평가한 몸의 세계 정점 — (가장 낮은 z, 가운데)."""
    dg = bpy.context.evaluated_depsgraph_get()
    dg.update()
    ev = body.evaluated_get(dg)
    me = ev.to_mesh()
    ps = [body.matrix_world @ v.co for v in me.vertices]
    ev.to_mesh_clear()
    lo = Vector((min(p.x for p in ps), min(p.y for p in ps), min(p.z for p in ps)))
    hi = Vector((max(p.x for p in ps), max(p.y for p in ps), max(p.z for p in ps)))
    return lo.z, (lo + hi) * 0.5


def craft_roll(arm, body, anchors):
    """`ROLL`: 시작 자세 → 공(끝 자세 + `BALL_BEND`)으로 말아 공의 가운데를 축으로 앞으로 한 바퀴 → 끝 자세. 엉덩이의 수평 자리는 쉬는 자세에
    두고(제자리 — 게임이 옮긴다), 프레임마다 가장 낮은 정점을 바닥 + `ROLL_MARGIN`에 붙인다. 원본이 간 쪽을 몸의 앞으로 돌린 각은 `TRAVEL_TURNS`에
    적는다(일어나기가 같은 각을 쓴다)."""
    src = ROLL["src"]
    hips_n = "mixamorig:Hips"
    rest = arm.data.bones[hips_n].head_local
    inv = arm.matrix_world.inverted()
    up = inv.to_3x3() @ Vector((0.0, 0.0, 1.0))
    set_src(arm, src, ROLL["start"])
    a = grab(arm)
    h0 = arm.pose.bones[hips_n].head.copy()
    set_src(arm, src, ROLL["end"])
    c_pose = grab(arm)
    h1 = arm.pose.bones[hips_n].head.copy()
    bends = list(BALL_BEND.items()) + [(n, TAIL_TUCK) for n in tail_bones(arm)]
    for name, deg in bends:  # 표의 차례 = 부모 → 자식
        pb = arm.pose.bones[name]
        lat = pb.matrix.to_3x3() @ arm.data.bones[name].matrix_local.to_3x3().inverted() @ Vector((1.0, 0.0, 0.0))
        turn_bone(arm, name, lat.normalized(), math.radians(deg))
    ball = grab(arm)
    d = h1 - h0
    turn = -math.atan2(d.x, -d.y)
    TRAVEL_TURNS[ROLL["name"]] = turn

    x_rest = arm.data.bones[hips_n].matrix_local.to_3x3().inverted() @ Vector((1.0, 0.0, 0.0))

    def placed(pa, pb_, u, square=0.0, extra=0.0):
        """섞은 자세 + 엉덩이를 쉬는 자리(수평)로 + 간 쪽을 앞으로(+ `extra` — `ROLL_FACE`로 옮겨 가는 몫) + `square`(0..1)만큼 몸의 옆축을
        구르는 옆축(+X)에 맞춘다 — 원본의 웅크린 끝 자세는 옆으로 27° 기울어 그대로 말면 공이 옆으로 누운 채(58°) 돌아 비스듬히 굴렀다."""
        put_pose(arm, pa, pb_, u)
        h = arm.pose.bones[hips_n].head
        shift_bone(arm, hips_n, Vector((rest.x - h.x, rest.y - h.y, 0.0)))
        turn_bone(arm, hips_n, Vector((0.0, 0.0, 1.0)), turn + extra)
        if square > 1e-4:
            pb = arm.pose.bones[hips_n]
            side = (pb.matrix.to_3x3() @ x_rest).normalized()
            q = side.rotation_difference(Vector((1.0, 0.0, 0.0)))
            ax, ang = q.to_axis_angle()
            if ang > 1e-5:
                turn_bone(arm, hips_n, ax, ang * square)

    face = math.radians(ROLL_FACE)
    placed(a, a, 0.0)
    corr_a = face - hips_yaw(arm)
    placed(c_pose, c_pose, 0.0)
    corr_c = face - hips_yaw(arm)
    corr_a, corr_c = (math.atan2(math.sin(c), math.cos(c)) for c in (corr_a, corr_c))
    TRAVEL_TURNS[ROLL["name"]] = turn + corr_c
    placed(ball, ball, 0.0, 1.0)
    _, mid = mesh_box(body)
    cen = inv @ mid
    lat = Vector((1.0, 0.0, 0.0))  # 돌린 뒤 앞 = −Y → 옆축 = +X(+각 = 머리가 앞·아래로)
    dur = anchors[-1][0]
    n = max(1, int(round(dur * OUT_FPS)))
    poses, hips, hands = [], [], []
    ci, co = ROLL["curl_in"], ROLL["curl_out"]
    for k in range(n + 1):
        t = k / OUT_FPS
        if t <= ci:
            u = smooth01(t / ci)
            placed(a, ball, u, u, corr_a * (1.0 - u))
        elif t < co:
            placed(ball, ball, 0.0, 1.0)
        else:
            u = smooth01((t - co) / max(dur - co, 1e-6))
            placed(ball, c_pose, u, 1.0 - u, corr_c * u)
        th = 2.0 * math.pi * keyed(ROLL["spin"], t)
        if th > 1e-5:
            rotate_about(arm, hips_n, lat, th, cen)
        low, _ = mesh_box(body)
        shift_bone(arm, hips_n, up * (ROLL_MARGIN - low))
        poses.append({pb.name: pb.matrix_basis.decompose() for pb in arm.pose.bones if pb.name not in ("lamp", "weapon", "shield", "bow")})
        hips.append(arm.pose.bones[hips_n].head.copy())
        hands.append((arm.pose.bones[HAND_R].head.copy(), arm.pose.bones["mixamorig:RightArm"].head.copy(),
                      arm.pose.bones[HAND_L].head.copy()))
    print(f"[skin] roll: ball turned {math.degrees(turn):+.0f}°, ends turned {math.degrees(corr_a):+.0f}°/{math.degrees(corr_c):+.0f}°"
          f" to {ROLL_FACE:+.0f}°, centre {mid.z:.2f} m, {n} frames")
    return n, poses, hips, hands


def sample(arm, src, anchors, upright=None, cap=None, end=None):
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
    for k, f in enumerate(times):
        go(f)
        if corr is not None:
            uprighten(arm, *corr)
        if cap is not None:
            cap_lean(arm, *cap)
        if end is not None:  # `END_UPRIGHT`: 끝으로 갈수록 상한이 90° → (몸, 머리)
            ramp = max(1.0, end[0] * OUT_FPS)
            w = smooth01((k - (n - ramp)) / ramp)
            if w > 1e-4:
                cap_lean(arm, 90.0 + (end[1] - 90.0) * w, 90.0 + (end[2] - 90.0) * w)
        tuck_pose(arm, src, f)
        poses.append({pb.name: pb.matrix_basis.decompose() for pb in arm.pose.bones if pb.name not in ("lamp", "weapon", "shield", "bow")})
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


TRAVEL_TURNS = {}  # 동작 → `face_travel`이 돌린 각

# 땅 위로(0239 — 사장 "머리가 큰게 문제구나 … 조금 점프 해서 구르면 괜찮나?"): 원본 구르기는 어른 몸의 것이라 큰 머리가 땅에 박혔다 —
# 원본 프레임마다 몸의 가장 낮은 정점을 재니 머리가 바닥 아래 0.57 m(키의 40%, 12~14), 닿을 때 다리·꼬리가 0.3 m(20~24).
# 그 원본을 쓰는 동작들의 엉덩이를 원본 프레임마다 "바닥 위로 필요한 만큼"을 덮는 매끈한 곡선(필요량마다 종 모양을 겹친 위 테두리)만큼
# 올린다 — 저절로 뛰어들며 구르는 꼴이 된다. 같은 원본이라 구르기 → 일어나기의 이음매도 같은 곡선. (원본 → 그 원본을 쓰는 동작들)
GROUND_CLEAR = {"Roll_Dodge_3": ("dodge_rise",)}  # 0240: 구르기는 공 굴리기(`ROLL`)가 프레임마다 바닥에 붙인다
CLEAR_MARGIN = 0.02  # m — 바닥에서 띄우는 몫
CLEAR_WIDTH = 0.5  # 원본 프레임 — 종 모양의 폭(0240: 일어나기만 — 닿을 몫이 작아 거의 프레임마다 그대로, 공 굴리기의 끝과 이음매가 맞게)


def clear_curve(arm, body, src):
    """원본 `src`의 정수 프레임마다 올릴 높이(m, 세계) — 몸의 가장 낮은 정점이 바닥 + `CLEAR_MARGIN` 위에 오게, 매끈하게."""
    sc = bpy.context.scene
    arm.animation_data.action = bpy.data.actions[src]
    dg = bpy.context.evaluated_depsgraph_get()
    end = int(math.ceil(bpy.data.actions[src].frame_range[1]))
    need = []
    for f in range(end + 1):
        for pb in arm.pose.bones:
            pb.location = (0, 0, 0)
            pb.rotation_quaternion = (1, 0, 0, 0)
            pb.scale = (1, 1, 1)
        sc.frame_set(f)
        tuck_pose(arm, src, f)
        dg.update()
        ev = body.evaluated_get(dg)
        me = ev.to_mesh()
        low = min((body.matrix_world @ v.co).z for v in me.vertices)
        ev.to_mesh_clear()
        need.append(max(0.0, CLEAR_MARGIN - low) if low < CLEAR_MARGIN else 0.0)
    curve = [max([r * math.exp(-((f - k) / CLEAR_WIDTH) ** 2) for k, r in enumerate(need)] + [0.0]) for f in range(end + 1)]
    print(f"[skin] clear {src}: need max {max(need):.2f} m at {need.index(max(need))}, lift max {max(curve):.2f} m")
    return curve


def lift_at(curve, f):
    i = min(max(int(math.floor(f)), 0), len(curve) - 1)
    j = min(i + 1, len(curve) - 1)
    u = min(max(f - i, 0.0), 1.0)
    return curve[i] + (curve[j] - curve[i]) * u


def face_travel(arm, name, poses, hips, way):
    """`FACE_TRAVEL`: 처음 → 끝 엉덩이 이동이 몸의 앞(또는 뒤)이 되게 몸 전체를 한 각만큼 돌린다(다른 동작 이름이면 그 각)."""
    d = hips[-1] - hips[0]
    if way in TRAVEL_TURNS:
        turn = TRAVEL_TURNS[way]
    else:
        if way == "back":
            d = -d
        turn = -math.atan2(d.x, -d.y)
    TRAVEL_TURNS[name] = turn
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


LOOK = (("mixamorig:Neck", 0.45), ("mixamorig:Head", 1.0))  # 목이 남은 돌림의 몫 · 머리가 나머지
LOOK_MAX = math.radians(110)


def look_ahead(arm, name, poses):
    """옆으로 서서 겨누는 동작(`BOW_AIM`, 0231 — 사장 "옆으로 서더라도 앞을 바라봐야지"): 몸은 겨누는 줄을 따라 옆으로 서도
    얼굴은 과녁(정면 −Y)을 보게 목·머리를 세로축으로 돌린다. 등뼈는 두지 않는다 — 팔(겨누는 줄)이 따라 돌지 않게."""
    head = arm.pose.bones["mixamorig:Head"]
    face = arm.data.bones["mixamorig:Head"].matrix_local.to_3x3().inverted() @ Vector((0, -1, 0))  # 쉬는 자세의 얼굴 앞(머리 뼈 공간)
    turned = []
    for pose in poses:
        for pb in arm.pose.bones:
            if pb.name in pose:
                loc, q, sc_ = pose[pb.name]
                pb.matrix_basis = Matrix.LocRotScale(loc, q, sc_)
            else:
                pb.matrix_basis.identity()
        bpy.context.view_layer.update()
        f = head.matrix.to_3x3() @ face
        left = -math.atan2(f.x, -f.y)
        left = max(-LOOK_MAX, min(LOOK_MAX, left))
        turned.append(left)
        for bn, share in LOOK:
            f = head.matrix.to_3x3() @ face
            rest = -math.atan2(f.x, -f.y)
            pb = arm.pose.bones[bn]
            loc, q, sc_ = pb.matrix_basis.decompose()
            a = pb.matrix.to_3x3() @ pb.matrix_basis.to_3x3().inverted()
            r = Matrix.Rotation(max(-LOOK_MAX, min(LOOK_MAX, rest)) * share, 3, "Z")
            pb.matrix_basis = Matrix.LocRotScale(loc, (a.inverted() @ r @ a @ q.to_matrix()).to_quaternion(), sc_)
            bpy.context.view_layer.update()
        for bn, _ in LOOK:
            pose[bn] = arm.pose.bones[bn].matrix_basis.decompose()
    print(f"[skin] look {name}: head turned {math.degrees(min(turned)):+.0f}..{math.degrees(max(turned)):+.0f}°")


# 서 있는 동안 고개(사장 2026-11-19 "서있을때 고개를 너무 숙이고 있는듯"): 대기 원본(Idle_02)은 얼굴이 쉬는 자세보다 18~20° 아래를
# 본다 — v2는 두건이 얼굴을 가려 안 보였고, 얼굴이 드러난 v3(0247)에서는 땅만 보는 것처럼 읽힌다. 목·머리를 몸의 옆축으로 들어
# 올린다(도). 몸짓 둘은 서기의 0초 자세에서 시작해 거기로 돌아오니(0153) 같은 값을 준다 — 이음매가 튀지 않게.
HEAD_LIFT = {"idle": 15.0, "idle_look": 15.0, "idle_shake": 15.0, "turn_l": 15.0, "turn_r": 15.0}
# 끝 자세의 고개(0256, 사장 "내려찍고 일어나기, 줍기는 끝 자세에서 고개가 숙여져 있음"): 끝의 이 초 동안 서기의 고개(`HEAD_LIFT` idle)로
# 서서히 든다 — 끝 자세가 서기로 이어지는 동작들. 중간(줍는 순간·내려찍은 순간)의 숙임은 그대로.
# 걷기·달리기의 쥔 팔(0260, 사장 "걷기, 달리기 무기가 손에 안 붙음"): 원본 걷기는 오른손을 엉덩이 아래(0.36~0.48 m — 서기 0.65 m)로
# 늘어뜨려 손이 망토 자락 속에 묻히고 칼만 밑으로 삐져나왔다(칼은 손뼈를 따라간다 — 손 메시와 1 cm 안). 오른팔 사슬의 로컬 돌림을
# 프레임마다 서기의 첫 자세 쪽으로 이 몫만큼 섞는다 — 흔들림은 조금 남는다.
ARM_HOLD = {"walk": 0.7, "walk_back": 0.7, "run": 0.5}
HOLD_BONES = ("mixamorig:RightShoulder", "mixamorig:RightArm", "mixamorig:RightForeArm", "mixamorig:RightHand")


def hold_arm(poses, ref, w):
    """`ARM_HOLD`: 오른팔 사슬의 로컬 돌림을 `ref`(서기 첫 자세) 쪽으로 `w`만큼."""
    for p in poses:
        for b in HOLD_BONES:
            if b not in p or b not in ref:
                continue
            loc, q, sc = p[b]
            qr = ref[b][1]
            if q.dot(qr) < 0:
                qr = -qr
            p[b] = (loc, q.slerp(qr, w), sc)


HEAD_SETTLE = {"pickup": 0.30, "plunge_land": 0.20, "land": 0.10, "dodge_rise": 0.15}
# 끝 자세를 세우기(0256): 끝의 이 초 동안 숙임 상한이 90° → (몸, 머리)도로 줄어든다 — 내려찍고 웅크린 몸이 끝에 서기로 일어나게.
END_UPRIGHT = {"plunge_land": (0.22, 15.0, 15.0), "pickup": (0.30, 15.0, 15.0)}
LIFT_SHARE = (("mixamorig:Neck", 0.4), ("mixamorig:Head", 0.6))


def lift_head(arm, name, poses, deg, weights=None):
    """`HEAD_LIFT`: 얼굴이 보는 쪽을 몸의 옆축으로 `deg`만큼 들어 올린다 — 목이 몫 0.4, 머리가 나머지(`look_ahead`와 같은 셈).
    `weights`가 있으면 장마다 그 몫만큼(`HEAD_SETTLE` — 끝에서만)."""
    head = arm.pose.bones["mixamorig:Head"]
    face = arm.data.bones["mixamorig:Head"].matrix_local.to_3x3().inverted() @ Vector((0, -1, 0))
    for k, pose in enumerate(poses):
        w = 1.0 if weights is None else weights[k]
        if w <= 1e-4:
            continue
        for pb in arm.pose.bones:
            if pb.name in pose:
                loc, q, sc_ = pose[pb.name]
                pb.matrix_basis = Matrix.LocRotScale(loc, q, sc_)
            else:
                pb.matrix_basis.identity()
        bpy.context.view_layer.update()
        f = (head.matrix.to_3x3() @ face).normalized()
        side = f.cross(Vector((0, 0, 1)))
        if side.length < 1e-4:
            continue
        side.normalize()
        for bn, share in LIFT_SHARE:
            pb = arm.pose.bones[bn]
            loc, q, sc_ = pb.matrix_basis.decompose()
            a = pb.matrix.to_3x3() @ pb.matrix_basis.to_3x3().inverted()
            r = Matrix.Rotation(math.radians(deg) * share * w, 3, side)
            pb.matrix_basis = Matrix.LocRotScale(loc, (a.inverted() @ r @ a @ q.to_matrix()).to_quaternion(), sc_)
            bpy.context.view_layer.update()
        for bn, _ in LIFT_SHARE:
            pose[bn] = arm.pose.bones[bn].matrix_basis.decompose()
    print(f"[skin] lift {name}: head +{deg:.0f}°")


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


def bow_pass(arm, act):
    """`bow` 뼈를 프레임마다 돌려 활대(활의 +z)가 하늘로 서고 화살(활의 −y)이 겨누는 줄(`BOW_AIM` — 오른손 → 왼손) 또는 몸 앞을
    보게(0231). 활은 왼손 주먹이 돌림 중심."""
    sc = bpy.context.scene
    ad = arm.animation_data
    ad.action = act
    n = int(act.frame_range[1])
    lh, rh = arm.pose.bones[HAND_L], arm.pose.bones[HAND_R]
    lr = arm.data.bones[HAND_L].matrix_local
    br = arm.data.bones["bow"].matrix_local
    rb = br.to_3x3().normalized()
    old_frame = basis(Vector((0, -1, 0)), Vector((1, 0, 0)))
    up = Vector((0, 0, 1))
    qs, prev = [], None
    for k in range(n + 1):
        sc.frame_set(k)
        q = (lh.matrix @ lr.inverted() @ br).to_3x3().normalized()
        if act.name in BOW_AIM:
            d = lh.head - rh.head
        else:
            d = arm.pose.bones["mixamorig:Hips"].matrix.to_3x3() @ Vector((0, 0, 1))  # 엉덩이 뼈의 z = 몸 앞(`own_yaw`와 같은 셈)
        if act.name not in BOW_AIM:
            d.z = 0.0
        if d.length < 1e-4:
            d = Vector((0, -1, 0))
        d.normalize()
        u = (up - d * d.dot(up)).normalized()
        e = u.cross(d)  # basis(d, e)의 셋째 열 d × e = 하늘
        rot = (q.inverted() @ basis(d, e) @ old_frame.transposed() @ rb).to_quaternion()
        if prev is not None and prev.dot(rot) < 0:
            rot.negate()
        prev = rot
        qs.append(tuple(rot))
    put(act, "bow", "rotation_quaternion", qs, reduce(qs, ROT_TOL))
    ad.action = None


def shield_pass(arm, act):
    """방패를 내미는 동작(`SHIELD_FRONT`)에서 `shield` 뼈를 프레임마다 돌려 판의 앞면이 앞을 보게 — 판 가운데(뼈의 머리)가 돌림 중심."""
    keys = SHIELD_FRONT.get(act.name)
    if not keys:
        return
    sc = bpy.context.scene
    ad = arm.animation_data
    ad.action = act
    n = int(act.frame_range[1])
    fore = arm.pose.bones[FORE_L]
    fr = arm.data.bones[FORE_L].matrix_local
    br = arm.data.bones["shield"].matrix_local
    rb = br.to_3x3().normalized()
    local_n = rb.inverted() @ SHIELD_OUT[0]
    qs, prev = [], None
    for k in range(n + 1):
        sc.frame_set(k)
        q = (fore.matrix @ fr.inverted() @ br).to_3x3().normalized()  # 이 자세에서 방패 뼈의 방향(제 돌림 없이)
        cur = (q @ local_n).normalized()
        g = keyed(keys, k / OUT_FPS)
        dq = cur.rotation_difference(SHIELD_FACE)
        turn = Matrix.Identity(3).to_quaternion().slerp(dq, g)
        rot = (q.inverted() @ turn.to_matrix() @ q).to_quaternion()
        if prev is not None and prev.dot(rot) < 0:
            rot.negate()
        prev = rot
        qs.append(tuple(rot))
    put(act, "shield", "rotation_quaternion", qs, reduce(qs, ROT_TOL))
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
    hold_ref = {}  # 서기의 첫 자세(첫 줄 `idle` — `ARM_HOLD`가 쓴다)
    ends = {}  # 동작마다 끝의 몸 쪽(`CHAIN`)
    lifts = {src: clear_curve(arm, body, src) for src in GROUND_CLEAR}
    up = arm.matrix_world.inverted().to_3x3() @ Vector((0.0, 0.0, 1.0))  # 세계 1 m 위 = 뼈대 공간의 이만큼
    for name, src, spec in CLIPS:
        anchors = anchors_of(arm, name, src, spec)
        crafted = name == ROLL["name"]
        if crafted:
            n, poses, hips, hands = craft_roll(arm, body, anchors)
        else:
            n, poses, hips, hands = sample(arm, src, anchors, UPRIGHT.get(name), LEAN_CAP.get(name), END_UPRIGHT.get(name))
        if name in GROUND_CLEAR.get(src, ()):
            for k in range(len(hips)):
                hips[k] = hips[k] + up * lift_at(lifts[src], src_at(anchors, k / OUT_FPS))
        if name in FACE_TRAVEL and not crafted:
            face_travel(arm, name, poses, hips, FACE_TRAVEL[name])
        if not crafted:  # 공 굴리기는 이미 제자리
            strip_travel(arm, poses, hips, name in LOOPS)
        if name == "idle":
            stand = sum(h.z for h in hips) / len(hips)
            hold_ref = dict(poses[0])
        if name in ARM_HOLD:
            hold_arm(poses, hold_ref, ARM_HOLD[name])
        if name in FLAT:
            flatten(arm, poses, hips, stand)
        if name in FACE_STILL:
            face_still(arm, name, poses)
        if name in AIM:
            face_aim(arm, name, poses, hands, AIM[name])
        if name in BOW_AIM:
            look_ahead(arm, name, poses)
        if name in HEAD_LIFT:
            lift_head(arm, name, poses, HEAD_LIFT[name])
        if name in HEAD_SETTLE:
            m = len(poses)
            ramp = max(1.0, HEAD_SETTLE[name] * OUT_FPS)
            ws = [smooth01((k - (m - 1 - ramp)) / ramp) for k in range(m)]
            lift_head(arm, name, poses, HEAD_LIFT["idle"], ws)
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
        bow_pass(arm, act)
        shield_pass(arm, act)
        made.append(act)
    for a in list(bpy.data.actions):
        if a not in made:
            bpy.data.actions.remove(a)
    return made


def texture(body, old):
    img = next(n.image for n in body.data.materials[0].node_tree.nodes if n.type == "TEX_IMAGE")
    img.scale(TEX, TEX)  # 쿼터뷰에서 몸은 100px 남짓이지만 숄더뷰·인형은 그 몇 배(0245)
    W.lift_purples(img)
    C.merge_textures([body], img, old["weapons"] + old["lamp"] + [old["shield"], old["bow"]], old["atlas"], NAME)


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
        C.Model.preview(m, path, None, 0.0, yaw=yaw, elev=elev, frame=frame,
                        size=int(os.environ.get("KR_PREVIEW_SIZE", "200")))
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
    if not os.environ.get("KR_NO_CONFINE"):
        confine_arms(arm, body)
    attach(arm, body, old)
    texture(body, old)
    craft_actions(arm)
    make_actions(arm, body)
    objs = [body] + old["weapons"] + old["lamp"] + [old["shield"], old["bow"]]
    for o in old["weapons"][1:] + [old["bow"]]:  # 미리보기에서는 칼만
        o.hide_render = True
    path = export(arm, objs, models_dir)
    preview(arm, objs, os.path.join(preview_dir, f"{NAME}.png"), "idle", 0.0)
    preview(arm, objs, os.path.join(preview_dir, f"{NAME}_front.png"), "idle", 0.0, yaw=-8.0, elev=14.0)
    # 방패(0223): 막는 자세(패링 동작의 민 자리)를 앞·옆에서 — 판의 앞면이 앞을 보는지 본다.
    preview(arm, objs, os.path.join(preview_dir, f"{NAME}_guard.png"), "block", 0.2, yaw=-20.0, elev=14.0)
    preview(arm, objs, os.path.join(preview_dir, f"{NAME}_guard_side.png"), "block", 0.2, yaw=-80.0, elev=14.0)
    if sheet_dir:
        if os.environ.get("KR_SHEET_BARE"):  # 맨손으로(무기·방패·활을 다 숨긴다 — 손과 옷의 움직임을 본다)
            for o in old["weapons"] + [old["shield"], old["bow"]]:
                o.hide_render = True
        only = [x for x in os.environ.get("KR_SHEET_ONLY", "").split(",") if x]  # 몇 동작만(이름 쉼표로)
        for name, _, _ in CLIPS:
            if only and name not in only:
                continue
            act = bpy.data.actions[name]
            n = int(act.frame_range[1])
            k = int(os.environ.get("KR_SHEET_N", "6"))
            for i in range(k):
                preview(arm, objs, os.path.join(sheet_dir, f"{name}_{i}.png"), name, n * i / (k - 1) / OUT_FPS,
                        yaw=float(os.environ.get("KR_SHEET_YAW", "-60")), elev=float(os.environ.get("KR_SHEET_ELEV", "25")),
                        frame=(Vector((0, 0, 0.55)), float(os.environ.get("KR_SHEET_SPAN", "2.4"))))
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_SHEET_DIR"))
