"""잠들지 않는 문지기(첫 보스, 키 3.2m).

0060 W3·0061(2026-11-05): 몸은 Meshy(AI) 모델 v2 "졸린 조약돌" — 나노바나나 그림(`tools/art/sources/ai/knellroot/warden_v2_*.png`) →
Meshy 7.1 Image to 3D(무료, CC BY 4.0) → `meshy_convert.py ... 3.2 2200 256`(줄이기 + 새 UV + 색 굽기) → 여기서 면 자리로 부위를 잘라
옛 뼈·동작에 붙인다. 핵(`glow_core`)·눈(`glow_eyes`)은 따로 잘라 스스로 빛나게. 옛 상자 문지기는 `make_blocks()`(되돌리기·비교용).

옛 설명(상자 문지기): 뿌리에 감긴 돌 갑옷의 수호자. 가슴의 큰 호박빛 핵(glow_core, 뼈 `core`에 따로 —
게임이 체력 반 아래에서 밝게 할 수 있다), 오른손에 돌판 몽둥이(뼈 `club`), 작은 머리에 눈(glow_eyes).

명세 뼈(root·body·head·arm_l·arm_r·leg_l·leg_r·core)에 무릎 뼈 shin_l·shin_r와 손목 뼈 club을 더했다 —
`wake`의 무릎 꿇은 자세와 걸음의 무릎 굽힘에 필요하다.

    blender --background --python tools/art/blender/gatekeeper.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "gatekeeper"

MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "warden_v2_low.glb")
# 부위 이름 → 뼈(옛 동작 표의 뼈 그대로 — 오른쪽 큰 주먹이 옛 몽둥이 뼈 `club`).
MESHY_PARTS = {"head_geo": "head", "body_geo": "body", "arm_l_geo": "arm_l", "arm_r_geo": "arm_r", "club_geo": "club",
               "leg_l_geo": "leg_l", "leg_r_geo": "leg_r", "shin_l_geo": "shin_l", "shin_r_geo": "shin_r",
               "glow_core": "core", "glow_eyes": "head"}
# 치수(키 3.2m, 앞 = -Y, 캐릭터 왼쪽 = +X) — 정면·옆 직교 렌더에 눈금을 그어 잰 것.
NECK_Z = 1.70  # 머리(뿌리 왕관 포함) 아래 끝
HIP_Z = 0.36  # 다리 위 끝
FOOT_Z = 0.14  # 발(정강이 뼈 — 무릎 꿇기·걸음에 따로 움직인다)
SHOULDER = (1.02, 0.12, 1.56)
WRIST = (1.24, 0.10, 0.80)  # 이 아래가 주먹(오른쪽 = `club`)
HAND = (1.30, 0.08, 0.45)
ARM_R = 0.40
EYE = (0.40, 2.21)  # 눈 가운데(±x, z) — 앞면만
EYE_R = (0.17, 0.11)
CORE = (0.0, 1.12)  # 배 핵 마름모 가운데(x, z) — 반 너비·반 높이
CORE_R = (0.28, 0.36)


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
        if y < -0.5 and ((ax - ex) / rx) ** 2 + ((z - ez) / rz) ** 2 <= 1.0:
            return "glow_eyes"
        return "head_geo"
    sx, sy, sz = SHOULDER
    hx, hy, hz = HAND
    p = Vector((ax, y, z))
    if ax > 0.9 and _seg_dist(p, (sx, sy, sz), (hx, hy, hz)) < ARM_R:
        side = "l" if x > 0 else "r"
        if side == "r" and z < WRIST[2]:
            return "club_geo"
        return f"arm_{side}_geo"
    if z < HIP_Z and ax < 0.85:
        side = "l" if x > 0 else "r"
        return f"shin_{side}_geo" if z < FOOT_Z else f"leg_{side}_geo"
    cx, cz = CORE
    rx, rz = CORE_R
    if y < -0.4 and abs(x - cx) / rx + abs(z - cz) / rz <= 1.0:
        return "glow_core"
    return "body_geo"


STONE_K = 0.72  # 돌(채도 낮은 칸)만 이만큼 어둡게 — Meshy 색이 밝은 회색이라 해 질 녘 성소에서 혼자 하얗게 떴다(0037: 바랜 잿빛 라벤더)
STONE_SAT = 0.18  # 이보다 채도가 낮으면 돌로 본다(호박색 눈·핵, 이끼, 보라 뿌리는 그대로)


def make():
    """0060 W3: Meshy 몸(졸린 조약돌), 뼈 이름·동작 표는 옛 문지기 그대로 — 뼈 자리만 새 비율에."""
    m = C.Model(NAME)
    sx, sy, sz = SHOULDER
    wx, wy, wz = WRIST
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0, HIP_Z), "root")
    m.bone("core", (CORE[0], -0.62, CORE[1]), "body")
    m.bone("head", (0, -0.05, NECK_Z), "body")
    m.bone("arm_l", (sx, sy, sz), "body")
    m.bone("arm_r", (-sx, sy, sz), "body")
    m.bone("club", (-wx, wy, wz), "arm_r")
    m.bone("leg_l", (0.40, 0.05, HIP_Z), "root")
    m.bone("leg_r", (-0.40, 0.05, HIP_Z), "root")
    m.bone("shin_l", (0.40, 0.05, FOOT_Z + 0.05), "leg_l")
    m.bone("shin_r", (-0.40, 0.05, FOOT_Z + 0.05), "leg_r")
    m.external(MESHY_LOW, classify, MESHY_PARTS)
    C.darken_grey(m.ext_img, STONE_K, STONE_SAT)
    m.build()
    acts(m)
    return m


def make_blocks():
    """옛 상자 문지기(0033~0037). 0060 W3부터 쓰지 않는다 — 비교·되돌리기용."""
    m = C.Model(NAME)
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0, 1.35), "root")
    m.bone("core", (0, -0.50, 2.08), "body")
    m.bone("head", (0, -0.05, 2.62), "body")
    m.bone("arm_l", (0.80, 0, 2.40), "body")
    m.bone("arm_r", (-0.80, 0, 2.40), "body")
    m.bone("club", (-0.94, -0.04, 1.02), "arm_r")
    m.bone("leg_l", (0.38, 0, 1.30), "root")
    m.bone("leg_r", (-0.38, 0, 1.30), "root")
    m.bone("shin_l", (0.40, 0, 0.76), "leg_l")
    m.bone("shin_r", (-0.40, 0, 0.76), "leg_r")

    g = m.geo("body_geo", "body")
    g.box((0, 0, 1.42), (0.95, 0.62, 0.36), "stone")  # 골반
    g.box((0, 0, 1.70), (0.82, 0.58, 0.30), "droot")  # 허리를 감은 뿌리
    g.box((0, 0.02, 2.13), (1.30, 0.86, 0.90), "wall", taper=(1.06, 1.0))  # 몸통(돌 쌓기)
    g.box((0, -0.44, 2.20), (0.95, 0.06, 0.62), "wall_light", drop=("+y",))  # 가슴판
    g.box((0, -0.49, 2.31), (0.48, 0.08, 0.10), "stone", drop=("+y",))  # 핵 둘레(위)
    g.box((0, -0.49, 1.85), (0.48, 0.08, 0.10), "stone", drop=("+y",))  # 핵 둘레(아래)
    g.box((0.21, -0.49, 2.08), (0.08, 0.08, 0.38), "stone", drop=("+y",))  # 핵 둘레(옆)
    g.box((-0.21, -0.49, 2.08), (0.08, 0.08, 0.38), "stone", drop=("+y",))
    for sx in (1, -1):
        g.box((0.80 * sx, 0, 2.50), (0.60, 0.76, 0.42), "wall", taper=(0.8, 0.85))  # 어깨 갑옷
    g.box((0.80, 0.02, 2.73), (0.40, 0.50, 0.05), "moss")  # 어깨 이끼
    g.box((-0.74, 0.08, 2.73), (0.28, 0.36, 0.05), "moss")
    g.box((0, 0.42, 2.34), (1.00, 0.30, 0.74), "bark", taper=(0.7, 0.8))  # 등의 뿌리 뭉치
    g.box((0.30, 0.52, 1.80), (0.12, 0.12, 0.60), "droot", rot=(10, -10, 0), taper=(0.4, 0.4), top=False)  # 등에서 늘어진 뿌리
    g.box((-0.25, 0.54, 1.86), (0.10, 0.10, 0.50), "root", rot=(14, 12, 0), taper=(0.4, 0.4), top=False)
    g.box((0.42, -0.47, 2.44), (0.55, 0.08, 0.10), "droot", rot=(0, -30, 0))  # 가슴을 비껴 감은 뿌리
    g.box((0.05, -0.43, 1.70), (1.00, 0.10, 0.12), "root", rot=(0, 8, 0))  # 배를 감은 뿌리
    g.box((-0.66, -0.36, 2.02), (0.12, 0.10, 0.86), "bark", rot=(0, -14, 0))  # 오른쪽 옆구리 뿌리
    g.box((0, -0.03, 2.61), (0.42, 0.38, 0.14), "stone")  # 목
    g = m.geo("glow_core", "core")
    g.box((0, -0.50, 2.08), (0.30, 0.12, 0.30), "amber", rot=(0, 45, 0), drop=("+y",))  # 핵(마름모)
    g.box((0, -0.565, 2.08), (0.13, 0.03, 0.13), "ember", rot=(0, 45, 0), drop=("+y",))  # 핵 속 불씨

    g = m.geo("head_geo", "head")
    g.box((0, -0.12, 2.80), (0.44, 0.44, 0.36), "lstone", taper=(0.85, 0.9))  # 머리
    g.box((0, -0.35, 2.89), (0.50, 0.08, 0.10), "stone")  # 이마
    g.box((0, -0.36, 2.67), (0.34, 0.06, 0.10), "stone")  # 턱
    for sx in (1, -1):
        g.box((0.19 * sx, -0.04, 3.04), (0.09, 0.09, 0.34), "droot", rot=(-30, 25 * sx, 0), taper=(0.3, 0.3))  # 뿌리 뿔
    g = m.geo("glow_eyes", "head")
    for sx in (1, -1):
        g.box((0.11 * sx, -0.345, 2.78), (0.11, 0.03, 0.05), "amber", drop=("+y",))

    for side, sx in (("l", 1), ("r", -1)):
        g = m.geo(f"arm_{side}_geo", f"arm_{side}")
        g.box((0.90 * sx, 0, 2.05), (0.42, 0.44, 0.62), "stone")  # 위팔
        g.box((0.92 * sx, 0, 1.74), (0.48, 0.50, 0.10), "droot")  # 팔꿈치 뿌리
        g.box((0.94 * sx, -0.02, 1.42), (0.50, 0.50, 0.60), "wall")  # 아래팔
        fist = (0.52, 0.52, 0.34) if side == "l" else (0.46, 0.46, 0.30)
        g.box((0.94 * sx, -0.04, 1.02), fist, "stone")  # 주먹
        if side == "l":
            g.box((1.12, -0.10, 1.25), (0.08, 0.08, 0.50), "root", rot=(8, 10, 0), taper=(0.4, 0.4), top=False)  # 늘어진 뿌리

    g = m.geo("club_geo", "club")
    cp = (-0.94, -0.04, 1.02)  # 손목에서 조금 바깥으로 벌어진 돌판
    g.box((-0.98, -0.06, 0.48), (0.66, 0.26, 1.00), "wall_light", pivot=cp, rot=(0, 6, 0), drop=("+z",))  # 돌판 몽둥이
    g.box((-0.98, -0.06, 0.86), (0.72, 0.32, 0.08), "droot", pivot=cp, rot=(0, 6, 0))  # 묶은 뿌리
    g.box((-0.98, -0.06, 0.34), (0.72, 0.32, 0.08), "droot", pivot=cp, rot=(0, 6, 0))
    g.box((-0.78, -0.06, 0.06), (0.26, 0.28, 0.14), "stone", pivot=cp, rot=(0, 6, 0))  # 깨진 끝
    g.box((-1.00, -0.21, 0.60), (0.40, 0.04, 0.30), "moss", pivot=cp, rot=(0, 6, 0))  # 앞면 이끼

    for side, sx in (("l", 1), ("r", -1)):
        g = m.geo(f"leg_{side}_geo", f"leg_{side}")
        g.box((0.40 * sx, 0, 1.02), (0.48, 0.52, 0.58), "stone")  # 넓적다리
        g = m.geo(f"shin_{side}_geo", f"shin_{side}")
        g.box((0.42 * sx, -0.24, 0.76), (0.36, 0.10, 0.24), "wall_light")  # 무릎판
        g.box((0.42 * sx, -0.02, 0.45), (0.52, 0.58, 0.56), "wall_light")  # 정강이
        g.box((0.42 * sx, -0.02, 0.56), (0.56, 0.62, 0.08), "droot")  # 정강이 뿌리
        g.box((0.42 * sx, -0.10, 0.10), (0.60, 0.80, 0.20), "stone", drop=("-z",))  # 발

    m.build()
    acts(m)
    return m


def legs(ll=0, sl=0, lr=0, sr=0, yl=0, yr=0):
    return {"leg_l": {"r": (ll, yl, 0)}, "shin_l": {"r": (sl, 0, 0)},
            "leg_r": {"r": (lr, yr, 0)}, "shin_r": {"r": (sr, 0, 0)}}


def k(*parts, **bones):
    out = {}
    for p in parts:
        out.update(p)
    out.update(bones)
    return out


REST = k(legs(), body={"r": (4, 0, 0)}, head={"r": (-4, 0, 0)}, arm_l={"r": (-6, -4, 0)}, arm_r={"r": (-6, 4, 0)},
         club={"r": (-16, 12, 0)}, core={"s": 1.0})


def acts(m):
    # idle 2.0초 고리: 무거운 숨. 핵이 숨에 맞춰 조금 부푼다.
    m.action("idle", 2.0, [
        (0.0, REST),
        (1.0, k(legs(-3, 6, -3, 6), root={"l": (0, 0, -0.04)}, body={"r": (7, 0, 0)}, head={"r": (-1, 0, 0)},
                arm_l={"r": (-4, -6, 0)}, arm_r={"r": (-4, 6, 0)}, club={"r": (-19, 12, 0)}, core={"s": 1.1})),
    ], loop=True)

    # walk 1.4초 고리: 쿵쿵 걷기, 몸이 디딘 쪽으로 기울고 몽둥이는 끌린다
    def step(s):
        return k(legs(-22 * s if s > 0 else 18, 4 if s > 0 else 22, 18 if s > 0 else -22, 22 if s > 0 else 4),
                 root={"l": (0, 0, -0.04)}, body={"r": (6, 5 * s, 6 * s)}, head={"r": (-6, 0, -5 * s)},
                 arm_l={"r": (12 * s, -6, 0)}, arm_r={"r": (-10 * s, 6, 0)}, club={"r": (6 * s, 0, 0)}, core={"s": 1.0})

    def pss(s):
        return k(legs(0 if s > 0 else -6, 0 if s > 0 else 40, -6 if s > 0 else 0, 40 if s > 0 else 0),
                 root={"l": (0, 0, 0.05)}, body={"r": (5, 0, 0)}, head={"r": (-5, 0, 0)},
                 arm_l={"r": (0, -6, 0)}, arm_r={"r": (0, 6, 0)}, club={"r": (0, 0, 0)}, core={"s": 1.0})
    m.action("walk", 1.4, [(0.0, step(1)), (0.35, pss(1)), (0.7, step(-1)), (1.05, pss(-1))], loop=True)

    # slam 1.5초: 0~0.9 몽둥이를 높이, 0.9~1.1 내려찍기, 1.1~1.5 돌아옴
    # 0073 과장: 정점에서 까치발로 크게 젖혀 늘어나고(왼팔은 반대로 뻗어 균형), 몽둥이를 더 뒤로 젖힌다. 내리칠 때 납작.
    up = k(legs(-12, 6, 10, 6), root={"l": (0, 0.08, 0.08)}, body={"r": (-24, 0, -22), "s": (0.95, 0.95, 1.1)},
           head={"r": (-18, 0, 16)},
           arm_r={"r": (-176, 14, 0)}, club={"r": (-40, 0, 0)}, arm_l={"r": (-40, -36, 0)}, core={"s": 1.2})
    hit = k(legs(-38, 40, 8, 44), root={"l": (0, -0.12, -0.32)}, body={"r": (34, 0, 8), "s": (1.1, 1.1, 0.86)},
            head={"r": (-12, 0, -6)},
            arm_r={"r": (-58, 6, 6)}, club={"r": (-44, 0, 0)}, arm_l={"r": (8, -24, 0)}, core={"s": 1.0})
    m.action("slam", 1.5, [
        (0.0, REST),
        (0.18, k(legs(-22, 24, -18, 24), root={"l": (0, -0.02, -0.12)}, body={"r": (14, 0, -6), "s": (1.06, 1.06, 0.9)},
                 head={"r": (4, 0, 4)}, arm_r={"r": (24, 8, 0)}, club={"r": (-10, 0, 0)}, arm_l={"r": (12, -10, 0)},
                 core={"s": 0.95})),
        (0.45, k(up, body={"r": (-12, 0, -14), "s": (0.98, 0.98, 1.04)}, arm_r={"r": (-130, 10, 0)},
                 club={"r": (-30, 0, 0)}, root={"l": (0, 0.04, 0.03)})),
        (0.9, up),
        (1.0, k(hit, root={"l": (0, -0.10, -0.26)}, body={"r": (28, 0, 6), "s": (1.1, 1.1, 0.86)}, arm_r={"r": (-70, 6, 6)})),
        (1.1, hit),
        (1.2, k(hit, root={"l": (0, -0.12, -0.30)})),
        (1.5, REST),
    ], linear=(0.9, 1.0))

    # charge_windup 0.8초: 왼어깨를 앞으로 낮추고 웅크린다(몸을 오른쪽으로 비틀어 왼어깨가 앞)
    crouch = k(legs(-30, 34, 26, 30), root={"l": (0, 0, -0.24)}, body={"r": (24, 0, -34)}, head={"r": (-14, 0, 26)},
               arm_l={"r": (-30, 0, -40)}, arm_r={"r": (8, 10, 0)}, club={"r": (-6, 0, 0)}, core={"s": 1.0})
    # 0073 과장: 한 번 몸을 크게 일으켜(숨 들이쉬기) 늘어났다가 더 깊이 웅크려 납작하게 — 용수철.
    m.action("charge_windup", 0.8, [
        (0.0, REST),
        (0.3, crouch),
        (0.55, k(crouch, root={"l": (0, 0.06, -0.10)}, body={"r": (6, 0, -26), "s": (0.95, 0.95, 1.1)},
                 head={"r": (-22, 0, 22)}, core={"s": 1.2})),
        (0.8, k(crouch, root={"l": (0, -0.04, -0.32)}, body={"r": (32, 0, -40), "s": (1.1, 1.1, 0.86)}, core={"s": 1.1})),
    ])

    # charge 0.5초 고리: 어깨를 앞세운 돌진(큰 걸음)
    lunge = dict(body={"r": (28, 0, -36)}, head={"r": (-16, 0, 26)}, arm_l={"r": (-35, 0, -45)},
                 arm_r={"r": (6, 10, 0)}, club={"r": (-6, 0, 0)}, core={"s": 1.0})
    m.action("charge", 0.5, [
        (0.0, k(legs(-38, 12, 30, 36), root={"l": (0, 0, -0.20)}, **lunge)),
        (0.125, k(legs(-8, 20, 0, 50), root={"l": (0, 0, -0.10)}, **lunge)),
        (0.25, k(legs(30, 36, -38, 12), root={"l": (0, 0, -0.20)}, **lunge)),
        (0.375, k(legs(0, 50, -8, 20), root={"l": (0, 0, -0.10)}, **lunge)),
    ], loop=True)

    # wave 1.6초: 0~1.0 두 팔을 들고, 1.0~1.2 주먹으로 땅을 친다(뿌리 파동), 1.2~1.6 돌아옴
    # 0073 과장: 들기 전에 움츠리고, 정점에서 까치발로 젖혀 두 팔을 활짝(V) — 핵이 부푼다. 땅 칠 때 납작.
    raise_ = k(legs(-6, 4, -6, 4), root={"l": (0, 0.08, 0.12)}, body={"r": (-24, 0, 0), "s": (0.94, 0.94, 1.12)},
               head={"r": (-26, 0, 0)},
               arm_l={"r": (-172, -30, 0)}, arm_r={"r": (-172, 30, 0)}, club={"r": (-14, 0, 0)}, core={"s": 1.35})
    pound = k(legs(-40, 64, -40, 64), root={"l": (0, 0.10, -0.36)}, body={"r": (50, 0, 0), "s": (1.12, 1.12, 0.84)},
              head={"r": (-30, 0, 0)},
              arm_l={"r": (-62, -6, 0)}, arm_r={"r": (-62, 6, 0)}, club={"r": (-30, 0, 0)}, core={"s": 1.0})
    m.action("wave", 1.6, [
        (0.0, REST),
        (0.22, k(legs(-24, 30, -24, 30), root={"l": (0, 0.0, -0.14)}, body={"r": (16, 0, 0), "s": (1.07, 1.07, 0.88)},
                 head={"r": (6, 0, 0)}, arm_l={"r": (20, -18, 0)}, arm_r={"r": (20, 18, 0)}, club={"r": (-6, 0, 0)},
                 core={"s": 0.9})),
        (0.6, k(raise_, arm_l={"r": (-128, -22, 0)}, arm_r={"r": (-128, 22, 0)},
                body={"r": (-12, 0, 0), "s": (0.97, 0.97, 1.06)}, root={"l": (0, 0.05, 0.06)}, core={"s": 1.2})),
        (1.0, raise_),
        (1.12, k(pound, root={"l": (0, 0.06, -0.30)}, body={"r": (36, 0, 0), "s": (1.1, 1.1, 0.86)},
                 arm_l={"r": (-90, -8, 0)}, arm_r={"r": (-90, 8, 0)})),
        (1.2, pound),
        (1.32, k(pound, root={"l": (0, 0.10, -0.34)})),
        (1.6, REST),
    ], linear=(1.0, 1.12))

    # hurt 0.4초: 뒤로 젖히고 핵이 번쩍 부푼다
    m.action("hurt", 0.4, [
        (0.0, REST),
        (0.08, k(legs(-8, 6, 10, 4), root={"l": (0, 0.10, -0.03)}, body={"r": (-14, 4, 6)}, head={"r": (-24, 0, 8)},
                 arm_l={"r": (8, -24, 0)}, arm_r={"r": (10, 22, 0)}, club={"r": (6, 0, 0)}, core={"s": 1.25})),
        (0.4, REST),
    ])

    # wake 2.0초: 무릎 꿇고 잠든 자세(핵이 작고 어둡다) → 일어서기. 등장 연출용.
    sleep = k(legs(-88, 88, -14, 84), root={"l": (0, 0.05, -0.50)}, body={"r": (40, 0, 0)}, head={"r": (32, 0, 0)},
              arm_l={"r": (-24, -6, 0)}, arm_r={"r": (-10, 6, 0)}, club={"r": (-30, 0, 0)}, core={"s": 0.6})
    m.action("wake", 2.0, [
        (0.0, sleep),
        (0.5, k(sleep, head={"r": (22, 0, 0)}, core={"s": 0.9})),
        (1.2, k(legs(-46, 52, -22, 46), root={"l": (0, 0.03, -0.26)}, body={"r": (20, 0, 0)}, head={"r": (2, 0, 0)},
                arm_l={"r": (-12, -10, 0)}, arm_r={"r": (-10, 10, 0)}, club={"r": (-10, 0, 0)}, core={"s": 1.1})),
        (1.6, k(legs(-6, 8, -4, 8), root={"l": (0, 0, -0.04)}, body={"r": (6, 0, 0)}, head={"r": (-12, 0, 0)},
                arm_l={"r": (-8, -22, 0)}, arm_r={"r": (-8, 22, 0)}, club={"r": (0, 0, 0)}, core={"s": 1.2})),
        (2.0, REST),
    ])


PREVIEWS = [("idle", 0.0, ""), ("slam", 0.9, "_slam")]
CHECK = [("slam", 0.18), ("slam", 0.9), ("wave", 0.22), ("charge_windup", 0.55), ("slam", 1.1), ("wave", 1.0), ("wave", 1.2), ("charge", 0.0), ("charge_windup", 0.8), ("walk", 0.0), ("walk", 0.35),
         ("wake", 0.0), ("wake", 1.2), ("hurt", 0.08)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, check_dir=None):
    m = make()
    path = m.export(models_dir)
    for act, t, suffix in PREVIEWS:
        m.preview(os.path.join(preview_dir, f"{NAME}{suffix}.png"), act, t)
    if check_dir:
        for act, t in CHECK:
            m.preview(os.path.join(check_dir, f"{NAME}_{act}_{int(t * 100):03d}.png"), act, t, yaw=-75, elev=12)
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
