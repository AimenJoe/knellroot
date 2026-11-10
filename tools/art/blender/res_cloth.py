"""꿰매는 주민(V03, 어린 양 — 0122 · 모델 0132).

Meshy 몸 — 크림빛 양털 덩어리 머리, 베이지 얼굴·늘어진 귀, 반쯤 감은 큰 눈, 어깨에 걸친 조각보 숄(바랜 장미·세이지·모래)과
십자 바느질, 발굽·둥근 손. 키 1.35m. 원본 `tools/art/sources/meshy/cloth_v1.glb`(Gemini 그림 v2 → Meshy "Patchwork Lamb",
CC BY 4.0) → `meshy_convert.py ... 1.35 700 128`. 동작은 제 일(`work`)과 돌아봄(`look`) — 제 일은 모델 앞(-Y)의 천막 천을
꿰맨다(`residents::WORK_FACE`).

    blender --background --python tools/art/blender/res_cloth.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import resident_parts as R  # noqa: E402

NAME = "res_cloth"
# 가운데가 잘 맞게 왔다(다리 사이 틈 x 0). 숄 자락이 어깨 밖(±0.3 남짓)까지 내려와 팔과 같이 움직인다.
CUT = R.Cut(head_z=0.72, arm_x=0.28, arm_z=(0.30, 0.64), leg_z=0.20, leg_split=0.0)
BONES = {"body": (0, 0, 0.2), "head": (0, 0, 0.72), "arm_l": (0.25, 0, 0.6), "arm_r": (-0.25, 0, 0.6),
         "leg_l": (0.1, 0, 0.2), "leg_r": (-0.1, 0, 0.2)}


def make():
    return R.make(NAME, "cloth_v1_low.glb", CUT, BONES, acts)


def k(**bones):
    return bones


def acts(m):
    # work 2.8초 고리: 왼손으로 앞의 천을 받쳐 들고(가슴 높이) 고개를 숙여 바늘 자리를 본다 → 오른손이 천에 바늘을 꽂고(0.5)
    # 실을 바깥으로 길게 당긴다(1.1) → 다시 천으로(1.6) → 한 번 더 당김(2.2). 실을 당길 때 몸이 조금 따라 돈다.
    base = k(body={"r": (6, 0, 0)}, head={"r": (16, 0, 0)}, arm_l={"r": (-62, 14, 0)}, arm_r={"r": (-58, -16, 0)})
    pull = dict(base, body={"r": (4, 0, -8)}, head={"r": (10, 0, -6)}, arm_r={"r": (-40, -52, 0)})
    m.action("work", 2.8, [
        (0.0, base),
        (0.5, dict(base, arm_r={"r": (-68, -6, 0)})),
        (1.1, pull),
        (1.6, dict(base, arm_r={"r": (-66, -8, 0)})),
        (2.2, pull),
    ], loop=True)

    # look 2.6초 고리: 손을 내리고 조용히 선다 — 고개를 조금 갸웃(참을성 있는 무심함).
    m.action("look", 2.6, [
        (0.0, k(body={"r": (2, 0, 0)}, head={"r": (4, 6, 0)}, arm_l={"r": (-6, 8, 0)}, arm_r={"r": (-6, -8, 0)})),
        (1.3, k(root={"l": (0, 0, -0.01)}, body={"r": (3, 0, 0), "s": (1.02, 1.02, 0.98)}, head={"r": (6, 3, 0)},
                arm_l={"r": (-4, 6, 0)}, arm_r={"r": (-4, -6, 0)})),
    ], loop=True)


PREVIEWS = [("look", 0.0, ""), ("work", 1.1, "_work")]
CHECK = [("work", 0.0), ("work", 0.5), ("work", 1.1), ("look", 1.3)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, check_dir=None):
    return R.build(make, NAME, PREVIEWS, CHECK, models_dir, preview_dir, check_dir)


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
