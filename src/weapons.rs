//! 무기 표(0051). 무기 = 공격의 **모양**(KB `concepts/action-rpg-stats-and-equipment.md`): 콤보의 타마다 시간·거리·각·피해·자세 피해,
//! 강타, RB(근접 무기 = 마법 탄, 원거리 무기 = 사격), 원거리 무기의 X(수류탄). 0048: 무기 한 줄 = (X, RB) 짝.
//!
//! 시간·거리·각·내딛기는 랩의 기준값(`attack.*`·`heavy.*`, `tuning.rs`)에 곱하는 **배율** — 손맛의 바탕은 랩 한 곳에서 맞추고,
//! 무기는 그 위의 모양만 갖는다(코드에 타별 숫자를 박지 않는다). 피해·자세 피해는 정수·점수 그대로(피해의 눈금은 `tuning::HP_UNIT` — 옛 1 = 10, 0158 N0).
//! 새 무기 = `Id` 한 줄 + `WEAPONS` 한 줄 + 동작(`tools/art/blender/wanderer.py`) + 모델 부위(같은 파일, 오른손 뼈) + 글(i18n).

/// 판정 모양.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// 앞쪽 부채꼴(각 = `arc`).
    Arc,
    /// 몸 둘레 원(회전 베기·내려찍기).
    Circle,
}

/// 휘두르는 쪽(0087): 그 동작의 칼이 어느 쪽에서 어느 쪽으로 지나가나 — 박자 부채꼴(`World::beat_cue`)이 칼이 지나간 쪽부터 사라진다.
/// 오른쪽·왼쪽은 캐릭터 기준(모델 앞 = +Z, 오른쪽 = -X — `tools/art/blender/common.py`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sweep {
    /// 오른쪽에서 왼쪽으로 가로 베기 — 오른쪽부터 사라진다.
    RightToLeft,
    /// 왼쪽에서 오른쪽으로 — 왼쪽부터.
    LeftToRight,
    /// 찌르기·내려치기·올려 치기(세로) — 양 가장자리부터, 가운데가 마지막.
    Center,
}

/// 동작 이름 → 휘두르는 쪽(`wanderer.py`의 키프레임에서 읽음: 몸·오른팔의 z 회전이 음수 = 오른쪽, 양수 = 왼쪽).
/// 새 동작을 무기 표에 쓰면 여기에도 한 줄(시험 `every_weapon_motion_has_a_sweep`).
pub const SWEEPS: [(&str, Sweep); 12] = [
    ("attack", Sweep::RightToLeft),
    ("attack2", Sweep::LeftToRight),
    ("attack3", Sweep::Center),
    ("heavy", Sweep::RightToLeft),
    ("hammer1", Sweep::RightToLeft),
    ("hammer2", Sweep::LeftToRight),
    ("hammer_slam", Sweep::Center),
    ("staff1", Sweep::Center),
    ("staff2", Sweep::RightToLeft),
    ("staff3", Sweep::Center),
    ("staff4", Sweep::LeftToRight),
    ("staff_release", Sweep::LeftToRight),
];

/// 동작의 휘두르는 쪽(표에 없으면 가운데).
pub fn sweep_of(anim: &str) -> Sweep {
    SWEEPS
        .iter()
        .find(|(a, _)| *a == anim)
        .map_or(Sweep::Center, |(_, s)| *s)
}

/// 콤보 한 타.
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    /// 선딜·활성·후딜 모두에 곱한다(1 = 랩 평타 그대로, 클수록 느리다).
    pub time: f32,
    /// 거리·각·내딛기 배율.
    pub reach: f32,
    pub arc: f32,
    pub lunge: f32,
    pub shape: Shape,
    pub damage: i32,
    /// 자세 피해(0051 — 적의 자세 막대를 채운다).
    pub poise: f32,
    /// 넉백 배율.
    pub knock: f32,
    /// 동작 이름(`wanderer.glb`) — 없으면 `attack`.
    pub anim: &'static str,
    /// 캔슬 시점(0057): 활성 뒤 후딜레이의 이만큼(비율)이 지나야 다음 행동으로 끊는다 — 0 = 활성 끝나자마자. 무기 성격.
    pub cancel: f32,
}

/// 강타(공격 꾹, 0030).
#[derive(Clone, Copy, Debug)]
pub struct Heavy {
    /// 선딜·활성·후딜 배율, 다 모으는 시간 배율.
    pub time: f32,
    pub charge: f32,
    pub reach: f32,
    pub arc: f32,
    pub lunge: f32,
    pub shape: Shape,
    /// 덜 모았을 때 피해(다 모으면 + `heavy.full bonus`).
    pub damage: i32,
    pub poise: f32,
    /// 모으는 동안 걷기 배율(0047의 `heavy.charge walk x`에 곱한다).
    pub walk: f32,
    /// 친 자리에 금이 가고 이 초 뒤 둘레를 다시 친다(망치 여진, PoE2 지진 착상): (늦음 s, 반지름 m, 피해, 자세 피해).
    pub aftershock: Option<(f32, f32, i32, f32)>,
    /// 자세 막대가 이만큼(0..1) 넘게 찬 적은 바로 깬다(망치 — PoE2의 "깨지기 직전" 적 즉시 깨기).
    pub breaks_primed: Option<f32>,
    pub anim_charge: &'static str,
    pub anim_heavy: &'static str,
    /// 캔슬 시점(0057): 강타의 활성 뒤 후딜레이 비율.
    pub cancel: f32,
}

/// RB.
#[derive(Clone, Copy, Debug)]
pub enum Rb {
    /// 근접 무기의 마법 탄(0048: 칸 3, 근접으로 맞히면 찬다).
    Magic(Bolt),
    /// 원거리 무기의 사격: 탄창, 다 쓰면 `reload` 초 뒤 가득, 회피하면 한 발 장전(PoE2 착상). 칸은 쓰지 않는다.
    Shot {
        bolt: Bolt,
        mag: u8,
        reload: f32,
        /// `cast.startup`·`cast.time` 배율(쏘기는 마법보다 빠르다).
        time: f32,
    },
}

/// 날아가는 탄의 모양.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bolt {
    /// `cast.speed`·`cast.range` 배율.
    pub speed: f32,
    pub range: f32,
    pub damage: i32,
    pub poise: f32,
    /// 적을 꿰뚫고 계속 간다.
    pub pierce: bool,
    /// 맞은 적이 이 초 휘청(0 = 없음).
    pub stagger: f32,
    /// 그림 색(0 = 마법 라벤더, 1 = 무거운 탄 놋쇠, 2 = 꿰뚫는 탄 민트, 3 = 쇠뇌 볼트).
    pub look: u8,
    /// 크기 배율(그림·맞는 반지름, 모은 탄은 크다 — 0056).
    pub scale: f32,
}

/// 원거리 무기의 X(0048: 근접 무기의 마법 자리를 수류탄이 맞바꾼다).
#[derive(Clone, Copy, Debug)]
pub struct Grenade {
    /// 도화선(초) — 땅을 튕기다 이 시간에 터진다. 쇠뇌 볼트에 맞으면 바로.
    pub fuse: f32,
    /// 한 번에 가질 수 있는 수, 하나가 다시 차는 초.
    pub count: u8,
    pub cooldown: f32,
    pub radius: f32,
    pub damage: i32,
    pub poise: f32,
    /// 떨구는 자리: 발 앞 이 거리(m) — 조준 쪽(스틱·커서), 없으면 바라보는 쪽. 멀리 던지지 않는다(0178 — 사장 "수류탄은 바로 앞에
    /// 떨어뜨리는 걸로", 예전 0051 자동 조준 7m · 0055 꾹 = 11m).
    pub drop: f32,
}

/// 무기 이름표.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Id {
    Sword,
    Hammer,
    Staff,
    Crossbow,
    /// 맨손(0076, 사장 2026-11-07 "무기 장착하지 않았을 경우에는 맨손 전투") — 무기 칸을 비운 것. 짧고 빠른 주먹 3연타.
    Fists,
}

/// 표 순서. 맨손은 끝에(옛 기록의 무기 번호가 그대로 읽히게 — `app::legacy_weapon`).
pub const IDS: [Id; 5] = [Id::Sword, Id::Hammer, Id::Staff, Id::Crossbow, Id::Fists];

impl Id {
    /// 기록 키(0066 — 든 무기는 슬롯 기록에, 키는 바꾸지 않는다).
    pub fn key(self) -> &'static str {
        match self {
            Id::Sword => "sword",
            Id::Hammer => "hammer",
            Id::Staff => "staff",
            Id::Crossbow => "crossbow",
            Id::Fists => "fists",
        }
    }

    pub fn by_key(k: &str) -> Option<Id> {
        IDS.into_iter().find(|i| i.key() == k)
    }
}

pub struct Weapon {
    pub id: Id,
    /// X 연타(근접 무기). 원거리 무기는 비어 있고 X가 수류탄.
    pub combo: &'static [Hit],
    /// 대시 공격(2026-11-06): 회피 끝무렵·직후에 공격 — 회피의 기세로 멀리 내딛는 한 타. 맞힌 뒤 콤보는 둘째 타부터.
    pub dash: Option<Hit>,
    pub heavy: Option<Heavy>,
    pub rb: Rb,
    pub grenade: Option<Grenade>,
    /// RB를 쥐어 다 모은 탄(0056): 근접 무기 = 모은 마법, 원거리 무기 = 모은 사격. 덜 모았으면 피해·자세 피해 반, 꿰뚫기·휘청 없음.
    pub charged: Bolt,
    /// 방랑자 모델에서 이 무기 부위 이름(다른 무기 부위는 숨긴다).
    pub node: &'static str,
    /// 공격 기력 배율(0117, 사장 "큰 무기는 더 많은 기력 — 높은 피해의 보정"): 평타·대시 공격 `stamina.attack`, 강타 `heavy.stamina`,
    /// 원거리 무기는 사격·모은 사격·공중 볼트에 곱한다. 맨손 < 지팡이 < 쇠뇌(0173: 0.7 → 0.9) < 검 < 망치.
    pub stamina: f32,
}

const fn hit(
    time: f32,
    reach: f32,
    arc: f32,
    lunge: f32,
    damage: i32,
    poise: f32,
    anim: &'static str,
) -> Hit {
    Hit {
        time,
        reach,
        arc,
        lunge,
        shape: Shape::Arc,
        damage,
        poise,
        knock: 1.0,
        anim,
        cancel: 0.0,
    }
}

/// 대시 공격의 타 번호(`PState::Attack::hit`) — 콤보 표 밖.
pub const DASH: u8 = u8::MAX;

impl Weapon {
    /// 타 번호 → 타(콤보 표 또는 대시 공격).
    pub fn hit(&self, i: u8) -> Option<&Hit> {
        if i == DASH {
            self.dash.as_ref()
        } else {
            self.combo.get(i as usize)
        }
    }

    /// 이 타 다음에 이을 타(마지막이면 None). 대시 공격은 첫 타 자리라 둘째 타로 잇는다.
    pub fn after(&self, i: u8) -> Option<u8> {
        let n = if i == DASH { 1 } else { i as usize + 1 };
        (n < self.combo.len()).then_some(n as u8)
    }
}

const MAGIC: Bolt = Bolt {
    speed: 1.0,
    range: 1.0,
    damage: 10,
    poise: 1.0,
    pierce: false,
    stagger: 0.0,
    look: 0,
    scale: 1.0,
};

/// 순례자의 검 — 기준. 3연타: 가로 → 되돌려 가로 → 내디디며 찌르기(좁고 길게, 자세 피해 큼).
const SWORD_COMBO: [Hit; 3] = [
    hit(1.0, 1.0, 1.0, 1.0, 10, 1.0, "attack"),
    hit(1.0, 1.0, 1.0, 1.0, 10, 1.0, "attack2"),
    Hit {
        knock: 1.6,
        cancel: 0.15,
        ..hit(1.2, 1.25, 0.45, 1.6, 20, 2.0, "attack3")
    },
];

/// 종치기 망치 — 느리고 넓은 2타(한 걸음씩 나아감, 자세 피해 큼), 강타는 뛰어올라 둥근 내려찍기 + 여진.
const HAMMER_COMBO: [Hit; 2] = [
    Hit {
        knock: 1.5,
        cancel: 0.4,
        ..hit(1.7, 1.15, 1.25, 1.4, 20, 3.0, "hammer1")
    },
    Hit {
        shape: Shape::Circle,
        knock: 2.0,
        cancel: 0.5,
        ..hit(2.0, 1.1, 1.0, 1.2, 30, 4.0, "hammer2")
    },
];

/// 순례자 지팡이 — 빠른 4연타, 넷째는 둘레 회전 마무리(느림).
const STAFF_COMBO: [Hit; 4] = [
    hit(0.7, 1.15, 0.8, 0.7, 10, 0.6, "staff1"),
    hit(0.7, 1.15, 0.8, 0.7, 10, 0.6, "staff2"),
    hit(0.75, 1.2, 0.9, 0.8, 10, 0.8, "staff3"),
    Hit {
        shape: Shape::Circle,
        knock: 1.8,
        ..hit(1.3, 1.25, 1.0, 0.4, 20, 2.0, "staff4")
    },
];

/// 맨손 — 짧고 빠른 주먹 3연타(가까이 붙어야 닿는다, 피해 10·자세 피해 작음), 셋째는 내디디며 한 방. 동작은 검의 것을 빈손으로
/// (주먹 동작은 아직 없다).
const FISTS_COMBO: [Hit; 3] = [
    hit(0.6, 0.7, 0.8, 0.6, 10, 0.5, "attack"),
    hit(0.6, 0.7, 0.8, 0.6, 10, 0.5, "attack2"),
    Hit {
        knock: 1.3,
        cancel: 0.1,
        ..hit(0.8, 0.8, 0.6, 1.2, 10, 1.2, "attack3")
    },
];

pub const WEAPONS: [Weapon; 5] = [
    Weapon {
        id: Id::Sword,
        combo: &SWORD_COMBO,
        // 달려들며 찌르기: 좁고 길게, 회피 거리만큼 더 내딛는다.
        dash: Some(Hit {
            knock: 1.4,
            cancel: 0.2,
            ..hit(1.0, 1.25, 0.5, 2.4, 20, 2.0, "attack3")
        }),
        heavy: Some(Heavy {
            time: 1.0,
            charge: 1.0,
            reach: 1.0,
            arc: 1.0,
            lunge: 1.0,
            shape: Shape::Arc,
            damage: 20,
            poise: 3.0,
            walk: 1.0,
            aftershock: None,
            breaks_primed: None,
            anim_charge: "charge",
            anim_heavy: "heavy",
            cancel: 0.2,
        }),
        rb: Rb::Magic(MAGIC),
        grenade: None,
        // 모은 마법: 크고 꿰뚫는 빛 덩이.
        charged: Bolt {
            speed: 1.2,
            range: 1.3,
            damage: 30,
            poise: 4.0,
            pierce: true,
            stagger: 0.6,
            scale: 1.8,
            ..MAGIC
        },
        node: "sword",
        stamina: 1.0,
    },
    Weapon {
        id: Id::Hammer,
        combo: &HAMMER_COMBO,
        // 달려들며 휘두르기: 한 바퀴 둘레(무거워 늦다).
        dash: Some(Hit {
            shape: Shape::Circle,
            knock: 1.8,
            cancel: 0.45,
            ..hit(1.6, 1.0, 1.0, 2.0, 20, 3.0, "hammer2")
        }),
        heavy: Some(Heavy {
            time: 1.3,
            charge: 1.4,
            reach: 1.0,
            arc: 1.0,
            lunge: 2.2,
            shape: Shape::Circle,
            damage: 30,
            poise: 6.0,
            walk: 0.75,
            aftershock: Some((1.5, 2.6, 20, 3.0)),
            breaks_primed: Some(0.6),
            anim_charge: "hammer_charge",
            anim_heavy: "hammer_slam",
            cancel: 0.55,
        }),
        rb: Rb::Magic(Bolt {
            speed: 0.65,
            range: 0.6,
            damage: 20,
            poise: 3.0,
            stagger: 0.5,
            look: 1,
            ..MAGIC
        }),
        grenade: None,
        // 모은 무거운 탄: 느리고 크다, 맞으면 오래 휘청·자세 피해 큼.
        charged: Bolt {
            speed: 0.6,
            range: 0.8,
            damage: 40,
            poise: 8.0,
            stagger: 1.0,
            look: 1,
            scale: 2.2,
            ..MAGIC
        },
        node: "w_hammer",
        stamina: 1.7,
    },
    Weapon {
        id: Id::Staff,
        combo: &STAFF_COMBO,
        // 달려들며 찌르기: 빠르고 가볍다.
        dash: Some(hit(0.8, 1.3, 0.6, 2.2, 10, 1.0, "staff3")),
        heavy: Some(Heavy {
            time: 0.9,
            charge: 0.9,
            reach: 1.0,
            arc: 1.0,
            lunge: 0.5,
            shape: Shape::Circle,
            damage: 20,
            poise: 2.5,
            walk: 1.2,
            aftershock: None,
            breaks_primed: None,
            anim_charge: "staff_charge",
            anim_heavy: "staff_release",
            cancel: 0.0,
        }),
        rb: Rb::Magic(Bolt {
            speed: 1.3,
            range: 1.2,
            pierce: true,
            look: 2,
            ..MAGIC
        }),
        grenade: None,
        // 모은 꿰뚫는 탄: 빠르고 멀리, 줄 선 적을 다 꿰뚫는다.
        charged: Bolt {
            speed: 1.6,
            range: 1.7,
            damage: 30,
            poise: 3.0,
            pierce: true,
            look: 2,
            scale: 1.5,
            ..MAGIC
        },
        node: "w_staff",
        stamina: 0.75,
    },
    Weapon {
        id: Id::Crossbow,
        combo: &[],
        dash: None,
        heavy: None,
        // 0173(사장 "쇠뇌는 전투도 편한데 데미지도 강해서 오버파워"): 잰 값 — 8m 밖에서 처음 4초 60~65/초(근접 무기 30~37/초의 두 배),
        // 오래 쏴도 35/초(검 25). 안전한 거리의 값은 피해로 치른다: 볼트 20 → 10(마법 탄과 같다), 한 발 0.24 → 0.36초,
        // 장전 1.2 → 1.5초, 기력 0.7 → 0.9. 고친 뒤 처음 4초 약 23/초 · 오래 15/초(근접의 여섯~일곱 할).
        rb: Rb::Shot {
            bolt: Bolt {
                speed: 1.7,
                range: 1.4,
                damage: 10,
                poise: 1.0,
                pierce: false,
                stagger: 0.0,
                look: 3,
                scale: 1.0,
            },
            mag: 4,
            reload: 1.5,
            time: 0.9,
        },
        grenade: Some(Grenade {
            fuse: 1.5,
            count: 2,
            cooldown: 4.0,
            radius: 2.2,
            damage: 30,
            poise: 5.0,
            drop: 1.4,
        }),
        // 모은 사격: 시위를 끝까지 당긴 무거운 볼트 — 꿰뚫고 밀어낸다(수류탄도 맞히면 터진다).
        charged: Bolt {
            speed: 2.2,
            range: 1.8,
            damage: 40,
            poise: 6.0,
            pierce: true,
            stagger: 0.4,
            look: 3,
            scale: 1.6,
        },
        node: "w_crossbow",
        stamina: 0.9,
    },
    Weapon {
        id: Id::Fists,
        combo: &FISTS_COMBO,
        // 달려들며 한 방.
        dash: Some(hit(0.7, 0.8, 0.6, 2.0, 10, 1.0, "attack3")),
        // 모아 치기: 검보다 짧고 빠르다, 피해 10(다 모으면 + `heavy.full bonus`).
        heavy: Some(Heavy {
            time: 0.8,
            charge: 0.8,
            reach: 0.7,
            arc: 0.8,
            lunge: 1.2,
            shape: Shape::Arc,
            damage: 10,
            poise: 2.0,
            walk: 1.2,
            aftershock: None,
            breaks_primed: None,
            anim_charge: "charge",
            anim_heavy: "heavy",
            cancel: 0.1,
        }),
        // 마법은 무기가 아니라 방랑자의 것 — 맨손이어도 쏜다.
        rb: Rb::Magic(MAGIC),
        grenade: None,
        charged: Bolt {
            speed: 1.2,
            range: 1.3,
            damage: 30,
            poise: 4.0,
            pierce: true,
            stagger: 0.6,
            scale: 1.8,
            ..MAGIC
        },
        // 든 것이 없다(다른 무기 부위는 모두 숨는다).
        node: "",
        stamina: 0.6,
    },
];

impl Id {
    pub fn def(self) -> &'static Weapon {
        WEAPONS
            .iter()
            .find(|w| w.id == self)
            .expect("WEAPONS has every Id")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_id_has_one_row_and_melee_weapons_have_a_combo() {
        for id in IDS {
            assert_eq!(WEAPONS.iter().filter(|w| w.id == id).count(), 1, "{id:?}");
            let w = id.def();
            match w.rb {
                Rb::Magic(_) => assert!(
                    !w.combo.is_empty() && w.grenade.is_none(),
                    "{id:?}: melee = X combo + RB magic"
                ),
                Rb::Shot { mag, .. } => assert!(
                    mag > 0 && w.grenade.is_some(),
                    "{id:?}: ranged = RB shot + X grenade"
                ),
            }
        }
        assert_eq!(Id::by_key("fists"), Some(Id::Fists));
    }

    /// 0087: 무기 표의 모든 평타·대시·강타 동작은 휘두르는 쪽 표에 있다(빠지면 박자 부채꼴이 말없이 가운데로 사라진다).
    #[test]
    fn every_weapon_motion_has_a_sweep() {
        for w in WEAPONS.iter() {
            let anims = w
                .combo
                .iter()
                .chain(w.dash.iter())
                .map(|h| h.anim)
                .chain(w.heavy.iter().map(|h| h.anim_heavy));
            for a in anims {
                assert!(SWEEPS.iter().any(|(n, _)| *n == a), "{:?}: {a}", w.id);
            }
        }
        // 검 1타는 오른쪽 → 왼쪽, 2타는 되돌려 왼쪽 → 오른쪽, 3타 찌르기는 가운데(wanderer.py 주석).
        assert_eq!(sweep_of("attack"), Sweep::RightToLeft);
        assert_eq!(sweep_of("attack2"), Sweep::LeftToRight);
        assert_eq!(sweep_of("attack3"), Sweep::Center);
    }

    // ── 눈금 옮기기(0158 N0): 체력·피해가 ×10이 되어도 몇 대에 쓰러지는가는 그대로 ──────────────────────────────

    /// 옮기기 전의 수(작은 정수 — 0158 N0 직전의 표를 그대로 적어 둔다, 고치지 않는다).
    /// 무기마다: 콤보 타들 · 대시 공격 · 강타(덜 모음) · 여진 · RB(마법/사격) · 모은 탄 · 수류탄. 없는 것은 0.
    struct Old {
        id: Id,
        combo: &'static [i32],
        dash: i32,
        heavy: i32,
        aftershock: i32,
        rb: i32,
        charged: i32,
        grenade: i32,
    }
    const OLD_WEAPONS: [Old; 5] = [
        Old {
            id: Id::Sword,
            combo: &[1, 1, 2],
            dash: 2,
            heavy: 2,
            aftershock: 0,
            rb: 1,
            charged: 3,
            grenade: 0,
        },
        Old {
            id: Id::Hammer,
            combo: &[2, 3],
            dash: 2,
            heavy: 3,
            aftershock: 2,
            rb: 2,
            charged: 4,
            grenade: 0,
        },
        Old {
            id: Id::Staff,
            combo: &[1, 1, 1, 2],
            dash: 1,
            heavy: 2,
            aftershock: 0,
            rb: 1,
            charged: 3,
            grenade: 0,
        },
        Old {
            id: Id::Crossbow,
            combo: &[],
            dash: 0,
            heavy: 0,
            aftershock: 0,
            // 0173: 쇠뇌를 낮췄다(사격 2 → 1, 모은 사격 5 → 4) — 눈금을 옮긴 것이 아니라 값을 바꾼 것이라 이 표의 줄도 같이 고친다.
            rb: 1,
            charged: 4,
            grenade: 3,
        },
        Old {
            id: Id::Fists,
            combo: &[1, 1, 1],
            dash: 1,
            heavy: 1,
            aftershock: 0,
            rb: 1,
            charged: 3,
            grenade: 0,
        },
    ];
    /// 옛 랩 값·상수: 다 모은 강타 덤 · 딱 맞춘 떼기 덤 · 내려찍기 · 치명타 덤 · 되받은 종 조각 · 큰 기술의 벽 쿵.
    const OLD_FULL_BONUS: i32 = 1;
    const OLD_PERFECT: i32 = 1;
    const OLD_PLUNGE: i32 = 2;
    const OLD_CRIT: i32 = 2;
    const OLD_SHARD_REFLECT: i32 = 3;
    const OLD_ULT_SLAM: i32 = 2;
    /// 옛 스킬 피해(표 순서: 휘돌이 · 가로지르기 · 끌어모으기 · 땅울림).
    const OLD_SKILLS: [i32; 4] = [1, 2, 0, 2];
    /// 옛 종 체력(표 `species::ALL` 순서: 잔재 · 가시짐승 · 문지기 · 등딱지짐승 · 꼬투리 · 종지기).
    const OLD_SPECIES_HP: [i32; 6] = [3, 2, 40, 5, 2, 58];
    /// 옛 등급 2·엘리트 배율(올림) — `tuning::TIERS[1].hp`·`ELITE_HP`.
    const OLD_TIER2: f32 = 1.5;
    const OLD_ELITE: f32 = 2.0;

    /// 한 무기의 모든 타(이름, 옛 피해, 새 피해) — 강타는 덜 모음·다 모음·딱 맞춤, 모은 탄은 덜 모음·다 모음·딱 맞춤(원거리만).
    fn attacks(o: &Old) -> Vec<(String, i32, i32)> {
        use crate::tuning::{half_up, idx, HP_UNIT, P, SPECS};
        let w = o.id.def();
        let full = SPECS[idx(P::HeavyFullBonus)].def as i32;
        let mut v = Vec::new();
        assert_eq!(w.combo.len(), o.combo.len(), "{:?} combo", o.id);
        for (i, (h, &old)) in w.combo.iter().zip(o.combo).enumerate() {
            v.push((format!("combo {i}"), old, h.damage));
        }
        v.push(("dash".into(), o.dash, w.dash.map_or(0, |h| h.damage)));
        let heavy = w.heavy.map_or(0, |h| h.damage);
        v.push(("heavy".into(), o.heavy, heavy));
        if w.heavy.is_some() {
            v.push(("heavy full".into(), o.heavy + OLD_FULL_BONUS, heavy + full));
            v.push((
                "heavy perfect".into(),
                o.heavy + OLD_FULL_BONUS + OLD_PERFECT,
                heavy + full + HP_UNIT,
            ));
        }
        let shock = w.heavy.and_then(|h| h.aftershock).map_or(0, |a| a.2);
        v.push(("aftershock".into(), o.aftershock, shock));
        let (rb, ranged) = match w.rb {
            Rb::Magic(b) => (b.damage, false),
            Rb::Shot { bolt, .. } => (bolt.damage, true),
        };
        v.push(("rb".into(), o.rb, rb));
        let c = w.charged.damage;
        v.push(("charged".into(), o.charged, c));
        v.push(("half charged".into(), (o.charged + 1) / 2, half_up(c)));
        if ranged {
            v.push(("perfect shot".into(), o.charged + OLD_PERFECT, c + HP_UNIT));
        }
        v.push((
            "grenade".into(),
            o.grenade,
            w.grenade.map_or(0, |g| g.damage),
        ));
        v
    }

    /// 무기와 상관없는 타(이름, 옛, 새): 내려찍기 · 스킬 넷 · 되받은 종 조각 · 큰 기술의 벽 쿵.
    fn other_attacks() -> Vec<(String, i32, i32)> {
        use crate::tuning::{idx, P, SHARD_REFLECT_DAMAGE, SPECS, ULT_SLAM_DAMAGE};
        let mut v = vec![(
            "plunge".to_string(),
            OLD_PLUNGE,
            SPECS[idx(P::PlungeDamage)].def as i32,
        )];
        for (sk, old) in crate::skills::SKILLS.iter().zip(OLD_SKILLS) {
            v.push((format!("skill {:?}", sk.id), old, sk.damage));
        }
        v.push(("shard".into(), OLD_SHARD_REFLECT, SHARD_REFLECT_DAMAGE));
        v.push(("ult slam".into(), OLD_ULT_SLAM, ULT_SLAM_DAMAGE));
        v
    }

    /// 적의 체력들(이름, 옛, 새): 종마다 등급 1, 잡몹은 등급 2·엘리트·등급 2 엘리트까지(옛 셈 = 작은 정수에 올림).
    fn healths() -> Vec<(String, i32, i32)> {
        use crate::enemy::{Affix, Enemy};
        use macroquad::prelude::Vec3;
        let up = |v: i32, k: f32| (v as f32 * k).ceil() as i32;
        let mut v = Vec::new();
        assert_eq!(crate::species::ALL.len(), OLD_SPECIES_HP.len());
        for (d, old) in crate::species::ALL.iter().zip(OLD_SPECIES_HP) {
            v.push((format!("{:?}", d.kind), old, d.hp));
            if d.boss {
                continue;
            }
            let make = |tier: u8, elite: bool| {
                let mut e = Enemy::new(d.kind, Vec3::ZERO);
                e.set_tier(tier);
                if elite {
                    e.make_elite(Affix::Hardened);
                }
                assert_eq!(e.hp, e.max_hp);
                e.max_hp
            };
            v.push((
                format!("{:?} tier 2", d.kind),
                up(old, OLD_TIER2),
                make(2, false),
            ));
            v.push((
                format!("{:?} elite", d.kind),
                up(old, OLD_ELITE),
                make(1, true),
            ));
            v.push((
                format!("{:?} tier 2 elite", d.kind),
                up(up(old, OLD_TIER2), OLD_ELITE),
                make(2, true),
            ));
        }
        v
    }

    /// 0158 N0 판정 기준: 모든 무기의 모든 타 × 모든 적(등급·엘리트 포함)에서 쓰러뜨리는 데 드는 대 수가 옮기기 전과 같다.
    /// 옛 수는 위의 표(작은 정수)에서, 새 수는 지금 표에서 — `ceil(새 체력 / 새 피해) == ceil(옛 체력 / 옛 피해)`.
    #[test]
    fn hits_to_kill_every_enemy_with_every_attack_are_what_they_were_before_the_scale_move() {
        let mut all = other_attacks();
        for o in &OLD_WEAPONS {
            for (name, old, new) in attacks(o) {
                all.push((format!("{:?} {name}", o.id), old, new));
            }
        }
        assert_eq!(
            OLD_WEAPONS.len(),
            WEAPONS.len(),
            "every weapon is in the old table"
        );
        let mut pairs = 0;
        for (foe, old_hp, new_hp) in healths() {
            for (atk, old_dmg, new_dmg) in &all {
                assert_eq!(
                    *old_dmg == 0,
                    *new_dmg == 0,
                    "{atk}: harmless stays harmless"
                );
                if *old_dmg == 0 {
                    continue;
                }
                let (old, new) = (
                    (old_hp + old_dmg - 1) / old_dmg,
                    (new_hp + new_dmg - 1) / new_dmg,
                );
                assert_eq!(new, old, "{atk} on {foe}: {new_dmg} into {new_hp}");
                pairs += 1;
            }
        }
        assert!(pairs > 500, "{pairs}");
    }

    /// 0158 N0: 옮기기만 했다 — 모든 체력·피해는 옛 수의 꼭 열 배다(대 수가 같은 것보다 센 조건: 덤·배율이 섞여도 그대로).
    #[test]
    fn every_health_and_damage_number_is_exactly_ten_times_the_old_one() {
        use crate::tuning::{idx, HP_UNIT, P, SPECS};
        assert_eq!(HP_UNIT, 10);
        for (foe, old, new) in healths() {
            assert_eq!(new, old * HP_UNIT, "{foe}");
        }
        for o in &OLD_WEAPONS {
            for (name, old, new) in attacks(o) {
                assert_eq!(new, old * HP_UNIT, "{:?} {name}", o.id);
            }
        }
        for (name, old, new) in other_attacks() {
            assert_eq!(new, old * HP_UNIT, "{name}");
        }
        assert_eq!(SPECS[idx(P::ParryCrit)].def as i32, OLD_CRIT * HP_UNIT);
    }

    /// 0158 N0: 종 체력·무기 피해·스킬 피해·장신구 보정은 모두 눈금(`HP_UNIT`)의 배수다 — 옛 한 칸보다 잔 수는 아직 없다
    /// (올림 규칙 `scale_up`·`half_up`이 옛 결과를 지키는 바탕. %를 쓰는 성장(G0)이 이 시험을 고친다).
    #[test]
    fn species_health_and_weapon_skill_and_charm_numbers_are_multiples_of_the_hp_unit() {
        use crate::tuning::HP_UNIT;
        let whole = |v: i32, what: &str| assert_eq!(v % HP_UNIT, 0, "{what}: {v}");
        for d in crate::species::ALL {
            whole(d.hp, &format!("{:?} hp", d.kind));
        }
        for w in WEAPONS.iter() {
            for h in w.combo.iter().chain(w.dash.iter()) {
                whole(h.damage, &format!("{:?} {}", w.id, h.anim));
            }
            if let Some(h) = w.heavy {
                whole(h.damage, &format!("{:?} heavy", w.id));
                if let Some((_, _, d, _)) = h.aftershock {
                    whole(d, &format!("{:?} aftershock", w.id));
                }
            }
            let rb = match w.rb {
                Rb::Magic(b) => b.damage,
                Rb::Shot { bolt, .. } => bolt.damage,
            };
            whole(rb, &format!("{:?} rb", w.id));
            whole(w.charged.damage, &format!("{:?} charged", w.id));
            if let Some(g) = w.grenade {
                whole(g.damage, &format!("{:?} grenade", w.id));
            }
        }
        for s in crate::skills::SKILLS.iter() {
            whole(s.damage, &format!("{:?}", s.id));
        }
        for c in crate::charms::TABLE.iter() {
            let m = c.mods;
            for v in [
                m.hp,
                m.heal,
                m.dmg,
                m.taken,
                m.heavy_dmg,
                m.light_dmg,
                m.opened_dmg,
            ] {
                whole(v, &format!("{:?}", c.id));
            }
        }
        whole(crate::tuning::PLAYER_HP, "player hp");
        whole(crate::tuning::ULT_SLAM_DAMAGE, "ult slam");
        whole(crate::tuning::SHARD_REFLECT_DAMAGE, "shard");
    }
}
