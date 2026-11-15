"""Meshy 리깅·동작 GLB 여러 장 → 줄인 스킨 몸 + 뼈 + 쓸 동작만 담은 원본 한 장(0193).

    blender -b -P tools/art/blender/meshy_rig.py -- <출력.glb> <A.glb> <B.glb> <C.glb> <D.glb>

- 사장이 Meshy Pro로 리깅·동작을 받은 방랑자 넷(셋 + 대기 하나)(같은 몸 "Hooded Raccoon Rogue", 뼈 이름·바인드 포즈가 같다 —
  역 바인드 행렬 md5가 같음을 확인). 몸은 첫 장에서, 동작은 표 `KEEP`에 든 것만 이름을 붙여 모은다.
- 몸: `meshy_convert.py`와 같은 길(정점 합치기 → 줄이기 → 새 UV → 원본 색을 Cycles로 굽기). 줄여도 뼈 가중치는
  남는다(Decimate는 정점 그룹을 보간한다). 굽는 동안은 뼈대를 쉬는 자세로.
- 크기·자리·제자리화·자르기는 하지 않는다 — 그건 `wanderer_skin.py`가 이 파일을 읽어 한다(이 단계는 느리고 한 번만).
- 원본 GLB 셋(각 14~16 MB)은 레포에 넣지 않는다 — 사장의 Meshy 계정에 있고, 이 파일이 쓸 것을 다 담는다.
"""
import os
import sys

import bmesh
import bpy

argv = sys.argv[sys.argv.index("--") + 1:]
dst, srcs = argv[0], argv[1:]
TARGET = 3000  # 내보낸 glTF 정점(UV 이음매로 갈린 것 포함)
TEX = 512
UV_ANGLE = 1.55

# 묶음마다 남길 동작(Meshy 이름 → 원본 안의 이름). 이름 없는 셋째 묶음은 만든 차례(UUIDv7) = `docs/art/meshy-motion-prompts.md`의 번호.
KEEP = {
    0: {"Combat_Stance": "Combat_Stance", "Archery_Shot": "Archery_Shot",  # 2026-11-15: 활쏘기 = 쇠뇌 겨누기·쏘기
        "Back_Jump": "Back_Jump"},  # 2026-11-15: 뒤로 뛰기 = 뒤로 피하기(0197)
    1: {n: n for n in ["Attack", "Basic_Jump", "Charged_Ground_Slam", "Dead", "Jump_Over_Obstacle_2", "Roll_Dodge_3",
                       "Running", "Skill_03", "Sword_Judgment", "Sword_Parry_Backward_2", "Thrust_Slash",
                       "Triple_Combo_Attack", "Walking", "Weapon_Combo", "Heavy_Hammer_Swing",
                       "Collect_Object"]},  # 2026-11-15: 줍기
    2: "numbered",
    3: {"Idle_02": "Idle_02"},
    # 2026-11-15 넷째 묶음(0198): 주먹(잽 둘·어퍼컷·앞차기)·막기·마법 시전·싸움 자세 뒷걸음
    4: {n: n for n in ["Left_Jab_from_Guard", "Right_Jab_from_Guard", "Right_Uppercut_from_Guard", "Spartan_Kick",
                       "Block10", "mage_soell_cast_8", "Walk_Fight_Back",
                       "Charged_Slash", "Shield_Push_Left"]},  # 0199: 검 모아 베기 · 패링 성공의 밀쳐내기  # 2026-11-14 사장이 다시 준 대기 동작(곧게 선 숨 고리) — Combat_Stance는 23° 숙였다
}

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
sc.render.fps = 24  # 가져오는 동작의 프레임 = 1/24초(자르는 표의 단위)
keep = []
arm = body = None
for bi, path in enumerate(srcs):
    before_obj = set(bpy.data.objects)
    before_act = set(bpy.data.actions)
    bpy.ops.import_scene.gltf(filepath=path)
    new_obj = [o for o in bpy.data.objects if o not in before_obj]
    new_act = [a for a in bpy.data.actions if a not in before_act]
    for a in new_act:
        base = a.name.replace("_SmartRigArmature", "")
        base = base.split(".")[0]
        a["src"] = base
    rule = KEEP[bi]
    if rule == "numbered":
        unnamed = sorted([a for a in new_act if a["src"].startswith("_")], key=lambda a: a["src"])
        for i, a in enumerate(unnamed):
            a.name = f"M{i + 1:02d}"
            keep.append(a)
    else:
        for a in new_act:
            if a["src"] in rule and rule[a["src"]] not in [k.name for k in keep]:
                a.name = rule[a["src"]]
                keep.append(a)
    if bi == 0:
        arm = next(o for o in new_obj if o.type == "ARMATURE")
        body = next(o for o in new_obj if o.type == "MESH" and o.vertex_groups)
        for o in new_obj:
            if o not in (arm, body):
                bpy.data.objects.remove(o)
    else:
        for o in new_obj:
            bpy.data.objects.remove(o)
for a in list(bpy.data.actions):
    if a not in keep:
        bpy.data.actions.remove(a)
for a in keep:
    a.use_fake_user = True
print("[meshy_rig] actions", sorted(a.name for a in keep))
arm.name = "rig"
arm.data.pose_position = "REST"
bpy.context.view_layer.update()

# --- 몸 줄이기(정점 합치기 → Decimate → 새 UV)
bm = bmesh.new()
bm.from_mesh(body.data)
n0 = len(bm.verts)
bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
print(f"[meshy_rig] merged {n0} -> {len(bm.verts)} verts")
bm.to_mesh(body.data)
bm.free()
body.data.update()


def gltf_verts(o):
    uv = o.data.uv_layers.active.data
    return len({(lp.vertex_index, round(uv[lp.index].uv.x, 5), round(uv[lp.index].uv.y, 5)) for lp in o.data.loops})


def select_only(o):
    bpy.ops.object.select_all(action="DESELECT")
    o.select_set(True)
    bpy.context.view_layer.objects.active = o


def make_low(ratio):
    lo = body.copy()
    lo.data = body.data.copy()
    lo.name = "body"
    bpy.context.collection.objects.link(lo)
    for md in list(lo.modifiers):
        lo.modifiers.remove(md)
    select_only(lo)
    md = lo.modifiers.new("dec", "DECIMATE")
    md.decimate_type = "COLLAPSE"
    md.ratio = ratio
    md.use_collapse_triangulate = True
    bpy.ops.object.modifier_apply(modifier=md.name)
    while lo.data.uv_layers:
        lo.data.uv_layers.remove(lo.data.uv_layers[0])
    lo.data.uv_layers.new(name="UVMap")
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=UV_ANGLE, island_margin=0.02)
    bpy.ops.object.mode_set(mode="OBJECT")
    am = lo.modifiers.new("rig", "ARMATURE")
    am.object = arm
    return lo


lo_r, hi_r, best = 0.002, 0.2, None
for _ in range(10):
    r = (lo_r + hi_r) / 2
    cand = make_low(r)
    if gltf_verts(cand) <= TARGET:
        if best:
            bpy.data.objects.remove(best[1])
        best, lo_r = (r, cand), r
    else:
        bpy.data.objects.remove(cand)
        hi_r = r
low = best[1]
for p in low.data.polygons:
    p.use_smooth = True

# --- 굽기: 원본 base color → 새 UV(쉬는 자세 — 두 몸이 같은 자리)
bake_px = TEX * 4
img = bpy.data.images.new("rig_base", bake_px, bake_px, alpha=False)
mat = bpy.data.materials.new("rig_base")
mat.use_nodes = True
nt = mat.node_tree
bsdf = nt.nodes["Principled BSDF"]
bsdf.inputs["Metallic"].default_value = 0.0
bsdf.inputs["Roughness"].default_value = 1.0
tex = nt.nodes.new("ShaderNodeTexImage")
tex.image = img
tex.interpolation = "Closest"
low.data.materials.clear()
low.data.materials.append(mat)
nt.nodes.active = tex
for m in body.data.materials:
    mt = m.node_tree
    b = next(n for n in mt.nodes if n.type == "BSDF_PRINCIPLED")
    col = b.inputs["Base Color"].links[0].from_socket
    em = mt.nodes.new("ShaderNodeEmission")
    mt.links.new(col, em.inputs["Color"])
    outn = next(n for n in mt.nodes if n.type == "OUTPUT_MATERIAL")
    mt.links.new(em.outputs["Emission"], outn.inputs["Surface"])
sc.render.engine = "CYCLES"
sc.cycles.samples = 4
sc.cycles.device = "CPU"
sc.cycles.use_denoising = False
bk = sc.render.bake
bk.use_pass_direct = False
bk.use_pass_indirect = False
bk.use_pass_color = True
bk.use_selected_to_active = True
bk.cage_extrusion = 0.03
bk.max_ray_distance = 0.1
bk.margin = bake_px // 32
bpy.ops.object.select_all(action="DESELECT")
body.select_set(True)
low.select_set(True)
bpy.context.view_layer.objects.active = low
bpy.ops.object.bake(type="EMIT", use_selected_to_active=True)
nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
img.scale(TEX, TEX)
img.file_format = "PNG"
img.pack()

bpy.data.objects.remove(body)
for im in list(bpy.data.images):
    if im != img:
        bpy.data.images.remove(im)
arm.data.pose_position = "POSE"
arm.animation_data.action = None
print(f"[meshy_rig] ratio {best[0]:.4f} glTF verts {gltf_verts(low)} tris {sum(len(p.vertices) - 2 for p in low.data.polygons)}")
select_only(low)
arm.select_set(True)
os.makedirs(os.path.dirname(os.path.abspath(dst)), exist_ok=True)
bpy.ops.export_scene.gltf(filepath=dst, export_format="GLB", export_yup=True, export_texcoords=True, export_normals=True,
                          export_materials="EXPORT", export_image_format="AUTO", export_cameras=False, export_lights=False,
                          export_skins=True, export_animations=True, export_animation_mode="ACTIONS",
                          export_force_sampling=False, export_optimize_animation_size=True, export_anim_slide_to_zero=False,
                          use_selection=True)
print("[meshy_rig] wrote", dst)
