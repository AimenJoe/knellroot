"""Knellroot 초안 3D 모델 전부 → assets/models/<이름>.glb + docs/art/previews/<이름>*.png, 끝에 check_glb 검사.

    blender --background --python tools/art/blender/build_all.py
    blender --background --python tools/art/blender/build_all.py -- <모델 폴더> <미리보기 폴더>   # 다른 곳에(검사용)

같은 입력이면 같은 결과(텍스처 잡음도 고정 시드). 고칠 때는 모델 스크립트를 고치고 이것을 다시 돌린다 — .glb·PNG는 손으로 고치지 않는다.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bellkeeper  # noqa: E402
import charms  # noqa: E402
import check_glb  # noqa: E402
import common as C  # noqa: E402
import gatekeeper  # noqa: E402
import pod  # noqa: E402
import props  # noqa: E402
import root_husk  # noqa: E402
import shellback  # noqa: E402
import thorn_beast  # noqa: E402
import wanderer  # noqa: E402


def main():
    models, previews = C.out_dirs()
    paths = []
    for mod in (wanderer, root_husk, thorn_beast, gatekeeper, shellback, pod, bellkeeper):
        paths.append(mod.build(models, previews))
    paths += props.build(models, previews)
    paths.append(charms.build(models, previews))
    print("\n[knellroot] check_glb")
    bad = check_glb.main(paths)
    if bad:
        raise SystemExit("check_glb: 문제 있음(위 '!!' 줄)")
    print("[knellroot] build_all 끝")


main()
