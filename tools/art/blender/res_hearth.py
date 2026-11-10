"""불 지키는 주민(V01, 늙은 오소리 — 0122 · 모델 0132).

Meshy 몸 — 크림빛 얼굴 띠·갈색 볼, 졸린 눈, 머리에 묶은 갈색 두건, 그을음빛 앞치마, 그을린 손발(Meshy가 짧은 꼬리를 더했다).
키 1.3m(방랑자보다 조금 작게). 원본 `tools/art/sources/meshy/hearth_v1.glb`(Gemini 그림 → Meshy "Little Apron Bandit",
CC BY 4.0) → `meshy_convert.py ... 1.3 700 128`. 주민은 싸우지 않는다 — 동작은 제 일(`work`)과 돌아봄(`look`) 둘
(`render::stage_props`가 방랑자 거리로 고른다). 제 일은 모델 앞(-Y)의 아궁이를 본다(`residents::WORK_FACE`).

    blender --background --python tools/art/blender/res_hearth.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import resident_parts as R  # noqa: E402

NAME = "res_hearth"
# 두건 매듭이 +X로 뻗어 meshy_convert(머리 가운데를 원점으로)가 몸을 −X로 0.07 비껴 두었다(다리 사이 틈 기준).
CUT = R.Cut(head_z=0.78, arm_x=0.24, arm_z=(0.28, 0.72), leg_z=0.17, leg_split=0.03, shift_x=0.07)
BONES = {"body": (0, 0, 0.18), "head": (0, -0.02, 0.78), "arm_l": (0.22, 0, 0.66), "arm_r": (-0.22, 0, 0.66),
         "leg_l": (0.12, 0, 0.18), "leg_r": (-0.10, 0, 0.18)}


def make():
    return R.make(NAME, "hearth_v1_low.glb", CUT, BONES, acts)


def k(**bones):
    return bones


def acts(m):
    # work 3.2초 고리: 앞의 아궁이로 허리를 숙여 들여다보고(0.6) 오른손으로 두 번 쑤신다(1.0·1.5) → 허리를 펴고
    # 늙은 등을 뒤로 젖혔다가(2.4) 다시 숙인다. 왼손은 무릎에.
    bend = k(body={"r": (22, 0, 0)}, head={"r": (14, 0, 0)}, arm_l={"r": (-10, 22, 0)}, arm_r={"r": (-40, 0, 0)},
             leg_l={"r": (-12, 0, 0)}, leg_r={"r": (-12, 0, 0)})
    m.action("work", 3.2, [
        (0.0, bend),
        (0.6, dict(bend, arm_r={"r": (-50, 0, 0)})),
        (1.0, dict(bend, arm_r={"r": (-72, 0, -6)})),
        (1.25, dict(bend, arm_r={"r": (-46, 0, 0)})),
        (1.5, dict(bend, arm_r={"r": (-74, 0, 6)})),
        (1.8, dict(bend, arm_r={"r": (-40, 0, 0)})),
        (2.4, k(body={"r": (-6, 0, 0)}, head={"r": (-12, 0, 0)}, arm_l={"r": (14, 18, 0)}, arm_r={"r": (14, -18, 0)},
                leg_l={"r": (0, 0, 0)}, leg_r={"r": (0, 0, 0)})),
        (2.8, k(body={"r": (4, 0, 0)}, head={"r": (6, 0, 0)}, arm_l={"r": (0, 10, 0)}, arm_r={"r": (0, -10, 0)},
                leg_l={"r": (-4, 0, 0)}, leg_r={"r": (-4, 0, 0)})),
    ], loop=True)

    # look 2.4초 고리: 조금 굽은 채 서서(늙은 어깨) 천천히 숨 — 고개가 아주 조금 갸웃. 손은 앞치마 앞에 늘어뜨림.
    m.action("look", 2.4, [
        (0.0, k(body={"r": (8, 0, 0)}, head={"r": (-6, 4, 0)}, arm_l={"r": (-6, 8, 0)}, arm_r={"r": (-6, -8, 0)})),
        (1.2, k(root={"l": (0, 0, -0.01)}, body={"r": (10, 0, 0), "s": (1.02, 1.02, 0.98)}, head={"r": (-4, -2, 0)},
                arm_l={"r": (-4, 6, 0)}, arm_r={"r": (-4, -6, 0)})),
    ], loop=True)


PREVIEWS = [("look", 0.0, ""), ("work", 1.0, "_work")]
CHECK = [("work", 0.0), ("work", 1.5), ("work", 2.4), ("look", 1.2)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, check_dir=None):
    return R.build(make, NAME, PREVIEWS, CHECK, models_dir, preview_dir, check_dir)


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
