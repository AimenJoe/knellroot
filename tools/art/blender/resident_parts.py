"""주민 모델 공용(0132): Meshy 몸 하나를 머리·몸·양팔·두 발로 자르고 뼈에 붙인다. 주민 셋(`res_*.py`)은 같은 꼴이라
자르는 치수(`Cut`)와 동작만 다르다. 산 것이라 잿빛을 어둡게 하지 않는다(적만 `darken_grey`).
"""
import os
from dataclasses import dataclass

from mathutils import Matrix

import common as C

PARTS = {"body_geo": "body", "head_geo": "head", "arm_l_geo": "arm_l", "arm_r_geo": "arm_r",
         "leg_l_geo": "leg_l", "leg_r_geo": "leg_r"}


@dataclass
class Cut:
    """자르는 치수(옮긴 자리, 앞 = -Y, 캐릭터 왼쪽 = +X) — 정면·옆·위 직교 렌더(텍스처째)로 잰다."""
    head_z: float  # 이보다 위는 머리
    arm_x: float  # 몸보다 밖이면 팔
    arm_z: tuple  # 팔이 있는 높이(어깨 아래 ~ 손)
    leg_z: float  # 이보다 아래는 발
    leg_split: float = 0.0  # 두 발 사이 틈
    shift_x: float = 0.0  # meshy_convert가 비껴 둔 만큼 옮김(머리 가운데로 원점을 잡아서)
    paw_x: float = 9.0  # 팔 높이보다 아래라도 이보다 밖이면 팔(늘어뜨린 큰 손끝 — 두더지)

    def classify(self, c):
        x, z = c.x + self.shift_x, c.z
        if z > self.head_z:
            return "head_geo"
        if (self.arm_z[0] < z < self.arm_z[1] and abs(x) > self.arm_x) or (z < self.arm_z[1] and abs(x) > self.paw_x):
            return "arm_l_geo" if x > 0 else "arm_r_geo"
        if z < self.leg_z:
            return "leg_l_geo" if x > self.leg_split else "leg_r_geo"
        return "body_geo"


def make(name, low, cut, bones, acts):
    """bones: {뼈: 머리 자리} — body(엉덩이)·head·arm_l·arm_r·leg_l·leg_r. 몸 뼈 아래는 body, 다리는 root."""
    m = C.Model(name)
    m.bone("root", (0, 0, 0))
    m.bone("body", bones["body"], "root")
    for b in ("head", "arm_l", "arm_r"):
        m.bone(b, bones[b], "body")
    for b in ("leg_l", "leg_r"):
        m.bone(b, bones[b], "root")
    objs = m.external(os.path.join(C.REPO, "tools", "art", "sources", "meshy", low), cut.classify, PARTS)
    if cut.shift_x:
        for o in objs.values():
            o.data.transform(Matrix.Translation((cut.shift_x, 0.0, 0.0)))
    m.build()
    acts(m)
    return m


def build(make_fn, name, previews, check, models_dir, preview_dir, check_dir=None):
    m = make_fn()
    path = m.export(models_dir)
    for act, t, suffix in previews:
        m.preview(os.path.join(preview_dir, f"{name}{suffix}.png"), act, t, yaw=-35)
    if check_dir:
        for act, t in check:
            m.preview(os.path.join(check_dir, f"{name}_{act}_{int(t * 100):03d}.png"), act, t, yaw=-70)
    return path
