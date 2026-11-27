"""Meshy API로 받은 동작 GLB들 → 방랑자 v3 뼈대에 옮겨 담은 동작만의 원본 한 장(0283).

    blender -b -P tools/art/blender/meshy_moves.py -- <출력.glb> <대상 리그.glb> <받은.glb>[=<클립>:<새 이름>,…] …

- 대상 리그 = `sources/meshy/raccoon_v3_rig.glb`(0247 — 게임 몸의 뼈대). 몸은 버리고 뼈대만 남긴다.
- 받은 GLB(`tools/art/meshy_animation.py apply`)는 Meshy가 다시 리깅한 몸 위의 동작이다(0283: v3 몸 원본은 리깅이 "관절을 못 찾는다"로
  실패해 v2 몸 원본을 리깅했다 — 뼈 이름은 같은 `mixamorig:*`). `retarget.py`가 바뀐 몫만 대상의 쉬는 자세에 얹는다(0247과 같은 길).
- 이름: 라이브러리 클립은 Meshy 이름 그대로(`mage_soell_cast_7`), Text to Motion 클립은 모두 `retarget_clip`이라 `=retarget_clip:T_withdraw`로
  붙인다. 이름을 적으면 적은 클립만 남는다(한 파일에 여러 동작을 받았어도 쓰는 것만).
- `wanderer_skin.py`가 `raccoon_v3_rig_anim.glb`(v2 동작 49개)와 함께 이 파일의 동작을 읽는다 — 원본 GLB(각 수 MB)는 레포에 넣지 않는다.
"""
import os
import sys

import bpy

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from retarget import retarget  # noqa: E402

argv = sys.argv[sys.argv.index("--") + 1:]
dst, target, srcs = argv[0], argv[1], argv[2:]

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.context.scene.render.fps = 24  # `meshy_rig.py`와 같은 프레임 단위(자르는 표 `CLIPS`의 수)
bpy.ops.import_scene.gltf(filepath=target)
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
for o in list(bpy.data.objects):
    if o is not arm:
        bpy.data.objects.remove(o)
for a in list(bpy.data.actions):
    bpy.data.actions.remove(a)
arm.name = "rig"
made = []
for spec in srcs:
    path, _, ren = spec.partition("=")
    rename = dict(p.split(":", 1) for p in ren.split(",") if p)
    got = retarget(path, arm, rename)
    made += [a for a in got if not rename or a.name in rename.values()]  # 이름을 적었으면 적은 클립만 남긴다
for a in list(bpy.data.actions):
    if a not in made:
        bpy.data.actions.remove(a)
for a in made:
    a.use_fake_user = True
print("[meshy_moves] actions", sorted(a.name for a in made))
bpy.ops.object.select_all(action="DESELECT")
arm.select_set(True)
os.makedirs(os.path.dirname(os.path.abspath(dst)), exist_ok=True)
bpy.ops.export_scene.gltf(filepath=dst, export_format="GLB", export_yup=True, export_cameras=False, export_lights=False,
                          export_skins=True, export_animations=True, export_animation_mode="ACTIONS",
                          export_force_sampling=False, export_optimize_animation_size=True, export_anim_slide_to_zero=False,
                          use_selection=True)
print("[meshy_moves] wrote", dst)
