"""다른 몸(같은 `mixamorig:*` 뼈 이름)의 동작을 이 뼈대로 옮기기 — 0247에서 `meshy_rig.py`에 있던 것을 0283에 떼어 냈다
(`meshy_rig.py` 몸 만들기와 `meshy_moves.py` 동작만 모으기가 같이 쓴다)."""
import bpy

def retarget(src_path, tgt, rename=None):
    """다른 몸(같은 `mixamorig:*` 뼈 이름)의 동작을 이 뼈대로 옮긴다(0247): 매 프레임 대상의 월드 회전 = Ps · Rs⁻¹ · Rt
    (Ps 원본 자세·Rs 원본 쉬는 자세·Rt 대상 쉬는 자세 — 원본이 쉬는 자세면 대상도 제 쉬는 자세).
    엉덩이의 움직임은 엉덩이 높이의 비로 줄인다. 원본에 없는 뼈(두건·망토·꼬리 `Bone_*`, 대상의 왼손가락)는 쉬는 자세 그대로.
    `rename`(0283): 원본 클립 이름 → 새 이름(Text to Motion 클립은 모두 `retarget_clip`이다). 옮겨 담은 액션들을 돌려준다."""
    from mathutils import Matrix, Quaternion, Vector

    before_o, before_a = set(bpy.data.objects), set(bpy.data.actions)
    bpy.ops.import_scene.gltf(filepath=src_path)
    new_o = [o for o in bpy.data.objects if o not in before_o]
    new_a = [a for a in bpy.data.actions if a not in before_a]
    src = next(o for o in new_o if o.type == "ARMATURE")
    for o in new_o:
        if o is not src:
            bpy.data.objects.remove(o)
    for a in new_a:
        a["clip"] = a.name[: -len(src.name) - 1] if a.name.endswith("_" + src.name) else a.name
    # 엉덩이 뼈의 z는 게임·`wanderer_skin.py`가 읽는 "몸 앞"이다(`own_yaw`) — v3 리깅의 엉덩이 뼈는 몸이 앞을 보는데도 13° 비뚤게
    # 돌아 있었다(척추·머리는 곧다). 쉬는 자세의 돌림(roll)만 몸 앞(−y)으로 맞춘다 — 몸의 꼴은 그대로(쉬는 자세 = 휨 없음).
    bpy.context.view_layer.objects.active = tgt
    bpy.ops.object.mode_set(mode="EDIT")
    eb = tgt.data.edit_bones["mixamorig:Hips"]
    eb.align_roll(tgt.matrix_world.inverted().to_3x3() @ Vector((0, -1, 0)))
    bpy.ops.object.mode_set(mode="OBJECT")
    for pb in src.pose.bones:
        pb.rotation_mode = "QUATERNION"
    for pb in tgt.pose.bones:
        pb.rotation_mode = "QUATERNION"
    bpy.context.view_layer.update()
    Sw, Tw = src.matrix_world.copy(), tgt.matrix_world.copy()
    Tw_q = Tw.to_quaternion()
    names = [b.name for b in tgt.data.bones if b.name in src.data.bones and b.name.startswith("mixamorig:")]

    def depth(b):
        d = 0
        while b.parent:
            b, d = b.parent, d + 1
        return d

    order = sorted(tgt.data.bones, key=depth)
    rest_arm = {b.name: b.matrix_local.copy() for b in tgt.data.bones}
    rs = {n: (Sw @ src.data.bones[n].matrix_local).to_quaternion() for n in names}
    rt = {n: (Tw @ rest_arm[n]).to_quaternion() for n in names}
    # 원본의 쉬는 자세에서 바뀐 몫(월드 회전)만 대상의 쉬는 자세에 얹는다(Pt = Ps · Rs⁻¹ · Rt). 처음에는 뼈 방향을 원본과 같게
    # 베꼈는데(쉬는 틀 맞춤), v3 리깅은 목뼈가 뒤로 29° 젖혀져 있어(몸통·목은 뼈의 자리가 몸마다 다르다) 목이 앞으로 꺾여 머리가
    # 가슴 앞으로 쏠렸다(사장 "기본 모션의 머리 각도가 이상해"). 두 몸의 쉬는 자세는 같은 A 자세라(팔 차 10° 안팎) 몫만 옮긴다.
    align = {n: Quaternion() for n in names}
    hip = "mixamorig:Hips"
    hs = (Sw @ src.data.bones[hip].head_local).z - min((Sw @ b.head_local).z for b in src.data.bones)
    ht = (Tw @ tgt.data.bones[hip].head_local).z - min((Tw @ b.head_local).z for b in tgt.data.bones)
    k = ht / hs
    print(f"[meshy_rig] retarget {len(names)} bones, hip ratio {k:.3f}")
    FEET = [f"mixamorig:{s}{p}" for s in ("Left", "Right") for p in ("Foot", "ToeBase", "Toe_End")]

    def feet_low(o, w):
        return min((w @ o.pose.bones[n].head).z for n in FEET if n in o.pose.bones)

    bpy.context.view_layer.update()
    s_feet0 = min((Sw @ src.data.bones[n].head_local).z for n in FEET if n in src.data.bones)
    t_feet0 = min((Tw @ tgt.data.bones[n].head_local).z for n in FEET if n in tgt.data.bones)
    sc = bpy.context.scene
    made = []
    for a in new_a:
        f0, f1 = int(a.frame_range[0]), int(a.frame_range[1])
        src.animation_data.action = a
        act = bpy.data.actions.new((rename or {}).get(a["clip"], a["clip"]))
        if not tgt.animation_data:
            tgt.animation_data_create()
        tgt.animation_data.action = act
        last = {}
        for f in range(f0, f1 + 1):
            sc.frame_set(f)
            posed = {}
            for b in order:
                n = b.name
                pb = tgt.pose.bones[n]
                par = b.parent
                rel = (rest_arm[par.name].to_quaternion().inverted() @ rest_arm[n].to_quaternion()) if par else rest_arm[n].to_quaternion()
                ppose = posed[par.name] if par else Quaternion()
                if n in names:
                    ps = (Sw @ src.pose.bones[n].matrix).to_quaternion()
                    want_world = ps @ rs[n].inverted() @ align[n].inverted() @ rt[n]
                    want_arm = Tw_q.inverted() @ want_world
                    basis = rel.inverted() @ ppose.inverted() @ want_arm
                    if n in last:
                        basis.make_compatible(last[n])
                    last[n] = basis
                    pb.rotation_quaternion = basis
                    pb.keyframe_insert("rotation_quaternion", frame=f, group=n)
                    posed[n] = want_arm
                else:
                    posed[n] = ppose @ rel
                if n == hip:
                    sm = Sw @ src.pose.bones[n].matrix
                    d = sm.translation - (Sw @ src.data.bones[n].matrix_local).translation
                    want = (Tw @ rest_arm[n]).translation + d * k
                    loc_arm = Tw.inverted() @ want
                    pb.location = rest_arm[n].to_quaternion().inverted() @ (loc_arm - rest_arm[n].translation)
                    pb.keyframe_insert("location", frame=f, group=n)
            # 발은 땅에(엉덩이 높이의 비만으로는 다리 길이·굽힘이 다른 몸의 발이 땅을 파고들었다 — 내려찍기 착지 −0.3m): 대상의 가장 낮은 발이
            # 원본의 가장 낮은 발이 쉬는 자세에서 오르내린 만큼(비 k)만 오르내리게 엉덩이를 위아래로 민다.
            bpy.context.view_layer.update()
            want_low = t_feet0 + (feet_low(src, Sw) - s_feet0) * k
            dz = want_low - feet_low(tgt, Tw)
            hp = tgt.pose.bones[hip]
            hp.location = hp.location + rest_arm[hip].to_quaternion().inverted() @ (Tw.inverted().to_3x3() @ Vector((0, 0, dz)))
            hp.keyframe_insert("location", frame=f, group=hip)
        act.use_fake_user = True
        made.append(act)
    tgt.animation_data.action = None
    bpy.data.objects.remove(src)
    for a in new_a:
        bpy.data.actions.remove(a)
    for pb in tgt.pose.bones:
        pb.rotation_quaternion = (1, 0, 0, 0)
        pb.location = (0, 0, 0)
    print("[meshy_rig] retargeted", sorted(a.name for a in made))
    return made
