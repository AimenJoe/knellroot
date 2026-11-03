"""성소의 소품 여덟: 벽 조각·부서진 기둥·뿌리 덤불·화로·성소 문·핵 결정·종·돌무더기. 파일마다 .glb 하나.

움직이는 것(문·핵 결정·종)만 뼈가 있다 — 문짝 door_l·door_r(바깥 모서리가 경첩), 결정 crystal, 종 bell·clapper.
가만히 있는 것은 메시 노드 하나(+ 빛나는 부위 glow_*).

    blender --background --python tools/art/blender/props.py            # 전부
    blender --background --python tools/art/blender/props.py -- gate    # 하나만(모델·미리보기 폴더는 기본값)
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

GROUND = ("-z",)


def wall_segment():
    """폭 2.0m(x)·두께 0.8m(y)·높이 약 3.0m. 바닥 가운데가 원점, x로 나란히 놓으면 이음매 없이 이어진다(돌 단 셋이 x ±1.0에 딱 맞음)."""
    m = C.Model("wall_segment")
    g = m.geo("wall_segment")
    g.box((0, 0, 0.15), (2.0, 0.86, 0.30), "wall_light", drop=GROUND)  # 굽돌
    g.box((0, 0, 0.70), (2.0, 0.80, 0.80), "wall", drop=GROUND)  # 1단
    g.box((0, 0, 1.50), (2.0, 0.80, 0.80), "wall", drop=GROUND)  # 2단
    g.box((0, 0, 2.20), (2.0, 0.80, 0.60), "wall", drop=GROUND)  # 3단
    g.box((-0.725, 0, 2.75), (0.55, 0.80, 0.50), "wall", drop=GROUND)  # 부서진 윗단(높낮이가 다름)
    g.box((-0.15, 0, 2.61), (0.60, 0.78, 0.22), "stone", drop=GROUND)
    g.box((0.40, 0.02, 2.72), (0.48, 0.72, 0.44), "wall", rot=(0, 0, 4), drop=GROUND)
    g.box((0.825, 0, 2.56), (0.35, 0.80, 0.12), "stone", drop=GROUND)
    g.box((-0.725, 0, 3.02), (0.48, 0.66, 0.05), "moss")  # 이끼
    g.box((-0.35, -0.42, 0.315), (0.90, 0.14, 0.04), "moss")
    g.box((0.45, -0.58, 0.20), (0.42, 0.34, 0.30), "stone", rot=(8, 0, 20))  # 떨어진 돌
    g.box((0.20, -0.42, 1.30), (0.12, 0.08, 1.20), "droot", rot=(0, 25, 0))  # 갈라진 틈을 따라 난 뿌리
    g.box((-0.06, -0.42, 0.62), (0.10, 0.08, 0.62), "root", rot=(0, -15, 0))
    g.box((0.42, -0.42, 2.02), (0.09, 0.07, 0.60), "droot", rot=(0, 40, 0))
    g.box((0.10, -0.405, 1.62), (0.06, 0.02, 0.70), "charcoal", rot=(0, 18, 0), top=False)  # 틈 그늘
    g.box((0.32, 0.0, 2.53), (0.10, 0.90, 0.08), "droot")  # 윗단을 넘어가는 뿌리
    m.build()
    return m, []


def pillar_broken():
    """폭 1.2m, 높이 약 2.9m, 윗부분이 부서진 기둥. 겹친 상자 둘(45°)로 팔각 기둥처럼."""
    m = C.Model("pillar_broken")
    g = m.geo("pillar_broken")
    g.box((0, 0, 0.15), (1.2, 1.2, 0.30), "wall_light", drop=GROUND)  # 받침
    g.box((0, 0, 0.38), (1.0, 1.0, 0.16), "lstone", drop=GROUND)
    for z in (0,):
        g.box((0, 0, 1.21), (0.80, 0.80, 1.50), "wall", rot=(0, 0, z), drop=GROUND)  # 기둥 몸
    g.box((0, 0, 1.21), (0.80, 0.80, 1.50), "wall", rot=(0, 0, 45), drop=GROUND)
    g.box((0.02, 0, 2.25), (0.76, 0.76, 0.60), "wall", rot=(0, 0, 8), drop=GROUND)  # 윗마디
    g.box((0.02, 0, 2.25), (0.76, 0.76, 0.60), "wall", rot=(0, 0, 53), drop=GROUND)
    g.box((0.12, 0.08, 2.70), (0.55, 0.60, 0.40), "stone", rot=(10, -8, 15), taper=(0.7, 0.6))  # 부서진 끝
    g.box((-0.18, -0.12, 2.60), (0.36, 0.40, 0.24), "stone", rot=(-6, 12, 30), taper=(0.6, 0.6))
    g.box((0.78, -0.40, 0.18), (0.40, 0.34, 0.30), "wall", rot=(5, 10, 35))  # 떨어진 조각
    g.box((0, -0.45, 0.95), (0.90, 0.08, 0.10), "droot", rot=(0, 22, 0))  # 감아 오르는 뿌리
    g.box((0.45, 0, 1.55), (0.08, 0.90, 0.10), "droot", rot=(-25, 0, 0))
    g.box((-0.44, -0.10, 1.35), (0.09, 0.09, 1.60), "root", rot=(0, -5, 0), taper=(0.6, 0.6))
    g.box((-0.28, 0.22, 0.475), (0.56, 0.52, 0.04), "moss")
    m.build()
    return m, []


def root_clump():
    """바닥에서 비틀려 솟은 뿌리 덤불. 바닥 약 1.5m, 높이 약 1m."""
    m = C.Model("root_clump")
    g = m.geo("root_clump")
    g.box((0, 0, 0.06), (1.30, 1.10, 0.14), "droot", taper=(0.8, 0.8), drop=GROUND)  # 흙 둔덕
    g.box((0, 0, 0.44), (0.38, 0.38, 0.84), "bark", rot=(6, -8, 12), taper=(0.45, 0.45))  # 줄기
    g.box((0.44, 0.10, 0.30), (0.18, 0.18, 0.74), "bark", rot=(0, 58, 10), taper=(0.5, 0.5))  # 뻗은 뿌리
    g.box((-0.42, 0.15, 0.30), (0.18, 0.18, 0.74), "root", rot=(0, -60, -15), taper=(0.5, 0.5))
    g.box((0.05, -0.45, 0.26), (0.16, 0.16, 0.66), "bark", rot=(62, 0, 20), taper=(0.5, 0.5))
    g.box((0.10, 0.45, 0.24), (0.15, 0.15, 0.62), "root", rot=(-58, 0, -10), taper=(0.5, 0.5))
    g.box((0.52, -0.28, 0.55), (0.12, 0.12, 0.50), "droot", rot=(30, 40, 0), taper=(0.5, 0.5))  # 휘어 오른 뿌리
    g.box((0.20, 0.06, 0.92), (0.10, 0.10, 0.32), "root", rot=(-30, 30, 0), taper=(0.3, 0.3))  # 끝 갈래
    g.box((-0.58, -0.36, 0.12), (0.12, 0.12, 0.42), "droot", rot=(40, -60, 0), taper=(0.5, 0.5))
    g.box((-0.25, -0.36, 0.13), (0.18, 0.16, 0.14), "bone", rot=(0, 0, 25))  # 뼈 조각
    g.box((0.22, 0.30, 0.14), (0.30, 0.25, 0.06), "moss")
    m.build()
    return m, []


def brazier():
    """짧은 받침 위 돌 그릇(그릇 위 0.84m), 불꽃(glow_flame) 끝 약 1.12m."""
    m = C.Model("brazier")
    g = m.geo("brazier")
    g.box((0, 0, 0.06), (0.56, 0.56, 0.12), "stone", drop=GROUND)  # 받침돌
    g.box((0, 0, 0.34), (0.24, 0.24, 0.44), "stone", taper=(0.8, 0.8), drop=GROUND)  # 기둥
    g.box((0, 0, 0.59), (0.34, 0.34, 0.06), "lstone")  # 목
    g.box((0, 0, 0.71), (0.48, 0.48, 0.20), "lstone", taper=(1.45, 1.45))  # 그릇(팔각: 둘을 45° 겹침)
    g.box((0, 0, 0.71), (0.48, 0.48, 0.20), "lstone", taper=(1.45, 1.45), rot=(0, 0, 45))
    g.box((0, 0, 0.82), (0.58, 0.58, 0.04), "charcoal")  # 숯
    g = m.geo("glow_flame")
    g.box((0, 0, 0.93), (0.32, 0.32, 0.22), "amber", taper=(0.5, 0.5), rot=(0, 0, 20), drop=GROUND)
    g.box((0.03, 0.02, 1.04), (0.16, 0.16, 0.16), "ember", taper=(0.35, 0.35), rot=(0, 0, 55), drop=GROUND)
    g.box((-0.11, 0.07, 0.96), (0.10, 0.10, 0.16), "amber", taper=(0.35, 0.35), rot=(0, -15, 10), drop=GROUND)
    m.build()
    return m, []


def gate():
    """성소 문: 폭 6m·높이 4.5m의 돌 문틀 + 문짝 둘. door_l = 모델의 왼쪽(+X, 앞에서 보면 오른쪽), door_r = -X.
    경첩은 바깥 모서리(x ±2.25). 동작 open 1.5초: 문짝이 앞(-Y, 보는 쪽)으로 100° 열린다. 0프레임은 닫힘."""
    m = C.Model("gate")
    m.bone("door_l", (2.25, 0, 0))
    m.bone("door_r", (-2.25, 0, 0))
    g = m.geo("gate_frame")
    for sx in (1, -1):
        g.box((2.65 * sx, 0, 2.05), (0.80, 1.00, 3.50), "wall")  # 문설주
        g.box((2.65 * sx, 0, 0.15), (1.00, 1.20, 0.30), "wall_light", drop=GROUND)  # 설주 받침
        g.box((2.05 * sx, 0, 3.55), (0.70, 0.90, 0.42), "stone", rot=(0, 45 * sx, 0))  # 모서리 버팀돌
    g.box((0, 0, 4.13), (6.00, 1.10, 0.74), "wall")  # 상인방
    g.box((0, -0.04, 3.95), (0.80, 1.12, 0.90), "wall_light", taper=(1.3, 1.0))  # 이맛돌
    g.box((1.20, -0.57, 4.00), (1.60, 0.10, 0.12), "droot", rot=(0, 12, 0))  # 뿌리
    g.box((2.25, -0.52, 2.60), (0.12, 0.10, 1.80), "bark", rot=(0, -6, 0))
    g.box((-1.50, 0, 4.52), (1.40, 0.90, 0.05), "moss")
    for side, sx in (("l", 1), ("r", -1)):
        g = m.geo(f"door_{side}_leaf", f"door_{side}")
        g.box((1.125 * sx, 0, 1.75), (2.25, 0.16, 3.48), "door", drop=GROUND)  # 문짝
        g.box((0.35 * sx, -0.10, 1.60), (0.14, 0.05, 0.16), "bronze")  # 문고리
    m.build()
    m.action("open", 1.5, [
        (0.0, {"door_l": {"r": (0, 0, 0)}, "door_r": {"r": (0, 0, 0)}}),
        (0.25, {"door_l": {"r": (0, 0, 5)}, "door_r": {"r": (0, 0, -5)}}),
        (0.4, {"door_l": {"r": (0, 0, 4)}, "door_r": {"r": (0, 0, -4)}}),
        (1.5, {"door_l": {"r": (0, 0, 100)}, "door_r": {"r": (0, 0, -100)}}),
    ])
    return m, [("open", 1.5, "_open")]


def core_crystal():
    """보스가 남기는 핵: 뿌리 요람 속 호박빛 결정(glow_core), 높이 0.6m. 동작 pulse 1.0초 고리(살짝 부풀고 뜸)."""
    m = C.Model("core_crystal")
    m.bone("crystal", (0, 0, 0.22))
    g = m.geo("core_cradle")
    g.box((0, 0, 0.04), (0.50, 0.50, 0.08), "droot", drop=GROUND)  # 바닥 뿌리
    for sx, sy, r in ((1, 0, (0, 22, 0)), (-1, 0, (0, -22, 0)), (0, 1, (-22, 0, 0)), (0, -1, (22, 0, 0))):
        g.box((0.17 * sx, 0.17 * sy, 0.17), (0.09, 0.09, 0.34), "bark", rot=r, taper=(0.6, 0.6))  # 요람 뿌리
    for sx, sy, r in ((1, 0, (0, -35, 0)), (-1, 0, (0, 35, 0)), (0, 1, (35, 0, 0)), (0, -1, (-35, 0, 0))):
        g.box((0.22 * sx, 0.22 * sy, 0.38), (0.07, 0.07, 0.16), "root", rot=r, taper=(0.4, 0.4))  # 안으로 굽은 끝
    g = m.geo("glow_core", "crystal")
    g.box((0, 0, 0.21), (0.18, 0.18, 0.16), "amber", taper=(1.6, 1.6), rot=(0, 0, 45))  # 아래(좁게)
    g.box((0, 0, 0.42), (0.29, 0.29, 0.26), "amber", taper=(0.12, 0.12), rot=(0, 0, 45))  # 위(뾰족)
    g.box((0.10, 0.05, 0.31), (0.08, 0.08, 0.18), "ember", rot=(0, 25, 0), taper=(0.3, 0.3))  # 곁 조각
    m.build()
    m.action("pulse", 1.0, [
        (0.0, {"crystal": {"s": 1.0, "l": (0, 0, 0)}}),
        (0.5, {"crystal": {"s": 1.07, "l": (0, 0, 0.03), "r": (0, 0, 10)}}),
    ], loop=True)
    return m, [("pulse", 0.5, "_pulse")]


def bell():
    """대들보에 매달린 금 간 어두운 청동 종, 전체 약 2.1m. 뼈 bell(대들보 아래 피벗)·clapper(추, 늦게 따라 흔들림).
    동작 toll 2.0초: 크게 흔들리다 잦아든다."""
    m = C.Model("bell")
    m.bone("bell", (0, 0, 1.90))
    m.bone("clapper", (0, 0, 1.75), "bell")
    g = m.geo("bell_frame")
    for sx in (1, -1):
        g.box((0.80 * sx, 0, 0.99), (0.16, 0.16, 1.82), "wood", drop=GROUND)  # 기둥
        g.box((0.80 * sx, 0, 0.08), (0.36, 0.42, 0.16), "stone", drop=GROUND)  # 주춧돌
        g.box((0.60 * sx, 0, 1.78), (0.40, 0.10, 0.10), "wood", rot=(0, 45 * sx, 0))  # 버팀대
    g.box((0, 0, 2.00), (1.90, 0.22, 0.20), "wood")  # 대들보
    g = m.geo("bell_body", "bell")
    g.box((0, 0, 1.84), (0.20, 0.14, 0.12), "charcoal")  # 고리
    g.box((0, 0, 1.70), (0.34, 0.34, 0.18), "bronze", taper=(0.7, 0.7))  # 종 머리
    g.box((0, 0, 1.60), (0.44, 0.44, 0.05), "bronze")  # 띠
    g.box((0, 0, 1.38), (0.56, 0.56, 0.44), "bronze", taper=(0.7, 0.7))  # 몸
    g.box((0, 0, 1.11), (0.74, 0.74, 0.12), "bronze", taper=(0.82, 0.82), top=False)  # 입술
    g.box((0.05, -0.248, 1.40), (0.035, 0.03, 0.36), "charcoal", rot=(-11, 16, 0), top=False)  # 금
    g.box((-0.01, -0.262, 1.22), (0.03, 0.03, 0.16), "charcoal", rot=(-11, -22, 0), top=False)
    g = m.geo("bell_clapper", "clapper")
    g.box((0, 0, 1.06), (0.10, 0.10, 0.16), "charcoal")  # 추
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
    """작은 돌무더기, 약 1m."""
    m = C.Model("rubble")
    g = m.geo("rubble")
    g.box((0, 0, 0.18), (0.50, 0.40, 0.36), "wall", rot=(0, 0, 12), drop=GROUND)
    g.box((0.36, 0.16, 0.12), (0.40, 0.35, 0.24), "stone", rot=(0, 0, -20), drop=GROUND)
    g.box((-0.33, -0.10, 0.13), (0.36, 0.42, 0.26), "wall_light", rot=(0, 0, 35), drop=GROUND)
    g.box((0.05, -0.33, 0.08), (0.30, 0.26, 0.16), "stone", rot=(0, 0, 50), drop=GROUND)
    g.box((0.10, 0.06, 0.46), (0.32, 0.26, 0.24), "lstone", rot=(12, 8, 25))
    g.box((-0.20, 0.32, 0.08), (0.22, 0.20, 0.16), "stone", rot=(0, 0, 10), drop=GROUND)
    g.box((-0.12, -0.02, 0.375), (0.22, 0.20, 0.03), "moss", rot=(0, 0, 12))
    m.build()
    return m, []


PROPS = {
    "wall_segment": wall_segment,
    "pillar_broken": pillar_broken,
    "root_clump": root_clump,
    "brazier": brazier,
    "gate": gate,
    "core_crystal": core_crystal,
    "bell": bell,
    "rubble": rubble,
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
