"""방랑자 v3 — 스킨 몸 + Meshy 동작(0193).

    blender -b -P tools/art/blender/wanderer_skin.py -- [모델 폴더] [미리보기 폴더]

- 몸·뼈·원본 동작: `tools/art/sources/meshy/raccoon_v2_rig.glb`(`meshy_rig.py`가 사장의 Meshy 리깅·동작 GLB 셋에서 만든 것).
  뼈는 Mixamo 이름(`mixamorig:*`) + 꼬리·망토·귀 뼈. 게임은 뼈 가중치로 CPU에서 몸을 휜다(`model::skin_into`).
- 무기 넷·등잔: `wanderer.py`의 코드 꼴 그대로(0157 — 생김새에 코드 꼴을 새로 붙이지 않는다; 든 것은 소품). 옛 손뼈의
  자리·방향에서 새 오른손뼈로 옮긴다: 날 = 손의 엄지 쪽(뼈의 z), 넓은 면 = 손등(뼈의 x) — 쥔 주먹의 마디 쪽으로 날이 선다.
  등잔은 왼손 밑 `lamp` 뼈(마실 때만 크기 1).
- 동작: 표 `CLIPS` — 게임 이름 = 원본 클립의 한 구간. 길이는 옛 동작 그대로(게임의 상태 시간·`Clip::at` 구간이 옛 동작에 맞춰져
  있다), 원본 구간 안의 "닿는 순간"(오른손 속도 봉우리)을 옛 동작의 그 순간에 맞춘다(구간마다 늘이고 줄이기).
  엉덩이의 수평 이동은 뺀다(게임이 몸을 옮긴다) — 되풀이 동작은 평균을, 한 번 하는 동작은 처음 → 끝 직선을 빼서 흔들림만 남긴다.
"""
import math
import os
import sys

import bpy
from mathutils import Matrix, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import wanderer as W  # noqa: E402

RIG = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "raccoon_v2_rig.glb")
NAME = "wanderer"
HEIGHT = 1.4  # 옛 방랑자와 같은 키(귀 끝까지)
OUT_FPS = 100  # 옛 동작처럼 — 길이(0.43·0.25 …)가 프레임에 딱 떨어진다. 키는 줄이므로 크기는 늘지 않는다
SRC_FPS = 24  # 원본 프레임 단위(`meshy_rig.py`가 24로 가져온다 — 아래 표의 수)
HAND_R, HAND_L = "mixamorig:RightHand", "mixamorig:LeftHand"
TEX = 256
GRIP_IN = 0.55  # 손목 → 가운데 손가락 뿌리의 이 몫이 주먹 가운데
# 쥔 방향(사장 2026-11-14 "무기가 손에 붙은 위치와 방향"): Meshy 동작의 손은 칼을 쥔 주먹이 아니라 펼친 손이고, 손의 돌림이
# 동작마다 제각각이다 — 원본 검 동작 열 곳의 닿는 순간을 손뼈 좌표로 재니 날 방향이 평균에서 22~72° 흩어졌다(손에 못 박은 한 방향은
# 어느 동작에선가 하늘·땅을 본다). 그래서 무기는 손뼈 밑 `weapon` 뼈에 달고 그 돌림을 프레임마다 몸에서 계산해 굽는다(`weapon_pass`):
# 날 = 어깨 → 주먹(위아래는 `BLADE_RISE`배 — 팔이 짧아 늘 아래로 처진다), 날이 앞서는 쪽 = 주먹이 가는 쪽(느리면 몸 앞).
BLADE_RISE = 0.5
EDGE_SPEED = 1.0  # m/s(뼈대 단위) — 이보다 빠르면 날이 손이 가는 쪽을 본다
EDGE_SMOOTH = 6  # 앞뒤 프레임(100fps) — 손 속도를 고르게
# 숙임 바로잡기(도, 엉덩이 → 머리가 수직에서 앞으로): 원본 Combat_Stance는 30° — 걷기(5°)와 이어지게 등뼈 셋으로 나눠 세운다.
UPRIGHT = {"idle": (8.0, 5.0)}  # (앞, 옆) 도 — 원본은 앞 23°·옆 18°(몸을 비스듬히 낮춘 싸움 자세)
SPINE = ("mixamorig:Spine", "mixamorig:Spine1", "mixamorig:Spine2")

# (게임 이름, 원본, 앵커[(게임 초, 원본 프레임)] 또는 ("loop", 시작, 끝, 길이) 또는 ("hit", 시작, 봉우리 찾기 (a, b), 끝, 길이, 게임의 순간)).
# 옛 동작 길이·순간은 옛 wanderer.glb에서 잰 값(오른손 속도 봉우리).
CLIPS = [
    # 서기·걷기·달리기 — 되풀이(길이: 서기·몸짓은 원본 그대로, 걷기는 옛 길이 — 걸음 시계가 옛 걷기 0.8초로 맞춰져 있다)
    ("idle", "Combat_Stance", ("loop", 0, 40, None)),
    ("walk", "Walking", ("loop", 0, 24.8, 0.8)),
    ("run", "Running", ("loop", 0, 15.2, None)),
    ("walk_back", "M02", ("cycle", 10, 48, 1.2)),  # 한 주기(두 걸음) 38프레임 — 걷기와 같은 배(1.29)로 빠르게
    ("idle_look", "M04", ("span", 0, 44, None)),
    ("idle_shake", "M04", ("span", 44, 71, None)),
    # 검 세 타(Triple_Combo_Attack 봉우리 18·36·54)
    ("attack", "Triple_Combo_Attack", ("hit", 6, (12, 24), 27, 0.43, 0.125)),
    ("attack2", "Triple_Combo_Attack", ("hit", 27, (30, 42), 45, 0.43, 0.142)),
    ("attack3", "Triple_Combo_Attack", ("hit", 45, (48, 60), 70, 0.52, 0.10)),
    ("charge", "Attack", ("span", 0, 21, 0.3)),
    ("heavy", "Attack", ("hit", 21, (22, 30), 50, 0.58, 0.133)),
    ("parry", "Sword_Parry_Backward_2", ("hit", 0, (2, 9), 12.8, 0.3, 0.06)),
    ("dodge", "Roll_Dodge_3", ("hit", 2, (10, 20), 30, 0.25, 0.1)),
    ("hurt", "Dead", ("hit", 0, (2, 6), 12, 0.35, 0.058)),
    ("fall", "Dead", ("span", 4, 71, 1.3)),
    ("wake", "M01", ("span", 0, 71, 4.0)),
    ("turn_l", "M03", ("hit", 0, (2, 10), 24, 0.25, 0.1)),
    ("turn_r", "M03", ("hit", 40, (44, 64), 68, 0.25, 0.075)),
    ("jump", "Basic_Jump", ("apex", 0.3)),
    ("land", "Basic_Jump", ("touch", 0.15)),
    ("plunge", "Charged_Ground_Slam", ("anchors", [(0.0, 18), (0.12, 30), (0.29, 39), (0.3, 41)])),
    ("drink", "M05", ("span", 0, 60, 0.8)),
    ("cast_charge", "M06", ("span", 4, 28, 0.3)),
    ("cast", "M06", ("hit", 28, (29, 40), 56, 0.4, 0.075)),
    ("throw", "M07", ("hit", 12, (24, 40), 56, 0.5, 0.175)),
    # 지팡이(M08 봉우리 11·24·29·42 — 찌르기·가로 치기·올려 치기·돌아 내려치기)
    ("staff1", "M08", ("hit", 2, (6, 14), 17, 0.3, 0.042)),
    ("staff2", "M08", ("hit", 17, (19, 26), 27, 0.3, 0.075)),
    ("staff3", "M08", ("hit", 27, (28, 34), 36, 0.32, 0.117)),
    ("staff4", "M08", ("hit", 36, (38, 48), 64, 0.56, 0.042)),
    ("staff_charge", "M09", ("span", 0, 22, 0.3)),
    ("staff_release", "M09", ("hit", 22, (24, 34), 56, 0.52, 0.05)),
    # 망치(M10 모으기·M11 두 타)
    ("hammer_charge", "M10", ("span", 0, 30, 0.3)),
    ("hammer1", "M11", ("hit", 10, (18, 30), 36, 0.73, 0.225)),
    ("hammer2", "M11", ("hit", 36, (42, 54), 72, 0.86, 0.233)),
    ("hammer_slam", "Charged_Ground_Slam", ("hit", 22, (34, 44), 64, 0.75, 0.108)),
    # 쇠뇌(M12)
    ("xbow_charge", "M12", ("span", 12, 48, 0.3)),
    ("xbow_shoot", "M12", ("hit", 48, (50, 58), 64, 0.24, 0.058)),
    # 기술(구간 = `skills.rs`의 `Clip::seg` — 시동 끝 = 닿는 순간)
    ("sk_whirl", "Skill_03", ("anchors", [(0.0, 16), (0.15, 24), (0.37, 33), (0.67, 39)])),
    ("sk_cut", "Thrust_Slash", ("anchors", [(0.0, 8), (0.12, 18), (0.32, 33), (0.57, 50)])),
    ("sk_gather", "M13", ("hitseg", 24, (40, 56), 76, [0.20, 0.0, 0.30])),
    ("sk_quake", "Charged_Ground_Slam", ("hitseg", 8, (34, 44), 66, [0.30, 0.0, 0.40])),
    ("sk_brace", "M14", ("anchors", [(0.0, 0), (0.10, 6), (1.30, 58), (1.65, 80)])),
    ("sk_echo", "M15", ("hitseg", 30, (42, 52), 64, [0.25, 0.0, 0.20])),
    ("sk_throw", "M16", ("anchors", [(0.0, 14), (0.18, 26), (0.28, 32), (0.53, 50)])),
    ("sk_embers", "M17", ("hitseg", 14, (26, 36), 56, [0.25, 0.0, 0.30])),
    ("ult_upend", "M18", ("anchors", [(0.0, 34), (0.30, 57), (0.50, 64), (1.0, 82)])),
    ("ult_flurry", "Weapon_Combo", ("anchors", [(0.0, 40), (0.30, 54), (0.50, 70)])),
    ("ult_line", "Sword_Judgment", ("anchors", [(0.0, 10), (0.30, 24), (0.45, 32), (0.90, 62)])),
]
LOOPS = {"idle", "walk", "run", "walk_back"}


def world_pt(arm, bone, which="head"):
    b = arm.data.bones[bone]
    return arm.matrix_world @ (b.head_local if which == "head" else b.tail_local)


def basis(a, b):
    a = a.normalized()
    b = (b - a * a.dot(b)).normalized()
    return Matrix((a, b, a.cross(b))).transposed()  # 열 = a, b, a×b


def load_old():
    """옛 빌더에서 무기·등잔과 손 자리만(동작·텍스처 합치기는 건너뛴다)."""
    W.acts = lambda m: None
    merge = C.merge_textures
    C.merge_textures = lambda *a, **k: None
    m = W.make()
    C.merge_textures = merge
    rig = m.rig
    bpy.context.view_layer.update()
    old = {
        "hand_r": rig.matrix_world @ rig.data.bones["hand_r"].head_local,
        "hand_l": rig.matrix_world @ rig.data.bones["hand_l"].head_local,
        "atlas": m.mat.node_tree.nodes["Image Texture"].image,
        "weapons": [m.objects[n] for n in ("sword", "w_hammer", "w_staff", "w_crossbow")],
        "lamp": [m.objects[n] for n in ("lamp_geo", "glow_lamp")],
    }
    keep = set(old["weapons"] + old["lamp"])
    for o in list(bpy.data.objects):
        if o not in keep:
            if o.type == "MESH" and o.parent and o.parent.type == "ARMATURE" and o not in keep:
                pass
    for o in keep:
        mw = o.matrix_world.copy()
        o.parent = None
        o.matrix_world = mw
    for o in list(bpy.data.objects):
        if o not in keep:
            bpy.data.objects.remove(o)
    for a in list(bpy.data.actions):
        bpy.data.actions.remove(a)
    return old


def load_rig():
    sc = bpy.context.scene
    sc.render.fps = SRC_FPS
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=RIG)
    new = [o for o in bpy.data.objects if o not in before]
    arm = next(o for o in new if o.type == "ARMATURE")
    body = next(o for o in new if o.type == "MESH" and o.vertex_groups)
    for o in new:
        if o not in (arm, body):
            bpy.data.objects.remove(o)
    arm.animation_data.action = None
    for a in bpy.data.actions:  # 가져오면 "<이름>_<뼈대 이름>"
        if a.name.endswith("_" + arm.name):
            a.name = a.name[: -len(arm.name) - 1]
    for pb in arm.pose.bones:
        pb.rotation_mode = "QUATERNION"
    return arm, body


def place(arm, body):
    """키 1.4 m, 엉덩이(쉬는 자세)가 원점 위."""
    bpy.context.view_layer.update()
    zs = [(arm.matrix_world @ v.co).z for v in body.data.vertices]  # 몸은 뼈대의 자식 — 쉬는 자세의 정점
    k = HEIGHT / (max(zs) - min(zs))
    hip = arm.data.bones["mixamorig:Hips"].head_local
    arm.scale = (k, k, k)
    arm.location = (-hip.x * k, -hip.y * k, -min(zs) * k)
    bpy.context.view_layer.update()
    return k


def add_bone(arm, name, parent, at_world):
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.select_all(action="DESELECT")
    arm.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    eb = arm.data.edit_bones
    b = eb.new(name)
    p = arm.matrix_world.inverted() @ at_world
    b.head = p
    b.tail = p + Vector((0, 0, 0.1))
    b.parent = eb[parent]
    bpy.ops.object.mode_set(mode="OBJECT")
    arm.pose.bones[name].rotation_mode = "QUATERNION"


def attach(arm, old):
    """무기 넷은 옛 방향 그대로 주먹 자리의 `weapon` 뼈(오른손 자식)에 — 돌림은 `weapon_pass`가 동작마다 굽는다.
    등잔은 자리만 왼손 밑 `lamp` 뼈로."""
    # 쉬는 자세에서 붙인다 — 가져온 뼈대는 마지막 동작의 자세를 들고 있어서, 그대로 붙이면 그 자세 기준으로 어긋나 붙는다
    # (0193 첫 판의 "무기가 손에서 떨어진" 까닭).
    arm.animation_data.action = None
    for pb in arm.pose.bones:
        pb.matrix_basis.identity()
    bpy.context.view_layer.update()
    grip = world_pt(arm, HAND_R).lerp(world_pt(arm, "mixamorig:RightHandMiddle1"), GRIP_IN)
    add_bone(arm, "weapon", HAND_R, grip)
    xf = Matrix.Translation(grip - old["hand_r"])
    for o in old["weapons"]:
        mw = xf @ o.matrix_world
        o.parent = arm
        o.parent_type = "BONE"
        o.parent_bone = "weapon"
        o.matrix_world = mw
    grip_l = world_pt(arm, HAND_L).lerp(world_pt(arm, "mixamorig:LeftHandMiddle1"), GRIP_IN)
    add_bone(arm, "lamp", HAND_L, grip_l)
    lf = Matrix.Translation(grip_l - old["hand_l"])
    for o in old["lamp"]:
        mw = lf @ o.matrix_world
        o.parent = arm
        o.parent_type = "BONE"
        o.parent_bone = "lamp"
        o.matrix_world = mw


# ---------------------------------------------------------------- 동작
def hand_speed(arm, src, f0, f1):
    """원본 [f0, f1] 프레임마다 오른손(엉덩이 기준) 속도 — 봉우리 찾기."""
    sc = bpy.context.scene
    arm.animation_data.action = bpy.data.actions[src]
    out, prev = {}, None
    for f in range(int(math.floor(f0)), int(math.ceil(f1)) + 1):
        sc.frame_set(f)
        p = arm.pose.bones[HAND_R].head - arm.pose.bones["mixamorig:Hips"].head
        out[f] = 0.0 if prev is None else (p - prev).length
        prev = p
    return out


def peak(arm, src, a, b):
    sp = hand_speed(arm, src, a - 1, b)
    return max(range(int(a), int(b) + 1), key=lambda f: sp.get(f, 0.0))


def hips_series(arm, src, frames):
    sc = bpy.context.scene
    arm.animation_data.action = bpy.data.actions[src]
    out = []
    for f in frames:
        sc.frame_set(int(f))
        out.append(arm.pose.bones["mixamorig:Hips"].head.z)
    return out


def anchors_of(arm, name, src, spec):
    kind = spec[0]
    act = bpy.data.actions[src]
    end = act.frame_range[1]
    if kind in ("loop", "span", "cycle"):
        _, s0, s1, dur = spec
        dur = dur if dur else (s1 - s0) / SRC_FPS
        return [(0.0, s0), (dur, s1)]
    if kind == "hit":
        _, s0, (a, b), s1, dur, tp = spec
        sp = peak(arm, src, a, b)
        print(f"[skin] {name}: {src} {s0}-{sp}-{s1} -> {dur}s (hit {tp})")
        return [(0.0, s0), (tp, sp), (dur, s1)]
    if kind == "hitseg":
        _, s0, (a, b), s1, seg = spec
        sp = peak(arm, src, a, b)
        print(f"[skin] {name}: {src} {s0}-{sp}-{s1} seg {seg}")
        return [(0.0, s0), (seg[0], sp), (sum(seg), s1)]
    if kind == "anchors":
        return spec[1]
    if kind in ("apex", "touch"):
        frames = list(range(0, int(end) + 1))
        zs = hips_series(arm, src, frames)
        top = max(frames, key=lambda f: zs[f])
        if kind == "apex":
            return [(0.0, top - 3), (spec[1], top + 3)]
        low = min(range(top, len(frames)), key=lambda f: zs[f])
        print(f"[skin] {name}: apex {top} touch {low}")
        return [(0.0, max(top, low - 2)), (spec[1], low + 4)]
    raise ValueError(kind)


def src_at(anchors, t):
    for (t0, f0), (t1, f1) in zip(anchors, anchors[1:]):
        if t <= t1 or (t1, f1) == anchors[-1]:
            u = 0.0 if t1 <= t0 else min(max((t - t0) / (t1 - t0), 0.0), 1.0)
            return f0 + (f1 - f0) * u
    return anchors[-1][1]


def lean_of(v):
    """엉덩이 → 머리 벡터의 (앞, 옆) 기울기(라디안, 뼈대 공간 — 앞 = −Y, 옆 = +X)."""
    return math.atan2(-v.y, v.z), math.atan2(v.x, v.z)


def spine_rot(fwd, side):
    """등뼈 하나가 맡는 몫(셋으로 나눔): 옆축(X)으로 `fwd`, 앞축(Y)으로 `side`."""
    k = len(SPINE)
    return Matrix.Rotation(side / k, 3, "Y") @ Matrix.Rotation(fwd / k, 3, "X")


def lean_after(snap, fwd, side):
    """등뼈를 돌린 뒤 엉덩이 → 머리의 기울기: 머리 자리를 위 등뼈부터 제 머리를 축으로 차례로 돌린다(부모가 돌면 아래 축도 따라 돈다)."""
    hips, pivots, head = snap
    r = spine_rot(fwd, side)
    x = head
    for p in reversed(pivots):
        x = p + r @ (x - p)
    return lean_of(x - hips)


def solve_upright(snaps, want):
    """클립 평균 기울기가 `want`(앞, 옆 도)가 되는 등뼈 돌림(앞, 옆 라디안)."""
    wf, ws = (math.radians(x) for x in want)
    fwd = side = 0.0
    for _ in range(8):
        got = [lean_after(s, fwd, side) for s in snaps]
        mf = sum(g[0] for g in got) / len(got)
        ms = sum(g[1] for g in got) / len(got)
        fwd -= mf - wf
        side -= ms - ws
    return fwd, side, (math.degrees(mf), math.degrees(ms))


def uprighten(arm, fwd, side):
    """등뼈 셋을 돌린다 — 바뀌기 전 자세에서 읽은 틀로 바탕 회전을 고친다(자식은 따라온다)."""
    r = spine_rot(fwd, side)
    frames = {n: arm.pose.bones[n].matrix.to_3x3() @ arm.pose.bones[n].matrix_basis.to_3x3().inverted() for n in SPINE}
    for n in SPINE:
        pb = arm.pose.bones[n]
        loc, q, sc = pb.matrix_basis.decompose()
        a = frames[n]
        rot = a.inverted() @ r @ a @ q.to_matrix()
        pb.matrix_basis = Matrix.LocRotScale(loc, rot.to_quaternion(), sc)


def sample(arm, src, anchors, upright=None):
    """게임 프레임(OUT_FPS)마다 원본을 읽는다 → {뼈: [(loc, quat, scale)]}, 엉덩이 머리(뼈대 공간).
    `upright`(도)면 클립 평균 숙임을 그 각까지 등뼈로 세운다(한 클립에 한 각 — 숨쉬는 흔들림은 남는다)."""
    sc = bpy.context.scene
    arm.animation_data.action = bpy.data.actions[src]
    dur = anchors[-1][0]
    n = max(1, int(round(dur * OUT_FPS)))
    times = [src_at(anchors, k / OUT_FPS) for k in range(n + 1)]

    def go(f):
        for pb in arm.pose.bones:  # 채널이 없는 뼈는 쉬는 자세
            pb.location = (0, 0, 0)
            pb.rotation_quaternion = (1, 0, 0, 0)
            pb.scale = (1, 1, 1)
        sc.frame_set(int(math.floor(f)), subframe=f - math.floor(f))

    corr = None
    if upright is not None:
        snaps = []
        for f in times:
            go(f)
            pb = arm.pose.bones
            snaps.append((pb["mixamorig:Hips"].head.copy(), [pb[n].head.copy() for n in SPINE], pb["mixamorig:Head"].head.copy()))
        before = [lean_of(h - p) for p, _, h in snaps]
        fwd, side, got = solve_upright(snaps, upright)
        corr = (fwd, side)
        print(f"[skin] {src}: lean fwd/side {math.degrees(sum(b[0] for b in before) / len(before)):.0f}/"
              f"{math.degrees(sum(b[1] for b in before) / len(before)):.0f} -> {got[0]:.0f}/{got[1]:.0f}")
    poses, hips = [], []
    for f in times:
        go(f)
        if corr is not None:
            uprighten(arm, *corr)
        poses.append({pb.name: pb.matrix_basis.decompose() for pb in arm.pose.bones if pb.name not in ("lamp", "weapon")})
        hips.append(arm.pose.bones["mixamorig:Hips"].head.copy())
    return n, poses, hips


def strip_travel(arm, poses, hips, loop):
    """엉덩이의 수평 이동을 뺀다(뼈대 공간 x·y). 되풀이 = 평균, 한 번 = 처음 → 끝 직선. 쉬는 자세의 엉덩이 자리가 기준."""
    b = arm.data.bones["mixamorig:Hips"]
    rest = b.head_local
    r_inv = b.matrix_local.to_3x3().inverted()
    n = len(hips)
    if loop:
        mx = sum(h.x for h in hips) / n
        my = sum(h.y for h in hips) / n
        line = [(mx, my)] * n
    else:
        a, z = hips[0], hips[-1]
        line = [(a.x + (z.x - a.x) * k / max(n - 1, 1), a.y + (z.y - a.y) * k / max(n - 1, 1)) for k in range(n)]
    for k in range(n):
        h = hips[k]
        want = Vector((rest.x + h.x - line[k][0], rest.y + h.y - line[k][1], h.z))
        loc, q, s = poses[k]["mixamorig:Hips"]
        poses[k]["mixamorig:Hips"] = (r_inv @ (want - rest), q, s)


ROT_TOL = 0.003  # 쿼터니언 성분 오차(≈ 0.35°) — 이 안이면 키를 줄인다
LOC_TOL = 0.002  # m(뼈대 공간, 줄이기 전 단위)


def reduce(rows, tol):
    """키 줄이기(RDP): rows[k] = 값들. 처음·끝 + 직선 보간 오차가 tol을 넘는 자리만 남긴 프레임 번호."""
    n = len(rows)
    keep = {0, n - 1}
    stack = [(0, n - 1)]
    while stack:
        a, b = stack.pop()
        if b - a < 2:
            continue
        worst, at = 0.0, None
        for k in range(a + 1, b):
            u = (k - a) / (b - a)
            err = max(abs(rows[a][i] + (rows[b][i] - rows[a][i]) * u - rows[k][i]) for i in range(len(rows[k])))
            if err > worst:
                worst, at = err, k
        if worst > tol:
            keep.add(at)
            stack += [(a, at), (at, b)]
    return sorted(keep)


def put(act, bn, path, rows, frames):
    for i in range(len(rows[0])):
        fc = act.fcurves.new(f'pose.bones["{bn}"].{path}', index=i, action_group=bn)
        fc.keyframe_points.add(len(frames))
        fc.keyframe_points.foreach_set("co", [v for k in frames for v in (k, rows[k][i])])
        for kp in fc.keyframe_points:
            kp.interpolation = "LINEAR"
        fc.update()


def moving_bones(arm, body):
    """채널을 둘 뼈: 몸 정점을 움직이는 뼈와 그 조상 + 손(무기·등잔이 붙는다). 가중치 없는 끝 뼈(손끝·발끝·머리 끝)는 뺀다 —
    채널 하나마다 glTF JSON이 수백 바이트라 파일이 커진다."""
    weighted = set()
    for v in body.data.vertices:
        for g in v.groups:
            if g.weight > 1e-3:
                weighted.add(body.vertex_groups[g.group].name)
    keep = set()
    for b in arm.data.bones:
        if b.name in weighted or b.name in (HAND_R, HAND_L):
            p = b
            while p:
                keep.add(p.name)
                p = p.parent
    return keep


def write_action(arm, name, n, poses, lamp, keep):
    """뼈마다 회전(엉덩이는 위치도) — 줄인 키, 직선 보간. 위치·크기가 쉬는 자세 그대로인 뼈는 채널을 두지 않는다."""
    act = bpy.data.actions.new(name)
    act.use_fake_user = True
    for bn in poses[0].keys():
        if bn not in keep:
            continue
        qs, prev = [], None
        for p in poses:
            q = p[bn][1].copy()
            if prev is not None and prev.dot(q) < 0:
                q.negate()
            qs.append(tuple(q))
            prev = q
        if any(max(abs(a - b) for a, b in zip(q, (1, 0, 0, 0))) > 1e-4 for q in qs):
            put(act, bn, "rotation_quaternion", qs, reduce(qs, ROT_TOL))
        if bn == "mixamorig:Hips":
            ls = [tuple(p[bn][0]) for p in poses]
            put(act, bn, "location", ls, reduce(ls, LOC_TOL))
    put(act, "lamp", "scale", [(lamp,) * 3, (lamp,) * 3], [0, 1] if n >= 1 else [0])
    act.fcurves.find('pose.bones["lamp"].scale', index=0).keyframe_points[-1].co.x = n
    for i in (1, 2):
        act.fcurves.find('pose.bones["lamp"].scale', index=i).keyframe_points[-1].co.x = n
    act.use_frame_range = True
    act.frame_start, act.frame_end = 0, n
    arm.animation_data.action = None
    tr = arm.animation_data.nla_tracks.new()
    tr.name = name
    tr.strips.new(name, 0, act)
    tr.mute = True
    return act


def weapon_pass(arm, act, loop):
    """굽힌 동작을 다시 돌며 `weapon` 뼈의 돌림을 프레임마다 정해 키로(머리 주석의 규칙)."""
    sc = bpy.context.scene
    ad = arm.animation_data
    ad.action = act
    n = int(act.frame_range[1])
    hand, sh, hips = (arm.pose.bones[k] for k in (HAND_R, "mixamorig:RightArm", "mixamorig:Hips"))
    hr = arm.data.bones[HAND_R].matrix_local
    br = arm.data.bones["weapon"].matrix_local
    rb = br.to_3x3().normalized()
    old_frame = basis(Vector((0, -1, 0)), Vector((1, 0, 0)))
    snaps = []
    for k in range(n + 1):
        sc.frame_set(k)
        q = (hand.matrix @ hr.inverted() @ br).to_3x3().normalized()
        snaps.append((hand.head.copy(), sh.head.copy(), hips.head.copy(), q))
    rel = [h - p for h, _, p, _ in snaps]

    def at(i):
        return rel[i % (n + 1)] if loop else rel[min(max(i, 0), n)]

    qs, prev_e, prev_q = [], None, None
    for k, (h, s_, _, q) in enumerate(snaps):
        d = h - s_
        d.z *= BLADE_RISE
        d.normalize()
        v = Vector()
        for j in range(-EDGE_SMOOTH, EDGE_SMOOTH + 1):
            v += (at(k + j + 1) - at(k + j - 1)) * (OUT_FPS / 2)
        v /= 2 * EDGE_SMOOTH + 1
        e0 = Vector((0, -1, 0)) - d * (-d.y)
        if e0.length < 1e-3:
            e0 = Vector((0, 0, 1)) - d * d.z
        e0.normalize()
        ev = v - d * v.dot(d)
        w = min(ev.length / EDGE_SPEED, 1.0)
        e = e0 * (1 - w) + (ev.normalized() if ev.length > 1e-6 else e0) * w
        e = (e - d * e.dot(d)).normalized()
        if prev_e is not None and e.dot(prev_e) < 0:  # 손이 되돌아도 날을 뒤집지 않는다(망치 머리가 홱 돌지 않게)
            e.negate()
        prev_e = e
        rot = (q.inverted() @ basis(d, e) @ old_frame.transposed() @ rb).to_quaternion()
        if prev_q is not None and prev_q.dot(rot) < 0:
            rot.negate()
        prev_q = rot
        qs.append(tuple(rot))
    put(act, "weapon", "rotation_quaternion", qs, reduce(qs, ROT_TOL))
    ad.action = None


def make_actions(arm, body):
    if arm.animation_data is None:
        arm.animation_data_create()
    keep = moving_bones(arm, body)
    print("[skin] bones with channels", len(keep), "of", len(arm.data.bones))
    made = []
    for name, src, spec in CLIPS:
        anchors = anchors_of(arm, name, src, spec)
        n, poses, hips = sample(arm, src, anchors, UPRIGHT.get(name))
        strip_travel(arm, poses, hips, name in LOOPS)
        if spec[0] == "cycle":  # 되풀이 이음매: 끝 = 처음
            poses[-1] = poses[0]
        act = write_action(arm, name, n, poses, 1.0 if name == "drink" else 0.0, keep)
        weapon_pass(arm, act, name in LOOPS)
        made.append(act)
    for a in list(bpy.data.actions):
        if a not in made:
            bpy.data.actions.remove(a)
    return made


def texture(body, old):
    img = next(n.image for n in body.data.materials[0].node_tree.nodes if n.type == "TEX_IMAGE")
    img.scale(TEX, TEX)  # 옛 방랑자와 같은 256 — 게임 거리에서 몸은 100px 남짓
    W.lift_purples(img)
    C.merge_textures([body], img, old["weapons"] + old["lamp"], old["atlas"], NAME)


def export(arm, objs, out_dir):
    sc = bpy.context.scene
    sc.render.fps = OUT_FPS
    arm.name = "root"  # 2차 동작(`secondary.rs`)이 몸 전체를 찌그러뜨리는 마디
    bpy.ops.object.select_all(action="DESELECT")
    for o in [arm] + objs:
        o.select_set(True)
    path = os.path.join(out_dir, NAME + ".glb")
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_yup=True, export_texcoords=True,
                              export_normals=True, export_materials="EXPORT", export_image_format="AUTO",
                              export_cameras=False, export_lights=False, export_extras=False, export_skins=True,
                              export_all_influences=False, export_animations=True, export_animation_mode="ACTIONS",
                              export_force_sampling=False, export_optimize_animation_size=True,
                              export_anim_slide_to_zero=False, export_bake_animation=False, use_selection=True)
    print("[skin] wrote", path)
    return path


def preview(arm, objs, path, act=None, t=0.0, yaw=-35.0, elev=20.0, frame=None):
    """`common.Model.preview`와 같은 Workbench 직교 한 장."""
    m = C.Model.__new__(C.Model)
    m.scene = bpy.context.scene
    m.rig = arm
    m.objects = {o.name: o for o in objs}
    sc = bpy.context.scene
    sc.render.fps = OUT_FPS
    ad = arm.animation_data
    for tr in ad.nla_tracks:
        tr.mute = True
    ad.action = bpy.data.actions[act] if act else None
    for pb in arm.pose.bones:
        if not act:
            pb.location = (0, 0, 0)
            pb.rotation_quaternion = (1, 0, 0, 0)
            pb.scale = (1, 1, 1)
    sc.frame_set(int(round(t * OUT_FPS)))
    _preview_keep_pose(m, path, yaw, elev, frame)


def _preview_keep_pose(m, path, yaw, elev, frame=None):
    keep = C.Model.pose_at
    C.Model.pose_at = lambda self, action=None, t=0.0: None
    try:
        C.Model.preview(m, path, None, 0.0, yaw=yaw, elev=elev, frame=frame, size=200)
    finally:
        C.Model.pose_at = keep


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, sheet_dir=None):
    old = load_old()
    arm, body = load_rig()
    place(arm, body)
    bpy.context.view_layer.objects.active = body  # 가져온 몸의 사용자 법선(glTF NORMAL)을 지워야 매끈/각진 표시가 법선을 정한다
    bpy.ops.mesh.customdata_custom_splitnormals_clear()
    for e in body.data.edges:  # 가져올 때 법선이 갈린 모서리에 붙은 "각진 모서리" 표시도 — 정점이 갈려 늘어난다
        e.use_edge_sharp = False
    if hasattr(body.data, "use_auto_smooth"):  # Blender 4.0: 켜져 있으면 각도로 법선을 갈라 정점이 는다
        body.data.use_auto_smooth = False
    W._flat_where_folded(body.data)  # 줄이며 접힌 작은 면(면 법선 ↔ 매끈한 법선이 반대)은 각지게 — `check_glb` 뒤집힌 면
    attach(arm, old)
    texture(body, old)
    make_actions(arm, body)
    objs = [body] + old["weapons"] + old["lamp"]
    for o in old["weapons"][1:]:  # 미리보기에서는 칼만
        o.hide_render = True
    path = export(arm, objs, models_dir)
    preview(arm, objs, os.path.join(preview_dir, f"{NAME}.png"), "idle", 0.0)
    preview(arm, objs, os.path.join(preview_dir, f"{NAME}_front.png"), "idle", 0.0, yaw=-8.0, elev=14.0)
    if sheet_dir:
        for name, _, _ in CLIPS:
            act = bpy.data.actions[name]
            n = int(act.frame_range[1])
            for i in range(6):
                preview(arm, objs, os.path.join(sheet_dir, f"{name}_{i}.png"), name, n * i / 5 / OUT_FPS,
                        yaw=-60.0, elev=25.0, frame=(Vector((0, 0, 0.55)), 2.4))
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_SHEET_DIR"))
