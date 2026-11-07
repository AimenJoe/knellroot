//! 무기 표(0051). 무기 = 공격의 **모양**(KB `concepts/action-rpg-stats-and-equipment.md`): 콤보의 타마다 시간·거리·각·피해·자세 피해,
//! 강타, RB(근접 무기 = 마법 탄, 원거리 무기 = 사격), 원거리 무기의 X(수류탄). 0048: 무기 한 줄 = (X, RB) 짝.
//!
//! 시간·거리·각·내딛기는 랩의 기준값(`attack.*`·`heavy.*`, `tuning.rs`)에 곱하는 **배율** — 손맛의 바탕은 랩 한 곳에서 맞추고,
//! 무기는 그 위의 모양만 갖는다(코드에 타별 숫자를 박지 않는다). 피해·자세 피해는 정수·점수 그대로.
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
    /// 던지는 거리(톡 = 자동 조준 자리까지 이 안에서, 꾹 = 가장 멀리).
    pub near: f32,
    pub far: f32,
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
    damage: 1,
    poise: 1.0,
    pierce: false,
    stagger: 0.0,
    look: 0,
    scale: 1.0,
};

/// 순례자의 검 — 기준. 3연타: 가로 → 되돌려 가로 → 내디디며 찌르기(좁고 길게, 자세 피해 큼).
const SWORD_COMBO: [Hit; 3] = [
    hit(1.0, 1.0, 1.0, 1.0, 1, 1.0, "attack"),
    hit(1.0, 1.0, 1.0, 1.0, 1, 1.0, "attack2"),
    Hit {
        knock: 1.6,
        cancel: 0.15,
        ..hit(1.2, 1.25, 0.45, 1.6, 2, 2.0, "attack3")
    },
];

/// 종치기 망치 — 느리고 넓은 2타(한 걸음씩 나아감, 자세 피해 큼), 강타는 뛰어올라 둥근 내려찍기 + 여진.
const HAMMER_COMBO: [Hit; 2] = [
    Hit {
        knock: 1.5,
        cancel: 0.4,
        ..hit(1.7, 1.15, 1.25, 1.4, 2, 3.0, "hammer1")
    },
    Hit {
        shape: Shape::Circle,
        knock: 2.0,
        cancel: 0.5,
        ..hit(2.0, 1.1, 1.0, 1.2, 3, 4.0, "hammer2")
    },
];

/// 순례자 지팡이 — 빠른 4연타, 넷째는 둘레 회전 마무리(느림).
const STAFF_COMBO: [Hit; 4] = [
    hit(0.7, 1.15, 0.8, 0.7, 1, 0.6, "staff1"),
    hit(0.7, 1.15, 0.8, 0.7, 1, 0.6, "staff2"),
    hit(0.75, 1.2, 0.9, 0.8, 1, 0.8, "staff3"),
    Hit {
        shape: Shape::Circle,
        knock: 1.8,
        ..hit(1.3, 1.25, 1.0, 0.4, 2, 2.0, "staff4")
    },
];

/// 맨손 — 짧고 빠른 주먹 3연타(가까이 붙어야 닿는다, 피해 1·자세 피해 작음), 셋째는 내디디며 한 방. 동작은 검의 것을 빈손으로
/// (주먹 동작은 아직 없다).
const FISTS_COMBO: [Hit; 3] = [
    hit(0.6, 0.7, 0.8, 0.6, 1, 0.5, "attack"),
    hit(0.6, 0.7, 0.8, 0.6, 1, 0.5, "attack2"),
    Hit {
        knock: 1.3,
        cancel: 0.1,
        ..hit(0.8, 0.8, 0.6, 1.2, 1, 1.2, "attack3")
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
            ..hit(1.0, 1.25, 0.5, 2.4, 2, 2.0, "attack3")
        }),
        heavy: Some(Heavy {
            time: 1.0,
            charge: 1.0,
            reach: 1.0,
            arc: 1.0,
            lunge: 1.0,
            shape: Shape::Arc,
            damage: 2,
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
            damage: 3,
            poise: 4.0,
            pierce: true,
            stagger: 0.6,
            scale: 1.8,
            ..MAGIC
        },
        node: "sword",
    },
    Weapon {
        id: Id::Hammer,
        combo: &HAMMER_COMBO,
        // 달려들며 휘두르기: 한 바퀴 둘레(무거워 늦다).
        dash: Some(Hit {
            shape: Shape::Circle,
            knock: 1.8,
            cancel: 0.45,
            ..hit(1.6, 1.0, 1.0, 2.0, 2, 3.0, "hammer2")
        }),
        heavy: Some(Heavy {
            time: 1.3,
            charge: 1.4,
            reach: 1.0,
            arc: 1.0,
            lunge: 2.2,
            shape: Shape::Circle,
            damage: 3,
            poise: 6.0,
            walk: 0.75,
            aftershock: Some((1.5, 2.6, 2, 3.0)),
            breaks_primed: Some(0.6),
            anim_charge: "hammer_charge",
            anim_heavy: "hammer_slam",
            cancel: 0.55,
        }),
        rb: Rb::Magic(Bolt {
            speed: 0.65,
            range: 0.6,
            damage: 2,
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
            damage: 4,
            poise: 8.0,
            stagger: 1.0,
            look: 1,
            scale: 2.2,
            ..MAGIC
        },
        node: "w_hammer",
    },
    Weapon {
        id: Id::Staff,
        combo: &STAFF_COMBO,
        // 달려들며 찌르기: 빠르고 가볍다.
        dash: Some(hit(0.8, 1.3, 0.6, 2.2, 1, 1.0, "staff3")),
        heavy: Some(Heavy {
            time: 0.9,
            charge: 0.9,
            reach: 1.0,
            arc: 1.0,
            lunge: 0.5,
            shape: Shape::Circle,
            damage: 2,
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
            damage: 3,
            poise: 3.0,
            pierce: true,
            look: 2,
            scale: 1.5,
            ..MAGIC
        },
        node: "w_staff",
    },
    Weapon {
        id: Id::Crossbow,
        combo: &[],
        dash: None,
        heavy: None,
        rb: Rb::Shot {
            bolt: Bolt {
                speed: 1.7,
                range: 1.4,
                damage: 2,
                poise: 1.5,
                pierce: false,
                stagger: 0.0,
                look: 3,
                scale: 1.0,
            },
            mag: 4,
            reload: 1.2,
            time: 0.6,
        },
        grenade: Some(Grenade {
            fuse: 1.5,
            count: 2,
            cooldown: 4.0,
            radius: 2.2,
            damage: 3,
            poise: 5.0,
            near: 7.0,
            far: 11.0,
        }),
        // 모은 사격: 시위를 끝까지 당긴 무거운 볼트 — 꿰뚫고 밀어낸다(수류탄도 맞히면 터진다).
        charged: Bolt {
            speed: 2.2,
            range: 1.8,
            damage: 5,
            poise: 6.0,
            pierce: true,
            stagger: 0.4,
            look: 3,
            scale: 1.6,
        },
        node: "w_crossbow",
    },
    Weapon {
        id: Id::Fists,
        combo: &FISTS_COMBO,
        // 달려들며 한 방.
        dash: Some(hit(0.7, 0.8, 0.6, 2.0, 1, 1.0, "attack3")),
        // 모아 치기: 검보다 짧고 빠르다, 피해 1(다 모으면 + `heavy.full bonus`).
        heavy: Some(Heavy {
            time: 0.8,
            charge: 0.8,
            reach: 0.7,
            arc: 0.8,
            lunge: 1.2,
            shape: Shape::Arc,
            damage: 1,
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
            damage: 3,
            poise: 4.0,
            pierce: true,
            stagger: 0.6,
            scale: 1.8,
            ..MAGIC
        },
        // 든 것이 없다(다른 무기 부위는 모두 숨는다).
        node: "",
    },
];

impl Id {
    pub fn def(self) -> &'static Weapon {
        WEAPONS
            .iter()
            .find(|w| w.id == self)
            .expect("WEAPONS has every Id")
    }

    /// 다음 무기(바꾸기 줄).
    pub fn next(self, step: i32) -> Id {
        let i = IDS.iter().position(|x| *x == self).unwrap_or(0) as i32;
        IDS[(i + step).rem_euclid(IDS.len() as i32) as usize]
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
        assert_eq!(Id::Fists.next(1), Id::Sword);
        assert_eq!(Id::Sword.next(-1), Id::Fists);
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
}
