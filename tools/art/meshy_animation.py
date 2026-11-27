#!/usr/bin/env python3
"""Meshy Animation · Text to Motion API로 리깅한 몸에 동작 받기 — 0283.

    python3 tools/art/meshy_animation.py library [--search 낱말] [--category Fighting]      # 동작 라이브러리(0 크레딧)
    python3 tools/art/meshy_animation.py motion "<영어 설명>" --duration 3 [--mode prime]     # Text to Motion(prime 10 · swift 3)
    python3 tools/art/meshy_animation.py apply <리깅 작업 번호> <나올.glb> --actions 99,137   # 라이브러리 동작 입히기(동작마다 3)
    python3 tools/art/meshy_animation.py apply <리깅 작업 번호> <나올.glb> --motion <motion 작업 번호>

받는 것: 리깅한 몸 + 동작 GLB(`--actions`는 동작마다 한 클립, 이름 = 라이브러리 이름 · `--motion`은 `retarget_clip`). 쓴 크레딧과
작업 번호를 찍는다(사장 2026-11-17: Meshy API는 불러도 된다, 크레딧만 알린다). 값은 https://docs.meshy.ai/en/api/pricing(2026-11-27 읽음):
리깅 5 · 동작 하나 3 · Text to Motion prime 10 / swift 3. 키는 코드에 두지 않는다 — 세션 프록시가 넣는다(밖에서는 `MESHY_API_KEY`).
리깅 작업 번호는 `meshy_rigging.py`가 찍는다(작업 결과는 며칠 뒤 지워진다 — `sources/meshy/README.md`에 적는다).
받은 GLB는 `blender/meshy_moves.py`가 방랑자 v3 뼈대로 옮겨 `sources/meshy/raccoon_v3_moves.glb`에 모은다.
"""
import json
import sys
import time
import urllib.parse

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from meshy_image3d import _get, _req  # noqa: E402

API = "https://api.meshy.ai/openapi/v1"


def _opts(argv, keys):
    out = {}
    for k in keys:
        if k in argv:
            i = argv.index(k)
            out[k] = argv[i + 1]
            argv = argv[:i] + argv[i + 2:]
    return argv, out


def _wait(url, every=6):
    while True:
        t = _req(url)
        if t["status"] in ("SUCCEEDED", "FAILED", "CANCELED"):
            return t
        time.sleep(every)


def library(argv):
    _, o = _opts(argv, ["--search", "--category"])
    q = {k[2:]: v for k, v in o.items()}
    rows = _req(API + "/animations/library" + ("?" + urllib.parse.urlencode(q) if q else ""))
    for a in rows:
        if a["rig_type"] == "biped":
            print(a["action_id"], a["key"], a["category"], a["sub_category"], a["preview_url"])


def motion(argv):
    argv, o = _opts(argv, ["--duration", "--mode"])
    if len(argv) != 1 or "--duration" not in o:
        sys.exit(__doc__)
    body = {"prompt": argv[0], "duration": float(o["--duration"]), "mode": o.get("--mode", "prime")}
    tid = _req(API + "/text-to-motion", body)["result"]
    print("task", tid, flush=True)
    t = _wait(API + "/text-to-motion/" + tid)
    if t["status"] != "SUCCEEDED":
        sys.exit("실패: " + json.dumps(t)[:800])
    print("motion", tid, body["mode"], body["duration"], "s credits", t.get("consumed_credits"))


def apply(argv):
    argv, o = _opts(argv, ["--actions", "--motion"])
    if len(argv) != 2 or len(o) != 1:
        sys.exit(__doc__)
    rig, out = argv
    body = {"rig_task_id": rig}
    if "--actions" in o:
        body["action_ids"] = [int(x) for x in o["--actions"].split(",")]
    else:
        body["motion_task_id"] = o["--motion"]
    tid = _req(API + "/animations", body)["result"]
    print("task", tid, flush=True)
    t = _wait(API + "/animations/" + tid)
    if t["status"] != "SUCCEEDED":
        sys.exit("실패: " + json.dumps(t)[:800])
    _get(t["result"]["animation_glb_url"], out)
    print(out, "credits", t.get("consumed_credits"), "task", tid)


def main(argv):
    cmds = {"library": library, "motion": motion, "apply": apply}
    if not argv or argv[0] not in cmds:
        sys.exit(__doc__)
    cmds[argv[0]](argv[1:])


if __name__ == "__main__":
    main(sys.argv[1:])
