"""Meshy GLB → 게임 예산 GLB(0038 실험): 줄이기 + 색 텍스처 하나(작게, 픽셀 그대로) + 키·방향 맞추기.

    blender -b -P tools/art/blender/meshy_convert.py -- <입력.glb> <출력.glb> [키 m=1.4] [정점 목표=2000] [텍스처 px=128]

- 우리 규칙(common.py): Z 위, 발 = z 0, 앞 = 블렌더 -Y. Meshy는 glTF +Z가 앞이라 가져오면 이미 -Y가 앞이다.
- 로더는 첫 이미지 하나만 읽는다(`src/model.rs`) → base color만 남기고 금속·노멀은 버린다(우리 렌더가 빛을 얹는다).
- 뼈·동작 없음(정지 모델). 부위 자르기·뼈 붙이기는 다음 단계.
- 정점 목표는 내보낸 glTF 정점(UV 이음매로 갈라진 것 포함) 기준으로 맞춘다.
- Meshy의 UV는 아주 잘게 쪼개져 있어 텍스처를 줄이면 조각 경계가 번진다(하얀 금) → 줄인 메시에 UV를 새로 펴고
  원본 색을 Cycles로 구워 넣는다(selected → active, 여백을 둬서 줄여도 안 번진다).
"""
import os
import sys

import bpy

argv = sys.argv[sys.argv.index("--") + 1:]
src, dst = argv[0], argv[1]
height = float(argv[2]) if len(argv) > 2 else 1.4
target = int(argv[3]) if len(argv) > 3 else 2000
tex_px = int(argv[4]) if len(argv) > 4 else 128
UV_ANGLE = 1.55  # 라디안 ≈ 89°: 조각을 크게 — 이음매가 적어야 같은 정점 예산에 삼각형이 많다

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=src)
meshes = [o for o in bpy.data.objects if o.type == "MESH"]
assert len(meshes) == 1, meshes
ob = meshes[0]
bpy.context.view_layer.objects.active = ob
ob.select_set(True)
bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

# 키·발 자리
zs = [v.co.z for v in ob.data.vertices]
xs = [v.co.x for v in ob.data.vertices]
ys = [v.co.y for v in ob.data.vertices]
k = height / (max(zs) - min(zs))
cx, cy = (max(xs) + min(xs)) / 2, (max(ys) + min(ys)) / 2
for v in ob.data.vertices:
    v.co.x = (v.co.x - cx) * k
    v.co.y = (v.co.y - cy) * k
    v.co.z = (v.co.z - min(zs)) * k
import bmesh  # noqa: E402

# glTF로 가져오면 UV 이음매마다 정점이 갈라져 있다 → 줄이면 경계가 벌어져 금이 간다. 먼저 합친다(원본 UV는 굽기에만 쓰니 상관없다).
_bm = bmesh.new()
_bm.from_mesh(ob.data)
_before = len(_bm.verts)
bmesh.ops.remove_doubles(_bm, verts=_bm.verts, dist=1e-4)
print(f"[meshy_convert] merged {_before} -> {len(_bm.verts)} verts")
_bm.to_mesh(ob.data)
_bm.free()
ob.data.update()  # 안 하면 Cycles가 옛 자리(키 1.9 m)의 원본을 봐서 굽기가 빗나간다(검정)


def gltf_verts(o):
    """UV가 다른 모서리는 glTF에서 정점이 갈린다 — (정점, uv) 쌍의 수."""
    uv = o.data.uv_layers.active.data
    seen = set()
    for loop in o.data.loops:
        u = uv[loop.index].uv
        seen.add((loop.vertex_index, round(u.x, 5), round(u.y, 5)))
    return len(seen)


def select_only(o):
    bpy.ops.object.select_all(action="DESELECT")
    o.select_set(True)
    bpy.context.view_layer.objects.active = o


def make_low(ratio):
    """원본을 복사해 줄이고 UV를 새로 편다."""
    lo = ob.copy()
    lo.data = ob.data.copy()
    lo.name = "low"
    bpy.context.collection.objects.link(lo)
    select_only(lo)
    mod = lo.modifiers.new("dec", "DECIMATE")
    mod.decimate_type = "COLLAPSE"
    mod.ratio = ratio
    mod.use_collapse_triangulate = True
    bpy.ops.object.modifier_apply(modifier=mod.name)
    while lo.data.uv_layers:
        lo.data.uv_layers.remove(lo.data.uv_layers[0])
    lo.data.uv_layers.new(name="UVMap")
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=UV_ANGLE, island_margin=0.02)
    bpy.ops.object.mode_set(mode="OBJECT")
    return lo


# 줄이기: 비율을 이분 탐색
lo_r, hi_r, best = 0.001, 0.2, None
for _ in range(10):
    r = (lo_r + hi_r) / 2
    cand = make_low(r)
    n = gltf_verts(cand)
    if n <= target:
        if best:
            bpy.data.objects.remove(best[1])
        best, lo_r = (r, cand), r
    else:
        bpy.data.objects.remove(cand)
        hi_r = r
low = best[1]
for p in low.data.polygons:
    p.use_smooth = True

# 굽기: 원본 base color → 새 UV 텍스처
bake_px = tex_px * 4
img = bpy.data.images.new("meshy_base", bake_px, bake_px, alpha=False)
mat = bpy.data.materials.new("meshy_base")
mat.use_nodes = True
nt = mat.node_tree
bsdf = nt.nodes["Principled BSDF"]
bsdf.inputs["Metallic"].default_value = 0.0
bsdf.inputs["Roughness"].default_value = 1.0
tex = nt.nodes.new("ShaderNodeTexImage")
tex.image = img
tex.interpolation = "Closest"
# 굽는 동안에는 잇지 않는다 — 구울 그림이 자기 재질의 입력이면 "Circular dependency"로 검게 나온다
low.data.materials.clear()
low.data.materials.append(mat)
nt.nodes.active = tex
# 원본 재질: base color를 발광으로 돌려 EMIT로 굽는다(빛·금속·패스 설정과 무관하게 색 그대로)
for m in ob.data.materials:
    mt = m.node_tree
    b = next(n for n in mt.nodes if n.type == "BSDF_PRINCIPLED")
    col = b.inputs["Base Color"].links[0].from_socket
    em = mt.nodes.new("ShaderNodeEmission")
    mt.links.new(col, em.inputs["Color"])
    outn = next(n for n in mt.nodes if n.type == "OUTPUT_MATERIAL")
    mt.links.new(em.outputs["Emission"], outn.inputs["Surface"])
sc = bpy.context.scene
sc.render.engine = "CYCLES"
sc.cycles.samples = 4
sc.cycles.device = "CPU"
sc.cycles.use_denoising = False  # apt Blender는 OpenImageDenoiser 없이 빌드돼, 켜져 있으면 굽기가 조용히 검정이 된다
bk = sc.render.bake
bk.use_pass_direct = False
bk.use_pass_indirect = False
bk.use_pass_color = True
bk.use_selected_to_active = True
bk.cage_extrusion = 0.03
bk.max_ray_distance = 0.1
bk.margin = bake_px // 32
bpy.ops.object.select_all(action="DESELECT")
ob.select_set(True)
low.select_set(True)
bpy.context.view_layer.objects.active = low
bpy.ops.object.bake(type="EMIT", use_selected_to_active=True)
nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
img.scale(tex_px, tex_px)
img.filepath_raw = os.path.splitext(dst)[0] + "_tex.png"
img.file_format = "PNG"
img.save()

bpy.data.objects.remove(ob)
for im in list(bpy.data.images):
    if im != img:
        bpy.data.images.remove(im)
select_only(low)
tris = sum(len(p.vertices) - 2 for p in low.data.polygons)
print(f"[meshy_convert] ratio {best[0]:.4f} glTF verts {gltf_verts(low)} tris {tris} tex {tex_px}")
bpy.ops.export_scene.gltf(filepath=dst, export_format="GLB", export_yup=True, export_texcoords=True,
                          export_normals=True, export_materials="EXPORT", export_image_format="AUTO",
                          export_cameras=False, export_lights=False, export_animations=False,
                          use_selection=True)
print("[meshy_convert] wrote", dst)
