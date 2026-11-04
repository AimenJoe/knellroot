"""0038 Meshy(이미지 → 3D)에 넣을 참고 그림: 방랑자(지금 모델)·잔재(시안)를 정면·옆·뒤·3/4로, 민짜 바탕, 1024px.

    blender --background --python tools/art/blender/meshy_refs.py
→ docs/art/meshy-0038/<이름>_{front,side,back,threeq}.png
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import concepts  # noqa: E402
import wanderer  # noqa: E402

OUT = os.path.join(C.REPO, "docs", "art", "meshy-0038")
VIEWS = (("front", 0.0, 4.0), ("side", -90.0, 4.0), ("back", 180.0, 4.0), ("threeq", -35.0, 18.0))


def shoot(m, name, action):
    for v, yaw, elev in VIEWS:
        m.preview(os.path.join(OUT, f"{name}_{v}.png"), action, 0.0, size=1024, yaw=yaw, elev=elev, floor=False)


os.makedirs(OUT, exist_ok=True)
m = wanderer.make()
shoot(m, "wanderer", "idle")
m = concepts.husk()
m.build()
shoot(m, "husk", None)
print("[meshy_refs] done")
