"""사슬에 묶인 권양지기(넷째 보스 — 작업층, 0208·0216 · 모델 0234).

Meshy 몸 — 늙은 오소리: 흰 줄 얼굴·졸린 호박빛 눈(`glow_eyes`), 두꺼운 몸통에 긴 팔·큰 손, 가슴의 금 사이로 호박빛 핵(`glow_core`)과
보라 결정, 어깨·등의 보라 결정, 허리의 가죽띠, **오른 손목(−X)의 쇠고랑**. 원본 `tools/art/sources/meshy/winchkeeper_v1.glb`
(나노바나나 Pro 그림 `tools/art/sources/ai/knellroot/winchkeeper_v1_front.png` → Meshy 7.1 API, CC BY 4.0) →
`meshy_convert.py ... 2.8 2300 256`(키 2.8 = 종 표의 `height`).
사슬·갈고리·추는 그리지 않았다(0208 — 코드·이펙트).

뼈·동작은 첫 문지기의 두 발 표 그대로(`gatekeeper.acts` — root·body·core·head·arm_l·arm_r·club(오른 손목 아래)·leg·shin):
idle · walk · slam(정점 0.9 · 맞음 1.0) · wave(사슬 휘두르기 — 1.0 · 1.12) · charge_windup · charge · hurt · wake.
권양지기만의 수 둘을 더한다: `hook`(쇠고랑 찬 오른팔을 뒤로 감았다 앞으로 내던진다 — 갈고리, 0.8 · 0.9)·
`hurl`(두 팔로 머리 위까지 들었다 앞으로 던진다 — 추, 0.9 · 1.0). 맞는 순간은 `render::STRIKE_KEYS`.

    blender --background --python tools/art/blender/winchkeeper.py
"""
import os
import sys

from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import gatekeeper as G  # noqa: E402

NAME = "winchkeeper"

MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "winchkeeper_v1_low.glb")
MESHY_PARTS = {"head_geo": "head", "glow_eyes": "head", "body_geo": "body", "glow_core": "core",
               "arm_l_geo": "arm_l", "arm_r_geo": "arm_r", "club_geo": "club",
               "leg_l_geo": "leg_l", "leg_r_geo": "leg_r", "shin_l_geo": "shin_l", "shin_r_geo": "shin_r"}
# 치수(키 2.8m, 앞 = -Y, 캐릭터 왼쪽 = +X) — 정면·옆 직교 렌더 눈금과 색 묶음의 면 자리로 잰 것.
NECK_Z = 1.95  # 머리 아래 끝(턱)
HEAD_X = 0.62  # 머리 폭(어깨 결정은 이보다 바깥 — 몸)
HIP_Z = 0.45  # 다리 위 끝(가랑이 ≈ 0.42)
FOOT_Z = 0.14
LEG_X = 0.42
LEG_IN = 0.66  # 다리는 이 안쪽(손은 바깥 — 손끝이 발 높이까지 내려온다)
SHOULDER = (0.85, 0.10, 1.85)
HAND = (1.10, 0.05, 0.20)
ARM_R = 0.52  # 팔 뒷면(y ≈ 0.5)까지
ARM_OUT = 0.95  # 이보다 바깥의 낮은 면은 다 팔(쇠고랑 고리는 손목 바깥 x ≈ −1.64로 나온다)
ARM_X = 0.62  # 이보다 바깥만 팔(몸통 옆면은 몸)
WRIST_Z = 0.60  # 이 아래가 오른손(`club`) — 쇠고랑(≈ 0.65~0.95)은 팔뚝에 남는다
CORE = (0.20, -0.33, 1.55)
GREY_K = 0.75  # 잿빛 몸을 어둡게(문지기 0234와 같은 값) — 호박빛·보라 결정·가죽띠는 그대로
GREY_SAT = 0.18

EYES, CORES = [], []


def _ambers():
    for c, (h, s, v) in C.face_colors(MESHY_LOW):
        if 0.04 < h < 0.15 and s > 0.45 and v > 0.5:
            (EYES if c[2] > 2.1 else CORES).append(c)


def classify(c):
    """면 가운데 자리 → 부위."""
    x, y, z = c.x, c.y, c.z
    ax = abs(x)
    p = (x, y, z)
    if C.near_any(EYES, p):
        return "glow_eyes"
    if C.near_any(CORES, p):
        return "glow_core"
    side = "l" if x > 0 else "r"
    if z > NECK_Z and ax < HEAD_X:
        return "head_geo"
    near_arm = G._seg_dist(Vector((ax, y, z)), SHOULDER, HAND) < ARM_R
    if ax > ARM_X and z < NECK_Z + 0.1 and (near_arm or (ax > ARM_OUT and z < 1.4)):
        if side == "r" and z < WRIST_Z:
            return "club_geo"
        return f"arm_{side}_geo"
    if z < HIP_Z and ax < LEG_IN:
        return f"shin_{side}_geo" if z < FOOT_Z else f"leg_{side}_geo"
    return "body_geo"


def make():
    EYES.clear(), CORES.clear()
    _ambers()
    print(f"[winchkeeper] eyes {len(EYES)} core {len(CORES)}")
    m = C.Model(NAME)
    sx, sy, sz = SHOULDER
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0, HIP_Z), "root")
    m.bone("core", CORE, "body")
    m.bone("head", (0, -0.05, NECK_Z), "body")
    m.bone("arm_l", (sx, sy, sz), "body")
    m.bone("arm_r", (-sx, sy, sz), "body")
    m.bone("club", (-HAND[0], HAND[1], WRIST_Z), "arm_r")
    m.bone("leg_l", (LEG_X, 0.1, HIP_Z), "root")
    m.bone("leg_r", (-LEG_X, 0.1, HIP_Z), "root")
    m.bone("shin_l", (LEG_X, 0.1, FOOT_Z + 0.05), "leg_l")
    m.bone("shin_r", (-LEG_X, 0.1, FOOT_Z + 0.05), "leg_r")
    m.external(MESHY_LOW, classify, MESHY_PARTS)
    C.darken_grey(m.ext_img, GREY_K, GREY_SAT)
    m.build()
    G.acts(m)
    acts(m)
    return m


def acts(m):
    """권양지기만의 수(첫 문지기의 표 위에)."""
    k, legs, REST = G.k, G.legs, G.REST

    # hook 1.3초: 쇠고랑 찬 오른팔을 뒤로 감고(몸을 비틀어 오른어깨가 뒤) 정점 0.8 → 0.9에 앞으로 내던진다(팔이 앞으로 곧게).
    wind = k(legs(-10, 12, 14, 10), root={"l": (0, 0.06, -0.06)}, body={"r": (-8, 0, -28)}, head={"r": (-6, 0, 18)},
             arm_r={"r": (52, 10, -24)}, club={"r": (-30, 0, 0)}, arm_l={"r": (-30, -30, 0)}, core={"s": 1.15})
    fling = k(legs(-30, 20, 16, 30), root={"l": (0, -0.14, -0.12)}, body={"r": (16, 0, 30), "s": (1.05, 1.05, 0.94)},
              head={"r": (-10, 0, -16)}, arm_r={"r": (-96, 8, 12)}, club={"r": (-8, 0, 0)}, arm_l={"r": (24, -16, 0)},
              core={"s": 1.0})
    m.action("hook", 1.3, [
        (0.0, REST),
        (0.3, k(wind, body={"r": (-4, 0, -18)}, arm_r={"r": (30, 8, -14)})),
        (0.8, wind),
        (0.9, fling),
        (1.0, k(fling, root={"l": (0, -0.16, -0.14)})),
        (1.3, REST),
    ], linear=(0.8, 0.9))

    # hurl 1.5초: 움츠렸다(0.2) 두 팔로 머리 위까지 들고(정점 0.9, 까치발로 젖힘) → 1.0에 앞으로 던진다(팔이 앞 아래, 몸을 숙임).
    lift = k(legs(-6, 4, -6, 4), root={"l": (0, 0.1, 0.1)}, body={"r": (-22, 0, 0), "s": (0.95, 0.95, 1.1)},
             head={"r": (-20, 0, 0)}, arm_l={"r": (-168, -18, 0)}, arm_r={"r": (-168, 18, 0)}, club={"r": (-10, 0, 0)},
             core={"s": 1.3})
    throw = k(legs(-34, 30, 10, 36), root={"l": (0, -0.16, -0.22)}, body={"r": (34, 0, 0), "s": (1.08, 1.08, 0.9)},
              head={"r": (-18, 0, 0)}, arm_l={"r": (-70, -10, 0)}, arm_r={"r": (-70, 10, 0)}, club={"r": (-20, 0, 0)},
              core={"s": 1.0})
    m.action("hurl", 1.5, [
        (0.0, REST),
        (0.2, k(legs(-22, 26, -22, 26), root={"l": (0, 0, -0.12)}, body={"r": (14, 0, 0), "s": (1.06, 1.06, 0.9)},
                head={"r": (6, 0, 0)}, arm_l={"r": (24, -16, 0)}, arm_r={"r": (24, 16, 0)}, core={"s": 0.9})),
        (0.6, k(lift, arm_l={"r": (-120, -20, 0)}, arm_r={"r": (-120, 20, 0)}, body={"r": (-10, 0, 0)})),
        (0.9, lift),
        (1.0, throw),
        (1.15, k(throw, root={"l": (0, -0.18, -0.24)})),
        (1.5, REST),
    ], linear=(0.9, 1.0))


PREVIEWS = [("idle", 0.0, ""), ("slam", 0.9, "_slam"), ("hook", 0.9, "_hook")]
CHECK = [("walk", 0.0), ("walk", 0.35), ("slam", 0.9), ("slam", 1.1), ("wave", 1.0), ("wave", 1.2), ("charge_windup", 0.8),
         ("hook", 0.8), ("hook", 0.9), ("hurl", 0.9), ("hurl", 1.0), ("wake", 0.0), ("hurt", 0.08)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, check_dir=None):
    m = make()
    path = m.export(models_dir)
    for act, t, suffix in PREVIEWS:
        m.preview(os.path.join(preview_dir, f"{NAME}{suffix}.png"), act, t)
    if check_dir:
        for act, t in CHECK:
            m.preview(os.path.join(check_dir, f"{NAME}_{act}_{int(t * 100):03d}.png"), act, t, yaw=-60, elev=12)
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
