"""고요한 사슴(성소의 필드 보스, 0270 D).

Meshy 몸 — 뿌리에 먹힌 큰 사슴: 바랜 잿빛 라벤더 몸, 뿔은 넓게 벌어진 각진 보라 뿌리 왕관(실루엣의 반), 목에는 뿌리 밧줄에 매인
금 간 청동 종(울리지 않는다), 낮게 둔 호박빛 눈(`glow_eyes`)·옆구리 금 사이의 호박빛 핵(`glow_core`), 발목에서 땅으로 내린 잔뿌리.
원본 `tools/art/sources/meshy/hushed_stag_v1.glb`(나노바나나 그림 `tools/art/sources/ai/knellroot/hushed_stag_v1_front.png` →
Meshy 6 API, 30 크레딧) → `meshy_convert.py ... 3.0 2400 256`(1분 44초 — 정점 2,375·삼각형 1,828, 뿔이 삼각형의 반).
줄인 몸은 뿔 끝까지 3.0m·몸 길이 2.1m — 문지기들보다 크게 1.2배로 쓴다(뿔 끝 3.6·머리 2.4·길이 2.5·뿔 폭 3.4).
가운데가 머리 밑이라(변환이 위 45% = 뿔로 가운데를 잡는다) 몸 가운데로 `SHIFT`만큼 민다.

부위는 면 가운데 자리와 색으로(치수는 정면·옆 눈금 그림 — 앞 = -Y): 뿔 = 머리 위의 보라, 머리 = 목줄(비스듬한 줄) 위, 종 = 가슴 앞의
청동 덩이, 다리 넷 = 배 아래, 나머지 몸(목줄·꼬리 포함).

뼈: root → body(**엉덩이**에 — 몸을 들면 앞이 들린다: 뒷발로 일어서기) → head → antlers · bell · leg_fl · leg_fr, 뒷다리 둘은 root에
(몸을 들어도 땅을 짚고 있다).

동작: idle 2.4↻ · walk 1.2↻ · slam 1.5(뿔 쓸기 — 감기 끝 0.9 · 맞음 1.0, `render::STRIKE_KEYS`) · charge_windup 0.9(머리를 낮추고
앞발로 땅을 긁는다) · charge 0.5↻ · rear 1.6(뒷발로 일어서 — 정점 0.9 · 내려 딛기 1.0) · recover 0.9 · hurt 0.4 ·
wake 2.4(0 = 다리를 접고 엎드려 머리를 내린 자세 — 기다리는 모습, 끝 = 선다).

    blender --background --python tools/art/blender/hushed_stag.py
"""
import os
import sys

from mathutils import Matrix

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "hushed_stag"

MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "hushed_stag_v1_low.glb")
MESHY_PARTS = {"body_geo": "body", "glow_core": "body", "head_geo": "head", "glow_eyes": "head", "antler_geo": "antlers",
               "bell_geo": "bell", "leg_fl_geo": "leg_fl", "leg_fr_geo": "leg_fr", "leg_bl_geo": "leg_bl", "leg_br_geo": "leg_br"}
SCALE = 1.2
SHIFT = 0.55  # 줄인 몸의 가운데(y 0)는 목 밑 — 몸 가운데(주둥이 -0.67 … 엉덩이 1.45)로
# 치수(줄이기 전 자리, 앞 = -Y, 캐릭터 왼쪽 = +X).
ANTLER_Z = 2.03  # 이보다 위는 모두 뿔(머리 꼭대기·귀 끝 ≈ 2.0)
NECK = (-0.35, 1.40, 0.727)  # 목줄 위쪽 줄: z = 1.40 + 0.727 (y + 0.35) — 이 줄 위(그리고 y < NECK_BACK)가 머리
NECK_BACK = 0.3
BELL = (-0.36, 0.80, 1.44, 0.3)  # 종: y < · z 아래 · z 위 · |x| <
LEG_Z = 0.58  # 이보다 아래는 다리(배 ≈ 0.5~0.6은 앞·뒷다리 사이라 y로 거른다)
FRONT_Y, BACK_Y = (0.55, 1.0)  # 앞다리는 이 y보다 앞, 뒷다리는 이 y보다 뒤
HIP = (1.25, 0.66)  # 엉덩이 뼈 (y, z) — 뒷다리 위 끝 높이: 더 높으면 몸을 들 때 뒷다리와의 이음이 벌어진다
GREY_K = 0.78  # 흰 잿빛 몸을 어둡게(문지기 0.75 꼴 — 몸 전체가 잿빛이라 0.6은 검게 묻힌다)
GREY_SAT = 0.2

EYES, CORE, BRONZE = [], [], []


def _purple(h, s, v):
    return 0.62 < h < 0.95 and s > 0.3


def _colored():
    """호박빛(눈·핵)·청동(종) 면."""
    for c, (h, s, v) in C.face_colors(MESHY_LOW):
        if 0.04 < h < 0.17 and s > 0.45 and v > 0.5:
            if c[2] > 1.55:
                EYES.append(c)
            elif c[1] < -0.3 and abs(c[0]) < 0.3:
                BRONZE.append(c)
            else:
                CORE.append(c)


PURPLE = []


def classify(c):
    """면 가운데 자리(가져온 그대로 — 밀고 키우기 전) → 부위."""
    x, y, z = c.x, c.y, c.z
    p = (x, y, z)
    if C.near_any(EYES, p):
        return "glow_eyes"
    if C.near_any(CORE, p):
        return "glow_core"
    purple = C.near_any(PURPLE, p)
    # 뿔의 아래 가지(눈썹 가지)는 머리 옆으로 내려온다(z 1.6까지) — 목줄(|x| < 0.4)과는 옆 자리로 가른다: 몸에 남기면 고개를 돌릴 때 허공에 뜬다.
    if z > ANTLER_Z or (purple and z > 1.9 and abs(x) > 0.15) or (purple and z > 1.6 and abs(x) > 0.42):
        return "antler_geo"
    by, bz0, bz1, bx = BELL
    if C.near_any(BRONZE, p) or (y < by and bz0 < z < bz1 and abs(x) < bx and not purple):
        return "bell_geo"
    if z < LEG_Z and (y < FRONT_Y or y > BACK_Y):
        return f"leg_{'f' if y < FRONT_Y else 'b'}{'l' if x > 0 else 'r'}_geo"
    y0, z0, k = NECK
    if not purple and y < NECK_BACK and z > z0 + k * (y - y0):
        return "head_geo"
    return "body_geo"


def make():
    EYES.clear(), CORE.clear(), BRONZE.clear(), PURPLE.clear()
    _colored()
    PURPLE.extend(c for c, hsv in C.face_colors(MESHY_LOW) if _purple(*hsv))
    print(f"[{NAME}] eyes {len(EYES)} core {len(CORE)} bronze {len(BRONZE)} purple {len(PURPLE)}")
    m = C.Model(NAME)
    s = SCALE

    def at(x, y, z):
        return (x * s, (y - SHIFT) * s, z * s)

    m.bone("root", (0, 0, 0))
    m.bone("body", at(0, *HIP), "root")
    m.bone("head", at(0, -0.05, 1.52), "body")
    m.bone("antlers", at(0, 0.0, 2.0), "head")
    m.bone("bell", at(0, -0.4, 1.42), "body")
    for n, x, y, parent in (("leg_fl", 0.25, 0.3, "body"), ("leg_fr", -0.25, 0.3, "body"),
                            ("leg_bl", 0.27, 1.27, "root"), ("leg_br", -0.27, 1.27, "root")):
        m.bone(n, at(x, y, LEG_Z + 0.06), parent)
    objs = m.external(MESHY_LOW, classify, MESHY_PARTS)
    for name in MESHY_PARTS:
        print(f"[{NAME}] {name}: {len(objs[name].data.polygons) if name in objs else 0} faces")
    for o in objs.values():
        o.data.transform(Matrix.Translation((0, -SHIFT, 0)))
        o.data.transform(Matrix.Scale(s, 4))
    # 각진 것은 뿌리만(0037): 뿔(뿌리 왕관)은 면마다 빛을 받게 — 몸은 매끈하게 둔다.
    for poly in objs["antler_geo"].data.polygons:
        poly.use_smooth = False
    C.darken_grey(m.ext_img, GREY_K, GREY_SAT)
    m.build()
    acts(m)
    return m


def k(*parts, **bones):
    out = {}
    for p in parts:
        out.update(p)
    out.update(bones)
    return out


def legs(fl, fr, bl, br):
    """다리 넷의 앞뒤 각(− = 앞으로)."""
    return {"leg_fl": {"r": (fl, 0, 0)}, "leg_fr": {"r": (fr, 0, 0)}, "leg_bl": {"r": (bl, 0, 0)}, "leg_br": {"r": (br, 0, 0)}}


def acts(m):
    s = SCALE

    def up(z, y=0.0):
        return {"l": (0, y * s, z * s)}

    # 숨: 가슴이 조금 부풀고 머리가 느리게 끄덕인다. 종은 흔들리기만(울리지 않는다).
    m.action("idle", 2.4, [
        (0.0, k(body={"s": 1.0}, head={"r": (0, 0, 0)}, bell={"r": (0, 0, 0)})),
        (1.2, k(body={"r": (-1.2, 0, 0), "s": (1.02, 1.0, 1.015)}, head={"r": (4, 0, 3)}, bell={"r": (5, 0, 0)},
                antlers={"r": (0, 0, 0)})),
    ], loop=True)
    # 걸음: 맞은편 다리끼리, 몸은 좌우로 조금 굴고 머리가 끄덕인다(무거운 몸 — 솟는 폭은 작게).
    m.action("walk", 1.2, [
        (0.0, k(legs(-24, 20, 18, -22), body={"r": (0, 2.5, 0)}, head={"r": (3, 0, -3)}, bell={"r": (8, 0, 0)})),
        (0.3, k(legs(-2, -2, -2, -2), root=up(0.05), body={"r": (-1.5, 0, 0)}, head={"r": (-2, 0, 0)}, bell={"r": (-6, 0, 0)})),
        (0.6, k(legs(20, -24, -22, 18), body={"r": (0, -2.5, 0)}, head={"r": (3, 0, 3)}, bell={"r": (8, 0, 0)})),
        (0.9, k(legs(-2, -2, -2, -2), root=up(0.05), body={"r": (-1.5, 0, 0)}, head={"r": (-2, 0, 0)}, bell={"r": (-6, 0, 0)})),
    ], loop=True)
    # 뿔 쓸기: 머리를 낮추며 한쪽으로 감았다가(정점 0.9) 반대쪽으로 크게 쓴다(맞음 1.0). 몸은 엉덩이 둘레로 따라 돈다.
    m.action("slam", 1.5, [
        (0.0, k()),
        (0.5, k(root=up(-0.06, 0.08), body={"r": (5, 0, 14)}, head={"r": (24, 0, 26)}, antlers={"s": 1.04}, bell={"r": (10, 0, 0)},
                **legs(-10, 6, 0, 0))),
        (0.9, k(root=up(-0.08, 0.12), body={"r": (7, 0, 20)}, head={"r": (30, 0, 36)}, antlers={"s": 1.08}, bell={"r": (14, 0, 0)},
                **legs(-14, 8, 0, 0))),
        (1.0, k(root=up(-0.08, -0.22), body={"r": (6, 0, -22)}, head={"r": (20, 0, -44)}, antlers={"s": 1.08}, bell={"r": (-20, 0, 0)},
                **legs(10, -16, 0, 0))),
        (1.2, k(root=up(-0.05, -0.16), body={"r": (4, 0, -18)}, head={"r": (12, 0, -34)}, bell={"r": (12, 0, 0)},
                **legs(8, -12, 0, 0))),
        (1.5, k()),
    ], linear=(0.9,))
    # 돌진 예고: 물러앉으며 머리를 푹 숙여 뿔을 앞으로 겨누고, 앞발로 땅을 두 번 긁는다.
    aimed = dict(root=up(-0.1, 0.16), body={"r": (8, 0, 0), "s": (1.04, 0.96, 0.95)}, head={"r": (42, 0, 0)}, antlers={"s": 1.06},
                 bell={"r": (-18, 0, 0)})
    m.action("charge_windup", 0.9, [
        (0.0, k()),
        (0.25, k(legs(-46, 0, 0, 0), root=up(-0.06, 0.1), body={"r": (5, 0, 0)}, head={"r": (26, 0, 0)}, bell={"r": (-10, 0, 0)})),
        (0.4, k(legs(14, 0, 0, 0), **aimed)),
        (0.55, k(legs(-46, 0, 0, 0), **aimed)),
        (0.7, k(legs(14, 0, 0, 0), **aimed)),
        (0.9, k(legs(-8, -8, -14, -14), **dict(aimed, root=up(-0.14, 0.2)))),
    ])
    # 돌진: 머리를 숙인 채 달린다(앞다리 뻗음 ↔ 모음).
    run = dict(body={"r": (6, 0, 0)}, head={"r": (40, 0, 0)}, antlers={"s": 1.06}, bell={"r": (-24, 0, 0)})
    m.action("charge", 0.5, [
        (0.0, k(legs(-46, -34, 40, 30), root=up(-0.06), **run)),
        (0.125, k(legs(-8, 4, 4, -6), root=up(0.05), **run)),
        (0.25, k(legs(30, 22, -36, -26), root=up(-0.06), **run)),
        (0.375, k(legs(4, -8, -6, 4), root=up(0.05), **run)),
    ], loop=True)
    # 뒷발로 일어서기: 몸(엉덩이 뼈)을 들어 앞발을 허공에 접었다가(정점 0.9) 쿵 내려 딛는다(1.0 — 뿌리가 솟는 순간).
    m.action("rear", 1.6, [
        (0.0, k()),
        (0.15, k(root=up(-0.08), body={"r": (4, 0, 0), "s": (1.04, 1.04, 0.94)}, head={"r": (10, 0, 0)})),
        (0.55, k(body={"r": (-34, 0, 0)}, head={"r": (6, 0, 0)}, bell={"r": (30, 0, 0)}, **legs(4, 28, -6, -6))),
        (0.9, k(body={"r": (-40, 0, 0)}, head={"r": (10, 0, 0)}, antlers={"s": 1.06}, bell={"r": (36, 0, 0)},
                **legs(30, 8, -8, -8))),
        (1.0, k(root=up(-0.12), body={"r": (5, 0, 0), "s": (1.07, 1.07, 0.9)}, head={"r": (22, 0, 0)}, bell={"r": (-22, 0, 0)},
                **legs(6, 6, 0, 0))),
        (1.25, k(root=up(-0.04), body={"r": (2, 0, 0)}, head={"r": (8, 0, 0)}, bell={"r": (10, 0, 0)})),
        (1.6, k()),
    ], linear=(0.9,))
    # 쉼(돌진·내려 딛기 뒤): 숙인 머리를 들며 좌우로 턴다 — 틈.
    m.action("recover", 0.9, [
        (0.0, k(body={"r": (6, 0, 0)}, head={"r": (38, 0, 0)}, bell={"r": (-16, 0, 0)}, **legs(-20, -12, 10, 6))),
        (0.25, k(body={"r": (2, 0, 0)}, head={"r": (16, 0, 16)}, bell={"r": (12, 0, 0)})),
        (0.5, k(head={"r": (8, 0, -14)}, bell={"r": (-8, 0, 0)})),
        (0.9, k()),
    ])
    m.action("hurt", 0.4, [
        (0.0, k(body={"r": (-5, 4, 0), "s": (0.97, 0.97, 1.04)}, head={"r": (-14, 0, 10)}, bell={"r": (18, 0, 0)})),
        (0.4, k()),
    ])
    # 깨어남: 0초 = 다리를 접고 엎드려 머리를 내린 채 기다리는 자세. 앞을 먼저 일으키고(무릎), 선다.
    down = k(legs(-84, -84, -84, -84), root=up(-0.5), body={"r": (2, 0, 0)}, head={"r": (34, 0, 0)}, bell={"r": (-12, 0, 0)})
    m.action("wake", 2.4, [
        (0.0, down),
        (0.8, down),
        (1.4, k(legs(-40, -52, -70, -70), root=up(-0.36), body={"r": (-10, 0, 0)}, head={"r": (-6, 0, 0)}, antlers={"s": 1.05},
                bell={"r": (14, 0, 0)})),
        (1.9, k(legs(-8, -12, -20, -20), root=up(-0.08), body={"r": (-3, 0, 0)}, head={"r": (-10, 0, 0)}, bell={"r": (-8, 0, 0)})),
        (2.4, k()),
    ])


PREVIEWS = [("idle", 0.0, ""), ("slam", 0.9, "_slam"), ("rear", 0.9, "_rear")]
CHECK = [("idle", 0.0), ("walk", 0.0), ("walk", 0.3), ("slam", 0.5), ("slam", 0.9), ("slam", 1.0), ("charge_windup", 0.25),
         ("charge_windup", 0.9), ("charge", 0.0), ("charge", 0.25), ("rear", 0.55), ("rear", 0.9), ("rear", 1.0), ("recover", 0.25),
         ("hurt", 0.05), ("wake", 0.0), ("wake", 1.4)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, check_dir=None):
    m = make()
    path = m.export(models_dir)
    for act, t, suffix in PREVIEWS:
        m.preview(os.path.join(preview_dir, f"{NAME}{suffix}.png"), act, t, yaw=-50)
    if check_dir:
        for act, t in CHECK:
            for yaw in (-70, 150):
                m.preview(os.path.join(check_dir, f"{NAME}_{act}_{int(t * 100):03d}_{yaw}.png"), act, t, yaw=yaw, elev=12)
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
