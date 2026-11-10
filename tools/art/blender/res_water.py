"""물 모으는 주민(V02, 작은 두더지 — 0122 · 모델 0132).

Meshy 몸 — 푸른 회색 둥근 몸·큰 머리, 반쯤 감은 큰 눈, 작은 분홍 코, 분홍 손바닥의 큰 파는 손, 하늘색 짧은 웃옷과 넓은 띠.
키 1.35m. 원본 `tools/art/sources/meshy/water_v1.glb`(Gemini 그림 → Meshy "Sleepy Blue Mole", CC BY 4.0)
→ `meshy_convert.py ... 1.35 700 128`. 동작은 제 일(`work`)과 돌아봄(`look`) — 제 일은 모델 앞(-Y)의 항아리를 본다
(`residents::WORK_FACE`).

    blender --background --python tools/art/blender/res_water.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import resident_parts as R  # noqa: E402

NAME = "res_water"
# 가운데가 잘 맞게 왔다(좌우 ±0.6). 웃옷 단이 0.2~0.27에서 ±0.4까지 퍼져 팔은 그 위만 — 단, 늘어뜨린 큰 손끝(±0.44 밖)은 아래도 팔.
CUT = R.Cut(head_z=0.74, arm_x=0.36, arm_z=(0.27, 0.70), leg_z=0.18, leg_split=0.0, paw_x=0.44)
BONES = {"body": (0, 0, 0.18), "head": (0, 0, 0.74), "arm_l": (0.33, 0, 0.62), "arm_r": (-0.33, 0, 0.62),
         "leg_l": (0.15, 0, 0.18), "leg_r": (-0.15, 0, 0.18)}


def make():
    return R.make(NAME, "water_v1_low.glb", CUT, BONES, acts)


def k(**bones):
    return bones


def acts(m):
    # work 3.6초 고리: 쪼그려 앉아 큰 두 손을 앞으로 모아 항아리를 받쳐 들고(0.5) 위(떨어지는 물)를 올려다보며 기다린다 →
    # 2.2 손을 조금 옮겨 고쳐 받고 → 고개를 숙여 항아리 안을 들여다본다(3.0) → 다시 올려다봄.
    hold = k(root={"l": (0, 0, -0.06)}, body={"r": (12, 0, 0)}, head={"r": (-14, 0, 0)},
             arm_l={"r": (-62, 18, 0)}, arm_r={"r": (-62, -18, 0)}, leg_l={"r": (-24, 0, 0)}, leg_r={"r": (-24, 0, 0)})
    m.action("work", 3.6, [
        (0.0, hold),
        (1.1, dict(hold, head={"r": (-18, 0, 4)}, body={"r": (12, 0, 0), "s": (1.02, 1.02, 0.98)})),
        (2.2, dict(hold, arm_l={"r": (-54, 12, 0)}, arm_r={"r": (-70, -24, 0)}, head={"r": (-10, 0, -4)})),
        (2.6, dict(hold, arm_l={"r": (-62, 18, 0)}, arm_r={"r": (-62, -18, 0)})),
        (3.0, dict(hold, head={"r": (16, 0, 0)}, body={"r": (16, 0, 0)})),
    ], loop=True)

    # look 2.2초 고리: 큰 손을 배 앞에 모으고(조심스러운 경계) 서서 숨 — 고개를 조금 움츠림.
    m.action("look", 2.2, [
        (0.0, k(body={"r": (4, 0, 0)}, head={"r": (6, 0, 0)}, arm_l={"r": (-24, 30, 0)}, arm_r={"r": (-24, -30, 0)})),
        (1.1, k(root={"l": (0, 0, -0.01)}, body={"r": (5, 0, 0), "s": (1.02, 1.02, 0.98)}, head={"r": (8, 0, 0)},
                arm_l={"r": (-28, 32, 0)}, arm_r={"r": (-28, -32, 0)})),
    ], loop=True)


PREVIEWS = [("look", 0.0, ""), ("work", 1.1, "_work")]
CHECK = [("work", 0.0), ("work", 2.2), ("work", 3.0), ("look", 1.1)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, check_dir=None):
    return R.build(make, NAME, PREVIEWS, CHECK, models_dir, preview_dir, check_dir)


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
