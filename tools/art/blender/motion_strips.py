"""동작 점검 띠(0038): 모델 스크립트의 동작마다 고르게 N 프레임을 한 줄로 찍는다.

    blender -b -P tools/art/blender/motion_strips.py -- <모델 모듈 이름> <출력 폴더> [프레임 수=6] [yaw=-35] [elev=25]
→ <출력>/<동작>.png 들. 띠를 이어 붙이는 건 PIL로(블렌더 파이썬에 PIL이 없다).
"""
import importlib
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

argv = sys.argv[sys.argv.index("--") + 1:]
mod = importlib.import_module(argv[0])
out = argv[1]
n = int(argv[2]) if len(argv) > 2 else 6
yaw = float(argv[3]) if len(argv) > 3 else -35.0
elev = float(argv[4]) if len(argv) > 4 else 25.0
os.makedirs(out, exist_ok=True)
m = mod.make()
for name, dur, loop in m.actions:
    for k in range(n):
        t = dur * k / (n if loop else n - 1)
        m.preview(os.path.join(out, f"{name}_{k}.png"), name, t, size=200, yaw=yaw, elev=elev)
