"""불 지키는 주민(V01, 늙은 오소리 — 0122 · 모델 0132).

Meshy 몸 — 크림빛 얼굴 띠·갈색 볼, 졸린 눈, 머리에 묶은 갈색 두건, 그을음빛 앞치마, 그을린 손발(Meshy가 짧은 꼬리를 더했다).
키 1.3m(방랑자보다 조금 작게). 원본 `tools/art/sources/meshy/hearth_v1.glb`(Gemini 그림 → Meshy "Little Apron Bandit",
CC BY 4.0) → `meshy_convert.py ... 1.3 700 128`. 주민은 싸우지 않는다 — 동작은 제 일(`work`)과 돌아봄(`look`) 둘
(`render::stage_props`가 방랑자 거리로 고른다). 제 일은 모델 앞(-Y)의 아궁이를 본다(`residents::WORK_FACE`).

    blender --background --python tools/art/blender/res_hearth.py
"""
import os
import sys

from mathutils import Matrix

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "res_hearth"

MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "hearth_v1_low.glb")
MESHY_PARTS = {"body_geo": "body", "head_geo": "head", "arm_l_geo": "arm_l", "arm_r_geo": "arm_r",
               "leg_l_geo": "leg_l", "leg_r_geo": "leg_r"}
# meshy_convert는 위 45%(머리)의 가운데를 원점에 둔다 — 두건 매듭이 +X로 뻗어 몸이 −X로 조금 비껴 왔다(다리 사이 틈 기준 0.07).
SHIFT_X = 0.07
# 치수(옮긴 자리, 앞 = -Y, 캐릭터 왼쪽 = +X) — 정면·옆·위 직교 렌더(텍스처째)로 잰 것.
HEAD_Z = 0.78  # 이보다 위는 머리(두건·귀)
ARM_X = 0.24  # 몸보다 밖이면 팔(어깨 아래 ~ 손)
ARM_Z = (0.28, 0.72)
LEG_Z = 0.17
LEG_SPLIT = 0.03  # 두 발 사이 틈


def classify(c):
    """면 가운데 자리(가져온 그대로) → 부위."""
    x, z = c.x + SHIFT_X, c.z
    if z > HEAD_Z:
        return "head_geo"
    if ARM_Z[0] < z < ARM_Z[1] and abs(x) > ARM_X:
        return "arm_l_geo" if x > 0 else "arm_r_geo"
    if z < LEG_Z:
        return "leg_l_geo" if x > LEG_SPLIT else "leg_r_geo"
    return "body_geo"


def make():
    m = C.Model(NAME)
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0, 0.18), "root")  # 엉덩이 — 숙이면 위가 앞으로
    m.bone("head", (0, -0.02, 0.78), "body")
    m.bone("arm_l", (0.22, 0, 0.66), "body")
    m.bone("arm_r", (-0.22, 0, 0.66), "body")
    m.bone("leg_l", (0.12, 0, 0.18), "root")
    m.bone("leg_r", (-0.10, 0, 0.18), "root")
    objs = m.external(MESHY_LOW, classify, MESHY_PARTS)
    for o in objs.values():
        o.data.transform(Matrix.Translation((SHIFT_X, 0.0, 0.0)))
    m.build()
    acts(m)
    return m


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
    m = make()
    path = m.export(models_dir)
    for act, t, suffix in PREVIEWS:
        m.preview(os.path.join(preview_dir, f"{NAME}{suffix}.png"), act, t, yaw=-35)
    if check_dir:
        for act, t in CHECK:
            m.preview(os.path.join(check_dir, f"{NAME}_{act}_{int(t * 100):03d}.png"), act, t, yaw=-70)
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
