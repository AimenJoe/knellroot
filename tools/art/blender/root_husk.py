"""뿌리의 잔재(기본 적, 키 1.5m — 뿌리 왕관 포함).

지금(0060 W3): Meshy 몸 — 색을 잃은 둥근 아기곰 같은 것, 머리에 보라 뿌리 왕관, 한쪽 눈은 빈 구멍·한쪽은 호박빛 불씨(glow_eye),
가슴을 가로지르는 뿌리 줄기. 원본 `tools/art/sources/meshy/husk_v2.glb`(Meshy 6 Lite, CC BY 4.0) → `meshy_convert.py ... 1.5 700 128`.
옛 상자 몸(구부정한 껍데기, 가슴 틈의 빛)은 `make_blocks()`에 남긴다.

    blender --background --python tools/art/blender/root_husk.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "root_husk"

MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "husk_v2_low.glb")
MESHY_PARTS = {"head_geo": "head", "body_geo": "body", "arm_l_geo": "arm_l", "arm_r_geo": "arm_r",
               "leg_l_geo": "leg_l", "leg_r_geo": "leg_r", "glow_eye": "head"}
# 치수(키 1.5m, 앞 = -Y, 캐릭터 왼쪽 = +X) — 정면·옆 직교 렌더에 눈금을 그어 잰 것.
NECK_Z = 0.665  # 머리 아래 끝(머리와 몸 사이 가장 좁은 곳)
HIP_Z = 0.19  # 다리 위 끝(가랑이 0.13 + 조금)
SHOULDER = (0.22, 0.0, 0.60)
HAND = (0.53, 0.0, 0.27)
ARM_R = 0.13
ARM_X = 0.26  # 이보다 바깥만 팔(몸통 옆면은 몸)
EYE = (-0.13, 0.90)  # 불씨 눈 가운데(x, z) — 캐릭터 오른쪽 눈, 앞면만
EYE_R = (0.085, 0.11)
GREY_K = 0.6  # 잿빛 몸을 이만큼 어둡게 — 그대로면 뜰에서 유령처럼 하얗게 떴다(문지기 0.72보다 더: 작고 여럿이라 바닥 위에서 튄다)
GREY_SAT = 0.18


def _seg_dist(p, a, b):
    from mathutils import Vector
    a, b = Vector(a), Vector(b)
    ab = b - a
    t = max(0.0, min(1.0, (p - a).dot(ab) / ab.length_squared))
    return (p - (a + ab * t)).length


def classify(c):
    """면 가운데 자리 → 부위."""
    from mathutils import Vector
    x, y, z = c.x, c.y, c.z
    ax = abs(x)
    if z > NECK_Z:
        ex, ez = EYE
        rx, rz = EYE_R
        if y < -0.15 and ((x - ex) / rx) ** 2 + ((z - ez) / rz) ** 2 <= 1.0:
            return "glow_eye"
        return "head_geo"
    side = "l" if x > 0 else "r"
    if ax > ARM_X and _seg_dist(Vector((ax, y, z)), SHOULDER, HAND) < ARM_R:
        return f"arm_{side}_geo"
    if z < HIP_Z and y < 0.2:  # 꼬리(뒤 +Y)는 몸
        return f"leg_{side}_geo"
    return "body_geo"


def make():
    """0060 W3: Meshy 몸, 뼈 이름·동작은 옛 잔재 그대로 — 뼈 자리만 새 비율에(머리가 크고 팔다리가 짧다)."""
    m = C.Model(NAME)
    sx, sy, sz = SHOULDER
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0, HIP_Z), "root")
    m.bone("head", (0, 0.0, NECK_Z), "body")
    m.bone("arm_l", (sx, sy, sz), "body")
    m.bone("arm_r", (-sx, sy, sz), "body")
    m.bone("leg_l", (0.12, 0, HIP_Z), "root")
    m.bone("leg_r", (-0.12, 0, HIP_Z), "root")
    m.external(MESHY_LOW, classify, MESHY_PARTS)
    C.darken_grey(m.ext_img, GREY_K, GREY_SAT)
    m.build()
    acts(m)
    return m


def make_blocks():
    m = C.Model(NAME)
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0, 0.76), "root")
    m.bone("head", (0, -0.28, 1.30), "body")
    m.bone("arm_l", (0.31, -0.16, 1.24), "body")
    m.bone("arm_r", (-0.31, -0.16, 1.24), "body")
    m.bone("leg_l", (0.14, 0, 0.74), "root")
    m.bone("leg_r", (-0.14, 0, 0.74), "root")

    lean = (28, 0, 0)  # 몸통은 앞으로 숙였다(가운데를 피벗으로 돌림)
    piv = (0, 0, 0.78)
    g = m.geo("body_geo", "body")
    g.box((0, 0, 0.84), (0.36, 0.26, 0.20), "droot", pivot=piv, rot=lean)  # 골반
    g.box((0, -0.02, 1.09), (0.50, 0.36, 0.40), "root", taper=(1.08, 1.0), pivot=piv, rot=lean)  # 몸통
    g.box((0, 0.13, 1.30), (0.44, 0.24, 0.26), "droot", taper=(0.7, 0.8), pivot=piv, rot=lean)  # 등 혹(뿌리 뭉치)
    g.box((-0.15, 0.10, 1.40), (0.14, 0.12, 0.30), "root", rot=(-20, -25, 0), taper=(0.35, 0.35))  # 등에서 솟은 뿌리
    g.box((0.12, 0.16, 1.34), (0.10, 0.10, 0.22), "droot", rot=(-35, 20, 0), taper=(0.3, 0.3))
    g.box((0.13, -0.20, 1.18), (0.16, 0.05, 0.20), "bone", pivot=piv, rot=lean, drop=("+y",))  # 갈비 조각
    g.box((-0.17, 0.0, 1.30), (0.20, 0.30, 0.06), "moss", pivot=piv, rot=lean)  # 어깨 이끼
    g.box((0.06, -0.10, 0.62), (0.07, 0.07, 0.30), "droot", rot=(10, -6, 0), taper=(0.4, 0.4), top=False)  # 늘어진 뿌리 가닥
    g.box((-0.10, 0.06, 0.64), (0.06, 0.06, 0.26), "root", rot=(-6, 8, 0), taper=(0.4, 0.4), top=False)
    g = m.geo("glow_core", "body")
    g.box((-0.02, -0.205, 1.07), (0.10, 0.05, 0.25), "amber", pivot=piv, rot=(28, 0, 12), drop=("+y",))  # 가슴 틈 빛

    g = m.geo("head_geo", "head")
    g.box((0, -0.38, 1.32), (0.25, 0.25, 0.25), "droot", rot=(10, 0, 0))  # 머리(뿌리 덩이)
    g.box((0.02, -0.50, 1.31), (0.17, 0.04, 0.15), "bone", rot=(10, 0, 6), drop=("+y",))  # 얼굴 자리의 뼈 조각(눈 없음)
    g.box((-0.07, -0.33, 1.48), (0.07, 0.07, 0.18), "root", rot=(-30, 20, 0), taper=(0.3, 0.3))  # 머리 뿌리 뿔

    for side, sx in (("l", 1), ("r", -1)):
        g = m.geo(f"arm_{side}_geo", f"arm_{side}")
        g.box((0.35 * sx, -0.17, 1.02), (0.14, 0.14, 0.44), "root", rot=(0, 6 * -sx, 0))  # 위팔
        g.box((0.37 * sx, -0.19, 0.66), (0.15, 0.15, 0.38), "droot")  # 아래팔
        g.box((0.37 * sx, -0.21, 0.42), (0.17, 0.12, 0.14), "bone")  # 손(뼈)
        g.box((0.37 * sx, -0.23, 0.31), (0.13, 0.05, 0.10), "droot", taper=(1.3, 1), top=False)  # 손가락 뿌리

    for side, sx in (("l", 1), ("r", -1)):
        g = m.geo(f"leg_{side}_geo", f"leg_{side}")
        g.box((0.15 * sx, 0, 0.52), (0.16, 0.17, 0.38), "droot", rot=(-8, 0, 0))  # 넓적다리
        g.box((0.16 * sx, -0.04, 0.16), (0.17, 0.25, 0.32), "root", drop=("-z",))  # 정강이·발

    m.build()
    acts(m)
    return m


def acts(m):
    base = {"body": {"r": (0, 0, 0)}, "arm_l": {"r": (-6, -4, 0)}, "arm_r": {"r": (-6, 4, 0)}}

    # idle 1.6초 고리: 옆으로 흔들흔들
    m.action("idle", 1.6, [
        (0.0, {"body": {"r": (0, -4, 2)}, "head": {"r": (0, 6, 0)}, "arm_l": {"r": (-4, -4, 0)}, "arm_r": {"r": (-8, 2, 0)}}),
        (0.8, {"body": {"r": (2, 4, -2)}, "head": {"r": (4, -6, 0)}, "arm_l": {"r": (-8, -2, 0)}, "arm_r": {"r": (-4, 6, 0)},
               "root": {"l": (0, 0, -0.015)}}),
    ], loop=True)

    # walk 1.0초 고리: 질질 끄는 걸음(왼쪽이 짧고 몸이 크게 흔들림), 팔은 늦게 흔들린다
    m.action("walk", 1.0, [
        (0.0, {"leg_l": {"r": (-24, 0, 0)}, "leg_r": {"r": (18, 0, 0)}, "body": {"r": (4, -8, 6)},
               "arm_l": {"r": (8, -6, 0)}, "arm_r": {"r": (-16, 4, 0)}, "head": {"r": (0, 8, -4)}}),
        (0.25, {"root": {"l": (0, 0, 0.03)}, "leg_l": {"r": (-4, 0, 0)}, "leg_r": {"r": (4, 0, 0)},
                "body": {"r": (2, 0, 0)}, "arm_l": {"r": (0, -4, 0)}, "arm_r": {"r": (-8, 4, 0)}}),
        (0.5, {"leg_l": {"r": (14, 0, 0)}, "leg_r": {"r": (-26, 0, 0)}, "body": {"r": (4, 8, -6)},
               "arm_l": {"r": (-16, -4, 0)}, "arm_r": {"r": (8, 6, 0)}, "head": {"r": (0, -8, 4)}}),
        (0.75, {"root": {"l": (0, 0, 0.02)}, "leg_l": {"r": (2, 0, 0)}, "leg_r": {"r": (-6, 0, 0)},
                "body": {"r": (2, 0, 0)}, "arm_l": {"r": (-6, -4, 0)}, "arm_r": {"r": (0, 4, 0)}}),
    ], loop=True)

    # attack 1.3초: 0~0.5 두 팔을 머리 위로(머리가 커서 바깥으로 벌려 든다), 0.5~0.7 내려찍기, 0.7~1.3 돌아옴.
    # 새 몸(0060 W3)은 머리가 몸만 해서 옛 값(몸 40°·엉덩이 −16cm)이면 얼굴부터 땅에 박혔다 → 몸은 반쯤만 숙이고 머리로 끄덕인다.
    # 0073 과장(사장 "귀여운 녀석들이 과장된 몸짓을 하는 건 더 귀엽고 어울려"): 들기 전에 한 번 움츠리고(0.14),
    # 정점에서는 까치발로 뒤로 크게 젖히며 위로 늘어나고, 내리칠 때 납작해진다 — 몸통 뼈 크기(부피 그대로, KB 찌그러짐·늘어남).
    # 정점 0.5·맞음 0.62는 게임 `render::STRIKE_KEYS`와 같다 — 바꾸면 거기도.
    m.action("attack", 1.3, [
        (0.0, base),
        (0.14, {"root": {"l": (0, -0.02, -0.07)}, "body": {"r": (14, 0, 0), "s": (1.1, 1.1, 0.82)},
                "head": {"r": (8, 0, 0)}, "arm_l": {"r": (22, -10, 0)}, "arm_r": {"r": (22, 10, 0)},
                "leg_l": {"r": (-16, 0, 0)}, "leg_r": {"r": (14, 0, 0)}}),
        (0.5, {"root": {"l": (0, 0.09, 0.13)}, "body": {"r": (-32, 0, 0), "s": (0.9, 0.9, 1.22)},
               "head": {"r": (-24, 0, 0)},
               "arm_l": {"r": (-176, -36, 0)}, "arm_r": {"r": (-176, 36, 0)},
               "leg_l": {"r": (-4, 0, 0)}, "leg_r": {"r": (6, 0, 0)}}),
        (0.62, {"root": {"l": (0, -0.06, -0.07)}, "body": {"r": (24, 0, 0), "s": (1.15, 1.15, 0.76)},
                "head": {"r": (16, 0, 0)},
                "arm_l": {"r": (-64, -10, 0)}, "arm_r": {"r": (-64, 10, 0)},
                "leg_l": {"r": (-20, 0, 0)}, "leg_r": {"r": (16, 0, 0)}}),
        (0.7, {"root": {"l": (0, -0.06, -0.08)}, "body": {"r": (26, 0, 0), "s": (1.06, 1.06, 0.9)},
               "head": {"r": (18, 0, 0)},
               "arm_l": {"r": (-56, -8, 0)}, "arm_r": {"r": (-56, 8, 0)},
               "leg_l": {"r": (-20, 0, 0)}, "leg_r": {"r": (16, 0, 0)}}),
        (0.95, {"root": {"l": (0, -0.04, -0.05)}, "body": {"r": (16, 0, 0), "s": (1.02, 1.02, 0.97)},
                "head": {"r": (10, 0, 0)},
                "arm_l": {"r": (-50, -8, 0)}, "arm_r": {"r": (-52, 8, 0)},
                "leg_l": {"r": (-14, 0, 0)}, "leg_r": {"r": (12, 0, 0)}}),
        (1.3, base),
    ], linear=(0.5,))

    # hurt 0.3초: 뒤로 꺾임(큰 머리가 젖혀진다)
    m.action("hurt", 0.3, [
        (0.0, base),
        (0.05, {"root": {"l": (0, 0.05, 0)}, "body": {"r": (-12, 4, 0)}, "head": {"r": (-16, 0, 8)},
                "arm_l": {"r": (14, -20, 0)}, "arm_r": {"r": (18, 22, 0)}}),
        (0.3, base),
    ])


PREVIEWS = [("idle", 0.0, ""), ("attack", 0.5, "_attack")]
CHECK = [("walk", 0.0), ("walk", 0.5), ("attack", 0.14), ("attack", 0.5), ("attack", 0.62), ("hurt", 0.05)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, check_dir=None):
    m = make()
    path = m.export(models_dir)
    for act, t, suffix in PREVIEWS:
        m.preview(os.path.join(preview_dir, f"{NAME}{suffix}.png"), act, t)
    if check_dir:
        for act, t in CHECK:
            m.preview(os.path.join(check_dir, f"{NAME}_{act}_{int(t * 100):03d}.png"), act, t)
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
