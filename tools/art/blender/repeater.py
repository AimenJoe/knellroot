"""되풀이꾼(작업층의 재활동체, 곡괭이 — 0208 5절 · 모델 0234).

Meshy 몸 — 잿빛 두더지 광부, 찌그러진 청동 안전모(꺼진 등), 큰 삽 같은 앞발 둘, 어깨의 보라 결정, 호박빛 반쯤 감긴 큰 눈(glow_eyes),
앞치마. 원본 `tools/art/sources/meshy/repeater_v1.glb`(나노바나나 Pro 그림 `tools/art/sources/ai/knellroot/repeater_v1_front.png`
→ Meshy 7.1 API, CC BY 4.0 — Meshy 6은 엉덩이 뒤로 큰 덩어리를 지어내서 7.1로 다시) → `meshy_convert.py ... 1.6 1400 128`
(7.1 메시는 정점 ~330 아래로 안 줄어 다리가 뾰족해졌다 — 1400으로).
곡괭이는 단단한 소품이라 코드 도형(0208 10절 — 그림에는 그리지 않았다): 오른손 뼈 `pick`, 쉬는 자세에서 자루가 앞(-Y)으로, 날은 세로.
동작은 잔재의 표 그대로(`root_husk.acts` — idle·walk·attack·hurt, 뼈 이름이 같다): 두 팔을 머리 위로 들었다 내려찍기 = 곡괭이질.

    blender --background --python tools/art/blender/repeater.py
"""
import os
import sys

from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import root_husk  # noqa: E402

NAME = "repeater"

MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "repeater_v1_low.glb")
MESHY_PARTS = {"head_geo": "head", "glow_eyes": "head", "body_geo": "body", "arm_l_geo": "arm_l", "arm_r_geo": "arm_r",
               "leg_l_geo": "leg_l", "leg_r_geo": "leg_r"}
# 치수(키 1.6m, 앞 = -Y, 캐릭터 왼쪽 = +X) — 정면·옆·위 직교 렌더에 0.05m 눈금을 그어 잰 것.
NECK_Z = 0.76  # 머리(안전모 포함) 아래 끝
HIP_Z = 0.17  # 다리 위 끝(가랑이)
SHOULDER = (0.28, 0.0, 0.62)
PAW = (0.64, -0.08, 0.28)  # 삽 앞발 가운데
ARM_R = 0.22
ARM_X = 0.31  # 이보다 바깥만 팔(몸통 옆면·어깨 결정은 몸)
PAW_X = 0.34
EYES = []
GREY_K = 0.6  # 잿빛 몸을 어둡게(다른 적과 같은 까닭·같은 값)
GREY_SAT = 0.18


def _eye_faces():
    """호박빛 눈 — 얼굴 앞의 채도 높은 주황 면(안전모의 청동은 채도가 낮다)."""
    return [c for c, (h, s, v) in C.face_colors(MESHY_LOW)
            if 0.04 < h < 0.15 and s > 0.45 and v > 0.5 and c[1] < -0.15 and 0.85 < c[2] < 1.1]


def classify(c):
    x, y, z = c.x, c.y, c.z
    if z > NECK_Z:
        return "glow_eyes" if C.near_any(EYES, (x, y, z)) else "head_geo"
    side = "l" if x > 0 else "r"
    ax = abs(x)
    if ax > ARM_X and root_husk._seg_dist(Vector((ax, y, z)), SHOULDER, PAW) < ARM_R:
        return f"arm_{side}_geo"
    if ax > PAW_X and z < 0.6:  # 앞발 밑의 발톱 — 몸통(|x| ≤ 0.32) 바깥의 낮은 면은 다 앞발
        return f"arm_{side}_geo"
    if z < HIP_Z:
        return f"leg_{side}_geo"
    return "body_geo"


def pickaxe(m):
    """오른손(−X)에 쥔 곡괭이: 자루는 앞으로, 끝에 세로 날(위는 뾰족·아래는 납작한 끌)."""
    px, py, pz = -PAW[0], PAW[1], PAW[2]
    g = m.geo("pick_geo", "pick")
    end = py - 0.62
    g.limb([(px, py + 0.06, pz), (px, end, pz + 0.02)], [0.03, 0.03], "k_wood", sides=6, caps=(True, True))
    g.limb([(px, end, pz + 0.02), (px, end + 0.05, pz + 0.30)], [0.05, 0.0], "k_bronzel", sides=4)
    g.limb([(px, end, pz + 0.02), (px, end + 0.08, pz - 0.24)], [0.05, 0.012], "k_bronze", sides=4)
    g.limb([(px, end + 0.04, pz + 0.02), (px, end - 0.04, pz + 0.02)], [0.06, 0.06], "k_bronze", sides=6, caps=(True, True))


def make():
    EYES[:] = _eye_faces()
    print(f"[repeater] eye faces {len(EYES)}")
    m = C.Model(NAME)
    sx, sy, sz = SHOULDER
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0, HIP_Z), "root")
    m.bone("head", (0, 0.0, NECK_Z), "body")
    m.bone("arm_l", (sx, sy, sz), "body")
    m.bone("arm_r", (-sx, sy, sz), "body")
    m.bone("pick", (-PAW[0], PAW[1], PAW[2]), "arm_r")
    m.bone("leg_l", (0.13, 0, HIP_Z), "root")
    m.bone("leg_r", (-0.13, 0, HIP_Z), "root")
    m.external(MESHY_LOW, classify, MESHY_PARTS)
    pickaxe(m)
    C.darken_grey(m.ext_img, GREY_K, GREY_SAT)
    m.build()
    root_husk.acts(m)
    return m


PREVIEWS = [("idle", 0.0, ""), ("attack", 0.5, "_attack")]
CHECK = [("walk", 0.0), ("walk", 0.5), ("attack", 0.14), ("attack", 0.5), ("attack", 0.62), ("hurt", 0.05)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, check_dir=None):
    m = make()
    path = m.export(models_dir)
    for act, t, suffix in PREVIEWS:
        m.preview(os.path.join(preview_dir, f"{NAME}{suffix}.png"), act, t)
    if check_dir:
        for act, t in CHECK:
            m.preview(os.path.join(check_dir, f"{NAME}_{act}_{int(t * 100):03d}.png"), act, t)
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
