#!/usr/bin/env python3
"""서막 숲 지도 생성기(0262) — MAP·HEIGHTS 두 격자를 같은 칸으로 만든다.

작게(성소의 1/3쯤): 남쪽 깨어난 빈터(P) → 쓰러진 둥치를 넘는 턱(뛰기 0.5m) → 좁은 숲길 → 잔재 둘(a — 해 보는 상대) →
북쪽 숲 끝(E — 성소로 가는 길, 옛 길의 기둥 둘이 표지). 숲의 가장자리는 벽이 아니라 바위 단 + 나무(`flora`): 걷는 땅에서
멀어질수록 높아지는 단(+1 → +2 → +3m)이라 나무가 겹겹이 선다. 벽(`#`)은 없다.

    python3 tools/maps/forest.py            # 미리보기만(지도 | 높이)
    python3 tools/maps/forest.py --write    # src/stage/forest.rs 의 MAP·HEIGHTS를 바꿔 쓴다

높이 글자: '.' = 0, 36진수 v -> (v - 4) * 0.5m. 쓴 뒤에는 cargo test stage:: (도달·높이)를 본다. 다시 돌려도 같은 지도(멱등).
"""
import sys

SRC = 'src/stage/forest.rs'
W, H = 25, 40
M = [[' '] * W for _ in range(H)]
HT = [['.'] * W for _ in range(H)]


def floor(x, y, h='.'):
    M[y][x] = '.'
    HT[y][x] = h


def disc(cx, cy, rx, ry, h='.'):
    for y in range(H):
        for x in range(W):
            if ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 <= 1.0:
                floor(x, y, h)


def path(points, half, h_of):
    """꺾은선을 따라 폭 2*half+1 칸의 길. 높이는 행으로(`h_of(y)`)."""
    for (x0, y0), (x1, y1) in zip(points, points[1:]):
        n = max(abs(x1 - x0), abs(y1 - y0)) * 4
        for i in range(n + 1):
            t = i / n
            x, y = round(x0 + (x1 - x0) * t), round(y0 + (y1 - y0) * t)
            for dy in range(-half, half + 1):
                for dx in range(-half, half + 1):
                    if 0 < x + dx < W - 1 and 0 < y + dy < H - 1:
                        floor(x + dx, y + dy, h_of(y + dy))


# 쓰러진 둥치(0262 — 뛰기 0.5m): 이 행보다 북쪽(작은 y)의 땅은 +0.5m.
TRUNK = 25
rise = lambda y: '5' if y < TRUNK else '.'

# 1. 깨어난 빈터(남쪽) — 둥근 터.
disc(12, 32, 6.2, 4.6)
# 2. 빈터에서 북쪽으로: 둥치를 넘는 길(곧게 3칸 폭) → 서쪽으로 휘는 숲길 → 잔재의 작은 빈터 → 동쪽으로 → 숲 끝.
path([(12, 28), (12, 22)], 2, rise)
path([(12, 22), (8, 18)], 1, rise)
disc(8, 16, 3.4, 2.6, '5')            # 잔재의 작은 빈터(첫 상대)
path([(8, 14), (11, 11), (15, 9)], 1, rise)
path([(15, 9), (14, 6), (12, 4)], 1, rise)
disc(12, 3, 3.2, 2.0, '5')            # 숲 끝(성소로 가는 길)

marks = {
    'P': [(12, 34)],                  # 깨어나는 자리(곁에 검이 놓인다 — 0220)
    'E': [(12, 2)],                   # 숲 끝 — 밟으면 "성소에 들다"
    'a': [(7, 16), (14, 9)],          # 잔재 둘(끈 짧게 — tuning::PROLOGUE_LEASH)
    'i': [(10, 3), (14, 3)],          # 옛 길의 기둥(숲 끝의 표지)
    'r': [(10, TRUNK), (14, TRUNK)],  # 둥치의 뿌리 끝(길 양옆)
    'u': [(11, TRUNK), (13, TRUNK), (9, 30), (16, 34)],  # 둥치·빈터의 돌무더기(지나간다)
}
for c, spots in marks.items():
    for x, y in spots:
        if M[y][x] != '.':
            raise SystemExit(f"{c} at ({x},{y}) lands on {M[y][x]!r}")
        M[y][x] = c

# 3. 가장자리: 걷는 땅에서의 칸 거리로 바위 단(+1 · +2 · +3m) — 단 위에 나무가 선다.
INF = 99
dist = [[0 if M[y][x] != ' ' else INF for x in range(W)] for y in range(H)]
changed = True
while changed:
    changed = False
    for y in range(H):
        for x in range(W):
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < W and 0 <= ny < H and dist[ny][nx] + 1 < dist[y][x]:
                    dist[y][x] = dist[ny][nx] + 1
                    changed = True
for y in range(H):
    for x in range(W):
        if M[y][x] == ' ':
            d = dist[y][x]
            # 둥치 북쪽(+0.5m)은 단도 그만큼 높게.
            base = 1 if y < TRUNK else 0
            HT[y][x] = '6789abc'[min(base + 2 * (d - 1), 6)] if d < INF else 'c'

# 빈 줄은 `Level::parse`가 건너뛴다(높이 격자와 어긋난다) — 바위만 있는 줄은 양 끝 모서리 칸을 벽 칸으로(바위 단 높이 그대로).
for y in range(H):
    if all(c == ' ' for c in M[y]):
        M[y][0] = M[y][W - 1] = '#'
out_m = '\n'.join(''.join(r) for r in M)
out_h = '\n'.join(''.join(r) for r in HT)
for y in range(H):
    print(f"{y:2} {''.join(M[y])}  {''.join(HT[y])}")
if '--write' in sys.argv:
    s = open(SRC).read()
    pre, rest = s.split('const MAP: &str = "\n', 1); _, rest = rest.split('";', 1)
    s = pre + 'const MAP: &str = "\n' + out_m + '\n";' + rest
    pre, rest = s.split('const HEIGHTS: &str = "\n', 1); _, rest = rest.split('";', 1)
    s = pre + 'const HEIGHTS: &str = "\n' + out_h + '\n";' + rest
    open(SRC, 'w').write(s)
    print('wrote', SRC)
