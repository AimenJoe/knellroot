"""가시짐승(돌진하는 적, 네 발, 높이 약 1.1m·길이 약 1.6m): 짙은 뿌리 몸에 등을 따라 뼈빛 가시, 호박빛 눈(glow_eyes).

    blender --background --python tools/art/blender/thorn_beast.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "thorn_beast"
LEGS = (("fl", 0.20, -0.26), ("fr", -0.20, -0.26), ("bl", 0.19, 0.34), ("br", -0.19, 0.34))


def make():
    m = C.Model(NAME)
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0.02, 0.72), "root")
    m.bone("head", (0, -0.42, 0.86), "body")
    for n, x, y in LEGS:
        m.bone(f"leg_{n}", (x, y, 0.64), "root")
    m.bone("tail", (0, 0.50, 0.86), "body")

    g = m.geo("body_geo", "body")
    g.box((0, 0.05, 0.74), (0.50, 0.86, 0.42), "droot", taper=(0.9, 1.0))  # 몸통
    g.box((0, -0.24, 0.78), (0.60, 0.42, 0.54), "root", taper=(0.82, 0.9))  # 가슴·어깨
    g.box((0, 0.34, 0.73), (0.50, 0.36, 0.44), "root", taper=(0.9, 0.9))  # 엉덩이
    g.box((0, -0.40, 0.95), (0.40, 0.22, 0.28), "droot", rot=(-20, 0, 0))  # 목 갈기
    for y, h in ((-0.36, 0.30), (-0.15, 0.36), (0.06, 0.32), (0.26, 0.26), (0.44, 0.20)):
        z = 1.00 if y < 0 else 0.93
        g.box((0, y + 0.03, z + h * 0.35), (0.12, 0.10, h), "bone", rot=(-28, 0, 0), taper=(0.15, 0.2))  # 등 가시

    g = m.geo("head_geo", "head")
    g.box((0, -0.60, 0.80), (0.36, 0.36, 0.32), "root")  # 머리
    g.box((0, -0.84, 0.72), (0.24, 0.22, 0.18), "droot", taper=(0.9, 1.0))  # 주둥이
    g.box((0, -0.80, 0.83), (0.20, 0.20, 0.04), "bone")  # 콧등 뼈
    for sx in (1, -1):
        g.box((0.11 * sx, -0.90, 0.68), (0.045, 0.045, 0.15), "bone", rot=(35, 0, 0), taper=(0.3, 0.3))  # 엄니
        g.box((0.13 * sx, -0.52, 1.00), (0.06, 0.08, 0.20), "bone", rot=(-45, 15 * sx, 0), taper=(0.2, 0.3))  # 귀 가시
    g = m.geo("glow_eyes", "head")
    for sx in (1, -1):
        g.box((0.10 * sx, -0.785, 0.87), (0.065, 0.03, 0.045), "amber", drop=("+y",))

    for n, x, y in LEGS:
        g = m.geo(f"leg_{n}_geo", f"leg_{n}")
        back = n[0] == "b"
        g.box((x, y, 0.43), (0.16, 0.22 if back else 0.18, 0.38), "droot", taper=(1.1, 1.1))  # 위다리
        g.box((x, y - 0.03, 0.13), (0.15, 0.21, 0.26), "root", drop=("-z",))  # 아래다리·발

    g = m.geo("tail_geo", "tail")
    g.box((0, 0.68, 0.92), (0.10, 0.38, 0.10), "droot", rot=(-18, 0, 0), pivot=(0, 0.50, 0.86))  # 꼬리
    g.box((0, 0.88, 0.99), (0.06, 0.14, 0.06), "bone", rot=(-18, 0, 0), pivot=(0, 0.50, 0.86), taper=(0.4, 0.4))  # 꼬리 가시

    m.build()
    acts(m)
    return m


def legs(fl, fr, bl, br):
    return {"leg_fl": {"r": (fl, 0, 0)}, "leg_fr": {"r": (fr, 0, 0)},
            "leg_bl": {"r": (bl, 0, 0)}, "leg_br": {"r": (br, 0, 0)}}


def k(*parts, **bones):
    out = {}
    for p in parts:
        out.update(p)
    for b, v in bones.items():
        out[b] = v
    return out


def acts(m):
    # idle 1.4초 고리: 숨(몸이 오르내림), 머리 조금, 꼬리 흔들
    m.action("idle", 1.4, [
        (0.0, k(body={"r": (0, 0, 0)}, head={"r": (2, 0, -4)}, tail={"r": (0, 0, 10)})),
        (0.7, k(root={"l": (0, 0, -0.02)}, body={"r": (2, 0, 0)}, head={"r": (6, 0, 4)}, tail={"r": (4, 0, -10)})),
    ], loop=True)

    # run 0.5초 고리: 질주(앞다리 뻗음 ↔ 모음)
    m.action("run", 0.5, [
        (0.0, k(legs(-42, -32, 38, 30), root={"l": (0, 0, -0.07)}, body={"r": (-6, 0, 0)}, head={"r": (6, 0, 0)}, tail={"r": (-6, 0, 0)})),
        (0.125, k(legs(0, 6, 0, -6), root={"l": (0, 0, 0.05)}, body={"r": (0, 0, 0)}, head={"r": (2, 0, 0)}, tail={"r": (6, 0, 0)})),
        (0.25, k(legs(30, 22, -34, -26), root={"l": (0, 0, -0.07)}, body={"r": (7, 0, 0)}, head={"r": (-4, 0, 0)}, tail={"r": (10, 0, 0)})),
        (0.375, k(legs(-6, 0, 6, 0), root={"l": (0, 0, 0.04)}, body={"r": (0, 0, 0)}, head={"r": (2, 0, 0)}, tail={"r": (0, 0, 0)})),
    ], loop=True)

    # charge_windup 0.6초: 머리를 낮추고 앞발로 땅을 긁는다(두 번)
    low = dict(root={"l": (0, 0.04, -0.09)}, body={"r": (10, 0, 0)}, head={"r": (28, 0, 0)}, tail={"r": (30, 0, 0)})
    m.action("charge_windup", 0.6, [
        (0.0, k(legs(0, 0, 0, 0))),
        (0.12, k(legs(-14, -22, -18, -18), **low)),
        (0.24, k(legs(-14, 34, -18, -18), **low)),
        (0.36, k(legs(-14, -16, -18, -18), **low)),
        (0.48, k(legs(-14, 34, -18, -18), **low)),
        (0.6, k(legs(-16, -10, -20, -20), **dict(low, head={"r": (32, 0, 0)}))),
    ])

    # charge 0.4초 고리: 몸을 길게 뻗은 돌진 자세
    lunge = dict(root={"l": (0, 0, -0.17)}, body={"r": (6, 0, 0)}, head={"r": (20, 0, 0)}, tail={"r": (-4, 0, 0)})
    m.action("charge", 0.4, [
        (0.0, k(legs(-58, -44, 56, 42), **lunge)),
        (0.1, k(legs(-30, -20, 28, 18), **dict(lunge, root={"l": (0, 0, -0.06)}))),
        (0.2, k(legs(-44, -58, 42, 56), **lunge)),
        (0.3, k(legs(-20, -30, 18, 28), **dict(lunge, root={"l": (0, 0, -0.06)}))),
    ], loop=True)

    # recover 0.7초: 미끄러져 서고(앞발 버팀) 머리를 털고 쉬는 자세로
    m.action("recover", 0.7, [
        (0.0, k(legs(-44, -58, 42, 56), root={"l": (0, 0, -0.17)}, body={"r": (6, 0, 0)}, head={"r": (20, 0, 0)})),
        (0.12, k(legs(-38, -40, -8, -12), root={"l": (0, 0, -0.08)}, body={"r": (-12, 0, 0)}, head={"r": (-14, 0, 0)}, tail={"r": (20, 0, 0)})),
        (0.3, k(legs(-12, -12, -4, -4), body={"r": (-4, 0, 0)}, head={"r": (4, 0, 18)}, tail={"r": (10, 0, -15)})),
        (0.45, k(legs(-4, -4, 0, 0), body={"r": (-2, 0, 0)}, head={"r": (4, 0, -14)}, tail={"r": (6, 0, 12)})),
        (0.7, k(legs(0, 0, 0, 0), body={"r": (0, 0, 0)}, head={"r": (2, 0, -4)}, tail={"r": (0, 0, 10)})),
    ])

    # hurt 0.3초: 움찔하며 옆으로 비틀림
    m.action("hurt", 0.3, [
        (0.0, k(legs(0, 0, 0, 0), head={"r": (2, 0, -4)}, tail={"r": (0, 0, 10)})),
        (0.05, k(legs(10, 10, -12, -12), root={"l": (0, 0.06, 0.02)}, body={"r": (-10, 8, 0)},
                 head={"r": (-26, 0, 12)}, tail={"r": (24, 0, 0)})),
        (0.3, k(legs(0, 0, 0, 0), head={"r": (2, 0, -4)}, tail={"r": (0, 0, 10)})),
    ])


PREVIEWS = [("idle", 0.0, ""), ("charge", 0.0, "_charge")]
CHECK = [("run", 0.0), ("run", 0.25), ("charge_windup", 0.24), ("recover", 0.12), ("hurt", 0.05)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, check_dir=None):
    m = make()
    path = m.export(models_dir)
    for act, t, suffix in PREVIEWS:
        m.preview(os.path.join(preview_dir, f"{NAME}{suffix}.png"), act, t, yaw=-50)
    if check_dir:
        for act, t in CHECK:
            m.preview(os.path.join(check_dir, f"{NAME}_{act}_{int(t * 100):03d}.png"), act, t, yaw=-70)
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
