#!/usr/bin/env python3
"""Meshy Image to 3D API로 참조 그림 한 장 → GLB — 0234.

    python3 tools/art/meshy_image3d.py <그림.png> <나올.glb> [--model meshy-6] [--polys 30000]

받는 것: GLB + 앞·오른쪽·뒤·왼쪽 미리보기(`<나올>_front.png` …, 평가용 — 레포에 넣지 않아도 된다). 쓴 크레딧과 작업 번호를 찍는다
(사장 2026-11-17: Meshy API는 불러도 된다, 크레딧만 알린다). 키는 코드에 두지 않는다 — 세션 프록시가 넣는다(밖에서는 `MESHY_API_KEY`).
설정의 까닭(KB `guides/ai-mesh-to-rigid-part-model.md`): 그늘이 텍스처에 구워지면 게임 빛이 한 번 더 얹힌다 → `remove_lighting`,
원본이 크면 변환이 오래 걸린다 → 3만 삼각형으로 리메시(어차피 1~2천 정점으로 줄인다), 금속·노멀 맵은 버리니 PBR 끔.
"""
import base64
import json
import os
import sys
import time
import urllib.request

BASE = "https://api.meshy.ai/openapi/v1/image-to-3d"


def _req(url, body=None):
    key = os.environ.get("MESHY_API_KEY", "proxy")
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json", "Authorization": "Bearer " + key})
    with urllib.request.urlopen(req, timeout=300) as r:
        return json.load(r)


def _get(url, path):
    with urllib.request.urlopen(url, timeout=600) as r, open(path, "wb") as f:
        f.write(r.read())


def main(argv):
    opts = {"--model": "meshy-6", "--polys": "30000"}
    for k in list(opts):
        if k in argv:
            i = argv.index(k)
            opts[k] = argv[i + 1]
            argv = argv[:i] + argv[i + 2:]
    if len(argv) != 2:
        sys.exit(__doc__)
    image, out = argv
    with open(image, "rb") as f:
        uri = "data:image/png;base64," + base64.b64encode(f.read()).decode()
    body = {
        "image_url": uri,
        "ai_model": opts["--model"],
        "should_texture": True,
        "enable_pbr": False,
        "should_remesh": True,
        "topology": "triangle",
        "target_polycount": int(opts["--polys"]),
        "remove_lighting": True,
        "image_enhancement": False,
        "target_formats": ["glb"],
        "multi_view_thumbnails": True,
    }
    tid = _req(BASE, body)["result"]
    print("task", tid, flush=True)
    while True:
        t = _req(BASE + "/" + tid)
        if t["status"] in ("SUCCEEDED", "FAILED", "CANCELED"):
            break
        time.sleep(10)
    if t["status"] != "SUCCEEDED":
        sys.exit("실패: " + json.dumps(t)[:800])
    _get(t["model_urls"]["glb"], out)
    stem = out[: -len(".glb")] if out.endswith(".glb") else out
    for side, url in (t.get("thumbnail_urls") or {}).items():
        _get(url, "{}_{}.png".format(stem, side))
    print(out, opts["--model"], "credits", t.get("consumed_credits"), "task", tid)


if __name__ == "__main__":
    main(sys.argv[1:])
