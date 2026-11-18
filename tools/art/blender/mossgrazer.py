"""이끼먹이(0185 — 잔뿌리의 정원의 비적대 생명체 · 2차 0234: 그림 → Meshy).

지금(0234): Meshy 몸 — 크림빛 둥근 몸, 등에 민트 이끼 방석(덩이 셋 + 새싹), 연두 귀 둘, 이끼 꼬리 뭉치, 낮게 둔 검은 큰 눈.
키 0.6m(귀·이끼 끝까지)·길이 0.87m. 원본 `tools/art/sources/meshy/mossgrazer_v1.glb`(나노바나나 Pro 그림
`tools/art/sources/ai/knellroot/mossgrazer_v1_front.png` → Meshy 6 API, CC BY 4.0) → `meshy_convert.py ... 0.6 650 128`.
옛 코드 몸(1차)은 `make_blocks()`에 남긴다.

정상 피식자: 작고 둥근 몸(키 0.5m 남짓), 귀여움 문법(0036 — 큰 머리·낮게 둔 큰 눈·입 없음·둥근 실루엣), 살아 있는 것이라
파스텔(0037 — 따뜻한 크림 몸 + 등의 연두 이끼). 침식된 적(잿빛 + 보라 뿌리)과 한눈에 갈린다.
동작: idle 1.8초 고리(머리를 숙여 이끼를 뜯고 오물오물) · walk 0.4초 고리(깡충 — 온몸으로, 0173) · hurt 0.3초.

    blender --background --python tools/art/blender/mossgrazer.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "mossgrazer"

MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "mossgrazer_v1_low.glb")
MESHY_PARTS = {"body_geo": "body", "head_geo": "head", "ear_l_geo": "ear_l", "ear_r_geo": "ear_r", "tail_geo": "tail"}
# 치수(앞 = -Y, 캐릭터 왼쪽 = +X) — 정면·옆·위 직교 렌더에 0.05m 눈금을 그어 잰 것.
LEG_Z = 0.10  # 이보다 아래는 다리 — 몸에 붙인다(깡충은 온몸으로, 0173)
HEAD_Y = -0.13  # 이보다 앞(위 LEG_Z)은 머리
EAR = (0.12, -0.20, 0.38)  # |x| 넘고, y 앞, z 위면 귀
TAIL_Y = 0.33  # 이보다 뒤(엉덩이 높이)는 이끼 꼬리 뭉치


def classify(c):
    x, y, z = c.x, c.y, c.z
    if z < LEG_Z:
        return "body_geo"
    ex, ey, ez = EAR
    if abs(x) > ex and y < ey and z > ez:
        return "ear_l_geo" if x > 0 else "ear_r_geo"
    if y < HEAD_Y:
        return "head_geo"
    if y > TAIL_Y and 0.14 < z < 0.38:
        return "tail_geo"
    return "body_geo"


def make():
    """0234: Meshy 몸, 뼈 이름·동작은 1차 그대로 — 뼈 자리만 새 몸에."""
    m = C.Model(NAME)
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0.05, 0.12), "root")
    m.bone("head", (0, -0.12, 0.22), "body")
    m.bone("ear_l", (0.17, -0.27, 0.40), "head")
    m.bone("ear_r", (-0.17, -0.27, 0.40), "head")
    m.bone("tail", (0, 0.34, 0.26), "body")
    m.external(MESHY_LOW, classify, MESHY_PARTS)
    m.build()
    acts(m)
    return m


def make_blocks():
    m = C.Model(NAME)
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0.02, 0.12), "root")
    m.bone("head", (0, -0.12, 0.26), "body")
    m.bone("ear_l", (0.11, -0.12, 0.44), "head")
    m.bone("ear_r", (-0.11, -0.12, 0.44), "head")
    m.bone("tail", (0, 0.26, 0.18), "body")

    body = m.geo("body_geo", "body")
    body.blob((0, 0.06, 0.2), (0.36, 0.42, 0.3), "p_cream", r=1.0, n=4)
    # 등의 이끼 방석 + 작은 새싹 셋(위로 솟은 가시가 아니라 둥근 덩이).
    moss = m.geo("moss_geo", "body")
    moss.blob((0, 0.08, 0.33), (0.32, 0.36, 0.12), "p_moss", r=1.0, n=3)
    for x, y in ((0.07, 0.02), (-0.06, 0.1), (0.02, 0.16)):
        moss.blob((x, y, 0.4), (0.07, 0.07, 0.08), "p_mossd", r=1.0, n=2)
    feet = m.geo("feet_geo", "body")
    for x, y in ((0.11, -0.06), (-0.11, -0.06), (0.11, 0.17), (-0.11, 0.17)):
        feet.blob((x, y, 0.035), (0.09, 0.11, 0.07), "p_cream", r=1.0, n=2)

    head = m.geo("head_geo", "head")
    head.blob((0, -0.16, 0.3), (0.34, 0.3, 0.28), "p_cream", r=1.0, n=4)
    # 큰 눈 둘, 낮게(머리 아래쪽 반), 입은 없다.
    eyes = m.geo("eyes_geo", "head")
    for x in (0.085, -0.085):
        eyes.blob((x, -0.305, 0.28), (0.075, 0.03, 0.09), "k_deep", r=1.0, n=3)
    for x, bone in ((0.11, "ear_l"), (-0.11, "ear_r")):
        ear = m.geo(bone + "_geo", bone)
        ear.blob((x, -0.12, 0.47), (0.07, 0.05, 0.12), "p_mossd", r=1.0, n=2)
    tail = m.geo("tail_geo", "tail")
    tail.blob((0, 0.3, 0.2), (0.1, 0.1, 0.1), "p_moss", r=1.0, n=2)

    m.build()
    acts(m)
    return m


def acts(m):
    def k(**b):
        return b

    # idle(뜯기) 1.8초 고리: 머리를 깊이 숙여(0.5) 오물오물(0.7·0.9) → 들어 둘레를 본다(1.4).
    m.action("idle", 1.8, [
        (0.0, k(head={"r": (0, 0, 0)}, body={"r": (0, 0, 0)})),
        (0.5, k(head={"r": (38, 0, 0)}, body={"r": (10, 0, 0)}, tail={"r": (-10, 0, 0)})),
        (0.7, k(head={"r": (34, 0, 4)}, body={"r": (10, 0, 0)})),
        (0.9, k(head={"r": (38, 0, -4)}, body={"r": (10, 0, 0)})),
        (1.4, k(head={"r": (-6, 0, 10)}, ear_l={"r": (0, -12, 0)}, ear_r={"r": (0, 12, 0)})),
    ], loop=True)
    # walk(깡충) 0.4초 고리: 웅크렸다 솟고(몸통 찌그러짐) 내려앉는다.
    m.action("walk", 0.4, [
        (0.0, k(root={"l": (0, 0, 0)}, body={"s": (1.08, 1.08, 0.88)})),
        (0.15, k(root={"l": (0, 0, 0.14)}, body={"r": (-8, 0, 0), "s": (0.94, 0.94, 1.1)},
                 ear_l={"r": (20, 0, 0)}, ear_r={"r": (20, 0, 0)}, tail={"r": (16, 0, 0)})),
        (0.3, k(root={"l": (0, 0, 0.04)}, body={"r": (6, 0, 0)})),
    ], loop=True)
    m.action("hurt", 0.3, [
        (0.0, k(body={"r": (-14, 0, 0), "s": (0.9, 0.9, 1.1)}, head={"r": (-12, 0, 0)})),
        (0.3, k()),
    ])


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR):
    m = make()
    path = m.export(models_dir)
    m.preview(os.path.join(preview_dir, NAME + ".png"), "idle", 0.0)
    m.preview(os.path.join(preview_dir, NAME + "_idle.png"), "idle", 0.5)
    return path


if __name__ == "__main__":
    print(build())
