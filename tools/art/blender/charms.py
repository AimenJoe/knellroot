"""장신구 아홉(0068 · 떠난 마을 셋 0127) + 덧옷 셋(0176): 장비 창 아이콘·도는 그림용 작은 물건 — `charms.glb`(장신구)·`garbs.glb`(덧옷)에 물건마다 메시 부위 하나(이름 = 기록 키).

게임은 부위 하나만 골라(다른 부위는 크기 0) 경계 상자 가운데에 놓고 그린다(`Renderer::draw_item`) — 파일 안의 자리는 미리보기용.
0061 체크리스트: 귀여움 문법이 먼저(둥근 덩어리 `blob`), 큰 색 덩어리 + 실루엣, 각진 것은 뿌리만(0037), 채도 높은 빨강 없음.
AI 사용: 모양·색은 Claude가 이 스크립트로 코드로 찍었다(그림 생성 AI·외부 에셋 없음). 2026-11-23 디테일(사장 "싸구려" — 덧옷 셋·꿰맨 천은 꼴부터, 껍질·짚신·돌 장갑은 덧붙임).

    blender --background --python tools/art/blender/charms.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

# 미리보기에서 나란히 보이게 x로 띄운다(게임은 부위마다 가운데로 옮기니 상관없다).
STEP = 0.55


def _hex(g, c, r, h, mat, rot=(0, 0, 0)):
    """낮은 육각 판(비늘판·징) — 각진 테에 밝은 선이 붙는다."""
    g.lathe(c, [(r, 0.0), (r, h * 0.5), (r * 0.72, h)], mat, sides=6, rot=rot, caps=(False, True))


def shell(g, x):
    """단단한 껍질(최대 체력 +1): 둥근 등딱지 — 물결진 밝은 테(둘레 혹) + 갈색 돔 + 육각 비늘판(가운데 하나·둘레 여섯, 판 사이 어두운 골)
    + 뒤의 가죽 끈 고리."""
    import math
    g.lathe((x, 0, 0.0), [(0.200, 0.0), (0.215, 0.022), (0.200, 0.048)], "p_boot", sides=12, smooth=True, squash=(1.0, 0.86),
            caps=(True, False))  # 테
    for k in range(12):
        a = 2 * math.pi * (k + 0.5) / 12
        g.blob((x + 0.212 * math.cos(a), 0.212 * 0.86 * math.sin(a), 0.026), (0.050, 0.050, 0.038), "p_boot", r=1.0, n=2)  # 테의 혹
    g.lathe((x, 0, 0.0), [(0.190, 0.040), (0.172, 0.105), (0.120, 0.160), (0.0, 0.185)], "brown", sides=12, smooth=True,
            squash=(1.0, 0.86), caps=(False, False))  # 돔
    _hex(g, (x, 0, 0.170), 0.062, 0.026, "p_belt")
    for k in range(6):
        a = 2 * math.pi * k / 6
        dx, dy = 0.118 * math.cos(a), 0.118 * 0.86 * math.sin(a)
        tilt = (-math.degrees(math.atan2(dy, 0.30)) * 1.1, math.degrees(math.atan2(dx, 0.30)) * 1.1, 0)
        _hex(g, (x + dx, dy, 0.140), 0.050, 0.022, "p_belt", rot=tilt)
    g.limb([(x - 0.05, 0.17, 0.05), (x - 0.03, 0.23, 0.10), (x + 0.03, 0.23, 0.10), (x + 0.05, 0.17, 0.05)], [0.012] * 4, "p_scarf",
           sides=5, smooth=True)  # 끈 고리
    g.blob((x, 0.175, 0.05), (0.05, 0.03, 0.03), "p_scarf", r=1.0, n=2)


def feet(g, x):
    """가벼운 발(회피 기력 −30% · 받는 피해 +1): 깃털 달린 작은 짚신 — 짚을 엮은 바닥(가로 결) + 앞코 매듭 + 엇건 끈 둘 + 뒤꿈치에서 솟은 깃
    (대에 감은 실 · 깃 끝의 작은 갈래)."""
    g.blob((x, 0, 0.03), (0.17, 0.33, 0.05), "p_belt", r=1.0, n=3)  # 바닥(짚)
    for k in range(7):
        y = -0.13 + k * 0.043
        w = 0.15 - 0.05 * abs(y) / 0.15
        g.blob((x, y, 0.056), (w, 0.022, 0.016), "wood" if k % 2 else "p_gold", r=1.0, n=2)  # 엮은 결
    g.blob((x, -0.145, 0.075), (0.050, 0.045, 0.040), "p_scarf", r=1.0, n=2)  # 앞코 매듭
    for y0, y1 in ((-0.10, 0.02), (0.02, -0.10)):
        g.limb([(x - 0.078, y0, 0.045), (x - 0.045, (y0 + y1) / 2 - 0.01, 0.105), (x + 0.045, (y0 + y1) / 2 + 0.01, 0.105),
                (x + 0.078, y1, 0.045)], [0.016, 0.017, 0.017, 0.016], "p_scarf", sides=5, smooth=True)  # 엇건 끈
    g.limb([(x - 0.070, 0.105, 0.045), (x, 0.130, 0.090), (x + 0.070, 0.105, 0.045)], [0.015, 0.016, 0.015], "p_scarf", sides=5,
           smooth=True)  # 뒤꿈치 끈
    # 깃: 납작한 잎 꼴이 뒤꿈치에서 비스듬히 솟아 끝이 뒤로 휜다 — 대는 밝게, 끝은 하늘빛.
    g.limb([(x, 0.125, 0.07), (x + 0.012, 0.150, 0.17), (x + 0.030, 0.185, 0.27), (x + 0.052, 0.235, 0.345), (x + 0.066, 0.285, 0.385)],
           [0.018, 0.050, 0.056, 0.038, 0.0], ["p_white", "p_white", "p_white", "p_sky"], sides=4, roll=0, flat=0.22)
    for dz, side in ((0.20, 1), (0.25, -1), (0.30, 1)):
        t = (dz - 0.07) / 0.315
        bx, by = x + 0.066 * t ** 1.4, 0.125 + 0.16 * t ** 1.6
        g.limb([(bx, by, dz), (bx + side * 0.055, by + 0.012, dz - 0.03)], [0.010, 0.0], "p_sky", sides=3, flat=0.4)  # 깃 갈래
    g.blob((x, 0.125, 0.075), (0.045, 0.045, 0.045), "p_sky", r=1.0, n=2)  # 꽂은 자리
    g.lathe((x + 0.004, 0.13, 0.085), [(0.026, 0.0), (0.028, 0.012), (0.026, 0.024), (0.028, 0.036)], "p_scarf", sides=6,
            smooth=True, caps=(False, False))  # 대에 감은 실


def ear(g, x):
    """종지기의 귀(패링 창 +0.06초): 작은 놋쇠 종 — 벌어진 입술 · 어깨의 띠 · 걸이 고리 · 아래로 보이는 추."""
    import math
    g.lathe((x, 0, 0.0), [(0.150, 0.0), (0.158, 0.014), (0.128, 0.040), (0.108, 0.110), (0.092, 0.170), (0.056, 0.215), (0.0, 0.232)],
            ["p_gold", "p_gold", "p_gold", "bronze", "p_gold", "p_gold"], sides=10, smooth=True, hard=(3, 4), caps=(True, False),
            cap_mat="bronze")
    g.blob((x, 0, -0.030), (0.074, 0.074, 0.074), "bronze", r=1.0, n=2)  # 추
    ring = [(x + 0.052 * math.cos(t), 0.0, 0.272 + 0.052 * math.sin(t)) for t in (2 * math.pi * k / 10 for k in range(11))]
    g.limb(ring, [0.016] * 11, "bronze", sides=5, smooth=True)  # 걸이 고리


def hunger(g, x):
    """굶주린 뿌리(처치 시 기력 · 등잔 −1): 굽은 송곳니를 보라 뿌리가 감아 오른다 — 뿌리만 각지게(0037)."""
    import math
    g.limb([(x - 0.02, 0, 0.0), (x - 0.03, 0, 0.11), (x - 0.01, 0, 0.22), (x + 0.04, 0, 0.31), (x + 0.10, 0, 0.365)],
           [0.085, 0.080, 0.062, 0.036, 0.0], "bone", sides=8, smooth=True, caps=(True, False))  # 송곳니
    coil = []
    for k in range(13):
        t = k / 12
        a = math.radians(-60 + 560 * t)
        r = 0.098 - 0.030 * t
        cx = x - 0.025 + 0.02 * t
        coil.append((cx + r * math.cos(a), r * math.sin(a), 0.015 + 0.20 * t))
    g.limb(coil, [0.034 - 0.018 * k / 12 for k in range(12)] + [0.0], ["root", "droot"] * 6, sides=4, roll=45)  # 감은 뿌리
    g.limb([(x + 0.07, -0.04, 0.02), (x + 0.15, -0.07, 0.035), (x + 0.21, -0.05, 0.085)], [0.030, 0.022, 0.0], "droot", sides=4,
           roll=45)  # 땅으로 뻗은 끝
    g.limb([(x - 0.09, 0.05, 0.02), (x - 0.16, 0.08, 0.03), (x - 0.20, 0.06, 0.07)], [0.026, 0.018, 0.0], "root", sides=4, roll=45)


def hand(g, x):
    """무거운 손(강타 피해 +1 — 0127, 예전 주는 피해 +1 · 공격 속도 −15%): 돌 장갑 — 손바닥 덩어리 + 마디 둘씩 굽은 손가락 넷(마디마다 쇠 징)
    + 엄지 + 손등의 이끼 조각 + 징 박은 손목 테."""
    import math
    g.blob((x, 0, 0.10), (0.20, 0.13, 0.20), "lstone", r=0.7, n=2)  # 손바닥
    for i, dx in enumerate((-0.066, -0.022, 0.022, 0.066)):
        up = 0.01 if i in (1, 2) else 0.0
        g.blob((x + dx, 0.0, 0.215 + up), (0.042, 0.11, 0.07), "stone", r=0.8, n=2)  # 첫 마디
        g.blob((x + dx, -0.035, 0.27 + up), (0.040, 0.07, 0.07), "stone", r=1.0, n=2, rot=(-25, 0, 0))  # 굽은 끝마디
        g.blob((x + dx, -0.062, 0.218 + up), (0.022, 0.016, 0.022), "steel", r=1.0, n=1)  # 마디 징
    g.blob((x - 0.12, -0.01, 0.13), (0.06, 0.09, 0.10), "stone", r=0.8, n=2, rot=(0, -30, 0))  # 엄지
    g.blob((x - 0.145, -0.045, 0.165), (0.018, 0.014, 0.018), "steel", r=1.0, n=1)
    for dx, dz, w in ((0.06, 0.12, 0.07), (-0.03, 0.07, 0.05), (0.09, 0.05, 0.035)):
        g.blob((x + dx, 0.065, dz), (w, 0.012, w * 0.8), "p_moss", r=1.0, n=2)  # 손등 이끼
    g.lathe((x, 0, -0.005), [(0.125, 0.0), (0.135, 0.018), (0.135, 0.040), (0.120, 0.058)], "moss", sides=10, smooth=True,
            squash=(1.0, 0.72), caps=(True, False))  # 손목 테
    for k in range(8):
        a = 2 * math.pi * (k + 0.5) / 8
        g.blob((x + 0.136 * math.cos(a), 0.136 * 0.72 * math.sin(a), 0.030), (0.022, 0.022, 0.022), "p_gold", r=1.0, n=1)  # 테 징


def pouch(g, x):
    """잔불 주머니(잔불 +50%): 배 부른 천 주머니 — 끈으로 조인 목 · 벌어진 주름 입 · 입에서 비치는 불씨 · 늘어진 끈 끝."""
    g.lathe((x, 0, 0.0), [(0.070, 0.0), (0.130, 0.025), (0.150, 0.090), (0.125, 0.160), (0.062, 0.205)], "p_mask", sides=10,
            smooth=True, caps=(True, False))  # 주머니
    g.lathe((x, 0, 0.0), [(0.062, 0.205), (0.070, 0.218), (0.062, 0.232)], "p_belt", sides=10, smooth=True, caps=(False, False))  # 끈
    g.lathe((x, 0, 0.0), [(0.062, 0.232), (0.105, 0.275), (0.070, 0.262)], "p_mask", sides=7, caps=(False, False))  # 주름 입
    g.blob((x, 0, 0.262), (0.105, 0.105, 0.060), "ember", r=1.0, n=2)  # 비치는 불씨
    g.blob((x + 0.035, 0.02, 0.305), (0.036, 0.036, 0.036), "amber", r=1.0, n=1)
    g.limb([(x + 0.066, -0.02, 0.218), (x + 0.120, -0.05, 0.205), (x + 0.150, -0.06, 0.140)], [0.013, 0.013, 0.011], "p_belt",
           sides=5, smooth=True)
    g.blob((x + 0.152, -0.06, 0.125), (0.040, 0.040, 0.040), "p_belt", r=1.0, n=1)  # 끈 끝 매듭


def cloth(g, x):
    """꿰맨 천(체력 +1 · 기력 회복 −10%, 0127): 개어 놓은 천 위에 끈으로 묶은 두루마리 — 천에 덧댄 조각(밝은 땀이 둘레를 엇건다)
    · 꽂힌 바늘의 실이 실패로 이어진다."""
    g.blob((x, 0, 0.035), (0.38, 0.30, 0.07), "p_scarf", r=0.6, n=2)  # 개어 놓은 천
    g.blob((x - 0.01, -0.01, 0.075), (0.34, 0.26, 0.025), "p_scarf", r=0.6, n=2, rot=(0, 0, 4))  # 위 겹
    g.limb([(x - 0.15, 0.065, 0.14), (x + 0.15, 0.065, 0.14)], [0.068, 0.068], "p_mask", sides=12, smooth=True, caps=(True, True))  # 두루마리
    for dx in (-0.08, 0.08):
        g.lathe((x + dx, 0.065, 0.14), [(0.071, -0.012), (0.074, 0.0), (0.071, 0.012)], "p_belt", sides=12, smooth=True,
                rot=(0, 90, 0), caps=(False, False))  # 묶은 끈
    g.lathe((x + 0.153, 0.065, 0.14), [(0.050, 0.0), (0.050, 0.003)], "p_cream", sides=12, rot=(0, 90, 0), caps=(False, True))  # 끝의 말린 결
    g.lathe((x + 0.156, 0.065, 0.14), [(0.024, 0.0), (0.024, 0.003)], "p_mask", sides=10, rot=(0, 90, 0), caps=(False, True))
    # 덧댄 조각: 기울인 네모 + 둘레의 엇건 땀.
    px, py = x - 0.06, -0.06
    g.stone((px, py, 0.093), (0.15, 0.12, 0.014), "k_clothd", bevel=0.004, rot=(0, 0, -10))
    import math
    c, s_ = math.cos(math.radians(-10)), math.sin(math.radians(-10))
    for (u, v, along) in [(-0.075 + 0.0375 * k, -0.06, 0) for k in range(5)] + [(-0.075 + 0.0375 * k, 0.06, 0) for k in range(5)] + \
            [(-0.075, -0.03 + 0.03 * k, 1) for k in range(3)] + [(0.075, -0.03 + 0.03 * k, 1) for k in range(3)]:
        g.box((px + u * c - v * s_, py + u * s_ + v * c, 0.102), (0.032, 0.008, 0.006), "p_cream",
              rot=(0, 0, -10 + (90 if along else 0) + 40))  # 땀
    g.limb([(x + 0.06, -0.08, 0.09), (x + 0.12, -0.10, 0.17), (x + 0.17, -0.12, 0.25)], [0.010, 0.009, 0.0], "p_steel", sides=4)  # 바늘
    g.blob((x + 0.06, -0.08, 0.095), (0.018, 0.018, 0.010), "p_steel", r=1.0, n=1)
    g.limb([(x + 0.155, -0.115, 0.235), (x + 0.20, -0.06, 0.20), (x + 0.24, 0.02, 0.12), (x + 0.25, 0.09, 0.07)],
           [0.005] * 4, "p_cream", sides=4, smooth=True)  # 실
    g.lathe((x + 0.25, 0.11, 0.0), [(0.034, 0.0), (0.040, 0.008), (0.028, 0.016), (0.028, 0.050), (0.040, 0.058), (0.034, 0.066)],
            ["wood", "wood", "p_cream", "wood", "wood"], sides=8, smooth=True, caps=(True, True))  # 실패


def jar(g, x):
    """물 받는 항아리(한 모금 +1 · 등잔 −1, 0127): 배 부른 흙 항아리 + 밝은 입술 + 고인 물 + 떨어지는 물방울."""
    g.lathe((x, 0, 0.0), [(0.080, 0.0), (0.140, 0.030), (0.160, 0.110), (0.135, 0.190), (0.095, 0.235)], "brown", sides=10,
            smooth=True, caps=(True, False))  # 몸
    g.lathe((x, 0, 0.0), [(0.095, 0.235), (0.118, 0.250), (0.112, 0.268), (0.088, 0.262)], "p_boot", sides=10, smooth=True,
            caps=(False, False))  # 입술
    g.lathe((x, 0, 0.0), [(0.090, 0.256), (0.0, 0.258)], "p_sky", sides=10, caps=(False, False))  # 물
    g.lathe((x, 0, 0.385), [(0.0, -0.040), (0.030, -0.018), (0.022, 0.012), (0.0, 0.052)], "p_sky", sides=6, smooth=True,
            caps=(False, False))  # 물방울


def bellshard(g, x):
    """깨진 종 조각(열린 틈 피해 +1, 0127): 옆으로 누운 종의 깨진 반쪽 — 바깥은 놋쇠, 안은 그을린 청동, 깨진 테는 각지고 밝다. 곁에 조각 둘."""
    # 종의 아래쪽 2/3만(위는 깨져 나갔다), 일곱 모로 각지게 — 깨진 테에 밝은 선이 붙는다.
    prof = [(0.165, 0.0), (0.172, 0.016), (0.140, 0.045), (0.118, 0.120), (0.104, 0.175)]
    g.lathe((x, 0, 0.115), prof, ["p_gold", "p_gold", "p_gold", "amber"], sides=7, rot=(0, 78, 20), caps=(False, False))
    g.lathe((x, 0, 0.115), [(r - 0.022, z) for r, z in prof], "bronze", sides=7, rot=(0, 78, 20), caps=(False, False))  # 안쪽
    g.box((x + 0.20, -0.10, 0.03), (0.11, 0.07, 0.03), "p_gold", rot=(8, -12, 30), taper=(0.5, 0.7))  # 떨어진 조각
    g.box((x + 0.13, 0.13, 0.025), (0.07, 0.06, 0.025), "amber", rot=(-6, 10, -25), taper=(0.6, 0.5))


def _at(profile, z):
    """`lathe` 윤곽의 높이 z에서의 반지름(선형)."""
    for (r0, z0), (r1, z1) in zip(profile, profile[1:]):
        if z0 <= z <= z1:
            return r0 + (r1 - r0) * (z - z0) / max(z1 - z0, 1e-6)
    return profile[-1][0]


def garb_cloak(g, x):
    """해진 순례 망토(덧옷, 0176 — 가볍다: 구르기에 기력이 덜 들고 맞으면 더 밀린다): 혼자 서 있는 바랜 망토 — 아래로 퍼지는 몸(앞뒤로 눌린 종 꼴)
    · 세로 주름 · 앞의 갈라진 여밈과 놋쇠 걸쇠 · 뒤로 넘긴 두건(안은 어둡게) · 해져 갈라진 자락 · 어깨의 덧댄 조각."""
    import math
    sq, z0 = 0.62, 0.035
    prof = [(0.205, 0.0), (0.200, 0.03), (0.170, 0.12), (0.125, 0.22), (0.090, 0.28), (0.060, 0.305)]
    g.lathe((x, 0, z0), prof, ["bone", "bone", "bone", "lstone", "lstone"], sides=14, smooth=True, squash=(1.0, sq),
            caps=(True, True))  # 몸
    for k, a in enumerate((-150, -115, -70, -30, 30, 70, 115, 150, 200, 250, 290, 340)):
        ar = math.radians(a)
        pts, radii = [], []
        for t in (0.0, 0.35, 0.7, 1.0):
            z = 0.25 - 0.24 * t
            r = _at(prof, z) + 0.006
            pts.append((x + r * math.cos(ar), r * sq * math.sin(ar), z0 + z))
            radii.append(0.008 + 0.018 * t)
        g.limb(pts, radii, "lstone" if k % 2 else "bone", sides=6, smooth=True)  # 주름
    # 앞 여밈: 앞(−y)의 갈라진 틈 — 어두운 띠.
    g.limb([(x, -0.062 * 1.0, z0 + 0.285), (x, -0.090 * sq * 1.25, z0 + 0.20), (x + 0.01, -0.125 * sq * 1.05, z0 + 0.10),
            (x + 0.02, -0.205 * sq, z0 + 0.0)], [0.010, 0.012, 0.014, 0.014], "slate", sides=4, flat=0.5)
    g.lathe((x - 0.005, -0.060, z0 + 0.272), [(0.026, 0.0), (0.026, 0.008), (0.016, 0.014)], "p_gold", sides=8,
            rot=(90, 0, 0), caps=(True, True))  # 걸쇠
    g.limb([(x - 0.03, -0.058, z0 + 0.272), (x - 0.06, -0.048, z0 + 0.26)], [0.006, 0.006], "p_gold", sides=4)  # 걸쇠 사슬
    # 목둘레 깃 + 등으로 넘겨 처진 두건(어두운 안쪽이 위로 보인다).
    g.lathe((x, 0, z0 + 0.285), [(0.072, 0.0), (0.084, 0.014), (0.074, 0.030)], "lstone", sides=12, smooth=True, squash=(1.0, 0.8),
            caps=(False, False))
    g.blob((x, 0.075, z0 + 0.255), (0.17, 0.10, 0.11), "bone", r=1.0, n=3, rot=(28, 0, 0))
    g.blob((x, 0.060, z0 + 0.300), (0.11, 0.05, 0.035), "slate", r=1.0, n=2, rot=(28, 0, 0))
    g.blob((x, 0.125, z0 + 0.19), (0.05, 0.04, 0.05), "lstone", r=1.0, n=2)  # 두건 끝
    # 해진 자락: 자락 둘레에서 뾰족하게 처진 조각(바닥에 닿기 전에 끝난다).
    for k, a in enumerate((-165, -138, -112, -85, -58, -32, -5, 25, 160, 195)):
        ar = math.radians(a)
        r = 0.212
        ln = (0.034, 0.022, 0.036, 0.026, 0.032, 0.020, 0.030, 0.024, 0.028, 0.026)[k]
        p0 = (x + r * math.cos(ar), r * sq * math.sin(ar), z0 + 0.012)
        p1 = (x + (r + 0.008) * math.cos(ar), (r + 0.008) * sq * math.sin(ar), z0 + 0.012 - ln)
        g.limb([p0, p1], [0.026, 0.0], "bone" if k % 2 else "lstone", sides=4, flat=0.35, roll=45)
    # 어깨의 덧댄 조각 + 땀.
    g.stone((x + 0.085, -0.05, z0 + 0.235), (0.06, 0.012, 0.05), "k_clothd", bevel=0.004, rot=(-30, -35, -32))
    for dz in (-0.018, 0.0, 0.018):
        g.box((x + 0.112, -0.063, z0 + 0.235 + dz), (0.016, 0.004, 0.004), "p_cream", rot=(-30, -35, -32))


def garb_bark(g, x):
    """뿌리껍질 덧옷(덧옷, 0176 — 무겁다: 덜 밀리고 덜 휘청이고 구르기에 기력이 더 든다): 어깨가 넓은 몸통 꼴(목 구멍은 어둡게)에 각진
    뿌리껍질 판이 비늘처럼 겹친다(각진 것은 뿌리만, 0037) · 둥근 어깨받이와 이끼 · 옆구리를 엮은 끈 · 허리띠와 놋쇠 고리."""
    import math
    sq = 0.62
    prof = [(0.145, 0.0), (0.150, 0.04), (0.135, 0.12), (0.160, 0.21), (0.180, 0.27), (0.120, 0.315), (0.070, 0.325)]
    g.lathe((x, 0, 0.0), prof, "brown", sides=14, smooth=True, squash=(1.0, sq), caps=(True, True))  # 몸
    g.lathe((x, 0, 0.312), [(0.080, 0.0), (0.090, 0.018), (0.082, 0.034), (0.066, 0.036)], "droot", sides=10, squash=(1.0, 0.72),
            caps=(False, False))  # 세운 깃(각진 껍질)
    g.lathe((x, 0, 0.328), [(0.066, 0.0), (0.0, 0.0)], "charcoal", sides=10, squash=(1.0, 0.72), caps=(False, False))  # 목 구멍
    # 비늘 판: 앞(−y) 반 둘레, 줄마다 엇갈려 — 아래 끝이 들려 겹쳐 보인다.
    for row, z in enumerate((0.075, 0.135, 0.195, 0.250)):
        r = _at(prof, z)
        for k in range(5):
            a = math.radians(-90 + (k - 2) * 26 + (13 if row % 2 else 0))
            px, py = x + r * math.cos(a), r * sq * math.sin(a)
            face = math.degrees(math.atan2(math.sin(a) * sq, math.cos(a))) + 90
            g.stone((px, py, z), (0.074, 0.022, 0.068), "root" if (row + k) % 2 else "droot", bevel=0.010,
                    rot=(10, 0, face), taper=(0.8, 1.0), edges="t", drop=("+y",))
    # 어깨받이 + 이끼.
    for side in (-1, 1):
        g.lathe((x + side * 0.155, 0.0, 0.255), [(0.090, 0.0), (0.088, 0.025), (0.062, 0.058), (0.0, 0.070)], "droot", sides=10,
                smooth=True, squash=(1.0, 1.15), rot=(0, side * 32, 0), caps=(True, False))  # 어깨받이
        g.lathe((x + side * 0.155, 0.0, 0.255), [(0.094, 0.0), (0.096, 0.010), (0.090, 0.018)], "root", sides=10,
                squash=(1.0, 1.15), rot=(0, side * 32, 0), caps=(False, False))  # 어깨받이 테
        g.blob((x + side * 0.185, 0.03, 0.312), (0.07, 0.06, 0.03), "p_moss", r=1.0, n=2, rot=(0, side * 32, 0))
        # 옆구리를 엮은 끈(엇갈린 X).
        r = _at(prof, 0.15)
        for k in range(3):
            z = 0.08 + k * 0.05
            g.limb([(x + side * (r + 0.004), -0.03, z), (x + side * (r + 0.008), 0.03, z + 0.04)], [0.007, 0.007], "p_belt",
                   sides=4, smooth=True)
            g.limb([(x + side * (r + 0.004), 0.03, z), (x + side * (r + 0.008), -0.03, z + 0.04)], [0.007, 0.007], "p_belt",
                   sides=4, smooth=True)
    g.lathe((x, 0, 0.020), [(0.152, 0.0), (0.156, 0.014), (0.152, 0.028)], "p_belt", sides=14, smooth=True, squash=(1.0, sq + 0.03),
            caps=(False, False))  # 허리띠
    g.lathe((x, -0.096, 0.034), [(0.030, 0.0), (0.030, 0.008), (0.020, 0.012)], "p_gold", sides=6, rot=(90, 0, 0),
            caps=(True, True))  # 고리


def garb_apron(g, x):
    """그을린 가죽 앞치마(덧옷, 0176 — 덜 휘청인다): 불 곁에서 일하던 사람의 것 — 아래로 넓어지며 앞으로 휜 가죽 판 · 가장자리를 도는 땀 ·
    그을린 자국 · 앞주머니(꽂힌 작은 망치) · 놋쇠 징 · 목끈과 나비로 맨 허리끈."""
    path = [(x, 0.0, 0.36), (x, -0.010, 0.25), (x, -0.020, 0.195), (x, -0.030, 0.12), (x, -0.020, 0.05), (x, 0.004, 0.0)]
    radii = [0.075, 0.080, 0.150, 0.165, 0.175, 0.180]
    g.limb(path, radii, "brown", sides=12, flat=0.14, smooth=True)  # 판(위·아래 끝은 감침 테가 가린다)
    for side in (-1, 1):
        g.limb([(px + side * (r - 0.012), py - 0.004, pz) for (px, py, pz), r in zip(path, radii)], [0.008] * len(path), "k_wood",
               sides=5, smooth=True)  # 가장자리 감침
    g.limb([(x - 0.155, -0.028, 0.205), (x, -0.040, 0.205), (x + 0.155, -0.028, 0.205)], [0.016, 0.016, 0.016], "p_belt", sides=6,
           flat=0.6, smooth=True)  # 허리 띠
    g.blob((x - 0.08, -0.036, 0.075), (0.09, 0.018, 0.07), "charcoal", r=1.0, n=3)  # 그을음
    g.blob((x + 0.09, -0.034, 0.24), (0.05, 0.02, 0.04), "charcoal", r=1.0, n=2)
    g.stone((x + 0.03, -0.058, 0.13), (0.16, 0.022, 0.085), "wood", bevel=0.008)  # 앞주머니
    g.box((x + 0.03, -0.068, 0.168), (0.15, 0.006, 0.008), "p_cream")  # 주머니 입 땀
    for dx in (-0.045, 0.105):
        g.blob((x + dx, -0.071, 0.168), (0.016, 0.012, 0.016), "p_gold", r=1.0, n=1)  # 징
    g.limb([(x + 0.075, -0.062, 0.14), (x + 0.085, -0.066, 0.22), (x + 0.09, -0.068, 0.25)], [0.010, 0.010, 0.010], "k_wood",
           sides=5)  # 망치 자루
    g.stone((x + 0.09, -0.068, 0.265), (0.065, 0.032, 0.032), "steel", bevel=0.006, rot=(0, 10, 0))  # 망치 머리
    g.limb([(x - 0.075, 0.0, 0.355), (x - 0.06, 0.0, 0.44), (x + 0.06, 0.0, 0.44), (x + 0.075, 0.0, 0.355)], [0.013] * 4, "p_belt",
           sides=5, smooth=True)  # 목끈
    for dx in (-0.075, 0.075):
        g.blob((x + dx, -0.012, 0.352), (0.020, 0.012, 0.020), "p_gold", r=1.0, n=1)
    # 허리끈: 허리 띠 양끝에서 옆으로 늘어진다(왼쪽은 나비 매듭).
    g.limb([(x + 0.155, -0.028, 0.205), (x + 0.20, -0.02, 0.19), (x + 0.22, -0.01, 0.12), (x + 0.215, 0.0, 0.06)],
           [0.012, 0.012, 0.011, 0.010], "p_belt", sides=5, smooth=True)
    g.blob((x - 0.175, -0.03, 0.205), (0.028, 0.022, 0.028), "p_belt", r=1.0, n=2)
    for dz in (0.03, -0.03):
        g.limb([(x - 0.185, -0.03, 0.205 + dz * 0.2), (x - 0.215, -0.034, 0.205 + dz * 1.0), (x - 0.240, -0.034, 0.205 + dz * 0.9),
                (x - 0.245, -0.032, 0.205 + dz * 0.3)], [0.011, 0.012, 0.012, 0.010], "p_belt", sides=5, smooth=True)  # 나비 고리
    g.limb([(x - 0.18, -0.03, 0.195), (x - 0.20, -0.02, 0.12)], [0.010, 0.008], "p_belt", sides=5, smooth=True)


# 이름 = 장신구 기록 키(`src/charms.rs` `key`, 바꾸지 않는다) = 게임이 찾는 부위 이름(`item_view.rs`).
CHARMS = [("shell", shell), ("feet", feet), ("ear", ear), ("hunger", hunger), ("hand", hand), ("pouch", pouch),
          ("cloth", cloth), ("jar", jar), ("bellshard", bellshard)]
# 덧옷(0176) — 부위 이름 = `src/garb.rs`의 `node`. 파일은 따로(`garbs.glb`, 2026-11-23): 메뉴는 파일 한 벌을 그리기 한 번에 넘기고
# 다른 부위를 크기 0으로 숨긴다 — 디테일을 넣은 뒤 한 파일로는 그리기 한 번의 색인 한도(0036)를 넘었다.
GARBS = [("garb_cloak", garb_cloak), ("garb_bark", garb_bark), ("garb_apron", garb_apron)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR):
    paths = []
    for name, parts in (("charms", CHARMS), ("garbs", GARBS)):
        m = C.Model(name)
        for i, (key, make) in enumerate(parts):
            make(m.geo(key), (i - (len(parts) - 1) / 2) * STEP)
        m.build()
        paths.append(m.export(models_dir))
        m.preview(os.path.join(preview_dir, f"{name}.png"), None, 0.0, size=640, floor=False)
    return paths


if __name__ == "__main__":
    build()
