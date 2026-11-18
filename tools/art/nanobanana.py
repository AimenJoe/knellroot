#!/usr/bin/env python3
"""나노바나나로 참조 그림 한 장 받기 — 0234.

    python3 tools/art/nanobanana.py PROMPT.txt OUT.png [첨부.png ...] [--model nano-banana-2] [--via meshy|gemini]

프롬프트는 `docs/art/gemini-prompts-*.md`의 블록을 공통 블록까지 붙여 넣은 글 파일. 첨부 그림은 참조로 싣는다(웹에서 그림을
붙이고 글을 쓰는 것과 같다, 다섯 장까지).
- `--via meshy`(기본): Meshy Image to Image API(`nano-banana`·`nano-banana-2`·`nano-banana-pro` …) — 첨부를 data URI로. 쓴 크레딧을 찍는다
  (사장 2026-11-17: Meshy API는 불러도 된다, 크레딧만 알린다).
- `--via gemini`: Gemini 이미지 API. 2026-11-18 이 세션의 키는 무료 등급이라 이미지 모델 한도가 0이었다(429).
API 키는 코드에 두지 않는다 — 세션 프록시가 넣는다(밖에서 쓰려면 `MESHY_API_KEY`·`GEMINI_API_KEY`).
받은 그림은 AI로 그린 것 — 숨기지 않는다(0064).
"""
import base64
import json
import os
import sys
import time
import urllib.request


def _req(url, body=None, key="proxy", header="Authorization", prefix="Bearer "):
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json", header: prefix + key})
    with urllib.request.urlopen(req, timeout=300) as r:
        return json.load(r)


def _b64(path):
    with open(path, "rb") as f:
        return base64.b64encode(f.read()).decode()


def via_meshy(model, prompt, attach, out):
    key = os.environ.get("MESHY_API_KEY", "proxy")
    base = "https://api.meshy.ai/openapi/v1/"
    if attach:
        body = {"ai_model": model, "prompt": prompt, "reference_image_urls": ["data:image/png;base64," + _b64(p) for p in attach]}
        kind = "image-to-image"
    else:
        body = {"ai_model": model, "prompt": prompt}
        kind = "text-to-image"
    tid = _req(base + kind, body, key)["result"]
    while True:
        t = _req(base + kind + "/" + tid, key=key)
        if t["status"] in ("SUCCEEDED", "FAILED", "CANCELED"):
            break
        time.sleep(5)
    if t["status"] != "SUCCEEDED":
        sys.exit("실패: " + json.dumps(t)[:800])
    with urllib.request.urlopen(t["image_urls"][0], timeout=300) as r, open(out, "wb") as f:
        f.write(r.read())
    print(out, model, "credits", t.get("consumed_credits"), "task", tid)


def via_gemini(model, prompt, attach, out):
    parts = [{"inline_data": {"mime_type": "image/png", "data": _b64(p)}} for p in attach] + [{"text": prompt}]
    body = {
        "contents": [{"role": "user", "parts": parts}],
        "generationConfig": {"responseModalities": ["IMAGE"], "imageConfig": {"aspectRatio": "1:1"}},
    }
    res = _req(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent".format(model),
        body,
        os.environ.get("GEMINI_API_KEY", "proxy"),
        header="x-goog-api-key",
        prefix="",
    )
    for cand in res.get("candidates", []):
        for part in cand.get("content", {}).get("parts", []):
            data = part.get("inlineData") or part.get("inline_data")
            if data:
                with open(out, "wb") as f:
                    f.write(base64.b64decode(data["data"]))
                print(out, model)
                return
    sys.exit("그림이 없다: " + json.dumps(res)[:800])


def main(argv):
    opts = {"--model": None, "--via": "meshy"}
    for k in list(opts):
        if k in argv:
            i = argv.index(k)
            opts[k] = argv[i + 1]
            argv = argv[:i] + argv[i + 2:]
    if len(argv) < 2:
        sys.exit(__doc__)
    prompt_path, out, attach = argv[0], argv[1], argv[2:]
    with open(prompt_path, encoding="utf-8") as f:
        prompt = f.read()
    if opts["--via"] == "gemini":
        via_gemini(opts["--model"] or "gemini-3-pro-image", prompt, attach, out)
    else:
        via_meshy(opts["--model"] or "nano-banana-2", prompt, attach, out)


if __name__ == "__main__":
    main(sys.argv[1:])
