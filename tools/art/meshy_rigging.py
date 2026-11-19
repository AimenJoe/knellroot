#!/usr/bin/env python3
"""Meshy Rigging API로 Image to 3D 결과(작업 번호)에 뼈 붙이기 — 0246.

    python3 tools/art/meshy_rigging.py <image-to-3d 작업 번호> <나올.glb> [--template mixamo] [--height 1.0] [--fingers]

받는 것: 뼈 붙은 GLB + 걷기·달리기 기본 동작 GLB(`<나올>_walking.glb`·`_running.glb`). 쓴 크레딧과 작업 번호를 찍는다
(사장 2026-11-17: Meshy API는 불러도 된다, 크레딧만 알린다). 키는 코드에 두지 않는다 — 세션 프록시가 넣는다(밖에서는 `MESHY_API_KEY`).
뼈 이름은 `--template`(기본 mixamo — 지금 방랑자의 동작 49개가 `mixamorig:*` 이름이라 이름으로 옮겨 쓸 수 있게).
"""
import json
import sys
import time

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from meshy_image3d import _get, _req  # noqa: E402

BASE = "https://api.meshy.ai/openapi/v1/rigging"


def main(argv):
    opts = {"--template": "mixamo", "--height": "1.0"}
    fingers = "--fingers" in argv
    argv = [a for a in argv if a != "--fingers"]
    for k in list(opts):
        if k in argv:
            i = argv.index(k)
            opts[k] = argv[i + 1]
            argv = argv[:i] + argv[i + 2:]
    if len(argv) != 2:
        sys.exit(__doc__)
    task, out = argv
    body = {
        "input_task_id": task,
        "height_meters": float(opts["--height"]),
        "skeleton_template": opts["--template"],
        "include_fingers": fingers,
    }
    tid = _req(BASE, body)["result"]
    print("task", tid, flush=True)
    while True:
        t = _req(BASE + "/" + tid)
        if t["status"] in ("SUCCEEDED", "FAILED", "CANCELED"):
            break
        time.sleep(8)
    if t["status"] != "SUCCEEDED":
        sys.exit("실패: " + json.dumps(t)[:800])
    r = t["result"]
    _get(r["rigged_character_glb_url"], out)
    stem = out[: -len(".glb")] if out.endswith(".glb") else out
    for name in ("walking", "running"):
        url = (r.get("basic_animations") or {}).get(name + "_glb_url")
        if url:
            _get(url, "{}_{}.glb".format(stem, name))
    print(out, opts["--template"], "credits", t.get("consumed_credits"), "task", tid)


if __name__ == "__main__":
    main(sys.argv[1:])
