"""꼬투리(멀리서 심는 적, 깡충 뛰는 네 발 — 0120 C2 · 모델 0132).

Meshy 몸 — 잿빛 동그란 날쥐·개구리 몸에 큰 머리, 늘어진 귀, 호박빛 큰 눈 둘(glow_eyes), 접힌 긴 뒷발, 등에 몸만 한
마른 올리브빛 씨앗 꼬투리(위가 갈라짐) + 틈에서 말려 나온 보라 뿌리 하나. 키 0.9m(뿌리 끝까지, 꼬투리 ≈ 0.75)·길이 0.9m.
원본 `tools/art/sources/meshy/pod_v1.glb`(Gemini 그림 → Meshy "Sproutback Frog", CC BY 4.0) → `meshy_convert.py ... 0.9 700 128`.

    blender --background --python tools/art/blender/pod.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "pod"

MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "pod_v1_low.glb")
MESHY_PARTS = {"body_geo": "body", "head_geo": "head", "glow_eyes": "head", "seed_geo": "seed",
               "leg_bl_geo": "leg_bl", "leg_br_geo": "leg_br"}
# 치수(앞 = -Y, 캐릭터 왼쪽 = +X) — 정면·옆·위 직교 렌더(텍스처째)에 눈금을 그어 잰 것.
HEAD_Y = -0.15  # 이보다 앞, HEAD_Z 위는 머리(귀 포함)
HEAD_Z = 0.16
EYE = (0.125, 0.40)  # 눈 가운데(±x, z)
EYE_R = (0.07, 0.07)
# 긴 뒷발은 앞발 바로 뒤까지 뻗어 있어 앞뒤 자리만으로는 못 가른다(아래에서 본 그림) → 뒷다리 = LEG_Y 뒤에서
# 옆(|x| > HIND_X)의 넓적다리 + 긴 발 한 덩어리. 앞발은 발끝만 작게 남아 돌리면 떨어진 조각처럼 보여 몸에 둔다.
LEG_Y = -0.24
HIND_Z = 0.30
HIND_X = 0.12
SEED_MIN_Y = -0.25  # 꼬투리 색 면이라도 이보다 앞은 아니다(머리의 그늘)
GREY_K = 0.6  # 잿빛 몸을 어둡게(다른 적과 같은 까닭·같은 값) — 올리브 꼬투리·보라 뿌리·호박빛 눈은 그대로
GREY_SAT = 0.18


def _seed_faces():
    """꼬투리(올리브)·뿌리(보라) 색 면의 가운데 자리 — 몸과 맞닿은 경계가 비스듬해 자리로는 못 가른다(0132)."""
    def pick(h, s, v):
        return (0.09 < h < 0.22 and s > 0.25 and v > 0.15) or (0.62 < h < 0.9 and s > 0.2)
    return [c for c, hsv in C.face_colors(MESHY_LOW) if pick(*hsv)]


SEED = []


def _is_seed(x, y, z):
    return C.near_any(SEED, (x, y, z))


def classify(c):
    """면 가운데 자리 → 부위."""
    x, y, z = c.x, c.y, c.z
    side = "l" if x > 0 else "r"
    if y > SEED_MIN_Y and _is_seed(x, y, z):
        return "seed_geo"
    if y < HEAD_Y and z > HEAD_Z:
        ex, ez = EYE
        rx, rz = EYE_R
        if y < -0.3 and ((abs(x) - ex) / rx) ** 2 + ((z - ez) / rz) ** 2 <= 1.0:
            return "glow_eyes"
        return "head_geo"
    if y >= LEG_Y and z < HIND_Z and abs(x) > HIND_X:
        return f"leg_b{side}_geo"
    return "body_geo"


def make():
    SEED[:] = _seed_faces()
    print(f"[pod] seed-colored faces {len(SEED)}")
    m = C.Model(NAME)
    m.bone("root", (0, 0, 0))
    # 몸 피벗은 바닥(앞발이 몸에 붙어 있다 — 납작해지거나 숙여도 앞발이 땅 아래로 들어가지 않게).
    m.bone("body", (0, 0.0, 0.0), "root")
    m.bone("head", (0, -0.15, 0.30), "body")
    m.bone("seed", (0, 0.08, 0.42), "body")  # 꼬투리 밑동(등에 붙은 곳)
    for n, x, y, z in (("bl", 0.2, 0.15, 0.26), ("br", -0.2, 0.15, 0.26)):
        m.bone(f"leg_{n}", (x, y, z), "root")
    m.external(MESHY_LOW, classify, MESHY_PARTS)
    C.darken_grey(m.ext_img, GREY_K, GREY_SAT)
    m.build()
    acts(m)
    return m


def legs(fl, fr, bl, br):
    """앞발(fl·fr)은 몸에 붙어 있어 쓰지 않는다 — 표를 네 발 꼴로 읽히게 둔 자리. 뒷다리 X+ = 발이 뒤로(밀기)."""
    return {"leg_bl": {"r": (bl, 0, 0)}, "leg_br": {"r": (br, 0, 0)}}


def k(*parts, **bones):
    out = {}
    for p in parts:
        out.update(p)
    for b, v in bones.items():
        out[b] = v
    return out


REST = k(legs(0, 0, 0, 0), head={"r": (0, 0, 0)}, seed={"r": (0, 0, 0)})


def acts(m):
    # idle 1.4초 고리: 숨, 머리가 갸웃, 꼬투리가 한 박자 늦게 흔들림
    m.action("idle", 1.4, [
        (0.0, k(body={"r": (0, 0, 0)}, head={"r": (0, 4, 0)}, seed={"r": (2, -3, 0)})),
        (0.7, k(body={"r": (2, 0, 0), "s": (1.03, 1.03, 0.97)},
                head={"r": (4, -4, 0)}, seed={"r": (-2, 3, 0)})),
    ], loop=True)

    # walk 0.5초 고리: 짧은 깡충(속도 1.6m/s) — 움츠림 → 뒷발로 밀기 → 떠서 앞발 뻗기 → 내려앉기. 꼬투리는 늦게 따라온다.
    m.action("walk", 0.5, [
        (0.0, k(legs(-6, -6, -18, -18), body={"r": (3, 0, 0), "s": (1.06, 1.06, 0.92)},
                head={"r": (-4, 0, 0)}, seed={"r": (10, 0, 0)})),
        (0.12, k(legs(-16, -16, 22, 22), root={"l": (0, 0, 0.06)}, body={"r": (-5, 0, 0), "s": (0.96, 0.96, 1.06)},
                 head={"r": (-6, 0, 0)}, seed={"r": (-6, 0, 0)})),
        (0.28, k(legs(-30, -30, 20, 20), root={"l": (0, 0, 0.08)}, body={"r": (-4, 0, 0)}, head={"r": (2, 0, 0)},
                 seed={"r": (-14, 0, 0)})),
        (0.4, k(legs(-12, -12, -10, -10), root={"l": (0, 0, 0.0)}, body={"r": (3, 0, 0), "s": (1.04, 1.04, 0.95)},
                head={"r": (4, 0, 0)}, seed={"r": (6, 0, 0)})),
    ], loop=True)

    # hop 0.4초(진행으로 본다 — render: 뛰기 동안 h.t/h.dur, 0132): 물러나기·턱 뛰기. 긴 뒷발을 다 펴서 미는 것이 보이게.
    # 0 움츠림 → 0.12 뒷발로 밀며(뒤로 26° — 넓적다리째 도는 덩어리라 더 돌리면 몸과 벌어진다) 몸이 길게 → 0.3 공중에서 몸을 말고 꼬투리가 뒤로 처짐 → 0.36 앞발로 받음 → 0.4 납작.
    m.action("hop", 0.4, [
        (0.0, k(legs(-4, -4, -20, -20), body={"r": (3, 0, 0), "s": (1.08, 1.08, 0.88)},
                head={"r": (-6, 0, 0)}, seed={"r": (8, 0, 0)})),
        (0.12, k(legs(-24, -24, 26, 26), body={"r": (-6, 0, 0), "s": (0.94, 0.94, 1.1)}, head={"r": (-10, 0, 0)},
                 seed={"r": (-8, 0, 0)})),
        (0.3, k(legs(-30, -30, 10, 10), body={"r": (-4, 0, 0)}, head={"r": (4, 0, 0)}, seed={"r": (-18, 0, 0)})),
        (0.36, k(legs(-10, -10, -14, -14), body={"r": (3, 0, 0), "s": (1.1, 1.1, 0.86)},
                 head={"r": (8, 0, 0)}, seed={"r": (12, 0, 0)})),
        (0.4, k(legs(0, 0, -6, -6), body={"r": (2, 0, 0), "s": (1.03, 1.03, 0.96)}, head={"r": (2, 0, 0)},
                seed={"r": (4, 0, 0)})),
    ])

    # plant 1.0초(0073 들기 → 버팀 → 터짐, render STRIKE_KEYS ("pod", "plant", 0.45, 0.6)):
    # 심는 자리는 예고 첫 틱에 정해진다 → 0.18 꼬투리를 앞으로 휙 젖혀 씨앗을 던진다(머리 숙임) → 0.45 납작 웅크려
    # 꼬투리가 부풀어 오른다(예고 동안 여기서 버틴다 — "곧 터진다") → 0.6 꼬투리를 꽉 오므리며 몸이 퐁 솟음(= 터지는 틱) → 쉼.
    m.action("plant", 1.0, [
        (0.0, REST),
        (0.08, k(legs(0, 0, -8, -8), body={"r": (-8, 0, 0)}, head={"r": (-10, 0, 0)}, seed={"r": (-22, 0, 0)})),
        (0.18, k(legs(-10, -10, 10, 10), body={"r": (3, 0, 0)}, head={"r": (14, 0, 0)}, seed={"r": (40, 0, 0)})),
        (0.32, k(legs(-4, -4, -16, -16), body={"r": (3, 0, 0), "s": (1.08, 1.08, 0.88)},
                 head={"r": (4, 0, 0)}, seed={"r": (0, 0, 0), "s": (1.12, 1.12, 1.12)})),
        (0.45, k(legs(-4, -4, -20, -20), body={"r": (3, 0, 0), "s": (1.1, 1.1, 0.85)},
                 head={"r": (6, 0, 0)}, seed={"r": (-6, 0, 0), "s": (1.22, 1.22, 1.22)})),
        (0.6, k(legs(-12, -12, 18, 18), root={"l": (0, 0, 0.04)}, body={"r": (-5, 0, 0), "s": (0.94, 0.94, 1.08)},
                head={"r": (-8, 0, 0)}, seed={"r": (6, 0, 0), "s": (0.82, 0.82, 0.82)})),
        (0.75, k(legs(-4, -4, -6, -6), body={"r": (3, 0, 0), "s": (1.04, 1.04, 0.96)}, seed={"r": (-4, 0, 0), "s": (1.04, 1.04, 1.04)})),
        (1.0, REST),
    ], linear=(0.45,))

    # hurt 0.3초: 움찔 뒤로, 꼬투리가 크게 흔들림
    m.action("hurt", 0.3, [
        (0.0, REST),
        (0.05, k(legs(10, 10, -10, -10), root={"l": (0, 0.05, 0.02)}, body={"r": (-12, 0, 6)}, head={"r": (-14, 0, -10)},
                 seed={"r": (-20, 8, 0)})),
        (0.3, REST),
    ])


PREVIEWS = [("idle", 0.0, ""), ("plant", 0.45, "_plant")]
CHECK = [("walk", 0.12), ("walk", 0.28), ("hop", 0.0), ("hop", 0.12), ("hop", 0.3), ("plant", 0.18), ("plant", 0.45),
         ("plant", 0.6), ("hurt", 0.05)]


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

