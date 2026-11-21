"""이끼 덮인 문지기(셋째 보스, 0185 — 모델 0234).

Meshy 몸 — 잿빛 둥근 거북 같은 짐승, 등에 겹친 돌 껍질 판 셋(이끼 덮임), 판 사이 틈으로 호박빛 핵(`glow_core`), 앞 판은 얼굴 위로
내민 챙, 껍질 가장자리에 늘어진 보라 잔뿌리, 낮게 둔 졸린 호박빛 눈(`glow_eyes`), 입 없음. 원본 `tools/art/sources/meshy/mossward_v1.glb`
(나노바나나 Pro 그림 `tools/art/sources/ai/knellroot/mossward_v1_front.png` → Meshy 7.1 API, CC BY 4.0) →
`meshy_convert.py ... 2.3 2000 256`. 줄인 몸이 길이 3.5m라(충돌 반지름 1.1) 0.85배로 쓴다(키 1.96·길이 3.0).
부위는 면 가운데 자리와 색으로: 껍질 = 판 아래 끝(`SHELL_Z`)보다 위 + 보라 잔뿌리, 핵 = 판 사이의 호박빛, 눈 = 얼굴의 호박빛.
옛 코드 몸(0185 1차)은 `make_blocks()`.

설계(story-arc 문턱지기 PROPOSED): 둥근 생명체의 원래 실루엣이 남은 몸, 등과 옆구리에 굳은 껍질·잔뿌리·돌가루·이끼.
귀여움 문법(0036 — 큰 머리·낮게 둔 큰 눈·입 없음·둥근 실루엣, 보스도), 침식 = 색을 잃은 잿빛(0037). 앞 = -Y.

동작(0185 그대로 `acts`): idle · walk · slam(낮게 웅크려 앞을 쓴다 — 정점 0.9 · 맞음 1.0, `render::STRIKE_KEYS`) · charge_windup ·
charge · shell(웅크려 껍질로 앞을 덮는다 — 정점 0.5 · 0.55, 그 뒤는 `recover`가 푼다) · recover · hurt · wake(0 = 웅크려 잠듦).

    blender --background --python tools/art/blender/mossward.py
"""
import os
import sys

from mathutils import Matrix

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "mossward"

MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "mossward_v1_low.glb")
MESHY_PARTS = {"body_geo": "body", "head_geo": "head", "glow_eyes": "head", "shell_geo": "shell", "glow_core": "core",
               "leg_fl_geo": "leg_fl", "leg_fr_geo": "leg_fr", "leg_bl_geo": "leg_bl", "leg_br_geo": "leg_br"}
SCALE = 0.85
# 치수(줄이기 전 자리, 앞 = -Y, 캐릭터 왼쪽 = +X) — 정면·옆 직교 렌더 눈금과 색 묶음의 면 자리로 잰 것.
SHELL_Z = 1.02  # 껍질 판 아래 끝(앞 챙 ≈ 0.97 — 얼굴 위) — 뒤 판은 꼬리 쪽으로 내려온다
SHELL_BACK = (0.67, 1.67, 0.86)  # 이 y부터 저 y까지 판 아래 끝이 SHELL_Z → 이 높이로
# 껍질 뼈(돌쩌귀)는 판 뒤 아래 끝에 둔다: 껍질 받기·잠든 자세는 껍질을 앞으로 숙여 챙이 얼굴을 덮는데, 뼈가 몸 가운데(y 0)면
# 뒤 판이 그만큼 들려(16°에 약 0.3m) 몸과의 이음이 벌어지고 속이 빈 두 부위(위가 열린 몸·아래가 열린 껍질) 너머로 앞의 눈까지
# 보였다(사장 "뒤에서 보면 뚫려 있다"). 뒤 끝을 돌쩌귀로 숙이면 판 끝은 모두 내려가기만(몸 속으로) 한다 — 이음이 안 벌어진다.
SHELL_HINGE = (1.59, 1.12)  # (y, z) — 껍질 판 아래 끝의 가장 뒤(줄인 몸 y 1.35·z 0.95)
HEAD_Y = -1.1  # 이보다 앞은 얼굴(앞다리 앞 끝 ≈ -0.76)
LEG_Z = 0.32  # 이보다 아래는 다리(배 ≈ 0.34)
LEG_X, FRONT_Y, BACK_Y = 0.47, -0.55, 0.72  # 발 가운데
GREY_K = 0.75  # 잿빛 몸·돌 껍질을 어둡게(0.6은 몸 전체가 검게 묻혔다 — 첫 문지기 돌 0.72 꼴) — 이끼·보라 뿌리·호박빛은 그대로
GREY_SAT = 0.18

EYES, CORE, ROOTS = [], [], []


def _colored():
    """호박빛(눈·핵)·보라(잔뿌리) 면."""
    for c, (h, s, v) in C.face_colors(MESHY_LOW):
        if 0.04 < h < 0.15 and s > 0.45 and v > 0.5:
            (EYES if c[1] < -1.4 and c[2] < 1.0 else CORE).append(c)
        elif 0.68 < h < 0.92 and s > 0.35:
            ROOTS.append(c)


def _shell_z(y):
    y0, y1, zb = SHELL_BACK
    t = min(1.0, max(0.0, (y - y0) / (y1 - y0)))
    return SHELL_Z + (zb - SHELL_Z) * t


def classify(c):
    """면 가운데 자리(가져온 그대로 — 줄이기 전) → 부위."""
    x, y, z = c.x, c.y, c.z
    p = (x, y, z)
    if C.near_any(EYES, p):
        return "glow_eyes"
    if C.near_any(CORE, p):
        return "glow_core"
    if C.near_any(ROOTS, p) or z > _shell_z(y):
        return "shell_geo"
    if z < LEG_Z and y > HEAD_Y:
        return f"leg_{'f' if y < 0.1 else 'b'}{'l' if x > 0 else 'r'}_geo"
    if y < HEAD_Y:
        return "head_geo"
    return "body_geo"


def make():
    EYES.clear(), CORE.clear(), ROOTS.clear()
    _colored()
    print(f"[mossward] eyes {len(EYES)} core {len(CORE)} roots {len(ROOTS)}")
    m = C.Model(NAME)
    s = SCALE

    def at(x, y, z):
        return (x * s, y * s, z * s)

    m.bone("root", (0, 0, 0))
    m.bone("body", at(0, 0.0, 0.6), "root")
    m.bone("head", at(0, HEAD_Y, 0.65), "body")
    m.bone("shell", at(0, *SHELL_HINGE), "body")
    m.bone("core", at(0, -0.35, 1.7), "shell")
    for n, x, y in (("leg_fl", LEG_X, FRONT_Y), ("leg_fr", -LEG_X, FRONT_Y), ("leg_bl", LEG_X, BACK_Y), ("leg_br", -LEG_X, BACK_Y)):
        m.bone(n, at(x, y, LEG_Z + 0.1), "body")
    objs = m.external(MESHY_LOW, classify, MESHY_PARTS)
    for o in objs.values():
        o.data.transform(Matrix.Scale(s, 4))
    C.darken_grey(m.ext_img, GREY_K, GREY_SAT)
    m.build()
    acts(m)
    return m


def make_blocks():
    """0185 1차 코드 몸(되돌리기·비교용)."""
    m = C.Model(NAME)
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0.1, 0.55), "root")
    m.bone("head", (0, -0.8, 0.85), "body")
    m.bone("shell", (0, 0.1, 1.5), "body")
    m.bone("core", (0, 0.25, 1.9), "shell")
    for n, x, y in (("leg_fl", 0.72, -0.55), ("leg_fr", -0.72, -0.55), ("leg_bl", 0.72, 0.75), ("leg_br", -0.72, 0.75)):
        m.bone(n, (x, y, 0.5), "body")

    # 몸: 둥근 원래 실루엣(잿빛) + 밝은 배.
    body = m.geo("body_geo", "body")
    body.blob((0, 0.1, 1.05), (2.1, 2.4, 1.55), "f_ash", r=1.0, n=4)
    body.blob((0, -0.35, 0.72), (1.5, 1.4, 0.75), "k_bone", r=1.0, n=3)

    # 등·옆구리의 굳은 껍질(겹친 큰 덩어리) + 이끼 + 돌가루 알갱이.
    shell = m.geo("shell_geo", "shell")
    for (x, y, z, sx, sy, sz, mat) in (
        (0.0, -0.25, 1.72, 1.55, 1.0, 0.5, "k_stone"),
        (0.0, 0.55, 1.78, 1.6, 1.1, 0.55, "k_light"),
        (0.62, 0.15, 1.5, 0.75, 1.5, 0.75, "k_stone"),
        (-0.62, 0.15, 1.5, 0.75, 1.5, 0.75, "k_stone"),
        (0.0, 1.05, 1.35, 1.3, 0.6, 0.7, "k_light"),
    ):
        shell.blob((x, y, z), (sx, sy, sz), mat, r=1.0, n=4)
    # 머리 위로 내민 챙(껍질 받기 때 얼굴을 덮는다).
    shell.blob((0, -0.85, 1.48), (1.35, 0.75, 0.32), "k_stone", r=1.0, n=3)
    for (x, y, z, s) in ((0.3, 0.2, 2.05, 0.55), (-0.35, 0.7, 2.06, 0.5), (0.1, -0.35, 1.98, 0.42), (-0.7, -0.1, 1.82, 0.32),
                         (0.75, 0.6, 1.8, 0.36)):
        shell.blob((x, y, z), (s, s, s * 0.38), "k_moss", r=1.0, n=2)
    for (x, y, z) in ((0.45, -0.5, 1.95), (-0.25, 0.95, 1.86), (0.9, 0.0, 1.55), (-0.88, 0.45, 1.5)):
        shell.blob((x, y, z), (0.12, 0.12, 0.1), "k_dark", r=1.0, n=2)
    # 잔뿌리: 껍질 가장자리에서 옆구리로 늘어진 가는 뿌리(각진 것은 뿌리만, 0037).
    for side in (1, -1):
        for (y, drop) in ((-0.3, 0.55), (0.35, 0.7), (0.9, 0.5)):
            x0 = side * 1.0
            shell.limb([(x0, y, 1.35), (x0 + side * 0.12, y + 0.05, 1.05), (x0 + side * 0.08, y + 0.12, 1.35 - drop - 0.25)],
                       [0.09, 0.07, 0.035], "k_root", sides=5, smooth=True)
    core = m.geo("glow_core", "core")
    core.blob((0, 0.25, 1.98), (0.32, 0.42, 0.22), "k_core", r=1.0, n=2)

    # 머리: 크고 둥글게, 몸 앞 아래쪽. 큰 눈 둘은 낮게, 입은 없다.
    head = m.geo("head_geo", "head")
    head.blob((0, -1.05, 0.85), (1.2, 0.95, 0.95), "f_ash", r=1.0, n=4)
    eyes = m.geo("glow_eyes", "head")
    for x in (0.27, -0.27):
        eyes.blob((x, -1.5, 0.76), (0.22, 0.07, 0.26), "k_amber", r=1.0, n=3)

    for n, x, y in (("leg_fl", 0.72, -0.55), ("leg_fr", -0.72, -0.55), ("leg_bl", 0.72, 0.75), ("leg_br", -0.72, 0.75)):
        leg = m.geo(n + "_geo", n)
        leg.blob((x, y, 0.24), (0.55, 0.6, 0.52), "f_ash_d", r=1.0, n=3)

    m.build()
    acts(m)
    return m


def acts(m):
    """동작 표(0185 그대로 — 뼈 이름이 같으면 몸이 바뀌어도 쓴다)."""
    def k(**b):
        return b

    legs_down = {}
    m.action("idle", 2.0, [
        (0.0, k(body={"s": 1.0}, head={"r": (0, 0, 0)})),
        (1.0, k(body={"s": (1.02, 1.02, 0.98)}, head={"r": (-4, 0, 0)}, shell={"r": (-1, 0, 0)}, core={"s": 1.12})),
    ], loop=True)
    m.action("walk", 1.4, [
        (0.0, k(body={"r": (0, 4, 0)}, leg_fl={"r": (-22, 0, 0)}, leg_br={"r": (-22, 0, 0)},
                leg_fr={"r": (18, 0, 0)}, leg_bl={"r": (18, 0, 0)})),
        (0.35, k(root={"l": (0, 0, 0.06)}, body={"r": (2, 0, 0)})),
        (0.7, k(body={"r": (0, -4, 0)}, leg_fl={"r": (18, 0, 0)}, leg_br={"r": (18, 0, 0)},
                leg_fr={"r": (-22, 0, 0)}, leg_bl={"r": (-22, 0, 0)})),
        (1.05, k(root={"l": (0, 0, 0.06)}, body={"r": (2, 0, 0)})),
    ], loop=True)
    # 쓸기: 낮게 웅크리며 몸을 한쪽으로 감았다가(정점 0.9) 반대쪽으로 쓴다(맞음 1.0).
    low = {"l": (0, 0.1, -0.22)}
    m.action("slam", 1.5, [
        (0.0, k()),
        (0.55, k(root=low, body={"r": (14, 0, 30), "s": (1.04, 1.04, 0.92)}, head={"r": (10, 0, 12)})),
        (0.9, k(root=low, body={"r": (16, 0, 38), "s": (1.05, 1.05, 0.9)}, head={"r": (12, 0, 16)})),
        (1.0, k(root={"l": (0, -0.25, -0.18)}, body={"r": (12, 0, -40)}, head={"r": (8, 0, -18)})),
        (1.2, k(root={"l": (0, -0.2, -0.1)}, body={"r": (8, 0, -34)}, head={"r": (4, 0, -12)})),
        (1.5, k()),
    ], linear=(0.9,))
    m.action("charge_windup", 0.8, [
        (0.0, k()),
        (0.65, k(root={"l": (0, 0.3, -0.18)}, body={"r": (18, 0, 0), "s": (1.06, 0.94, 0.92)}, head={"r": (14, 0, 0)},
                 leg_bl={"r": (-20, 0, 0)}, leg_br={"r": (-20, 0, 0)})),
        (0.8, k(root={"l": (0, 0.32, -0.2)}, body={"r": (20, 0, 0), "s": (1.06, 0.94, 0.9)}, head={"r": (16, 0, 0)},
                leg_bl={"r": (-22, 0, 0)}, leg_br={"r": (-22, 0, 0)})),
    ])
    m.action("charge", 0.5, [
        (0.0, k(body={"r": (14, 0, 0)}, head={"r": (12, 0, 0)}, leg_fl={"r": (-30, 0, 0)}, leg_fr={"r": (25, 0, 0)},
                leg_bl={"r": (25, 0, 0)}, leg_br={"r": (-30, 0, 0)})),
        (0.12, k(root={"l": (0, 0, 0.12)}, body={"r": (12, 0, 0)}, head={"r": (12, 0, 0)})),
        (0.25, k(body={"r": (14, 0, 0)}, head={"r": (12, 0, 0)}, leg_fl={"r": (25, 0, 0)}, leg_fr={"r": (-30, 0, 0)},
                 leg_bl={"r": (-30, 0, 0)}, leg_br={"r": (25, 0, 0)})),
        (0.37, k(root={"l": (0, 0, 0.12)}, body={"r": (12, 0, 0)}, head={"r": (12, 0, 0)})),
    ], loop=True)
    # 껍질 받기: 머리를 몸 안으로 끌어들이고(뒤·아래), 몸을 낮추고, 챙(껍질)을 앞으로 덮는다. 0.5부터 버틴다.
    # 껍질은 뒤 끝 돌쩌귀(`SHELL_HINGE`)로 숙인다 — 각은 챙이 내려오는 거리(≈0.45m)를 옛 가운데 뼈 16°와 맞춘 9°.
    curled = k(root={"l": (0, 0, -0.32)}, body={"r": (6, 0, 0), "s": (1.06, 1.06, 0.86)},
               head={"l": (0, 0.42, -0.18), "r": (-8, 0, 0), "s": 0.92}, shell={"r": (9, 0, 0)},
               leg_fl={"r": (0, 0, 0), "s": 0.85}, leg_fr={"s": 0.85}, leg_bl={"s": 0.85}, leg_br={"s": 0.85})
    m.action("shell", 1.6, [(0.0, k()), (0.35, curled), (0.5, curled), (0.55, curled), (1.6, curled)])
    # 푼다(쉼 — 껍질 뒤·돌진 뒤): 웅크림에서 천천히 일어선다.
    m.action("recover", 0.8, [(0.0, curled), (0.8, k())])
    m.action("hurt", 0.4, [
        (0.0, k(body={"r": (-10, 0, 0), "s": (0.96, 0.96, 1.05)}, head={"r": (-14, 0, 0)})),
        (0.4, k()),
    ])
    # 깨어남: 0초 = 웅크려 잠든 자세(머리 묻고 챙 덮음), 끝 = 선다.
    asleep = k(root={"l": (0, 0, -0.38)}, body={"r": (8, 0, 0), "s": (1.06, 1.06, 0.84)},
               head={"l": (0, 0.45, -0.22), "r": (12, 0, 0), "s": 0.92}, shell={"r": (10, 0, 0)}, core={"s": 0.7},
               leg_fl={"s": 0.8}, leg_fr={"s": 0.8}, leg_bl={"s": 0.8}, leg_br={"s": 0.8})
    m.action("wake", 2.0, [(0.0, asleep), (0.8, asleep), (1.4, k(head={"r": (-12, 0, 0)}, core={"s": 1.2})), (2.0, k())])
    del legs_down


PREVIEWS = [("idle", 0.0, ""), ("slam", 0.9, "_slam"), ("shell", 0.5, "_shell")]
CHECK = [("walk", 0.0), ("walk", 0.35), ("slam", 0.55), ("slam", 0.9), ("slam", 1.0), ("charge_windup", 0.8), ("charge", 0.0),
         ("shell", 0.5), ("hurt", 0.05), ("wake", 0.0), ("wake", 1.4)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, check_dir=None):
    m = make()
    path = m.export(models_dir)
    for act, t, suffix in PREVIEWS:
        m.preview(os.path.join(preview_dir, f"{NAME}{suffix}.png"), act, t)
    if check_dir:
        for act, t in CHECK:
            m.preview(os.path.join(check_dir, f"{NAME}_{act}_{int(t * 100):03d}.png"), act, t, yaw=-70, elev=12)
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
