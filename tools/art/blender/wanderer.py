"""방랑자(플레이어, 키 1.8m): 후드 망토·청록 목도리·바랜 갈색 띠와 장화·후드 그늘 속 불씨 눈 두 점·오른손의 검.

    blender --background --python tools/art/blender/wanderer.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "wanderer"


S = 1.15  # 시안(concepts.py variant_b)의 치수 → 게임 키(≈ 1.4m)


def v(*xs):
    return tuple(x * S for x in xs)


# 0038: 몸은 Meshy(AI) 모델 — Gemini 그림 → Meshy Image to 3D(CC BY 4.0) → `meshy_convert.py`로 줄이고 색을 구운 것.
MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "raccoon_v1_low.glb")
CX = 0.0  # 몸 가운데 x(머리로 맞춘 뒤 — 두 손끝이 ±0.35로 맞는다)
# 부위 이름 → 뼈
MESHY_PARTS = {"head_geo": "head", "body_geo": "body", "arm_l_geo": "arm_l", "arm_r_geo": "arm_r",
               "leg_l_geo": "leg_l", "leg_r_geo": "leg_r", "tail_geo": "tail"}


# 팔 = 어깨 → 손끝 선분 둘레의 원기둥(목도리 끝·옷자락이 섞이지 않게). (어깨, 손끝, 반지름), x는 CX 기준.
ARM = ((0.235, -0.07, 0.42), (0.30, -0.085, 0.30), 0.072)  # 장갑 손 + 소매 끝


def _seg_dist(p, a, b):
    from mathutils import Vector
    a, b = Vector(a), Vector(b)
    ab = b - a
    t = max(0.0, min(1.0, (p - a).dot(ab) / ab.length_squared))
    return (p - (a + ab * t)).length


def classify(c):
    """면 가운데 자리 → 부위(앞 = -Y, 캐릭터 왼쪽 = +X, 키 1.4m). 치수는 높이 띠마다 정점을 찍어 본 것."""
    from mathutils import Vector
    x, y, z = c.x - CX, c.y, c.z
    if z > 0.628:  # 두건 아래 가장자리 — 목도리는 몸에 남겨 고개를 젖혀도 자른 자리가 덜 드러난다
        return "head_geo"
    if z < 0.125:
        return "leg_l_geo" if x > 0 else "leg_r_geo"
    (sx, sy, sz), (hx, hy, hz), r = ARM
    p = Vector((x, y, z))
    if _seg_dist(p, (-sx, sy, sz), (-hx, hy, hz)) < r:
        return "arm_r_geo"
    if _seg_dist(p, (sx, sy, sz), (hx, hy, hz)) < r:
        return "arm_l_geo"
    if z < 0.62 and (y > 0.17 or (y > 0.06 and x > 0.14)):
        return "tail_geo"
    return "body_geo"


# 뼈 자리(새 몸, 키 1.4m). 손은 장갑 가운데, 어깨는 소매 위.
HAND = (0.29, -0.085, 0.31)
SHOULDER = (0.185, -0.05, 0.52)


def make():
    """0038: Meshy 몸 + 코드로 만든 칼·등잔. 뼈 이름·동작 표는 0037과 같다."""
    m = C.Model(NAME)
    m.loc_scale = 0.55
    hx, hy, hz = HAND
    sx, sy, sz = SHOULDER
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0, 0.13), "root")
    m.bone("head", (0, -0.02, 0.62), "body")
    m.bone("arm_l", (sx, sy, sz), "body")
    m.bone("arm_r", (-sx, sy, sz), "body")
    m.bone("hand_r", (-hx, hy, hz), "arm_r")
    m.bone("hand_l", (hx, hy, hz), "arm_l")
    m.bone("leg_l", (0.08, -0.07, 0.125), "root")
    m.bone("leg_r", (-0.08, -0.07, 0.125), "root")
    m.bone("cloak", (0, 0.15, 0.50), "body")  # 0037 동작 표가 부르는 뼈 — 새 몸에는 따로 흔들 망토가 없다
    m.bone("tail", (0.10, 0.16, 0.24), "body")
    m.bone("lamp", (hx, hy - 0.05, hz - 0.04), "hand_l")
    m.hide("lamp")

    m.external(MESHY_LOW, classify, MESHY_PARTS)

    # 칼(오른손) — 0037과 같은 꼴, 손 자리만 새 몸에 맞춤.
    g = m.geo("sword", "hand_r")
    x0, y0, z0 = -hx, hy, hz
    g.blob((x0, y0 + 0.06, z0), (0.05, 0.04, 0.05), "p_gold", r=0.9, n=2)  # 칼자루 끝 구슬
    g.box((x0, y0, z0), (0.035, 0.09, 0.035), "p_boot")  # 손잡이
    g.blob((x0, y0 - 0.06, z0), (0.15, 0.035, 0.05), "p_gold", r=0.8, n=2)  # 코등이
    g.box((x0, y0 - 0.31, z0), (0.06, 0.46, 0.022), "p_steel")  # 칼날
    g.box((x0, y0 - 0.29, z0), (0.016, 0.38, 0.026), "p_white")  # 날 가운데 밝은 줄
    g.box((x0, y0 - 0.575, z0), (0.06, 0.022, 0.08), "p_steel", rot=(90, 0, 0), taper=(0.15, 1))  # 칼끝

    # 등잔(왼손, 마실 때만 — 0031).
    g = m.geo("lamp_geo", "lamp")
    g.blob((hx, hy - 0.06, hz - 0.10), (0.115, 0.115, 0.115), "p_lamp", r=0.7, n=3, taper=(0.75, 0.75))
    g = m.geo("glow_lamp", "lamp")
    g.box((hx, hy - 0.06, hz - 0.02), (0.045, 0.045, 0.07), "ember", taper=(0.4, 0.4))

    m.build()
    acts(m)
    return m


def make_blocks():
    """0037 코드 모델(둥근 상자). 0038부터 쓰지 않는다 — 비교·되돌리기용으로 남긴다."""
    m = C.Model(NAME)
    # 0037 B "작은 동물 방랑자": 둥근 너구리 얼굴(눈 둘레 무늬·크림 주둥이)·두건 밖 귀·줄무늬 꼬리, 콩 모양 민트 망토, 복숭아 목도리,
    # 짧은 팔·작은 장화, 허리의 작은 금빛 종. 2등신, 눈은 크고 낮게, 입 없음(docs/art/direction.md).
    # 뼈 이름은 그대로(동작 표를 같이 쓴다) + 꼬리. 동작의 위치 이동은 몸에 맞게 줄인다.
    m.loc_scale = 0.55
    m.bone("root", (0, 0, 0))
    m.bone("body", v(0, 0, 0.16), "root")
    m.bone("head", v(0, 0, 0.56), "body")
    m.bone("arm_l", v(0.20, 0, 0.47), "body")
    m.bone("arm_r", v(-0.20, 0, 0.47), "body")
    m.bone("hand_r", v(-0.225, -0.02, 0.29), "arm_r")
    m.bone("leg_l", v(0.09, 0, 0.14), "root")
    m.bone("leg_r", v(-0.09, 0, 0.14), "root")
    m.bone("cloak", v(0, 0.15, 0.55), "body")
    m.bone("tail", v(0, 0.20, 0.22), "body")
    m.bone("hand_l", v(0.225, -0.02, 0.29), "arm_l")
    m.bone("lamp", v(0.225, -0.07, 0.25), "hand_l")
    m.hide("lamp")

    g = m.geo("body_geo", "body")
    g.blob(v(0, 0, 0.36), v(0.42, 0.36, 0.42), "p_cloak", r=0.7, taper=(0.72, 0.76))  # 콩 모양 망토 몸
    g.blob(v(0, 0, 0.17), v(0.43, 0.37, 0.05), "p_cream", r=0.7, n=3)  # 밑단 테
    g.blob(v(0, -0.02, 0.56), v(0.38, 0.32, 0.09), "p_scarf", r=0.7, n=3)  # 목도리
    g.blob(v(0.10, -0.165, 0.47), v(0.08, 0.04, 0.13), "p_scarf", r=0.7, n=2, rot=(0, 10, 0))  # 목도리 끝
    g.blob(v(-0.205, -0.05, 0.21), v(0.07, 0.07, 0.08), "p_gold", r=0.8, n=2, taper=(0.55, 0.55))  # 허리의 작은 종
    g.blob(v(-0.205, -0.05, 0.175), v(0.025, 0.025, 0.025), "p_boot", r=0.9, n=2)  # 종 추

    g = m.geo("head_geo", "head")
    g.blob(v(0, 0.0, 0.82), v(0.66, 0.56, 0.54), "p_hair", r=0.95, n=5)  # 털 얼굴(캐러멜)
    g.blob(v(0, -0.215, 0.72), v(0.30, 0.16, 0.20), "p_cream", r=1.0, n=4)  # 주둥이(작고 둥글게, 아래로)
    for x in (0.165, -0.165):
        g.blob(v(x * 0.85, -0.225, 0.805), v(0.15, 0.05, 0.16), "p_mask", r=1.0, n=3)  # 눈 둘레 무늬(눈보다 조금 크게)
        g.blob(v(x * 0.82, -0.255, 0.80), v(0.105, 0.03, 0.13), "p_eye", r=1.0, n=3)  # 눈(크고 낮게)
        sx = 1 if x > 0 else -1
        g.box(v(x * 0.82 - 0.02 * sx, -0.273, 0.83), v(0.034, 0.01, 0.038), "p_white", drop=("+y",))  # 눈빛
        g.blob(v(x * 1.32, -0.235, 0.715), v(0.07, 0.02, 0.035), "p_blush", r=0.9, n=2)  # 볼
    g.blob(v(0, -0.30, 0.745), v(0.065, 0.04, 0.045), "p_eye", r=1.0, n=2)  # 코
    g.blob(v(0, 0.10, 1.00), v(0.62, 0.44, 0.34), "p_hood", r=1.0, n=4)  # 작은 두건(뒤·위, 둥글게)
    g.blob(v(0, 0.24, 0.92), v(0.40, 0.10, 0.26), "p_hood", r=0.8, n=3)  # 두건 뒷자락
    for x in (0.22, -0.22):
        g.blob(v(x, 0.0, 1.12), v(0.15, 0.10, 0.18), "p_hair", r=0.85, n=3)  # 귀(두건 밖으로)
        g.blob(v(x, -0.05, 1.11), v(0.08, 0.03, 0.10), "p_blush", r=0.9, n=2)  # 귀 안쪽

    for side, sx in (("l", 1), ("r", -1)):
        g = m.geo(f"arm_{side}_geo", f"arm_{side}")
        g.blob(v(0.215 * sx, -0.01, 0.40), v(0.12, 0.13, 0.18), "p_cloak", r=0.8, n=3)  # 짧고 통통한 소매
        g.blob(v(0.225 * sx, -0.02, 0.29), v(0.11, 0.11, 0.09), "p_hair", r=0.95, n=3)  # 앞발(손)

    g = m.geo("sword", "hand_r")
    y0 = -0.02 * S
    hx, hz = -0.225 * S, 0.29 * S
    g.blob((hx, y0 + 0.06, hz), (0.05, 0.04, 0.05), "p_gold", r=0.9, n=2)  # 칼자루 끝 구슬
    g.box((hx, y0, hz), (0.035, 0.09, 0.035), "p_boot")  # 손잡이
    g.blob((hx, y0 - 0.06, hz), (0.15, 0.035, 0.05), "p_gold", r=0.8, n=2)  # 코등이
    g.box((hx, y0 - 0.31, hz), (0.06, 0.46, 0.022), "p_steel")  # 칼날
    g.box((hx, y0 - 0.29, hz), (0.016, 0.38, 0.026), "p_white")  # 날 가운데 밝은 줄
    g.box((hx, y0 - 0.575, hz), (0.06, 0.022, 0.08), "p_steel", rot=(90, 0, 0), taper=(0.15, 1))  # 칼끝

    for side, sx in (("l", 1), ("r", -1)):
        g = m.geo(f"leg_{side}_geo", f"leg_{side}")
        g.blob(v(0.09 * sx, -0.03, 0.06), v(0.15, 0.20, 0.13), "p_boot", r=0.7, n=3)  # 작은 장화(망토 밑으로)

    g = m.geo("tail_geo", "tail")
    for k, (y, z, w, mat) in enumerate(((0.27, 0.21, 0.17, "p_hair"), (0.33, 0.29, 0.16, "p_mask"),
                                        (0.37, 0.38, 0.145, "p_hair"), (0.395, 0.465, 0.12, "p_cream"))):
        g.blob(v(0, y, z), v(w, w, w * 0.8), mat, r=0.95, n=3)  # 줄무늬 꼬리(위로 말림)

    g = m.geo("lamp_geo", "lamp")
    g.blob(v(0.225, -0.08, 0.20), v(0.10, 0.10, 0.10), "p_lamp", r=0.7, n=3, taper=(0.75, 0.75))  # 등잔 몸
    g = m.geo("glow_lamp", "lamp")
    g.box(v(0.225, -0.08, 0.27), v(0.04, 0.04, 0.06), "ember", taper=(0.4, 0.4))  # 불꽃

    g = m.geo("cloak_geo", "cloak")
    g.blob(v(0, 0.16, 0.36), v(0.36, 0.06, 0.38), "p_hood", r=0.5, n=3, taper=(0.8, 1))  # 등 망토

    m.build()
    acts(m)
    return m


def acts(m):
    # 기본 자세(쉬기): 칼끝이 앞아래로.
    rest_hand = {"r": (50, 0, 0)}

    # idle 1.2초 고리: 숨 쉬기(몸이 조금 오르내리고 망토가 따라 흔들림)
    m.action("idle", 1.2, [
        (0.0, {"root": {"l": (0, 0, 0)}, "body": {"r": (3, 0, 0)}, "head": {"r": (-2, 0, 0)},
               "arm_l": {"r": (-4, -4, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand,
               "cloak": {"r": (6, 0, 0)}, "tail": {"r": (0, 0, -14)}}),
        (0.6, {"tail": {"r": (0, 0, 14)}, "root": {"l": (0, 0, -0.018)}, "body": {"r": (5, 0, 0)}, "head": {"r": (0, 4, 0)},
               "arm_l": {"r": (-2, -6, 0)}, "arm_r": {"r": (-10, 8, 0)}, "hand_r": {"r": (54, 0, 0)},
               "cloak": {"r": (9, 0, 0)}}),
    ], loop=True)

    # walk 0.8초 고리: 다리 ±28°, 팔은 반대로, 지날 때 몸이 오른다
    st = 28
    def walk_key(s):
        # 0038: 뒤뚱 — 내딛는 발 쪽으로 몸이 기울고(Y) 머리는 반대로 조금 버틴다, 꼬리는 크게 반대로.
        return {"root": {"l": (0, 0, 0)}, "body": {"r": (6, -7 * s, 4 * s)},
                "leg_l": {"r": (-st * s, 0, 0)}, "leg_r": {"r": (st * s, 0, 0)},
                "arm_l": {"r": (18 * s, -4, 0)}, "arm_r": {"r": (-16 - 12 * s, 6, 0)}, "hand_r": rest_hand,
                "head": {"r": (-3, 4 * s, -3 * s)}, "cloak": {"r": (14, 0, -3 * s)}, "tail": {"r": (-8, 6 * s, 24 * s)}}
    def pass_key():
        return {"root": {"l": (0, 0, 0.05)}, "body": {"r": (6, 0, 0)}, "leg_l": {"r": (0, 0, 0)},
                "leg_r": {"r": (0, 0, 0)}, "cloak": {"r": (20, 0, 0)}, "head": {"r": (-6, 0, 0)}}
    m.action("walk", 0.8, [(0.0, walk_key(1)), (0.2, pass_key()), (0.4, walk_key(-1)), (0.6, pass_key())], loop=True)

    # attack 0.43초: 0~0.08 칼을 오른쪽 뒤로 당김, 0.08~0.18 앞을 가로질러 베기(오른쪽 → 왼쪽, 조금 위에서 아래로), 0.18~0.43 돌아옴
    m.action("attack", 0.43, [
        (0.0, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand,
               "arm_l": {"r": (-4, -4, 0)}}),
        (0.08, {"root": {"l": (0, 0, -0.04)}, "body": {"r": (4, -6, -38)}, "head": {"r": (0, 0, 20)},
                "arm_r": {"r": (-95, 0, -75)}, "hand_r": {"r": (82, 0, -20)},
                "arm_l": {"r": (-30, -25, 0)}, "leg_l": {"r": (-22, 0, 0)}, "leg_r": {"r": (14, 0, 0)},
                "cloak": {"r": (8, 0, 10)}}),
        (0.18, {"root": {"l": (0, -0.05, -0.07)}, "body": {"r": (14, 6, 40)}, "head": {"r": (0, 0, -24)},
                "arm_r": {"r": (-78, 0, 70)}, "hand_r": {"r": (88, 0, 25)},
                "arm_l": {"r": (15, -30, 0)}, "leg_l": {"r": (-30, 0, 0)}, "leg_r": {"r": (22, 0, 0)},
                "cloak": {"r": (30, 0, -18)}}),
        (0.26, {"root": {"l": (0, -0.05, -0.06)}, "body": {"r": (12, 4, 34)}, "head": {"r": (0, 0, -18)},
                "arm_r": {"r": (-62, 0, 62)}, "hand_r": {"r": (80, 0, 20)},
                "arm_l": {"r": (10, -24, 0)}, "leg_l": {"r": (-26, 0, 0)}, "leg_r": {"r": (20, 0, 0)},
                "cloak": {"r": (22, 0, -10)}}),
        (0.43, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand,
                "arm_l": {"r": (-4, -4, 0)}, "cloak": {"r": (6, 0, 0)}}),
    ], linear=(0.08,))

    # dodge 0.25초: 낮게 웅크려 앞으로 몸을 던지는 기울기(구르기 비슷)
    m.action("dodge", 0.25, [
        (0.0, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand}),
        (0.06, {"root": {"l": (0, -0.05, -0.28)}, "body": {"r": (48, 0, 0)}, "head": {"r": (10, 0, 0)},
                "leg_l": {"r": (-62, 0, 0)}, "leg_r": {"r": (38, 0, 0)},
                "arm_l": {"r": (38, -15, 0)}, "arm_r": {"r": (40, 15, 0)}, "hand_r": {"r": (100, 0, 0)},
                "cloak": {"r": (18, 0, 0)}}),
        (0.15, {"root": {"l": (0, -0.08, -0.30)}, "body": {"r": (50, 0, 0)}, "head": {"r": (4, 0, 0)},
                "leg_l": {"r": (-58, 0, 0)}, "leg_r": {"r": (42, 0, 0)},
                "arm_l": {"r": (-10, -20, 0)}, "arm_r": {"r": (-20, 20, 0)}, "hand_r": {"r": (90, 0, 0)},
                "cloak": {"r": (22, 0, 0)}}),
        (0.25, {"root": {"l": (0, 0, -0.06)}, "body": {"r": (14, 0, 0)}, "head": {"r": (0, 0, 0)},
                "leg_l": {"r": (-14, 0, 0)}, "leg_r": {"r": (10, 0, 0)},
                "arm_l": {"r": (-6, -6, 0)}, "arm_r": {"r": (-14, 8, 0)}, "hand_r": rest_hand,
                "cloak": {"r": (24, 0, 0)}}),
    ])

    # hurt 0.35초: 뒤로 젖혀 밀림
    m.action("hurt", 0.35, [
        (0.0, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand}),
        (0.06, {"root": {"l": (0, 0.06, -0.03)}, "body": {"r": (-22, 0, 6)}, "head": {"r": (-22, 0, 0)},
                "arm_l": {"r": (-30, -38, 0)}, "arm_r": {"r": (-40, 35, 0)}, "hand_r": {"r": (30, 0, 0)},
                "leg_l": {"r": (-10, 0, 0)}, "leg_r": {"r": (12, 0, 0)}, "cloak": {"r": (-10, 0, 0)}}),
        (0.16, {"root": {"l": (0, 0.05, -0.05)}, "body": {"r": (-14, 0, 4)}, "head": {"r": (-10, 0, 0)},
                "arm_l": {"r": (-20, -28, 0)}, "arm_r": {"r": (-30, 25, 0)}, "hand_r": {"r": (38, 0, 0)},
                "leg_l": {"r": (-8, 0, 0)}, "leg_r": {"r": (10, 0, 0)}, "cloak": {"r": (-4, 0, 0)}}),
        (0.35, {"body": {"r": (3, 0, 0)}, "arm_l": {"r": (-4, -4, 0)}, "arm_r": {"r": (-12, 6, 0)},
                "hand_r": rest_hand, "cloak": {"r": (6, 0, 0)}}),
    ])

    # 0034 turn_r / turn_l 0.25초: 몸을 낮췄다가(0.06) 윗몸이 도는 쪽으로 먼저 휙 감기고(0.13) 몸이 따라와 선다.
    # 게임은 같은 동안 몸 방향을 천천히-빠르게-천천히 돌린다(world `PState::Turn`). r의 z: + = 위에서 보아 반시계(왼쪽으로).
    def turn_keys(side):  # side: -1 오른쪽, +1 왼쪽
        return [
            (0.0, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand}),
            (0.06, {"root": {"l": (0, 0, -0.28)}, "body": {"r": (30, 0, 8 * side)}, "head": {"r": (10, 0, 0)},
                    "leg_l": {"r": (-40, -16, 0)}, "leg_r": {"r": (30, 16, 0)},
                    "arm_l": {"r": (-40, -20, 20)}, "arm_r": {"r": (-44, 20, -20)}, "hand_r": {"r": (95, 0, 0)},
                    "cloak": {"r": (18, 0, 0)}}),
            (0.13, {"root": {"l": (0, 0, -0.26)}, "body": {"r": (24, 0, 48 * side)}, "head": {"r": (6, 0, 20 * side)},
                    "leg_l": {"r": (-36, -16, 0)}, "leg_r": {"r": (28, 16, 0)},
                    "arm_l": {"r": (-30, -40, 60 * side)}, "arm_r": {"r": (-34, 40, 60 * side)}, "hand_r": {"r": (95, 0, 0)},
                    "cloak": {"r": (40, 0, -40 * side)}}),
            (0.25, {"root": {"l": (0, 0, -0.04)}, "body": {"r": (6, 0, 6 * side)}, "head": {"r": (0, 0, 0)},
                    "leg_l": {"r": (-4, -6, 0)}, "leg_r": {"r": (4, 6, 0)},
                    "arm_l": {"r": (-6, -6, 0)}, "arm_r": {"r": (-14, 8, 0)}, "hand_r": rest_hand,
                    "cloak": {"r": (14, 0, -10 * side)}}),
        ]
    m.action("turn_r", 0.25, turn_keys(-1), linear=(0.06,))
    m.action("turn_l", 0.25, turn_keys(1), linear=(0.06,))

    # 0034 락온 걸음: 칼을 앞에 겨눈 채 옆으로 게걸음(다리를 벌렸다 모은다), 뒷걸음. 0.6초 고리.
    guard_arm = {"arm_r": {"r": (-62, 4, -22)}, "hand_r": {"r": (22, 0, -16)}, "arm_l": {"r": (-30, -18, 10)}}
    def strafe_key(open_, side):
        k = {"root": {"l": (0, 0, -0.03 if open_ else 0.0)}, "body": {"r": (8, 0, 6 * side)},
             "head": {"r": (0, 0, -4 * side)},
             "leg_l": {"r": (0, (-16 if open_ else 4), 0)}, "leg_r": {"r": (0, (16 if open_ else -4), 0)},
             "cloak": {"r": (12, (8 * side if open_ else 0), 0)}}
        k.update(guard_arm)
        return k
    for name, side in (("strafe_l", 1), ("strafe_r", -1)):
        m.action(name, 0.6, [(0.0, strafe_key(False, side)), (0.15, strafe_key(True, side)),
                             (0.3, strafe_key(False, side)), (0.45, strafe_key(True, side))], loop=True)
    def back_key(sw):
        k = {"root": {"l": (0, 0, 0)}, "body": {"r": (2, 0, 3 * sw)},
             "leg_l": {"r": (20 * sw, 0, 0)}, "leg_r": {"r": (-20 * sw, 0, 0)},
             "head": {"r": (0, 0, 0)}, "cloak": {"r": (6, 0, 0)}}
        k.update(guard_arm)
        return k
    m.action("walk_back", 0.7, [(0.0, back_key(1)), (0.175, dict(back_key(0), root={"l": (0, 0, 0.025)})),
                                (0.35, back_key(-1)), (0.525, dict(back_key(0), root={"l": (0, 0, 0.025)}))], loop=True)


    # 0025·0027 동작. 손 자세 기준은 위와 같다(월드 축 오일러, X− = 팔을 앞으로 든다).
    # parry 0.3초: 칼을 가슴 앞에 가로로 들어 막는다(왼팔은 칼등을 받친다).
    guard = {"root": {"l": (0, 0, -0.05)}, "body": {"r": (6, 0, -14)}, "head": {"r": (-4, 0, 8)},
             "arm_r": {"r": (-82, 0, -38)}, "hand_r": {"r": (8, 0, -78)},
             "arm_l": {"r": (-62, -14, 30)}, "leg_l": {"r": (-16, 0, 0)}, "leg_r": {"r": (12, 0, 0)},
             "cloak": {"r": (10, 0, 0)}}
    m.action("parry", 0.3, [
        (0.0, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand}),
        (0.05, guard),
        (0.3, guard),
    ])

    # jump 0.3초(체공 동안 첫 자세를 쥔다): 다리를 접고 팔을 벌려 균형.
    air = {"root": {"l": (0, 0, 0.02)}, "body": {"r": (12, 0, 0)}, "head": {"r": (-6, 0, 0)},
           "leg_l": {"r": (-58, 0, 0)}, "leg_r": {"r": (-24, 0, 0)},
           "arm_l": {"r": (-34, -34, 0)}, "arm_r": {"r": (-28, 30, 0)}, "hand_r": {"r": (62, 0, 0)},
           "cloak": {"r": (38, 0, 0)}}
    m.action("jump", 0.3, [(0.0, air), (0.3, air)])

    # plunge 0.3초: 두 손으로 칼을 머리 위로 들었다가(0.1) 아래로 내리꽂는다.
    m.action("plunge", 0.3, [
        (0.0, {"body": {"r": (-6, 0, 0)}, "head": {"r": (-10, 0, 0)},
               "arm_r": {"r": (-168, 8, 0)}, "arm_l": {"r": (-164, -8, 0)}, "hand_r": {"r": (-20, 0, 0)},
               "leg_l": {"r": (-40, 0, 0)}, "leg_r": {"r": (-20, 0, 0)}, "cloak": {"r": (30, 0, 0)}}),
        (0.12, {"body": {"r": (-10, 0, 0)}, "head": {"r": (-12, 0, 0)},
                "arm_r": {"r": (-175, 8, 0)}, "arm_l": {"r": (-170, -8, 0)}, "hand_r": {"r": (-30, 0, 0)},
                "leg_l": {"r": (-46, 0, 0)}, "leg_r": {"r": (-24, 0, 0)}, "cloak": {"r": (40, 0, 0)}}),
        (0.3, {"root": {"l": (0, -0.04, -0.12)}, "body": {"r": (34, 0, 0)}, "head": {"r": (14, 0, 0)},
               "arm_r": {"r": (-58, 8, 0)}, "arm_l": {"r": (-54, -8, 0)}, "hand_r": {"r": (118, 0, 0)},
               "leg_l": {"r": (-34, 0, 0)}, "leg_r": {"r": (18, 0, 0)}, "cloak": {"r": (12, 0, 0)}}),
    ])

    # 0030 강공격. charge 0.3초: 평타 끝(칼이 왼쪽으로 쓸려 간 자세)에서 칼을 오른쪽 어깨 뒤로 높이 끌어올리고 몸을 비틀어 낮춘다,
    # 0.2초부터는 그 자세를 쥔다(모으는 동안). 첫 키는 평타 0.18초와 같아 이어진다.
    # 0034: 두 손으로 칼을 오른 어깨 뒤 높이 들고 몸을 낮게 비틀어 감는다(왼손이 칼자루로 온다), 망토가 뒤로 날린다.
    windup = {"root": {"l": (0, 0.04, -0.20)}, "body": {"r": (14, -10, -60)}, "head": {"r": (-4, 0, 36)},
              "arm_r": {"r": (-162, 0, -62)}, "hand_r": {"r": (30, 0, -40)},
              "arm_l": {"r": (-150, 10, -70)}, "leg_l": {"r": (-34, -12, 0)}, "leg_r": {"r": (24, 12, 0)},
              "cloak": {"r": (28, 0, 20)}}
    m.action("charge", 0.3, [
        (0.0, {"root": {"l": (0, -0.05, -0.07)}, "body": {"r": (14, 6, 40)}, "head": {"r": (0, 0, -24)},
               "arm_r": {"r": (-78, 0, 70)}, "hand_r": {"r": (88, 0, 25)},
               "arm_l": {"r": (15, -30, 0)}, "leg_l": {"r": (-30, 0, 0)}, "leg_r": {"r": (22, 0, 0)},
               "cloak": {"r": (30, 0, -18)}}),
        (0.2, windup),
        (0.3, windup),
    ])

    # heavy 0.58초(선딜 0.06 + 활성 0.12 + 후딜 0.4): 끌어올린 칼을 크게 앞을 가로질러 낮게 내려친다(평타보다 넓고 깊게),
    # 끝나면 칼끝이 땅 가까이에 머물렀다(무게) 천천히 돌아온다.
    m.action("heavy", 0.58, [
        (0.0, windup),
        (0.06, {"root": {"l": (0, 0.02, -0.22)}, "body": {"r": (16, -12, -66)}, "head": {"r": (-2, 0, 40)},
                "arm_r": {"r": (-170, 0, -70)}, "hand_r": {"r": (26, 0, -44)},
                "arm_l": {"r": (-158, 10, -76)}, "leg_l": {"r": (-36, -12, 0)}, "leg_r": {"r": (26, 12, 0)},
                "cloak": {"r": (32, 0, 22)}}),
        # 0.18: 앞으로 크게 내디디며 낮게 내리친다(칼끝이 땅 가까이, 두 손).
        (0.18, {"root": {"l": (0, -0.18, -0.24)}, "body": {"r": (34, 10, 46)}, "head": {"r": (10, 0, -26)},
                "arm_r": {"r": (-42, 0, 88)}, "hand_r": {"r": (108, 0, 42)},
                "arm_l": {"r": (-48, -10, 70)}, "leg_l": {"r": (-52, 0, 0)}, "leg_r": {"r": (36, 0, 0)},
                "cloak": {"r": (44, 0, -28)}}),
        (0.36, {"root": {"l": (0, -0.17, -0.22)}, "body": {"r": (30, 8, 42)}, "head": {"r": (8, 0, -22)},
                "arm_r": {"r": (-36, 0, 84)}, "hand_r": {"r": (110, 0, 38)},
                "arm_l": {"r": (-40, -10, 64)}, "leg_l": {"r": (-50, 0, 0)}, "leg_r": {"r": (34, 0, 0)},
                "cloak": {"r": (30, 0, -16)}}),
        (0.58, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand,
                "arm_l": {"r": (-4, -4, 0)}, "cloak": {"r": (6, 0, 0)}}),
    ], linear=(0.06,))

    # 0031 drink 0.8초: 왼손을 들어(0.35초에 입에 닿음 = 삼키는 순간) 고개를 젖혔다가 내린다. 칼은 오른손에 쉬는 자세 그대로.
    sip = {"body": {"r": (-4, 0, 0)}, "head": {"r": (-24, 0, 0)},
           "arm_l": {"r": (-112, -6, -40)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand,
           "cloak": {"r": (4, 0, 0)}}
    m.action("drink", 0.8, [
        (0.0, {"lamp": {"s": 0.0}, "body": {"r": (3, 0, 0)}, "arm_l": {"r": (-4, -4, 0)}, "arm_r": {"r": (-12, 6, 0)},
               "hand_r": rest_hand, "cloak": {"r": (6, 0, 0)}}),
        (0.05, {"lamp": {"s": 1.0}}),
        (0.25, {"body": {"r": (2, 0, 0)}, "head": {"r": (-8, 0, 0)},
                "arm_l": {"r": (-95, -6, -30)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand}),
        (0.35, sip),
        (0.55, sip),
        (0.75, {"lamp": {"s": 1.0}}),
        (0.8, {"lamp": {"s": 0.0}, "body": {"r": (3, 0, 0)}, "head": {"r": (0, 0, 0)}, "arm_l": {"r": (-4, -4, 0)},
               "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand, "cloak": {"r": (6, 0, 0)}}),
    ])

    # land 0.15초: 내려앉아 무릎을 굽혔다 편다.
    m.action("land", 0.15, [
        (0.0, {"root": {"l": (0, 0, -0.16)}, "body": {"r": (22, 0, 0)}, "head": {"r": (6, 0, 0)},
               "leg_l": {"r": (-34, 0, 0)}, "leg_r": {"r": (24, 0, 0)},
               "arm_l": {"r": (-20, -30, 0)}, "arm_r": {"r": (-20, 30, 0)}, "hand_r": rest_hand,
               "cloak": {"r": (4, 0, 0)}}),
        (0.15, {"body": {"r": (3, 0, 0)}, "arm_l": {"r": (-4, -4, 0)}, "arm_r": {"r": (-12, 6, 0)},
                "hand_r": rest_hand, "cloak": {"r": (6, 0, 0)}}),
    ])


PREVIEWS = [("idle", 0.0, ""), ("idle", 0.0, "_front"), ("attack", 0.15, "_attack"), ("parry", 0.1, "_parry"),
            ("jump", 0.1, "_jump"), ("plunge", 0.06, "_plunge_up"), ("plunge", 0.29, "_plunge_down"),
            ("charge", 0.3, "_charge"), ("heavy", 0.18, "_heavy"),
            ("drink", 0.4, "_drink"), ("turn_r", 0.13, "_turn"), ("strafe_l", 0.15, "_strafe")]
CHECK = [("walk", 0.0), ("walk", 0.2), ("attack", 0.08), ("attack", 0.18), ("dodge", 0.15), ("hurt", 0.06), ("turn_r", 0.06), ("turn_r", 0.13)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, check_dir=None):
    m = make()
    path = m.export(models_dir)
    for act, t, suffix in PREVIEWS:
        # 얼굴을 보려고 정면에 가깝게 한 장(0036).
        kw = {"yaw": -8.0, "elev": 14.0} if suffix == "_front" else {}
        m.preview(os.path.join(preview_dir, f"{NAME}{suffix}.png"), act, t, **kw)
    if check_dir:
        for act, t in CHECK:
            m.preview(os.path.join(check_dir, f"{NAME}_{act}_{int(t * 100):03d}.png"), act, t)
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
