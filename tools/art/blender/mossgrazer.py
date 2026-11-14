"""이끼먹이(0185 — 잔뿌리의 정원의 비적대 생명체, 1차 코드 모델 · 2차는 그림 → Meshy).

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


def make():
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
    return m


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR):
    m = make()
    path = m.export(models_dir)
    m.preview(os.path.join(preview_dir, NAME + ".png"), "idle", 0.0)
    m.preview(os.path.join(preview_dir, NAME + "_idle.png"), "idle", 0.5)
    return path


if __name__ == "__main__":
    print(build())
