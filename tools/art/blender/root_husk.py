"""뿌리의 잔재(기본 적, 키 1.5m): 뿌리에 덮인 구부정한 껍데기. 얼굴 없음 — 가슴 틈의 어두운 호박빛(glow_core)만.
긴 팔, 뼈 조각·이끼가 군데군데.

    blender --background --python tools/art/blender/root_husk.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "root_husk"


def make():
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

    # attack 1.3초: 0~0.5 두 팔을 머리 위로, 0.5~0.7 내려찍기, 0.7~1.3 돌아옴
    m.action("attack", 1.3, [
        (0.0, base),
        (0.5, {"root": {"l": (0, 0.06, 0.02)}, "body": {"r": (-26, 0, 0)}, "head": {"r": (-20, 0, 0)},
               "arm_l": {"r": (-168, -12, 0)}, "arm_r": {"r": (-168, 12, 0)},
               "leg_l": {"r": (-10, 0, 0)}, "leg_r": {"r": (12, 0, 0)}}),
        (0.62, {"root": {"l": (0, -0.08, -0.12)}, "body": {"r": (34, 0, 0)}, "head": {"r": (14, 0, 0)},
                "arm_l": {"r": (-62, -6, 0)}, "arm_r": {"r": (-62, 6, 0)},
                "leg_l": {"r": (-26, 0, 0)}, "leg_r": {"r": (18, 0, 0)}}),
        (0.7, {"root": {"l": (0, -0.08, -0.16)}, "body": {"r": (40, 0, 0)}, "head": {"r": (18, 0, 0)},
               "arm_l": {"r": (-48, -6, 0)}, "arm_r": {"r": (-48, 6, 0)},
               "leg_l": {"r": (-28, 0, 0)}, "leg_r": {"r": (20, 0, 0)}}),
        (0.95, {"root": {"l": (0, -0.06, -0.12)}, "body": {"r": (34, 0, 0)}, "head": {"r": (12, 0, 0)},
                "arm_l": {"r": (-42, -6, 0)}, "arm_r": {"r": (-44, 6, 0)},
                "leg_l": {"r": (-24, 0, 0)}, "leg_r": {"r": (18, 0, 0)}}),
        (1.3, base),
    ], linear=(0.5,))

    # hurt 0.3초: 뒤로 꺾임
    m.action("hurt", 0.3, [
        (0.0, base),
        (0.05, {"root": {"l": (0, 0.06, 0)}, "body": {"r": (-20, 4, 0)}, "head": {"r": (-25, 0, 8)},
                "arm_l": {"r": (14, -20, 0)}, "arm_r": {"r": (18, 22, 0)}}),
        (0.3, base),
    ])


PREVIEWS = [("idle", 0.0, ""), ("attack", 0.5, "_attack")]
CHECK = [("walk", 0.0), ("walk", 0.5), ("attack", 0.7), ("hurt", 0.05)]


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
