"""성소의 소품 여덟(벽 조각·부서진 기둥·뿌리 덤불·화로·성소 문·핵 결정·종·돌무더기) + 손대는 물건 셋(장신구 상자·새긴 돌·문 장치, 0139).
파일마다 .glb 하나.

움직이는 것(문·핵 결정·종)만 뼈가 있다 — 문짝 door_l·door_r(바깥 모서리가 경첩), 결정 crystal, 종 bell·clapper.
가만히 있는 것은 메시 노드 하나(+ 빛나는 부위 glow_*).

v2(0139): 0061 그림 기준으로 다시 빚었다 — 상자 + 돌 쌓기 무늬 칸 대신 `common.Geo`의 깎은 돌(`stone`)·돌린 꼴(`lathe`)·
꺾인 관(`limb`)과 무늬 없는 단색 칸(`k_*`). 돌은 큰 덩어리 몇 개로 쌓아 사이 금을 기하로 내고(렌더의 선 둘이 긋는다),
둥근 것(기둥 북·그릇·종)은 매끈하게, 뿌리·결정·테만 각지게(0037: 각진 것 = 뿌리). 치수·노드·뼈·동작 이름은 v1 그대로.

    blender --background --python tools/art/blender/props.py            # 전부
    blender --background --python tools/art/blender/props.py -- gate    # 하나만(모델·미리보기 폴더는 기본값)
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

GROUND = ("-z",)


def root(g, pts, r0, mat="k_droot", tip="k_root", sides=4, roll=0.0):
    """뿌리 한 줄기: 밑동 반지름 r0에서 끝으로 가늘어져 뾰족하게. 끝 마디만 밝은 보라(tip)."""
    n = len(pts)
    radii = [r0 * (1 - i / (n - 1)) ** 0.8 for i in range(n)]
    mats = [mat] * (n - 2) + [tip]
    g.limb(pts, radii, mats, sides=sides, roll=roll)


def wall_segment():
    """폭 2.0m(x)·두께 0.8m(y)·높이 약 3.0m. 바닥 가운데가 원점, x로 나란히 놓으면 이어진다(끝 면이 x ±1.0).
    큰 마름돌 여덟 덩어리를 엇갈려 쌓고 오른쪽 위가 무너졌다 — 돌 사이 금은 깎인 모서리가 만든다."""
    m = C.Model("wall_segment")
    g = m.geo("wall_segment")
    g.stone((0, 0, 0.14), (2.0, 0.92, 0.28), "k_dark", bevel=0.05, edges="t", drop=GROUND)  # 굽돌
    # 1단(z 0.28–1.08)
    g.stone((-0.44, 0, 0.68), (1.12, 0.80, 0.80), "k_stone", bevel=0.07, drop=("-z", "+z"))
    g.stone((0.56, 0, 0.68), (0.88, 0.80, 0.80), "k_dark", bevel=0.07, drop=("-z", "+z"))
    # 2단(z 1.08–1.82)
    g.stone((-0.62, 0.01, 1.45), (0.76, 0.78, 0.74), "k_dark", bevel=0.07, drop=("-z", "+z"))
    g.stone((0.38, 0, 1.45), (1.24, 0.80, 0.74), "k_stone", bevel=0.07, drop=GROUND)
    # 3단(z 1.82–2.48): 오른쪽은 무너져 낮은 조각만
    g.stone((-0.475, 0, 2.15), (1.05, 0.80, 0.66), "k_stone", bevel=0.07, drop=GROUND)
    g.stone((0.36, 0.02, 2.11), (0.60, 0.74, 0.58), "k_light", bevel=0.07, drop=GROUND, taper=(0.86, 0.94), shift=(-0.03, 0))
    g.stone((0.83, 0, 1.95), (0.34, 0.76, 0.26), "k_dark", bevel=0.06, drop=GROUND, taper=(0.7, 0.9), shift=(-0.03, 0))
    # 4단: 왼쪽만 남고, 그 곁에 비뚤게 얹힌 돌
    g.stone((-0.64, 0, 2.74), (0.72, 0.78, 0.52), "k_stone", bevel=0.07, drop=GROUND)
    g.stone((-0.02, 0.02, 2.64), (0.46, 0.66, 0.30), "k_dark", bevel=0.06, rot=(3, -7, 9), edges="tvb")
    # 이끼(윗면과 굽돌 턱)
    g.stone((-0.64, 0.02, 3.02), (0.56, 0.62, 0.05), "k_moss", bevel=0.02, edges="t", drop=GROUND)
    g.stone((-0.38, -0.43, 0.30), (0.84, 0.10, 0.05), "k_moss", bevel=0.02, edges="t", drop=GROUND)
    g.stone((0.50, -0.64, 0.15), (0.40, 0.32, 0.28), "k_stone", bevel=0.06, rot=(7, 0, 22), edges="tvb")  # 떨어진 돌
    # 틈을 따라 오른 뿌리(앞 = -Y) — 꼭대기를 넘어간다
    root(g, [(0.62, -0.50, 0.02), (0.44, -0.47, 0.72), (0.14, -0.46, 1.36), (0.20, -0.45, 2.02), (0.08, -0.30, 2.52), (0.16, 0.05, 2.60)], 0.10)
    root(g, [(0.15, -0.46, 1.34), (-0.22, -0.45, 1.66), (-0.40, -0.44, 2.10)], 0.055, tip="k_rootl")
    m.build()
    return m, []


def pillar_broken():
    """폭 1.2m, 높이 약 2.9m, 윗부분이 부서진 기둥. 네모 받침 + 둥근 북 셋(위는 비스듬히 부러짐) + 굴러떨어진 조각."""
    m = C.Model("pillar_broken")
    g = m.geo("pillar_broken")
    g.stone((0, 0, 0.15), (1.2, 1.2, 0.30), "k_dark", bevel=0.06, drop=GROUND)  # 받침
    g.lathe((0, 0, 0.30), [(0.53, 0.0), (0.53, 0.10), (0.44, 0.20)], "k_light", sides=10, smooth=True, caps=(False, False))  # 받침 고리
    # 북 셋: 이음매마다 조금 좁혔다 넓어진다(둥글게 — 툰 띠가 감긴다). 위로 갈수록 살짝 가늘다.
    g.lathe((0, 0, 0.50), [(0.40, 0.0), (0.42, 0.06), (0.42, 0.76), (0.39, 0.82)], "k_stone", sides=10, smooth=True, caps=(False, False))
    g.lathe((0.01, 0, 1.32), [(0.385, 0.0), (0.405, 0.06), (0.40, 0.72), (0.37, 0.78)], "k_stone", sides=10, smooth=True, caps=(False, True),
            cap_mat="k_dark")
    # 부러진 윗북: 한쪽으로 밀리고 기울었다, 윗면은 비스듬한 깨진 면(밝은 돌)
    g.lathe((0.04, 0.02, 2.10), [(0.375, 0.0), (0.39, 0.06), (0.385, 0.50), (0.30, 0.74)], "k_stone", sides=10, smooth=True, hard=(2,),
            rot=(6, -5, 20), caps=(False, True), cap_mat="k_light")
    g.stone((0.10, 0.06, 2.84), (0.30, 0.34, 0.18), "k_light", bevel=0.05, rot=(10, -8, 15), taper=(0.6, 0.6), edges="tv")  # 깨진 끝
    # 굴러떨어진 북 조각(옆으로 누움)
    g.lathe((0.80, -0.44, 0.24), [(0.24, -0.20), (0.25, -0.14), (0.25, 0.14), (0.20, 0.20)], "k_dark", sides=8, smooth=True,
            rot=(90, 0, 35), caps=(True, True), cap_mat="k_stone")
    # 감아 오르는 뿌리
    root(g, [(-0.58, -0.36, 0.02), (-0.50, -0.30, 0.60), (-0.20, -0.46, 1.10), (0.24, -0.40, 1.50), (0.44, -0.10, 1.90), (0.30, 0.20, 2.20)], 0.09)
    root(g, [(-0.20, -0.46, 1.08), (-0.44, -0.20, 1.50), (-0.42, 0.02, 1.92)], 0.05, tip="k_rootl")
    g.stone((-0.30, 0.30, 0.32), (0.50, 0.46, 0.05), "k_moss", bevel=0.02, edges="t", drop=GROUND)
    m.build()
    return m, []


def root_clump():
    """바닥에서 비틀려 솟은 뿌리 덤불. 바닥 약 1.5m, 높이 약 1m. 굵은 밑동에서 갈래가 휘어 나오고 끝은 가시처럼 뾰족하다."""
    m = C.Model("root_clump")
    g = m.geo("root_clump")
    g.lathe((0, 0, 0), [(0.68, 0.0), (0.56, 0.08), (0.30, 0.15)], "k_droot", sides=7, smooth=True, squash=(1.0, 0.86), caps=(False, True))  # 흙 둔덕
    # 가운데 줄기: 비틀려 솟아 끝이 갈라진다
    root(g, [(-0.04, 0.02, 0.10), (0.03, -0.03, 0.42), (-0.08, 0.05, 0.74), (0.06, 0.10, 0.98), (0.20, 0.06, 1.10)], 0.20, tip="k_rootl")
    root(g, [(-0.07, 0.05, 0.72), (-0.24, -0.04, 0.92), (-0.30, -0.16, 1.02)], 0.07, tip="k_rootl")
    # 둘레로 뻗은 뿌리(땅을 기다 끝이 들린다)
    root(g, [(0.10, 0.02, 0.22), (0.46, 0.10, 0.20), (0.74, 0.20, 0.10), (0.92, 0.22, 0.34)], 0.12, mat="k_root", tip="k_rootl")
    root(g, [(-0.10, 0.06, 0.22), (-0.44, 0.20, 0.24), (-0.70, 0.16, 0.12), (-0.84, 0.02, 0.40)], 0.12)
    root(g, [(0.02, -0.12, 0.20), (0.12, -0.46, 0.26), (0.06, -0.72, 0.12), (-0.10, -0.86, 0.30)], 0.11, mat="k_root", tip="k_rootl")
    root(g, [(0.0, 0.12, 0.20), (0.14, 0.46, 0.22), (0.06, 0.70, 0.10), (0.20, 0.82, 0.26)], 0.10)
    # 휘어 오른 가시 둘
    root(g, [(0.34, -0.24, 0.14), (0.52, -0.34, 0.46), (0.44, -0.30, 0.78)], 0.08, tip="k_rootl")
    root(g, [(-0.40, -0.30, 0.12), (-0.58, -0.40, 0.36), (-0.52, -0.30, 0.60)], 0.07, mat="k_root", tip="k_rootl")
    g.stone((-0.24, -0.40, 0.13), (0.20, 0.14, 0.10), "k_bone", bevel=0.03, rot=(0, 8, 25), edges="tv", drop=GROUND)  # 뼈 조각
    g.stone((0.30, 0.34, 0.13), (0.30, 0.24, 0.05), "k_moss", bevel=0.02, edges="t", drop=GROUND)
    m.build()
    return m, []


def brazier():
    """쉼터의 화로: 돌 받침 위 넓은 그릇(테 위 0.86m), 불꽃(glow_flame) 끝 약 1.42m. "아직 색이 타는 곳" — 멀리서도 그릇과 불꽃이 따로 읽히게."""
    m = C.Model("brazier")
    g = m.geo("brazier")
    g.stone((0, 0, 0.07), (0.62, 0.62, 0.14), "k_dark", bevel=0.05, edges="tv", drop=GROUND)  # 받침돌
    g.lathe((0, 0, 0.14), [(0.21, 0.0), (0.13, 0.08), (0.11, 0.36), (0.17, 0.44)], "k_stone", sides=8, smooth=True, caps=(False, False))  # 기둥
    # 그릇: 아래는 둥글게, 테는 각지게(밝은 선이 붙는 테)
    g.lathe((0, 0, 0.58), [(0.16, 0.0), (0.30, 0.08), (0.37, 0.20)], "k_stone", sides=8, smooth=True, caps=(True, False))
    g.lathe((0, 0, 0.78), [(0.37, 0.0), (0.39, 0.03), (0.39, 0.08), (0.33, 0.08)], ["k_light", "k_light", "k_light"], sides=8, caps=(False, False))
    g.lathe((0, 0, 0.80), [(0.33, 0.0), (0.33, 0.02)], "k_deep", sides=8, caps=(False, True), cap_mat="k_deep")  # 숯
    g = m.geo("glow_flame")
    # 불꽃: 밑동이 가장 밝고(엷은 불씨) 끝으로 갈수록 짙은 호박빛 — 휘어 오른 큰 혀 하나 + 곁 혀 둘(위에서 봐도 색 띠가 보인다)
    g.limb([(0.0, 0.0, 0.80), (0.0, 0.0, 0.90), (0.02, -0.01, 1.02), (0.05, -0.02, 1.16), (0.02, 0.0, 1.29), (-0.04, 0.02, 1.42)],
           [0.20, 0.25, 0.20, 0.12, 0.06, 0.0], ["k_ember", "k_ember", "k_amber", "k_amber", "k_flame"], sides=6, smooth=True)
    g.limb([(0.16, 0.08, 0.82), (0.19, 0.10, 0.92), (0.24, 0.12, 1.02), (0.22, 0.10, 1.14)],
           [0.09, 0.10, 0.055, 0.0], ["k_ember", "k_amber", "k_flame"], sides=5, smooth=True)
    g.limb([(-0.15, -0.10, 0.82), (-0.18, -0.12, 0.90), (-0.22, -0.13, 0.98), (-0.19, -0.11, 1.07)],
           [0.08, 0.09, 0.05, 0.0], ["k_ember", "k_amber", "k_flame"], sides=5, smooth=True)
    m.build()
    return m, []


def gate():
    """성소 문: 폭 6m·높이 4.5m의 돌 문틀 + 문짝 둘. door_l = 모델의 왼쪽(+X, 앞에서 보면 오른쪽), door_r = -X.
    경첩은 바깥 모서리(x ±2.25). 동작 open 1.5초: 문짝이 앞(-Y, 보는 쪽)으로 100° 열린다. 0프레임은 닫힘.
    문틀은 큰 돌 셋씩 쌓은 설주 + 상인방 돌 셋(가운데 이맛돌), 문짝은 널 넷 + 청동 띠 둘 + 고리."""
    m = C.Model("gate")
    m.bone("door_l", (2.25, 0, 0))
    m.bone("door_r", (-2.25, 0, 0))
    g = m.geo("gate_frame")
    for sx in (1, -1):
        g.stone((2.65 * sx, 0, 0.16), (1.02, 1.22, 0.32), "k_dark", bevel=0.07, drop=GROUND)  # 설주 받침
        g.stone((2.65 * sx, 0, 0.92), (0.82, 1.02, 1.20), "k_stone", bevel=0.08, drop=("-z", "+z"))  # 설주 돌 셋
        g.stone((2.66 * sx, 0, 2.08), (0.78, 0.98, 1.12), "k_dark" if sx > 0 else "k_stone", bevel=0.08, drop=("-z", "+z"))
        g.stone((2.65 * sx, 0, 3.20), (0.82, 1.02, 1.12), "k_stone" if sx > 0 else "k_dark", bevel=0.08, drop=("-z", "+z"))
        g.stone((2.02 * sx, 0, 3.52), (0.56, 0.92, 0.46), "k_light", bevel=0.07, rot=(0, 38 * sx, 0), edges="tvb")  # 모서리 버팀돌
        g.stone((1.72 * sx, 0, 4.14), (2.56, 1.10, 0.74), "k_stone", bevel=0.09)  # 상인방 좌우
    g.stone((0, -0.05, 4.06), (0.92, 1.20, 0.96), "k_light", bevel=0.09, taper=(1.22, 1.0))  # 이맛돌(가운데, 앞으로 내밈)
    g.stone((-1.50, 0, 4.53), (1.40, 0.90, 0.06), "k_moss", bevel=0.02, edges="t", drop=GROUND)
    # 상인방을 타고 내린 뿌리(앞 = -Y)
    root(g, [(2.20, -0.60, 4.50), (1.60, -0.62, 4.16), (1.10, -0.60, 4.02), (0.72, -0.58, 3.70), (0.80, -0.56, 3.30)], 0.11)
    root(g, [(2.30, -0.56, 3.60), (2.22, -0.56, 2.90), (2.34, -0.56, 2.20), (2.26, -0.56, 1.60)], 0.08, mat="k_root", tip="k_rootl")
    for side, sx in (("l", 1), ("r", -1)):
        g = m.geo(f"door_{side}_leaf", f"door_{side}")
        # 널 넷(폭이 조금씩 다르고 번갈아 어둡다) — 사이 금은 깎인 모서리
        x = 0.0
        for i, w in enumerate((0.60, 0.52, 0.58, 0.55)):
            g.stone(((x + w / 2) * sx, 0, 1.75), (w, 0.16, 3.48), "k_wood" if i % 2 == 0 else "k_woodl", bevel=0.03, edges="v", drop=GROUND)
            x += w
        for z in (0.80, 2.70):
            g.stone((1.125 * sx, -0.09, z), (2.21, 0.05, 0.22), "k_bronze", bevel=0.02, edges="t", drop=("+y",))  # 청동 띠
            for bx in (0.30, 1.125, 1.95):
                g.lathe((bx * sx, -0.115, z), [(0.06, 0.0), (0.0, 0.05)], "k_bronzel", sides=4, rot=(90, 0, 0), caps=(False, False))  # 못
        g.lathe((0.30 * sx, -0.11, 1.72), [(0.13, 0.0), (0.13, 0.04), (0.08, 0.04)], "k_bronzel", sides=8, rot=(90, 0, 0), caps=(False, True),
                cap_mat="k_deep")  # 문고리
    m.build()
    m.action("open", 1.5, [
        (0.0, {"door_l": {"r": (0, 0, 0)}, "door_r": {"r": (0, 0, 0)}}),
        (0.25, {"door_l": {"r": (0, 0, 5)}, "door_r": {"r": (0, 0, -5)}}),
        (0.4, {"door_l": {"r": (0, 0, 4)}, "door_r": {"r": (0, 0, -4)}}),
        (1.5, {"door_l": {"r": (0, 0, 100)}, "door_r": {"r": (0, 0, -100)}}),
    ])
    return m, [("open", 1.5, "_open")]


def core_crystal():
    """보스가 남기는 핵: 뿌리 요람 속 호박빛 결정(glow_core), 높이 0.6m. 동작 pulse 1.0초 고리(살짝 부풀고 뜸).
    결정은 여섯 모 쌍뿔(각진 면 — 밝은 선이 붙는다) + 곁 조각, 요람은 발톱처럼 감아 쥔 뿌리 다섯."""
    m = C.Model("core_crystal")
    m.bone("crystal", (0, 0, 0.22))
    g = m.geo("core_cradle")
    g.lathe((0, 0, 0), [(0.27, 0.0), (0.22, 0.05), (0.10, 0.08)], "k_droot", sides=6, smooth=True, caps=(False, True))  # 바닥 뿌리 둔덕
    import math
    for i in range(5):
        a = math.radians(72 * i + 18)
        cx, sy = math.cos(a), math.sin(a)
        root(g, [(0.10 * cx, 0.10 * sy, 0.04), (0.24 * cx, 0.24 * sy, 0.14), (0.25 * cx, 0.25 * sy, 0.32), (0.15 * cx, 0.15 * sy, 0.47)],
             0.055, mat="k_root" if i % 2 else "k_droot", tip="k_rootl")
    g = m.geo("glow_core", "crystal")
    g.lathe((0, 0, 0.10), [(0.0, 0.0), (0.14, 0.14), (0.16, 0.30), (0.0, 0.52)], ["k_amber", "k_ember", "k_ember"], sides=6, caps=(False, False))
    g.lathe((0.11, 0.05, 0.14), [(0.0, 0.0), (0.05, 0.08), (0.0, 0.24)], ["k_amber", "k_core"], sides=4, rot=(0, 22, 0), caps=(False, False))  # 곁 조각
    m.build()
    m.action("pulse", 1.0, [
        (0.0, {"crystal": {"s": 1.0, "l": (0, 0, 0)}}),
        (0.5, {"crystal": {"s": 1.07, "l": (0, 0, 0.03), "r": (0, 0, 10)}}),
    ], loop=True)
    return m, [("pulse", 0.5, "_pulse")]


def bell():
    """대들보에 매달린 금 간 청동 종, 전체 약 2.1m. 뼈 bell(대들보 아래 피벗)·clapper(추, 늦게 따라 흔들림).
    동작 toll 2.0초: 크게 흔들리다 잦아든다. 종 몸은 돌려 만든 둥근 꼴(입술 z 1.05 — 내던지는 종 0133이 이 높이를 쓴다), 입술 테만 각지게."""
    m = C.Model("bell")
    m.bone("bell", (0, 0, 1.90))
    m.bone("clapper", (0, 0, 1.75), "bell")
    g = m.geo("bell_frame")
    for sx in (1, -1):
        g.stone((0.80 * sx, 0, 1.00), (0.17, 0.17, 1.84), "k_wood", bevel=0.03, edges="v", drop=GROUND)  # 기둥
        g.stone((0.80 * sx, 0, 0.09), (0.40, 0.46, 0.18), "k_stone", bevel=0.05, drop=GROUND)  # 주춧돌
        g.limb([(0.74 * sx, 0, 1.52), (0.42 * sx, 0, 1.90)], [0.06, 0.06], "k_woodl", roll=45)  # 버팀대
    g.stone((0, 0, 2.00), (1.96, 0.22, 0.20), "k_woodl", bevel=0.035, edges="tvb")  # 대들보
    g = m.geo("bell_body", "bell")
    g.lathe((0, 0, 1.80), [(0.05, 0.0), (0.05, 0.10)], "k_deep", sides=4, caps=(False, True))  # 고리
    g.lathe((0, 0, 1.05), [(0.37, 0.07), (0.33, 0.12), (0.27, 0.36), (0.23, 0.58), (0.15, 0.72), (0.0, 0.77)], "k_bronze", sides=10, smooth=True,
            caps=(False, False))  # 몸(어깨까지 둥글게)
    g.lathe((0, 0, 1.05), [(0.33, 0.0), (0.39, 0.0), (0.39, 0.07), (0.37, 0.07)], "k_bronzel", sides=10, caps=(False, False))  # 입술 테
    g.lathe((0, 0, 1.05), [(0.33, 0.0), (0.33, 0.01)], "k_deep", sides=10, caps=(True, False), cap_mat="k_deep")  # 속(아래에서 보이는 어둠)
    g.lathe((0, 0, 1.55), [(0.255, 0.0), (0.262, 0.025), (0.248, 0.05)], "k_bronzel", sides=10, caps=(False, False))  # 띠
    # 금(앞 = -Y, 입술에서 위로 번개꼴)
    g.limb([(0.05, -0.385, 1.06), (0.02, -0.34, 1.20), (0.09, -0.305, 1.34), (0.04, -0.275, 1.50)], [0.03, 0.028, 0.02, 0.0], "k_deep", sides=3, roll=30)
    g = m.geo("bell_clapper", "clapper")
    g.limb([(0, 0, 1.72), (0, 0, 1.16)], [0.02, 0.02], "k_deep", sides=3)
    g.lathe((0, 0, 1.00), [(0.0, 0.0), (0.07, 0.06), (0.07, 0.12), (0.0, 0.18)], "k_deep", sides=6, smooth=True, caps=(False, False))  # 추
    m.build()
    m.action("toll", 2.0, [
        (0.0, {"bell": {"r": (0, 0, 0)}, "clapper": {"r": (0, 0, 0)}}),
        (0.3, {"bell": {"r": (28, 0, 0)}, "clapper": {"r": (-14, 0, 0)}}),
        (0.75, {"bell": {"r": (-22, 0, 0)}, "clapper": {"r": (18, 0, 0)}}),
        (1.15, {"bell": {"r": (14, 0, 0)}, "clapper": {"r": (-12, 0, 0)}}),
        (1.5, {"bell": {"r": (-7, 0, 0)}, "clapper": {"r": (8, 0, 0)}}),
        (1.8, {"bell": {"r": (2, 0, 0)}, "clapper": {"r": (-3, 0, 0)}}),
        (2.0, {"bell": {"r": (0, 0, 0)}, "clapper": {"r": (0, 0, 0)}}),
    ])
    return m, [("toll", 0.3, "_toll")]


def rubble():
    """작은 돌무더기, 약 1m. 무너진 마름돌 여섯 — 크기·기울기·밝기가 다 다르다."""
    m = C.Model("rubble")
    g = m.geo("rubble")
    g.stone((0.02, 0.02, 0.17), (0.52, 0.42, 0.34), "k_stone", bevel=0.07, rot=(0, 0, 12), taper=(0.9, 0.9), drop=GROUND)
    g.stone((0.38, 0.16, 0.11), (0.40, 0.34, 0.22), "k_dark", bevel=0.06, rot=(0, 0, -20), taper=(0.85, 0.9), drop=GROUND)
    g.stone((-0.34, -0.10, 0.12), (0.36, 0.42, 0.24), "k_light", bevel=0.06, rot=(0, 0, 35), taper=(0.9, 0.85), drop=GROUND)
    g.stone((0.06, -0.34, 0.08), (0.30, 0.26, 0.16), "k_dark", bevel=0.05, rot=(0, 0, 50), drop=GROUND)
    g.stone((0.10, 0.06, 0.45), (0.34, 0.28, 0.24), "k_light", bevel=0.06, rot=(14, 9, 25), edges="tvb")
    g.stone((-0.22, 0.32, 0.07), (0.24, 0.20, 0.14), "k_stone", bevel=0.045, rot=(0, 0, 10), drop=GROUND)
    g.stone((-0.34, -0.10, 0.255), (0.24, 0.26, 0.04), "k_moss", bevel=0.015, rot=(0, 0, 35), edges="t", drop=GROUND)
    m.build()
    return m, []


def chest():
    """장신구 상자(0064 M3·0139): 갈색 나무 궤 0.86×0.56m + 둥근 뚜껑 + 놋쇠 띠 둘 + 호박빛 자물쇠(glow_lock — 닫힌 동안 게임이 숨 쉬듯 밝힌다).
    앞 = -Y. 뼈 lid(뒤 모서리 경첩). 동작 open 0.4초: 뚜껑이 뒤로 젖혀진다(끝 = 열린 자세 — 게임은 열린 상자를 끝 프레임으로 둔다)."""
    m = C.Model("chest")
    m.bone("lid", (0, 0.27, 0.54))
    g = m.geo("chest_body")
    g.stone((0, 0, 0.035), (0.92, 0.62, 0.07), "k_chestl", bevel=0.02, edges="t", drop=GROUND)  # 굽
    g.stone((0, 0, 0.29), (0.86, 0.56, 0.50), "k_chest", bevel=0.035, edges="v", drop=GROUND, top_mat="k_deep")  # 몸(열면 속이 어둡다)
    for sx in (1, -1):
        g.stone((0.27 * sx, 0, 0.29), (0.11, 0.60, 0.50), "k_brass", bevel=0.02, edges="v", drop=("-z", "+z"))  # 띠
    g = m.geo("chest_lid", "lid")
    g.stone((0, 0, 0.63), (0.90, 0.60, 0.18), "k_chestl", bevel=0.075, edges="t")  # 뚜껑(윗모서리를 크게 깎아 둥글게)
    for sx in (1, -1):
        g.stone((0.27 * sx, 0, 0.635), (0.11, 0.64, 0.20), "k_brass", bevel=0.08, edges="t", drop=GROUND)
    g = m.geo("glow_lock")
    g.stone((0, -0.295, 0.45), (0.15, 0.05, 0.17), "k_amber", bevel=0.025, edges="v", taper=(1.0, 1.0))
    g.stone((0, -0.325, 0.43), (0.05, 0.02, 0.07), "k_core", bevel=0.0)  # 열쇠 구멍 자리의 밝은 점
    m.build()
    m.action("open", 0.4, [
        (0.0, {"lid": {"r": (0, 0, 0)}}),
        (0.26, {"lid": {"r": (-118, 0, 0)}}),
        (0.4, {"lid": {"r": (-109, 0, 0)}}),
    ])
    return m, [("open", 0.4, "_open")]


def stone_tablet():
    """새긴 돌(0129·0139): 위가 좁은 선돌 0.62×1.1m + 새긴 줄 셋(glow_carve — 곁에 서면 게임이 밝힌다) + 이끼 낀 밑동. 앞 = -Y.
    기울기·방향은 게임이 준다."""
    m = C.Model("stone_tablet")
    g = m.geo("stone_tablet")
    g.stone((0, 0, 0.05), (0.78, 0.34, 0.10), "k_moss", bevel=0.03, edges="t", drop=GROUND)  # 밑동
    g.stone((0, 0, 0.58), (0.62, 0.20, 1.10), "k_light", bevel=0.06, taper=(0.80, 0.86), shift=(0.02, 0), drop=GROUND)
    g.stone((0.21, -0.02, 1.05), (0.20, 0.18, 0.16), "k_stone", bevel=0.04, rot=(0, 28, 0), edges="tvb")  # 깨져 나간 어깨의 그늘
    g = m.geo("glow_carve")
    for k in range(3):
        g.stone((0.0, -0.094, 0.82 - k * 0.2), (0.40 - k * 0.08, 0.03, 0.045), "k_bone", bevel=0.0)
    m.build()
    return m, []


def lever():
    """문 장치(0113·0139): 돌 기둥 0.5×0.9m + 앞에 달린 손잡이(뼈 handle, 경첩 z 0.8) + 손잡이 끝 호박빛 쥘 곳(glow_grip — 당기기 전에만).
    앞 = -Y. 동작 pull 0.4초: 손잡이가 뒤로 선 자세(0초)에서 앞으로 넘어간다(끝 = 당긴 자세)."""
    m = C.Model("lever")
    m.bone("handle", (0, -0.27, 0.80))
    g = m.geo("lever_post")
    g.stone((0, 0, 0.06), (0.62, 0.62, 0.12), "k_dark", bevel=0.04, drop=GROUND)
    g.stone((0, 0, 0.50), (0.48, 0.48, 0.78), "k_stone", bevel=0.06, taper=(0.9, 0.9), drop=GROUND)
    g.stone((0, 0, 0.905), (0.52, 0.52, 0.07), "k_light", bevel=0.03, drop=GROUND)  # 갓돌
    g.stone((0, -0.25, 0.80), (0.22, 0.10, 0.18), "k_deep", bevel=0.03, edges="tvb")  # 경첩 쇠
    g = m.geo("lever_handle", "handle")
    g.limb([(0, -0.27, 0.80), (0, -0.27, 1.42)], [0.05, 0.04], "k_bronze", roll=45, caps=(False, True))
    g = m.geo("glow_grip", "handle")
    g.lathe((0, -0.27, 1.38), [(0.0, 0.0), (0.09, 0.05), (0.09, 0.12), (0.0, 0.18)], "k_amber", sides=6, smooth=True, caps=(False, False))
    m.build()
    m.action("pull", 0.4, [
        (0.0, {"handle": {"r": (-28.6, 0, 0)}}),
        (0.28, {"handle": {"r": (64, 0, 0)}}),
        (0.4, {"handle": {"r": (57.3, 0, 0)}}),
    ])
    return m, [("pull", 0.4, "_pull")]


PROPS = {
    "wall_segment": wall_segment,
    "pillar_broken": pillar_broken,
    "root_clump": root_clump,
    "brazier": brazier,
    "gate": gate,
    "core_crystal": core_crystal,
    "bell": bell,
    "rubble": rubble,
    "chest": chest,
    "stone_tablet": stone_tablet,
    "lever": lever,
}


def build_one(name, models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR):
    m, extra = PROPS[name]()
    path = m.export(models_dir)
    m.preview(os.path.join(preview_dir, f"{name}.png"), None, 0.0)
    for act, t, suffix in extra:
        m.preview(os.path.join(preview_dir, f"{name}{suffix}.png"), act, t)
    return path


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, only=None):
    return [build_one(n, models_dir, preview_dir) for n in PROPS if not only or n in only]


if __name__ == "__main__":
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    build(only=[a for a in argv if a in PROPS] or None)
