"""등짐꾼(작업층의 재활동체, 네 발 — 0208·0216 W2 · 모델 0234).

Meshy 몸 — 잿빛 둥근 카피바라 짐꾼, 등에 모래빛 큰 돌짐(돌 서넛 + 배를 감은 끈 둘, 돌 사이에 보라 결정 뿌리), 어깨의 결정,
호박빛 큰 눈 둘(glow_eyes). 원본 `tools/art/sources/meshy/packbearer_v1.glb`(나노바나나 Pro 그림
`tools/art/sources/ai/knellroot/packbearer_v1_front.png` → Meshy 6 API, CC BY 4.0) → `meshy_convert.py ... 1.5 800 128`.
줄인 몸이 길이 1.95m라 충돌 반지름(0.65)에 견줘 길어 0.85배로 줄여 쓴다(키 1.27·길이 1.66).

짐은 따로 부위 `load_geo`(뼈 `load`, 몸의 자식) — 자세가 처음 깨지면 게임이 숨긴다(`Enemy::load_lost`, 0216 W2).
짐·끈은 자리로는 몸과 안 갈려 색(모래빛 갈색)으로 고른다(0132 꼬투리와 같은 꼴), 짐 위 결정은 높이로.
동작은 가시짐승의 표 그대로(`thorn_beast.acts` — idle·run·charge_windup·charge·recover·hurt, 뼈 이름이 같다). 꼬리 뼈는 붙은 부위 없이 둔다.

    blender --background --python tools/art/blender/packbearer.py
"""
import os
import sys

from mathutils import Matrix

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import thorn_beast  # noqa: E402

NAME = "packbearer"

MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "packbearer_v1_low.glb")
MESHY_PARTS = {"body_geo": "body", "head_geo": "head", "glow_eyes": "head", "load_geo": "load",
               "leg_fl_geo": "leg_fl", "leg_fr_geo": "leg_fr", "leg_bl_geo": "leg_bl", "leg_br_geo": "leg_br"}
SCALE = 0.85
# 치수(줄이기 전 자리, 앞 = -Y, 캐릭터 왼쪽 = +X) — 정면·옆·위 직교 렌더에 0.05m 눈금을 그어 잰 것.
LEG_Z = 0.24  # 이보다 아래는 다리(배 ≈ 0.26)
FRONT_LEGS_Y = 0.2  # 다리 칸: 이보다 앞은 앞다리(앞발 y ≈ −0.3, 뒷발 ≈ 0.7)
HEAD_Y = -0.45  # 이보다 앞은 머리(볼 뒤)
EYE = (0.27, -0.55, 0.9)  # 눈 가운데(±x, 이보다 앞 y, z)
EYE_R = (0.08, 0.08)
LOAD_ROOTS_Z = 1.1  # 이보다 위의 결정 뿌리는 짐에 박힌 것(어깨의 결정 ≈ 0.8~1.0은 몸)
LOAD_Y = (-0.42, 0.9)  # 짐·끈의 앞뒤 끝(얼굴의 갈색 칸은 빼려고)
GREY_K = 0.6  # 잿빛 몸을 어둡게(다른 적과 같은 까닭·같은 값) — 모래빛 짐·보라 결정·호박빛 눈은 그대로
GREY_SAT = 0.18

LOAD = []


def _load_faces():
    def pick(x, y, z, h, s, v):
        if not (LOAD_Y[0] < y < LOAD_Y[1]) or z < 0.12:
            return False
        sand = 0.03 < h < 0.17 and s >= 0.2
        root = 0.7 < h < 0.9 and s > 0.4 and z > LOAD_ROOTS_Z
        return sand or root
    return [c for c, hsv in C.face_colors(MESHY_LOW) if pick(*c, *hsv)]


def classify(c):
    """면 가운데 자리(가져온 그대로 — 줄이기 전) → 부위."""
    x, y, z = c.x, c.y, c.z
    if C.near_any(LOAD, (x, y, z)):
        return "load_geo"
    side = "l" if x > 0 else "r"
    if z < LEG_Z:
        return f"leg_{'f' if y < FRONT_LEGS_Y else 'b'}{side}_geo"
    if y < HEAD_Y:
        ex, ey, ez = EYE
        rx, rz = EYE_R
        if y < ey and ((abs(x) - ex) / rx) ** 2 + ((z - ez) / rz) ** 2 <= 1.0:
            return "glow_eyes"
        return "head_geo"
    return "body_geo"


def make():
    LOAD[:] = _load_faces()
    print(f"[packbearer] load faces {len(LOAD)}")
    m = C.Model(NAME)
    s = SCALE

    def at(x, y, z):
        return (x * s, y * s, z * s)

    m.bone("root", (0, 0, 0))
    m.bone("body", at(0, 0.1, 0.55), "root")
    m.bone("head", at(0, HEAD_Y, 0.7), "body")
    for n, x, y in (("fl", 0.25, -0.3), ("fr", -0.25, -0.3), ("bl", 0.25, 0.7), ("br", -0.25, 0.7)):
        m.bone(f"leg_{n}", at(x, y, LEG_Z + 0.03), "root")
    m.bone("tail", at(0, 0.95, 0.55), "body")
    m.bone("load", at(0, 0.2, 1.0), "body")
    objs = m.external(MESHY_LOW, classify, MESHY_PARTS)
    for o in objs.values():
        o.data.transform(Matrix.Scale(s, 4))
    C.darken_grey(m.ext_img, GREY_K, GREY_SAT)
    m.build()
    thorn_beast.acts(m, leg_k=0.45, rear_k=0.5)  # 짧은 뭉툭발·무거운 짐: 다리는 반쯤, 앞만 조금 든다
    return m


PREVIEWS = [("idle", 0.0, ""), ("charge", 0.0, "_charge")]
CHECK = [("run", 0.0), ("run", 0.25), ("charge_windup", 0.22), ("charge_windup", 0.46), ("charge_windup", 0.6), ("recover", 0.12), ("hurt", 0.05)]


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
