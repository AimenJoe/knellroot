"""등딱지짐승(앞을 막는 적, 네 발 — 0120 C2 · 모델 0132).

Meshy 몸 — 잿빛 큰 등딱지(앞이 챙처럼 얼굴을 덮는다), 등딱지 금에서 솟은 보라 결정 뿌리, 챙 아래 호박빛 큰 눈 둘(glow_eyes),
짧고 굵은 다리 넷. 키 1.2m(뿌리 끝까지, 등딱지 ≈ 0.95)·길이 1.25m·폭 1.0m. 원본 `tools/art/sources/meshy/shellback_v1.glb`
(Gemini 그림 → Meshy, CC BY 4.0) → `meshy_convert.py ... 1.2 700 128`.

    blender --background --python tools/art/blender/shellback.py
"""
import os
import sys

from mathutils import Matrix

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "shellback"

MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "shellback_v1_low.glb")
MESHY_PARTS = {"body_geo": "body", "head_geo": "head", "glow_eyes": "head",
               "leg_fl_geo": "leg_fl", "leg_fr_geo": "leg_fr", "leg_bl_geo": "leg_bl", "leg_br_geo": "leg_br"}
# meshy_convert는 위 45%의 가운데를 원점에 둔다 — 한쪽(+X)에 솟은 결정이 가운데를 끌어 몸이 −X로 0.11m 비껴 있다.
# 자를 때는 옮긴 자리(x + SHIFT_X)로 재고, 부위를 옮겨 머리·다리 가운데가 원점에 오게 한다.
SHIFT_X = 0.11
# 치수(옮긴 자리, 앞 = -Y, 캐릭터 왼쪽 = +X) — 정면·옆 직교 렌더에 눈금을 그어 잰 것.
LEG_Z = 0.20  # 이보다 아래는 다리(배딱지 아래 ≈ 0.22)
HEAD_Y = -0.28  # 이보다 앞, 챙 아래(HEAD_Z)·가운데(HEAD_X)면 얼굴
HEAD_Z = (0.17, 0.50)
HEAD_X = 0.26
EYE = (0.10, -0.47, 0.365)  # 눈 가운데(±x, y, z) — 텍스처의 호박빛 점을 3D로 되짚어 잰 것
EYE_R = (0.055, 0.06)
LEGS = (("fl", 0.24, -0.20), ("fr", -0.24, -0.20), ("bl", 0.22, 0.26), ("br", -0.22, 0.26))
GREY_K = 0.6  # 잿빛 몸을 어둡게(잔재·가시짐승과 같은 까닭·같은 값)
GREY_SAT = 0.18


def classify(c):
    """면 가운데 자리(가져온 그대로) → 부위."""
    x, y, z = c.x + SHIFT_X, c.y, c.z
    side = "l" if x > 0 else "r"
    if y < HEAD_Y and HEAD_Z[0] < z < HEAD_Z[1] and abs(x) < HEAD_X:
        ex, ey, ez = EYE
        rx, rz = EYE_R
        if y < ey + 0.06 and ((abs(x) - ex) / rx) ** 2 + ((z - ez) / rz) ** 2 <= 1.0:
            return "glow_eyes"
        return "head_geo"
    if z < LEG_Z:
        return f"leg_{'f' if y < 0.0 else 'b'}{side}_geo"
    return "body_geo"


def make():
    m = C.Model(NAME)
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0.0, 0.40), "root")
    m.bone("head", (0, -0.30, 0.34), "body")
    for n, x, y in LEGS:
        m.bone(f"leg_{n}", (x, y, LEG_Z + 0.03), "root")
    objs = m.external(MESHY_LOW, classify, MESHY_PARTS)
    for o in objs.values():
        o.data.transform(Matrix.Translation((SHIFT_X, 0.0, 0.0)))
    C.darken_grey(m.ext_img, GREY_K, GREY_SAT)
    m.build()
    acts(m)
    return m


def legs(fl, fr, bl, br):
    return {"leg_fl": {"r": (fl, 0, 0)}, "leg_fr": {"r": (fr, 0, 0)},
            "leg_bl": {"r": (bl, 0, 0)}, "leg_br": {"r": (br, 0, 0)}}


def k(*parts, **bones):
    out = {}
    for p in parts:
        out.update(p)
    for b, v in bones.items():
        out[b] = v
    return out


# 머리를 등딱지 안으로(챙 아래로 쏙): 뒤로·조금 아래로.
TUCK = {"l": (0, 0.12, -0.03)}
# 앞을 드는 자세: root를 뒷발(y 0.26, 땅) 둘레로 θ 들고, 뒷다리는 +θ로 되돌려 땅을 짚게(가시짐승 charge_windup과 같은 셈).
REAR_Y = 0.26


def rear(theta):
    import math
    t = math.radians(theta)
    return {"r": (-theta, 0, 0), "l": (0, REAR_Y * (1 - math.cos(t)), REAR_Y * math.sin(t))}


def acts(m):
    # idle 1.6초 고리: 숨(등딱지가 오르내림), 머리가 조금 나왔다 들어감
    m.action("idle", 1.6, [
        (0.0, k(body={"r": (0, 0, 0)}, head={"r": (0, 0, -3)})),
        (0.8, k(root={"l": (0, 0, -0.012)}, body={"r": (1.5, 0, 0), "s": (1.02, 1.02, 0.98)},
                head={"r": (4, 0, 3), "l": (0, -0.02, 0)})),
    ], loop=True)

    # walk 1.0초 고리: 대각선 두 발씩(왼앞+오른뒤 ↔ 오른앞+왼뒤), 등딱지가 좌우로 기우뚱.
    # 0173: 짧은 발만으로는 게임 거리에서 걷는 것으로 안 읽혔다 → 온몸이 디딘 쪽으로 기울고(root 굴림), 디딜 때 납작·뜰 때 볼록,
    # 머리는 걸음마다 내밀었다 들어간다.
    m.action("walk", 1.0, [
        (0.0, k(legs(-32, 24, 24, -32), root={"l": (0, 0, -0.02), "r": (0, -8, 0)},
                body={"r": (2, -4, 5), "s": (1.05, 1.05, 0.92)}, head={"r": (0, 0, -9), "l": (0, -0.03, 0)})),
        (0.25, k(legs(0, 0, 0, 0), root={"l": (0, 0, 0.07), "r": (0, 0, 0)},
                 body={"r": (-3, 0, 0), "s": (0.97, 0.97, 1.06)}, head={"r": (5, 0, 0), "l": (0, 0.02, 0)})),
        (0.5, k(legs(24, -32, -32, 24), root={"l": (0, 0, -0.02), "r": (0, 8, 0)},
                body={"r": (2, 4, -5), "s": (1.05, 1.05, 0.92)}, head={"r": (0, 0, 9), "l": (0, -0.03, 0)})),
        (0.75, k(legs(0, 0, 0, 0), root={"l": (0, 0, 0.07), "r": (0, 0, 0)},
                 body={"r": (-3, 0, 0), "s": (0.97, 0.97, 1.06)}, head={"r": (5, 0, 0), "l": (0, 0.02, 0)})),
    ], loop=True)

    # attack 1.3초(0073 들기 → 버팀 → 내리침, render STRIKE_KEYS ("shellback", "attack", 0.6, 0.72)):
    # 움츠림(0.15) → 머리를 쏙 넣고 앞을 35° 들어 앞발이 허공에 늘어짐(0.6 정점 — 예고 동안 여기서 버틴다)
    # → 0.72 쿵 내려앉아 등딱지가 납작(= 맞는 틱, 둘레 원판) → 버팀 → 1.3 쉬는 자세.
    up = rear(35)
    m.action("attack", 1.3, [
        (0.0, k(legs(0, 0, 0, 0))),
        (0.15, k(legs(-8, -8, 8, 8), root={"l": (0, 0.02, -0.04)}, body={"r": (4, 0, 0), "s": (1.08, 1.08, 0.88)},
                 head=TUCK)),
        (0.45, k(legs(14, 22, 35, 35), root=up, body={"r": (-4, 0, 0), "s": (0.96, 0.96, 1.06)}, head=TUCK)),
        (0.6, k(legs(10, 26, 38, 38), root=rear(38), body={"r": (-6, 0, 0), "s": (0.95, 0.95, 1.08)}, head=TUCK)),
        (0.72, k(legs(14, 14, -8, -8), root={"l": (0, 0, -0.05)}, body={"r": (4, 0, 0), "s": (1.16, 1.16, 0.8)},
                 head=TUCK)),
        (0.9, k(legs(10, 10, -6, -6), root={"l": (0, 0, -0.04)}, body={"r": (3, 0, 0), "s": (1.1, 1.1, 0.86)},
                head={"l": (0, 0.08, -0.02)})),
        (1.3, k(legs(0, 0, 0, 0), body={"r": (0, 0, 0)}, head={"r": (0, 0, -3)})),
    ], linear=(0.6,))

    # hurt 0.3초: 머리를 쏙 넣고 등딱지가 뒤로 밀림
    m.action("hurt", 0.3, [
        (0.0, k(legs(0, 0, 0, 0), head={"r": (0, 0, -3)})),
        (0.05, k(legs(10, 10, -8, -8), root={"l": (0, 0.06, 0.01)}, body={"r": (-8, 6, 0)}, head=TUCK)),
        (0.3, k(legs(0, 0, 0, 0), head={"r": (0, 0, -3)})),
    ])


PREVIEWS = [("idle", 0.0, ""), ("attack", 0.6, "_attack")]
CHECK = [("walk", 0.0), ("walk", 0.25), ("walk", 0.5), ("attack", 0.15), ("attack", 0.6), ("attack", 0.72), ("attack", 0.9), ("hurt", 0.05)]


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
