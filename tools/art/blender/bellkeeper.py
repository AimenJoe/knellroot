"""깨진 종지기(마을의 보스 — 0125 · 모델 0132).

Meshy 몸 — 두건 쓴 큰 머리, 아래로 퍼지는 종 모양 잿빛 옷, 졸린 호박빛 눈 둘(glow_eyes), 어깨에서 앞으로 내려간 금 안의
호박빛 핵(glow_core, 뼈 `core`), 금과 어깨를 덮은 보라 뿌리, 짧은 발, 왼손(+X)에 고리로 든 깨진 청동 종(뼈 `bell`).
키 3.0m(두건 끝까지). 원본 `tools/art/sources/meshy/bellkeeper_v1.glb`(Gemini 그림 → Meshy "Hollow Lantern Bellringer",
CC BY 4.0) → `meshy_convert.py ... 3.0 1500 256`. 종은 Meshy가 몸 옆이 아니라 왼손 **앞**(y −0.4 ~ −1.2)에 두었다.

    blender --background --python tools/art/blender/bellkeeper.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "bellkeeper"

MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "bellkeeper_v1_low.glb")
MESHY_PARTS = {"body_geo": "body", "head_geo": "head", "glow_eyes": "head", "glow_core": "core",
               "arm_l_geo": "arm_l", "arm_r_geo": "arm_r", "bell_geo": "bell",
               "leg_l_geo": "leg_l", "leg_r_geo": "leg_r"}
# 치수(앞 = -Y, 캐릭터 왼쪽 = +X) — 정면·옆·위 직교 렌더(텍스처째)와 부위 색칠 그림으로 잰 것.
HEAD_Z = 2.05  # 이보다 위(가운데)는 두건 머리
EYES_Z = 2.0  # 호박빛 면 중 이보다 위는 눈, 아래는 가슴 금의 핵
LEG_Z = 0.30  # 옷자락 아래 짧은 발
GREY_K = 0.6  # 잿빛 몸을 어둡게(다른 적과 같은 까닭·같은 값) — 청동 종·보라 뿌리·호박빛은 그대로
GREY_SAT = 0.18


def _bronze(h, s, v):
    return 0.04 < h < 0.2 and s > 0.18 and v < 0.6


def _amber(h, s, v):
    return 0.04 < h < 0.16 and s > 0.5 and v > 0.6


BELL, AMBER = [], []


def classify(c):
    """면 가운데 자리 → 부위. 종(청동)·눈과 핵(호박빛)은 색으로(`C.face_colors`), 나머지는 자리로."""
    x, y, z = c.x, c.y, c.z
    p = (x, y, z)
    if x > 0.4 and y < -0.3 and z < 1.35 and C.near_any(BELL, p):
        return "bell_geo"
    if C.near_any(AMBER, p):
        return "glow_eyes" if z > EYES_Z else "glow_core"
    if z > HEAD_Z and abs(x) < 0.8:
        return "head_geo"
    # 팔은 옷 옆구리에 붙어 내려온다 — 옷보다 밖(|x| > 0.9 남짓)이거나 어깨 높이만 팔. 왼팔은 앞으로 뻗어 종 고리를 쥔다.
    if 0.72 < z < 1.75 and (x < -0.9 or (x < -0.72 and z > 1.45)):
        return "arm_r_geo"
    if 0.95 < z < 1.85 and (x > 0.82 or (x > 0.62 and z > 1.5) or (x > 0.5 and y < -0.55)):
        return "arm_l_geo"
    if z < LEG_Z and abs(x) < 0.6:
        return "leg_l_geo" if x > 0 else "leg_r_geo"
    return "body_geo"


def make():
    faces = C.face_colors(MESHY_LOW)
    BELL[:] = [c for c, hsv in faces if _bronze(*hsv)]
    AMBER[:] = [c for c, hsv in faces if _amber(*hsv)]
    m = C.Model(NAME)
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0.05, 0.35), "root")  # 옷자락 밑동 — 숙이고 비틀면 위가 크게 움직인다
    m.bone("head", (0, -0.1, 2.05), "body")
    m.bone("core", (-0.22, -0.52, 1.35), "body")
    m.bone("arm_l", (0.68, -0.1, 1.72), "body")  # 종을 든 팔(+X)
    m.bone("bell", (0.95, -0.8, 1.12), "arm_l")  # 종 고리를 쥔 손
    m.bone("arm_r", (-0.75, 0.0, 1.68), "body")
    m.bone("leg_l", (0.25, 0.05, 0.32), "root")
    m.bone("leg_r", (-0.28, 0.05, 0.32), "root")
    m.external(MESHY_LOW, classify, MESHY_PARTS)
    C.darken_grey(m.ext_img, GREY_K, GREY_SAT)
    m.build()
    acts(m)
    return m


def k(**bones):
    return bones


REST = k(body={"r": (0, 0, 0)}, head={"r": (0, 0, 0)}, arm_l={"r": (0, 0, 0)}, arm_r={"r": (0, 0, 0)},
         bell={"r": (0, 0, 0)}, leg_l={"r": (0, 0, 0)}, leg_r={"r": (0, 0, 0)}, core={"s": (1, 1, 1)})


def acts(m):
    # idle 2.0초 고리: 무거운 숨(옷이 오르내림), 핵이 숨 따라 커졌다 작아짐, 종이 조금 흔들림
    m.action("idle", 2.0, [
        (0.0, k(body={"r": (0, 2, 0)}, head={"r": (4, 0, 0)}, bell={"r": (3, 0, 0)}, core={"s": (1, 1, 1)})),
        (1.0, k(root={"l": (0, 0, -0.03)}, body={"r": (2, -2, 0), "s": (1.02, 1.02, 0.98)}, head={"r": (8, 0, 0)},
                bell={"r": (-3, 0, 0)}, core={"s": (1.18, 1.18, 1.18)})),
    ], loop=True)

    # walk 1.4초 고리: 뒤뚱(옷이 좌우로 기울고 짧은 발이 번갈아), 종은 늦게 흔들림
    m.action("walk", 1.4, [
        (0.0, k(body={"r": (2, 6, 0)}, leg_l={"r": (-22, 0, 0)}, leg_r={"r": (18, 0, 0)}, bell={"r": (10, 0, 0)},
                arm_r={"r": (10, 0, 0)}, head={"r": (4, -3, 0)})),
        (0.35, k(root={"l": (0, 0, 0.04)}, body={"r": (2, 0, 0)}, leg_l={"r": (0, 0, 0)}, leg_r={"r": (0, 0, 0)},
                 bell={"r": (0, 0, 0)}, arm_r={"r": (0, 0, 0)})),
        (0.7, k(body={"r": (2, -6, 0)}, leg_l={"r": (18, 0, 0)}, leg_r={"r": (-22, 0, 0)}, bell={"r": (-10, 0, 0)},
                arm_r={"r": (-10, 0, 0)}, head={"r": (4, 3, 0)})),
        (1.05, k(root={"l": (0, 0, 0.04)}, body={"r": (2, 0, 0)}, leg_l={"r": (0, 0, 0)}, leg_r={"r": (0, 0, 0)},
                 bell={"r": (0, 0, 0)}, arm_r={"r": (0, 0, 0)})),
    ], loop=True)

    # toll 1.6초(울림 고리, render STRIKE_KEYS ("bellkeeper", "toll", 1.0, 1.12)): 종을 머리 위로 들어(1.0 정점 —
    # 예고 동안 버팀, 종은 아래로 늘어진 채) → 1.12 앞 땅에 내리쳐 울린다(고리가 퍼지는 틱) → 웅크린 채 버팀 → 쉼.
    up = k(body={"r": (-10, 0, 0)}, head={"r": (-14, 0, 0)}, arm_l={"r": (-112, 0, 0)}, bell={"r": (100, 0, 0)},
           arm_r={"r": (-40, 0, 20)}, core={"s": (1.25, 1.25, 1.25)})
    m.action("toll", 1.6, [
        (0.0, REST),
        (0.25, k(root={"l": (0, 0, -0.06)}, body={"r": (8, 0, 0), "s": (1.04, 1.04, 0.94)}, arm_l={"r": (-20, 0, 0)},
                 bell={"r": (10, 0, 0)}, head={"r": (10, 0, 0)})),
        (0.75, up),
        (1.0, dict(up, arm_l={"r": (-118, 0, 0)}, root={"l": (0, 0, 0.05)})),
        (1.12, k(root={"l": (0, 0, -0.1)}, body={"r": (16, 0, 0), "s": (1.06, 1.06, 0.92)}, head={"r": (16, 0, 0)},
                 arm_l={"r": (10, 0, 0)}, bell={"r": (14, 0, 0)}, arm_r={"r": (-10, 0, 0)}, core={"s": (0.9, 0.9, 0.9)})),
        (1.3, k(root={"l": (0, 0, -0.08)}, body={"r": (14, 0, 0), "s": (1.05, 1.05, 0.94)}, head={"r": (14, 0, 0)},
                arm_l={"r": (8, 0, 0)}, bell={"r": (6, 0, 0)})),
        (1.6, REST),
    ], linear=(1.0,))

    # slam 1.5초(앞 부채꼴 휘두르기, STRIKE_KEYS 0.9·1.0): 몸을 왼쪽으로 비틀어 종을 뒤로 끌어 올리고(0.9 정점, 버팀)
    # → 1.0 오른쪽으로 크게 돌며 종이 앞을 가로지른다(맞는 틱) → 끝까지 휘둘러 버팀 → 쉼.
    wind = k(body={"r": (-4, 0, 38)}, head={"r": (0, 0, -18)}, arm_l={"r": (24, -38, 0)}, bell={"r": (20, 0, 0)},
             arm_r={"r": (-20, 20, 0)}, core={"s": (1.2, 1.2, 1.2)})
    m.action("slam", 1.5, [
        (0.0, REST),
        (0.6, wind),
        (0.9, dict(wind, body={"r": (-6, 0, 44)})),
        (1.0, k(body={"r": (8, 0, -34)}, head={"r": (6, 0, 10)}, arm_l={"r": (-70, -30, 0)}, bell={"r": (-30, -40, 0)},
                arm_r={"r": (10, -10, 0)})),
        (1.2, k(body={"r": (8, 0, -44)}, head={"r": (6, 0, 14)}, arm_l={"r": (-60, -20, 0)}, bell={"r": (-10, -20, 0)},
                arm_r={"r": (10, -10, 0)})),
        (1.5, REST),
    ], linear=(0.9,))

    # throw 1.2초(종 조각 부채꼴, STRIKE_KEYS 0.6·0.7): 종을 가슴 앞으로 들어 입을 앞으로 젖히고(0.6 정점, 버팀)
    # → 0.7 앞으로 확 털어 조각을 뿌린다 → 쉼.
    m.action("throw", 1.2, [
        (0.0, REST),
        (0.45, k(body={"r": (-8, 0, 6)}, head={"r": (-6, 0, 0)}, arm_l={"r": (-62, 0, 0)}, bell={"r": (40, 0, 0)},
                 arm_r={"r": (-20, 0, 10)})),
        (0.6, k(body={"r": (-10, 0, 8)}, head={"r": (-8, 0, 0)}, arm_l={"r": (-70, 0, 0)}, bell={"r": (52, 0, 0)},
                arm_r={"r": (-24, 0, 10)}, core={"s": (1.2, 1.2, 1.2)})),
        (0.7, k(body={"r": (10, 0, -4)}, head={"r": (8, 0, 0)}, arm_l={"r": (-34, 0, 0)}, bell={"r": (-36, 0, 0)},
                arm_r={"r": (6, 0, 0)})),
        (0.85, k(body={"r": (8, 0, -2)}, head={"r": (6, 0, 0)}, arm_l={"r": (-28, 0, 0)}, bell={"r": (-12, 0, 0)})),
        (1.2, REST),
    ], linear=(0.6,))

    # charge_windup 0.8초: 몸을 앞으로 숙이고 종을 뒤로 끌어 둔다(돌진 = 종 끌기, 0125)
    drag = k(body={"r": (22, 0, 0)}, head={"r": (10, 0, 0)}, arm_l={"r": (46, 0, 0)}, bell={"r": (-30, 0, 0)},
             arm_r={"r": (30, 0, 0)})
    m.action("charge_windup", 0.8, [
        (0.0, REST),
        (0.3, k(root={"l": (0, 0, -0.1)}, body={"r": (10, 0, 0), "s": (1.06, 1.06, 0.9)}, head={"r": (14, 0, 0)},
                arm_l={"r": (20, 0, 0)}, core={"s": (1.2, 1.2, 1.2)})),
        (0.8, dict(drag, root={"l": (0, 0, -0.06)}, leg_l={"r": (-20, 0, 0)}, leg_r={"r": (24, 0, 0)})),
    ])
    # charge 0.5초 고리: 숙인 채 짧은 발로 쿵쿵, 끌리는 종이 튄다
    m.action("charge", 0.5, [
        (0.0, dict(drag, leg_l={"r": (-28, 0, 0)}, leg_r={"r": (24, 0, 0)}, bell={"r": (-26, 0, 0)})),
        (0.125, dict(drag, root={"l": (0, 0, 0.05)}, bell={"r": (-38, 0, 0)})),
        (0.25, dict(drag, leg_l={"r": (24, 0, 0)}, leg_r={"r": (-28, 0, 0)}, bell={"r": (-26, 0, 0)})),
        (0.375, dict(drag, root={"l": (0, 0, 0.05)}, bell={"r": (-38, 0, 0)})),
    ], loop=True)
    # recover 0.8초: 미끄러져 서서 몸을 일으킨다
    m.action("recover", 0.8, [
        (0.0, dict(drag)),
        (0.2, k(body={"r": (-8, 0, 0)}, head={"r": (-6, 0, 0)}, arm_l={"r": (-20, 0, 0)}, bell={"r": (24, 0, 0)},
                leg_l={"r": (-16, 0, 0)}, leg_r={"r": (-16, 0, 0)})),
        (0.5, k(body={"r": (4, 0, 0)}, head={"r": (6, 0, 0)}, arm_l={"r": (4, 0, 0)}, bell={"r": (-8, 0, 0)})),
        (0.8, REST),
    ])

    # hurt 0.4초: 뒤로 휘청, 핵이 움찔 작아짐
    m.action("hurt", 0.4, [
        (0.0, REST),
        (0.08, k(root={"l": (0, 0.08, 0)}, body={"r": (-10, 4, 0)}, head={"r": (-14, 0, 8)}, arm_l={"r": (10, 0, 0)},
                 bell={"r": (18, 0, 0)}, arm_r={"r": (-16, 0, 0)}, core={"s": (0.8, 0.8, 0.8)})),
        (0.4, REST),
    ])

    # wake 2.0초(잠든 보스가 깨어남, render: 깨어남 진행대로): 옷자락째 앞으로 푹 숙이고 종을 땅에 내려놓은 채 핵이 꺼져
    # 있다 → 핵이 켜지며 고개를 들고 → 몸을 일으켜 종을 고쳐 쥔다.
    slump = k(root={"l": (0, 0, -0.12)}, body={"r": (26, 0, 4)}, head={"r": (30, 0, 0)}, arm_l={"r": (14, 0, 0)},
              bell={"r": (-6, 0, 0)}, arm_r={"r": (14, 0, -6)}, core={"s": (0.5, 0.5, 0.5)})
    m.action("wake", 2.0, [
        (0.0, slump),
        (0.6, dict(slump, core={"s": (1.0, 1.0, 1.0)})),
        (1.2, k(root={"l": (0, 0, -0.06)}, body={"r": (12, 0, 2)}, head={"r": (-6, 0, 0)}, arm_l={"r": (6, 0, 0)},
                bell={"r": (-2, 0, 0)}, arm_r={"r": (6, 0, 0)}, core={"s": (1.3, 1.3, 1.3)})),
        (1.6, k(body={"r": (-4, 0, 0)}, head={"r": (-8, 0, 0)}, arm_l={"r": (-10, 0, 0)}, bell={"r": (8, 0, 0)},
                core={"s": (1.15, 1.15, 1.15)})),
        (2.0, REST),
    ])


PREVIEWS = [("idle", 0.0, ""), ("toll", 1.0, "_toll")]
CHECK = [("walk", 0.0), ("toll", 1.0), ("toll", 1.12), ("slam", 0.9), ("slam", 1.0), ("throw", 0.6), ("throw", 0.7),
         ("charge", 0.0), ("hurt", 0.08), ("wake", 0.0), ("wake", 1.2)]


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
