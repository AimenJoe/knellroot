//! 아이템 인스턴스와 랜덤 옵션(0223 V4 — 사장 답 1 "적게, 숫자만"). 아이템 하나 = 바탕(무기·방패·장신구) + 등급 + 옵션 0~2줄 + 벼린 단.
//! 옵션은 **숫자만**(`stats::Mods`의 한 칸씩 — 동작을 바꾸는 효과는 고유 아이템의 몫, V5), 세기는 세 단, **지역이 세기의 상한**이다.
//! 받는 피해를 줄이는 옵션·패링 창을 넓히는 옵션은 없다("방어는 손" — 0095 원칙 2, 시험).
//!
//! 이 파일은 순수한 규칙이다: 옵션 표·굴리기(씨앗이 같으면 같은 아이템)·기록의 글 꼴·`Mods`. 가방과 떨굼은 다음 걸음(V4b·V4c).

use crate::stats::Mods;
use crate::{charms, shield, weapons};

/// 아이템의 바탕.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base {
    Weapon(weapons::Id),
    Shield(shield::Id),
    Charm(charms::Id),
}

/// 등급: 옵션의 줄 수(보통 0 · 단단한 1 · 드문 2). 고유는 정해진 것(굴리지 않는다 — 표는 V5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rarity {
    Plain,
    Sturdy,
    Rare,
    Unique,
}

impl Rarity {
    /// 굴리는 옵션의 줄 수.
    pub fn lines(self) -> usize {
        match self {
            Rarity::Plain => 0,
            Rarity::Sturdy => 1,
            Rarity::Rare | Rarity::Unique => 2,
        }
    }

    /// 기록의 글자(바꾸지 않는다).
    fn key(self) -> &'static str {
        match self {
            Rarity::Plain => "p",
            Rarity::Sturdy => "s",
            Rarity::Rare => "r",
            Rarity::Unique => "u",
        }
    }

    fn by_key(k: &str) -> Option<Rarity> {
        [Rarity::Plain, Rarity::Sturdy, Rarity::Rare, Rarity::Unique]
            .into_iter()
            .find(|r| r.key() == k)
    }
}

/// 옵션이 바꾸는 것(숫자 하나).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    /// 주는 피해 +(타마다).
    Dmg,
    /// 강타 피해 +.
    Heavy,
    /// 사격 피해 +(원거리 무기).
    Shot,
    /// 자세 피해 +%.
    Poise,
    /// 공격 빠르기 +%.
    Swift,
    /// 열린 틈의 타 피해 +.
    Opened,
    /// 최대 기력 +.
    Stamina,
    /// 기력 회복 +%.
    Regen,
    /// 쓰러뜨릴 때 기력 +.
    KillStamina,
    /// 잔불 +%.
    Ember,
    /// 최대 체력 +.
    Hp,
    /// 스킬 대기 −%.
    Cooldown,
    /// 타입 게이지 +%.
    Build,
}

pub const STATS: [Stat; 13] = [
    Stat::Dmg,
    Stat::Heavy,
    Stat::Shot,
    Stat::Poise,
    Stat::Swift,
    Stat::Opened,
    Stat::Stamina,
    Stat::Regen,
    Stat::KillStamina,
    Stat::Ember,
    Stat::Hp,
    Stat::Cooldown,
    Stat::Build,
];

/// 세기의 단 수(1~3). 지역이 상한이다(`tier_cap`).
pub const TIERS: u8 = 3;

impl Stat {
    /// 기록 키(바꾸지 않는다).
    pub fn key(self) -> &'static str {
        match self {
            Stat::Dmg => "dmg",
            Stat::Heavy => "heavy",
            Stat::Shot => "shot",
            Stat::Poise => "poise",
            Stat::Swift => "swift",
            Stat::Opened => "opened",
            Stat::Stamina => "stamina",
            Stat::Regen => "regen",
            Stat::KillStamina => "kill",
            Stat::Ember => "ember",
            Stat::Hp => "hp",
            Stat::Cooldown => "cd",
            Stat::Build => "build",
        }
    }

    pub fn by_key(k: &str) -> Option<Stat> {
        STATS.into_iter().find(|s| s.key() == k)
    }

    /// 단마다의 값(화면에 보이는 수 — 더하기는 그 수, %는 퍼센트).
    pub fn values(self) -> [f32; TIERS as usize] {
        match self {
            // 피해 줄은 벼리기 한 단(+5, 0180)보다 작게: 보스 모의(허수아비)에서 +2/+4/+6은 싸움을 18/31/43% 줄였고
            // 벼리기 한 단은 37% — 굴려 얻는 줄 하나가 정해진 수의 숫돌로 사는 단을 넘지 않는다(0223 검증).
            Stat::Dmg => [1.0, 2.0, 3.0],
            Stat::Heavy => [3.0, 5.0, 7.0],
            Stat::Shot => [1.0, 2.0, 3.0],
            Stat::Poise => [8.0, 14.0, 20.0],
            Stat::Swift => [4.0, 7.0, 10.0],
            Stat::Opened => [3.0, 5.0, 8.0],
            Stat::Stamina => [8.0, 14.0, 20.0],
            Stat::Regen => [6.0, 10.0, 15.0],
            Stat::KillStamina => [4.0, 7.0, 10.0],
            Stat::Ember => [6.0, 10.0, 15.0],
            Stat::Hp => [5.0, 10.0, 15.0],
            Stat::Cooldown => [4.0, 7.0, 10.0],
            Stat::Build => [10.0, 18.0, 25.0],
        }
    }

    pub fn value(self, tier: u8) -> f32 {
        self.values()[(tier.clamp(1, TIERS) - 1) as usize]
    }

    /// 값이 퍼센트인가(화면의 "+8%" — 아니면 "+4").
    pub fn percent(self) -> bool {
        matches!(
            self,
            Stat::Poise | Stat::Swift | Stat::Regen | Stat::Ember | Stat::Cooldown | Stat::Build
        )
    }

    /// 이 옵션이 붙을 수 있는 바탕인가: 무기는 치는 것(원거리는 사격·근접은 강타와 자세), 방패는 버티는 몸, 장신구는 그 밖.
    pub fn fits(self, base: Base) -> bool {
        match base {
            Base::Weapon(w) => {
                let ranged = w.is_ranged();
                match self {
                    Stat::Dmg | Stat::Swift | Stat::Opened | Stat::Build | Stat::Cooldown => true,
                    Stat::Heavy | Stat::Poise => !ranged,
                    Stat::Shot => ranged,
                    _ => false,
                }
            }
            Base::Shield(_) => matches!(
                self,
                Stat::Stamina | Stat::Regen | Stat::Hp | Stat::KillStamina
            ),
            Base::Charm(_) => matches!(
                self,
                Stat::Stamina
                    | Stat::Regen
                    | Stat::Hp
                    | Stat::KillStamina
                    | Stat::Ember
                    | Stat::Cooldown
                    | Stat::Build
            ),
        }
    }

    /// 이 옵션 한 줄의 보정.
    pub fn mods(self, tier: u8) -> Mods {
        let v = self.value(tier);
        let up = 1.0 + v / 100.0;
        let d = Mods::default();
        match self {
            Stat::Dmg => Mods { dmg: v as i32, ..d },
            Stat::Heavy => Mods {
                heavy_dmg: v as i32,
                ..d
            },
            Stat::Shot => Mods {
                shot_dmg: v as i32,
                ..d
            },
            Stat::Poise => Mods { poise: up, ..d },
            Stat::Swift => Mods { atk_speed: up, ..d },
            Stat::Opened => Mods {
                opened_dmg: v as i32,
                ..d
            },
            Stat::Stamina => Mods {
                stamina_max: v,
                ..d
            },
            Stat::Regen => Mods {
                stamina_regen: up,
                ..d
            },
            Stat::KillStamina => Mods {
                kill_stamina: v,
                ..d
            },
            Stat::Ember => Mods { ember: up, ..d },
            Stat::Hp => Mods { hp: v as i32, ..d },
            Stat::Cooldown => Mods {
                skill_cd: 1.0 - v / 100.0,
                ..d
            },
            Stat::Build => Mods { ail: up, ..d },
        }
    }
}

/// 옵션 한 줄: 무엇을·몇 단으로.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Affix {
    pub stat: Stat,
    pub tier: u8,
}

/// 옵션의 줄 수 상한.
pub const AFFIXES: usize = 2;

/// 아이템 하나.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    pub base: Base,
    pub rarity: Rarity,
    pub affixes: [Option<Affix>; AFFIXES],
    /// 벼린 단(0180 — 무기만, 아이템 하나에 붙는다).
    pub forge: u8,
}

impl Item {
    /// 옵션 없는 보통 아이템(옛 기록의 무기·장신구·방패가 이것으로 옮겨진다).
    pub fn plain(base: Base) -> Item {
        Item {
            base,
            rarity: Rarity::Plain,
            affixes: [None; AFFIXES],
            forge: 0,
        }
    }

    /// 옵션들의 보정(바탕의 몫 — 무기 표·장신구 표·벼린 단 — 은 저마다의 자리에서 그대로 더해진다).
    pub fn mods(&self) -> Vec<Mods> {
        self.affixes
            .iter()
            .flatten()
            .map(|a| a.stat.mods(a.tier))
            .collect()
    }

    /// 거두면 돌아오는 잔불(0223 D — 화폐는 늘리지 않는다): 등급의 바탕 + 옵션 단의 합마다.
    pub fn salvage(&self) -> u32 {
        let base = match self.rarity {
            Rarity::Plain => SALVAGE[0],
            Rarity::Sturdy => SALVAGE[1],
            Rarity::Rare => SALVAGE[2],
            Rarity::Unique => SALVAGE[3],
        };
        let tiers: u32 = self.affixes.iter().flatten().map(|a| a.tier as u32).sum();
        base + tiers * SALVAGE_PER_TIER
    }

    /// 기록의 글(바꾸지 않는다): `<바탕>|<등급>|<옵션.단>+<옵션.단>|<벼린 단>` — 예 `w.sword|r|dmg.2+poise.1|0`.
    pub fn encode(&self) -> String {
        let base = match self.base {
            Base::Weapon(w) => format!("w.{}", w.key()),
            Base::Shield(s) => format!("s.{}", s.key()),
            Base::Charm(c) => format!("c.{}", charms::def(c).key),
        };
        let affixes: Vec<String> = self
            .affixes
            .iter()
            .flatten()
            .map(|a| format!("{}.{}", a.stat.key(), a.tier))
            .collect();
        format!(
            "{base}|{}|{}|{}",
            self.rarity.key(),
            affixes.join("+"),
            self.forge
        )
    }

    /// 기록의 글 → 아이템. 모르는 바탕·등급이면 None(그 줄은 버린다), 모르는 옵션·바탕에 안 맞는 옵션은 그 줄만 버린다.
    pub fn decode(text: &str) -> Option<Item> {
        let mut parts = text.split('|');
        let (kind, key) = parts.next()?.split_once('.')?;
        let base = match kind {
            "w" => Base::Weapon(weapons::Id::by_key(key)?),
            "s" => Base::Shield(shield::Id::by_key(key)?),
            "c" => Base::Charm(charms::by_key(key)?),
            _ => return None,
        };
        let rarity = Rarity::by_key(parts.next()?)?;
        let mut affixes = [None; AFFIXES];
        let mut n = 0;
        for a in parts.next().unwrap_or("").split('+') {
            let Some((stat, tier)) = a.split_once('.') else {
                continue;
            };
            let (Some(stat), Ok(tier)) = (Stat::by_key(stat), tier.parse::<u8>()) else {
                continue;
            };
            let dup = affixes.iter().flatten().any(|x: &Affix| x.stat == stat);
            if n < rarity.lines() && stat.fits(base) && (1..=TIERS).contains(&tier) && !dup {
                affixes[n] = Some(Affix { stat, tier });
                n += 1;
            }
        }
        let forge = parts
            .next()
            .and_then(|f| f.parse::<u8>().ok())
            .unwrap_or(0)
            .min(crate::forge::RANKS);
        Some(Item {
            base,
            rarity,
            affixes,
            forge: if matches!(base, Base::Weapon(_)) {
                forge
            } else {
                0
            },
        })
    }
}

/// 낀 자리들이 가리키는 아이템(가방의 번호). 바탕은 기록의 옛 칸(든 무기·부무기·방패·장신구 셋)이 말하고, 이것은 그 바탕의
/// **어느 것**인지를 말한다.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gear {
    pub main: Option<u32>,
    pub sub: Option<u32>,
    pub shield: Option<u32>,
    pub charms: [Option<u32>; charms::SLOTS],
}

/// 지금 낀 바탕들(기록의 옛 칸에서 — 맨손은 아이템이 아니다).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Worn {
    pub main: Option<weapons::Id>,
    pub sub: Option<weapons::Id>,
    pub shield: Option<shield::Id>,
    pub charms: [Option<charms::Id>; charms::SLOTS],
}

/// 가방(0223 V4b): 가진 아이템들(번호는 바뀌지 않는다)과 낀 자리.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bag {
    pub items: Vec<(u32, Item)>,
    next: u32,
    pub gear: Gear,
}

/// 기록의 키(바꾸지 않는다): `item.<번호>` = 아이템의 글 · `item.next` = 다음 번호 · `gear.*` = 낀 자리의 번호.
const KEY_ITEM: &str = "item.";
const KEY_NEXT: &str = "item.next";
const KEY_GEAR: [&str; 3] = ["gear.main", "gear.sub", "gear.shield"];
const KEY_GEAR_CHARM: &str = "gear.charm.";

impl Bag {
    pub fn get(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|(i, _)| *i == id).map(|(_, it)| it)
    }

    /// 지금까지 넣은 수(다음 번호) — 떨굼의 씨앗에 섞는다(같은 자리를 다시 쓰러뜨려도 다른 것이 나오게).
    pub fn serial(&self) -> u32 {
        self.next
    }

    /// 가방에 넣는다 — 새 번호를 돌려준다.
    pub fn add(&mut self, item: Item) -> u32 {
        let id = self.next.max(
            self.items
                .iter()
                .map(|(i, _)| i.saturating_add(1))
                .max()
                .unwrap_or(0),
        );
        self.next = id.saturating_add(1);
        self.items.push((id, item));
        id
    }

    /// 가방에서 뺀다(거두기).
    pub fn remove(&mut self, id: u32) -> Option<Item> {
        let at = self.items.iter().position(|(i, _)| *i == id)?;
        Some(self.items.remove(at).1)
    }

    /// 거둘 수 있는 여벌(0223 V4): 낀 자리가 가리키지 않고, 같은 바탕의 다른 아이템이 남는 것 — 거둬도 가진 바탕이 사라지지 않는다.
    pub fn spares(&self) -> Vec<u32> {
        let g = &self.gear;
        let worn = [
            g.main,
            g.sub,
            g.shield,
            g.charms[0],
            g.charms[1],
            g.charms[2],
        ];
        self.items
            .iter()
            .filter(|(id, it)| !worn.contains(&Some(*id)) && self.of(it.base).count() > 1)
            .map(|(id, _)| *id)
            .collect()
    }

    /// 그 바탕의 아이템들.
    pub fn of(&self, base: Base) -> impl Iterator<Item = (u32, &Item)> {
        self.items
            .iter()
            .filter(move |(_, it)| it.base == base)
            .map(|(i, it)| (*i, it))
    }

    /// 자리가 가리키는 아이템이 그 바탕의 것이면 그것 — 아니면(막 바꿔 껴서 자리가 아직 안 맞춰졌으면) 그 바탕의 첫 아이템.
    fn held(&self, id: Option<u32>, base: Option<Base>) -> Option<&Item> {
        let base = base?;
        id.and_then(|i| self.get(i))
            .filter(|it| it.base == base)
            .or_else(|| self.of(base).next().map(|(_, it)| it))
    }

    /// 가방을 기록의 옛 칸에 맞춘다: 가진 바탕마다 아이템이 적어도 하나(없으면 보통 것 하나 — 옛 기록이 이렇게 옮겨진다),
    /// 낀 자리는 낀 바탕의 아이템을 가리킨다(맞으면 그대로, 아니면 그 바탕의 아직 안 낀 첫 아이템). 맨손은 아이템이 없다.
    pub fn settle(&mut self, owned: &[Base], worn: &Worn) {
        for &b in owned {
            if b != Base::Weapon(weapons::Id::Fists) && self.of(b).next().is_none() {
                self.add(Item::plain(b));
            }
        }
        let mut used: Vec<u32> = Vec::new();
        let g = self.gear;
        let was = [
            g.main,
            g.sub,
            g.shield,
            g.charms[0],
            g.charms[1],
            g.charms[2],
        ];
        let mut fix = |bag: &Bag, slot: Option<u32>, base: Option<Base>| -> Option<u32> {
            let base = base.filter(|b| *b != Base::Weapon(weapons::Id::Fists))?;
            let keep = slot
                .filter(|id| bag.get(*id).is_some_and(|it| it.base == base) && !used.contains(id));
            // 자리가 안 맞으면: 방금까지 다른 자리에 끼고 있던 그 바탕의 아이템이 먼저(주·부무기를 맞바꾸거나 장신구의 칸을
            // 옮겨도 끼던 그 아이템이 따라간다 — 바탕의 첫 아이템(보통 것)으로 슬쩍 바뀌지 않는다), 없으면 그 바탕의 첫 아이템.
            let id = keep
                .or_else(|| {
                    was.iter().flatten().copied().find(|i| {
                        bag.get(*i).is_some_and(|it| it.base == base) && !used.contains(i)
                    })
                })
                .or_else(|| bag.of(base).map(|(i, _)| i).find(|i| !used.contains(i)))?;
            used.push(id);
            Some(id)
        };
        let main = fix(self, g.main, worn.main.map(Base::Weapon));
        let sub = fix(self, g.sub, worn.sub.map(Base::Weapon));
        let shield = fix(self, g.shield, worn.shield.map(Base::Shield));
        let mut charms_at = [None; charms::SLOTS];
        for (i, slot) in charms_at.iter_mut().enumerate() {
            *slot = fix(self, g.charms[i], worn.charms[i].map(Base::Charm));
        }
        self.gear = Gear {
            main,
            sub,
            shield,
            charms: charms_at,
        };
    }

    /// 낀 아이템들의 옵션 보정: **든** 무기(부무기는 들었을 때만 — 벼린 단과 같은 규칙) + 방패 + 장신구 셋.
    pub fn mods(&self, worn: &Worn) -> Vec<Mods> {
        let g = &self.gear;
        let mut v = Vec::new();
        let slots = [
            (g.main, worn.main.map(Base::Weapon)),
            (g.shield, worn.shield.map(Base::Shield)),
            (g.charms[0], worn.charms[0].map(Base::Charm)),
            (g.charms[1], worn.charms[1].map(Base::Charm)),
            (g.charms[2], worn.charms[2].map(Base::Charm)),
        ];
        for (id, base) in slots {
            if let Some(it) = self.held(id, base) {
                v.extend(it.mods());
            }
        }
        v
    }

    /// 주무기와 부무기를 바꿔 들었다 — 자리의 아이템도 맞바뀐다.
    pub fn swap_weapons(&mut self) {
        std::mem::swap(&mut self.gear.main, &mut self.gear.sub);
    }

    /// 기록에서 읽는다(모르는 글의 아이템은 버린다).
    pub fn read(r: &crate::save::Record) -> Bag {
        let mut items: Vec<(u32, Item)> = r
            .with_prefix(KEY_ITEM)
            .filter_map(|(id, text)| Some((id.parse::<u32>().ok()?, Item::decode(text)?)))
            .collect();
        items.sort_by_key(|(i, _)| *i);
        // 손댄 기록의 같은 번호(`item.7`·`item.007`)는 하나만.
        items.dedup_by_key(|(i, _)| *i);
        let at = |k: &str| r.num(k);
        let mut charms_at = [None; charms::SLOTS];
        for (i, slot) in charms_at.iter_mut().enumerate() {
            *slot = at(&format!("{KEY_GEAR_CHARM}{i}"));
        }
        Bag {
            next: at(KEY_NEXT).unwrap_or(0),
            items,
            gear: Gear {
                main: at(KEY_GEAR[0]),
                sub: at(KEY_GEAR[1]),
                shield: at(KEY_GEAR[2]),
                charms: charms_at,
            },
        }
    }

    /// 기록에 쓴다 — 옛 줄을 다 지우고 지금 것만. 빈 가방은 아무 줄도 쓰지 않는다(빈 슬롯이 "기록 있음"이 되지 않게).
    pub fn write(&self, r: &mut crate::save::Record) {
        r.remove_prefix(KEY_ITEM);
        r.remove_prefix("gear.");
        if self.items.is_empty() {
            return;
        }
        for (id, it) in &self.items {
            r.set(&format!("{KEY_ITEM}{id}"), it.encode());
        }
        r.set(KEY_NEXT, self.next);
        let g = &self.gear;
        for (k, v) in KEY_GEAR.into_iter().zip([g.main, g.sub, g.shield]) {
            if let Some(v) = v {
                r.set(k, v);
            }
        }
        for (i, v) in g.charms.iter().enumerate() {
            if let Some(v) = v {
                r.set(&format!("{KEY_GEAR_CHARM}{i}"), v);
            }
        }
    }
}

/// 가방의 상한(0223 D) — 가득이면 새로 얻은 여벌은 잔불로 거둔다(처음 얻는 바탕은 상한과 상관없이 든다).
pub const BAG_CAP: usize = 30;

/// 거두는 값: 등급마다의 바탕(보통·단단한·드문·고유)과 옵션 단 하나의 몫.
pub const SALVAGE: [u32; 4] = [4, 10, 20, 40];
pub const SALVAGE_PER_TIER: u32 = 4;

/// 지역(스테이지 번호 0~)마다의 옵션 세기 상한: 성소 1 · 마을 2 · 정원 3 · 작업층 3.
pub fn tier_cap(stage: usize) -> u8 {
    match stage {
        0 => 1,
        1 => 2,
        _ => TIERS,
    }
}

/// 굴리는 난수(씨앗이 같으면 같은 아이템 — 시험·되살아난 상자가 같은 것을 낸다).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        // 0은 xorshift에서 멈춘다 — 섞어서 넣는다.
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        ((self.next() >> 33) as usize) % n.max(1)
    }
}

/// 여벌의 바탕을 고른다(0223 V4c): 씨앗을 섞어 무게대로 — 지금 낀 바탕은 `WORN_WEIGHT`배(쓰는 것의 여벌이 더 자주 나온다 —
/// 견줄 것이 생긴다), 나머지는 1. 씨앗의 낮은 자리(자리 번호)만 달라도 다른 바탕이 나온다.
pub fn pick_base(pool: &[Base], worn: &[Base], seed: u64) -> Option<Base> {
    let weight = |b: &Base| if worn.contains(b) { WORN_WEIGHT } else { 1 };
    let total: usize = pool.iter().map(weight).sum();
    if total == 0 {
        return None;
    }
    let mut r = Rng::new(seed ^ 0x0B45_E5ED).below(total);
    for b in pool {
        let w = weight(b);
        if r < w {
            return Some(*b);
        }
        r -= w;
    }
    pool.last().copied()
}

/// 낀 바탕의 무게(여벌 고르기).
pub const WORN_WEIGHT: usize = 3;

/// 등급을 굴린다: 떨군 것의 급(`luck` 0 = 잡몹·상자 · 1 = 엘리트 · 2 = 문지기)이 높을수록 드문 쪽으로.
/// 백에 몇 — 잡몹·상자 [보통 60 · 단단한 32 · 드문 8], 엘리트 [20 · 55 · 25], 문지기 [0 · 40 · 60].
pub fn roll_rarity(luck: u8, seed: u64) -> Rarity {
    let odds = RARITY_ODDS[(luck as usize).min(RARITY_ODDS.len() - 1)];
    let r = Rng::new(seed ^ 0xA5A5).below(100) as u32;
    if r < odds[0] {
        Rarity::Plain
    } else if r < odds[0] + odds[1] {
        Rarity::Sturdy
    } else {
        Rarity::Rare
    }
}

pub const RARITY_ODDS: [[u32; 3]; 3] = [[60, 32, 8], [20, 55, 25], [0, 40, 60]];

/// 옵션을 굴려 아이템을 만든다: 등급만큼의 줄(서로 다른 옵션 — 바탕에 맞는 것만), 세기는 1~`cap`(낮은 단이 더 흔하다).
pub fn roll(base: Base, rarity: Rarity, cap: u8, seed: u64) -> Item {
    let mut rng = Rng::new(seed);
    let cap = cap.clamp(1, TIERS);
    let mut pool: Vec<Stat> = STATS.into_iter().filter(|s| s.fits(base)).collect();
    let mut affixes = [None; AFFIXES];
    let lines = if rarity == Rarity::Unique {
        0
    } else {
        rarity.lines()
    };
    for slot in affixes.iter_mut().take(lines) {
        if pool.is_empty() {
            break;
        }
        let stat = pool.remove(rng.below(pool.len()));
        // 단: 둘을 굴려 낮은 쪽(높은 단은 드물다).
        let tier = 1 + rng.below(cap as usize).min(rng.below(cap as usize)) as u8;
        *slot = Some(Affix { stat, tier });
    }
    Item {
        base,
        rarity,
        affixes,
        forge: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::Stats;

    fn bases() -> Vec<Base> {
        let mut v: Vec<Base> = weapons::IDS.into_iter().map(Base::Weapon).collect();
        v.extend(shield::TABLE.iter().map(|d| Base::Shield(d.id)));
        v.extend(charms::TABLE.iter().map(|d| Base::Charm(d.id)));
        v
    }

    /// 여벌의 바탕: 자리 번호만 다른 씨앗들이 여러 바탕을 낸다(한 판의 엘리트가 모두 같은 것을 주지 않는다), 낀 바탕이 더 자주.
    #[test]
    fn spare_bases_vary_by_spot_and_lean_to_what_is_worn() {
        let pool = bases();
        let worn = [pool[0]];
        let seed = |i: u64| (3u64 << 48) ^ (2u64 << 32) ^ (5u64 << 12) ^ i;
        let mut seen = std::collections::BTreeSet::new();
        for i in 0..6 {
            let b = pick_base(&pool, &worn, seed(i)).unwrap();
            seen.insert(format!("{b:?}"));
        }
        assert!(seen.len() >= 3, "six spots gave only {seen:?}");
        let n = 4000;
        let hits = (0..n)
            .filter(|&i| pick_base(&pool, &worn, seed(i)) == Some(pool[0]))
            .count() as f32;
        let fair = WORN_WEIGHT as f32 / (pool.len() - 1 + WORN_WEIGHT) as f32;
        assert!(
            (hits / n as f32 - fair).abs() < 0.03,
            "{hits} of {n}, want {fair}"
        );
        assert_eq!(pick_base(&[], &worn, 1), None);
    }

    /// 등급 = 옵션의 줄 수. 줄마다 다른 옵션이고 바탕에 맞는 것만, 세기는 지역의 상한 안. 씨앗이 같으면 같은 아이템.
    #[test]
    fn a_roll_gives_as_many_lines_as_the_rarity_all_fitting_distinct_and_under_the_cap() {
        for base in bases() {
            for (rarity, lines) in [(Rarity::Plain, 0), (Rarity::Sturdy, 1), (Rarity::Rare, 2)] {
                for cap in 1..=TIERS {
                    for seed in 0..200u64 {
                        let it = roll(base, rarity, cap, seed);
                        let got: Vec<Affix> = it.affixes.iter().flatten().copied().collect();
                        assert_eq!(got.len(), lines, "{base:?} {rarity:?}");
                        assert!(got.iter().all(|a| a.stat.fits(base)));
                        assert!(got.iter().all(|a| (1..=cap).contains(&a.tier)));
                        assert!(lines < 2 || got[0].stat != got[1].stat);
                        assert_eq!(it, roll(base, rarity, cap, seed), "same seed, same item");
                    }
                }
            }
        }
    }

    /// 높은 단은 드물다: 상한 3에서 3단은 1단보다 적게 나온다(그래도 나온다).
    #[test]
    fn high_tiers_are_rarer_but_do_drop() {
        let base = Base::Weapon(weapons::Id::Sword);
        let mut n = [0u32; 3];
        for seed in 0..3000u64 {
            let it = roll(base, Rarity::Sturdy, 3, seed);
            n[(it.affixes[0].unwrap().tier - 1) as usize] += 1;
        }
        assert!(n[0] > n[1] && n[1] > n[2] && n[2] > 100, "{n:?}");
    }

    /// 모든 옵션이 어딘가에는 붙는다(붙을 바탕이 없는 옵션은 없다), 바탕마다 굴릴 옵션이 둘 이상이다.
    #[test]
    fn every_stat_has_a_home_and_every_base_has_two_to_roll() {
        for s in STATS {
            assert!(bases().into_iter().any(|b| s.fits(b)), "{s:?}");
            assert_eq!(Stat::by_key(s.key()), Some(s));
            let v = s.values();
            assert!(v[0] > 0.0 && v[0] < v[1] && v[1] < v[2], "{s:?}");
        }
        for b in bases() {
            assert!(
                STATS.into_iter().filter(|s| s.fits(b)).count() >= 2,
                "{b:?}"
            );
        }
    }

    /// 0223 D·0095 원칙 2: 옵션은 받는 피해를 줄이지도, 패링 창·회피를 넓히지도 않는다 — 방어는 손이다. 한 줄은 능력치를 낫게만 바꾼다.
    #[test]
    fn no_affix_touches_defence_or_the_parry_window() {
        let none = Stats::default();
        for s in STATS {
            for tier in 1..=TIERS {
                let got = Stats::compute(&[s.mods(tier)]);
                assert_eq!(
                    (got.taken, got.parry, got.dodge_cost, got.knock, got.flinch),
                    (
                        none.taken,
                        none.parry,
                        none.dodge_cost,
                        none.knock,
                        none.flinch
                    ),
                    "{s:?}"
                );
                assert_ne!(got, none, "{s:?} does something");
                assert!(got.skill_cd <= 1.0 && got.ail >= 1.0);
            }
        }
    }

    /// 기록의 글로 갔다 와도 같은 아이템. 모르는 바탕은 버리고, 모르는 옵션·안 맞는 옵션·넘치는 줄은 그 줄만 버린다.
    #[test]
    fn items_round_trip_through_the_record_text() {
        for base in bases() {
            for seed in 0..40u64 {
                let mut it = roll(base, roll_rarity((seed % 3) as u8, seed), 3, seed);
                if matches!(base, Base::Weapon(_)) {
                    it.forge = (seed % 4) as u8;
                }
                assert_eq!(Item::decode(&it.encode()), Some(it), "{}", it.encode());
            }
        }
        assert_eq!(
            Item::plain(Base::Weapon(weapons::Id::Sword)).encode(),
            "w.sword|p||0"
        );
        assert_eq!(Item::decode("w.nothing|p||0"), None);
        assert_eq!(Item::decode("x.sword|p||0"), None);
        // 안 맞는 옵션(검에 사격)·모르는 옵션은 빠지고 맞는 줄은 남는다. 보통 등급에 적힌 옵션은 버린다.
        let it = Item::decode("w.sword|r|shot.2+zzz.1+dmg.3|1").unwrap();
        assert_eq!(
            it.affixes,
            [
                Some(Affix {
                    stat: Stat::Dmg,
                    tier: 3
                }),
                None
            ]
        );
        assert_eq!(it.forge, 1);
        assert_eq!(
            Item::decode("w.sword|p|dmg.3|0").unwrap().affixes,
            [None; 2]
        );
        // 방패·장신구는 벼리지 않는다.
        assert_eq!(Item::decode("s.pilgrim|p||3").unwrap().forge, 0);
    }

    /// 급이 높은 것이 떨군 아이템일수록 드문 쪽이다 — 문지기는 보통을 떨구지 않는다. 지역의 상한은 성소 1 · 마을 2 · 그 뒤 3.
    #[test]
    fn better_sources_roll_rarer_and_regions_cap_the_tier() {
        let count = |luck: u8| {
            let mut n = [0u32; 3];
            for seed in 0..2000u64 {
                n[roll_rarity(luck, seed) as usize] += 1;
            }
            n
        };
        let (mob, elite, boss) = (count(0), count(1), count(2));
        assert!(mob[0] > mob[1] && mob[1] > mob[2] && mob[2] > 0, "{mob:?}");
        assert!(elite[2] > mob[2] && elite[0] < mob[0], "{elite:?}");
        assert_eq!(boss[0], 0);
        assert!(boss[2] > boss[1], "{boss:?}");
        for odds in RARITY_ODDS {
            assert_eq!(odds.iter().sum::<u32>(), 100);
        }
        assert_eq!(
            [tier_cap(0), tier_cap(1), tier_cap(2), tier_cap(3)],
            [1, 2, 3, 3]
        );
    }

    fn worn_sword_and(sub: Option<weapons::Id>) -> Worn {
        Worn {
            main: Some(weapons::Id::Sword),
            sub,
            shield: Some(shield::Id::Pilgrim),
            charms: [Some(charms::Id::Ear), None, None],
        }
    }

    /// 가방 맞추기(V4b): 가진 바탕마다 아이템이 하나는 생기고(이미 있으면 그대로 — 두 번 맞춰도 같다), 낀 자리는 낀 바탕의 아이템을
    /// 가리킨다. 맨손은 아이템이 없다. 이미 가리키던 아이템이 그 바탕의 것이면 바꾸지 않는다(더 좋은 것을 골라 낀 것을 지킨다).
    #[test]
    fn settling_gives_every_owned_base_an_item_and_points_gear_at_what_is_worn() {
        let sword = Base::Weapon(weapons::Id::Sword);
        let owned = [
            sword,
            Base::Weapon(weapons::Id::Hammer),
            Base::Weapon(weapons::Id::Fists),
            Base::Shield(shield::Id::Pilgrim),
            Base::Charm(charms::Id::Ear),
        ];
        let worn = worn_sword_and(Some(weapons::Id::Hammer));
        let mut bag = Bag::default();
        bag.settle(&owned, &worn);
        assert_eq!(bag.items.len(), 4, "no item for bare fists");
        assert!(bag.items.iter().all(|(_, it)| it.rarity == Rarity::Plain));
        let g = bag.gear;
        assert_eq!(bag.get(g.main.unwrap()).unwrap().base, sword);
        assert_eq!(
            bag.get(g.sub.unwrap()).unwrap().base,
            Base::Weapon(weapons::Id::Hammer)
        );
        assert!(g.shield.is_some() && g.charms[0].is_some() && g.charms[1].is_none());
        let once = bag.clone();
        bag.settle(&owned, &worn);
        assert_eq!(bag, once, "settling twice changes nothing");
        // 더 좋은 검을 주워 그것을 꼈다 — 맞춰도 그대로다. 맨손으로 바꾸면 주무기 자리는 빈다.
        let better = bag.add(roll(sword, Rarity::Rare, 3, 7));
        bag.gear.main = Some(better);
        bag.settle(&owned, &worn);
        assert_eq!(bag.gear.main, Some(better));
        let bare = Worn {
            main: Some(weapons::Id::Fists),
            ..worn
        };
        bag.settle(&owned, &bare);
        assert_eq!(bag.gear.main, None);
    }

    /// 옵션은 낀 것만 — 무기는 **든** 것만(부무기의 옵션은 바꿔 들어야 먹는다), 방패·장신구는 낀 동안.
    #[test]
    fn only_worn_items_count_and_a_weapon_only_while_held() {
        let (sword, hammer) = (
            Base::Weapon(weapons::Id::Sword),
            Base::Weapon(weapons::Id::Hammer),
        );
        let affix = |stat| [Some(Affix { stat, tier: 2 }), None];
        let item = |base, stat| Item {
            base,
            rarity: Rarity::Sturdy,
            affixes: affix(stat),
            forge: 0,
        };
        let mut bag = Bag::default();
        let s = bag.add(item(sword, Stat::Dmg));
        let h = bag.add(item(hammer, Stat::Heavy));
        let spare = bag.add(item(sword, Stat::Swift));
        let c = bag.add(item(Base::Charm(charms::Id::Ear), Stat::Hp));
        bag.gear = Gear {
            main: Some(s),
            sub: Some(h),
            shield: None,
            charms: [Some(c), None, None],
        };
        let worn = Worn {
            main: Some(weapons::Id::Sword),
            sub: Some(weapons::Id::Hammer),
            shield: None,
            charms: [Some(charms::Id::Ear), None, None],
        };
        let st = Stats::compute(&bag.mods(&worn));
        let none = Stats::default();
        assert_eq!(st.dmg, none.dmg + Stat::Dmg.value(2) as i32);
        assert_eq!(st.max_hp, none.max_hp + 10);
        assert_eq!(st.heavy_dmg, none.heavy_dmg, "the off weapon waits");
        assert_eq!(st.atk_speed, none.atk_speed, "the spare stays in the bag");
        // 바꿔 든다.
        bag.swap_weapons();
        let swapped = Worn {
            main: worn.sub,
            sub: worn.main,
            ..worn
        };
        let st = Stats::compute(&bag.mods(&swapped));
        assert_eq!(
            (st.dmg, st.heavy_dmg),
            (none.dmg, none.heavy_dmg + Stat::Heavy.value(2) as i32)
        );
        let _ = spare;
    }

    /// 주·부무기를 맞바꾸거나 장신구의 칸을 옮겨도 끼던 그 아이템이 따라간다 — 바탕의 첫 아이템(보통 것)으로 슬쩍 바뀌지 않는다.
    #[test]
    fn worn_items_follow_their_base_when_slots_are_exchanged() {
        let (sword, hammer) = (
            Base::Weapon(weapons::Id::Sword),
            Base::Weapon(weapons::Id::Hammer),
        );
        let (ear, hand) = (Base::Charm(charms::Id::Ear), Base::Charm(charms::Id::Hand));
        let mut bag = Bag::default();
        for b in [sword, hammer, ear, hand] {
            bag.add(Item::plain(b));
        }
        let rs = bag.add(roll(sword, Rarity::Rare, 3, 1));
        let rh = bag.add(roll(hammer, Rarity::Rare, 3, 2));
        let re = bag.add(roll(ear, Rarity::Rare, 3, 3));
        bag.gear = Gear {
            main: Some(rs),
            sub: Some(rh),
            shield: None,
            charms: [Some(re), None, None],
        };
        let owned = [sword, hammer, ear, hand];
        // 장비 탭에서 망치를 주무기로(검이 부무기로), 귀를 둘째 칸으로 옮기고 첫 칸에 손.
        let worn = Worn {
            main: Some(weapons::Id::Hammer),
            sub: Some(weapons::Id::Sword),
            shield: None,
            charms: [Some(charms::Id::Hand), Some(charms::Id::Ear), None],
        };
        bag.settle(&owned, &worn);
        assert_eq!((bag.gear.main, bag.gear.sub), (Some(rh), Some(rs)));
        assert_eq!(bag.gear.charms[1], Some(re));
        assert_eq!(bag.get(bag.gear.charms[0].unwrap()).unwrap().base, hand);
    }

    /// 거둘 수 있는 것은 여벌뿐: 낀 것은 아니고, 그 바탕의 마지막 하나도 아니다(가진 바탕이 사라지지 않는다).
    #[test]
    fn only_spares_can_be_salvaged_never_the_worn_or_the_last_of_a_base() {
        let (sword, hammer) = (
            Base::Weapon(weapons::Id::Sword),
            Base::Weapon(weapons::Id::Hammer),
        );
        let mut bag = Bag::default();
        let worn = bag.add(Item::plain(sword));
        let spare = bag.add(roll(sword, Rarity::Sturdy, 1, 3));
        let only = bag.add(Item::plain(hammer));
        bag.gear.main = Some(worn);
        assert_eq!(bag.spares(), [spare]);
        assert!(bag.remove(spare).is_some());
        assert!(
            bag.spares().is_empty(),
            "the worn sword and the only hammer stay"
        );
        assert!(bag.get(only).is_some() && bag.remove(spare).is_none());
        // 끼지 않은 바탕이 둘이면 하나는 거둘 수 있고, 거두고 나면 남은 하나는 못 거둔다.
        let second = bag.add(Item::plain(hammer));
        assert_eq!(bag.spares(), [only, second]);
        bag.remove(only);
        assert!(bag.spares().is_empty());
    }

    /// 가방은 기록에 갔다 와도 같다. 빈 가방은 아무 줄도 쓰지 않고, 없어진 아이템의 옛 줄은 지워진다.
    #[test]
    fn the_bag_round_trips_and_an_empty_one_writes_nothing() {
        use crate::save::Record;
        let mut r = Record::parse("clears=1\n");
        Bag::default().write(&mut r);
        assert_eq!(r.to_text(), "clears=1\n");
        let mut bag = Bag::default();
        bag.settle(
            &[
                Base::Weapon(weapons::Id::Sword),
                Base::Shield(shield::Id::Pilgrim),
                Base::Charm(charms::Id::Ear),
            ],
            &worn_sword_and(None),
        );
        let extra = bag.add(roll(Base::Weapon(weapons::Id::Sword), Rarity::Rare, 3, 11));
        bag.write(&mut r);
        assert_eq!(Bag::read(&r), bag);
        // 하나를 없애고 다시 쓰면 그 줄은 남지 않는다 — 번호는 다시 쓰지 않는다(다음 아이템은 새 번호).
        bag.items.retain(|(i, _)| *i != extra);
        bag.write(&mut r);
        assert!(r.get(&format!("item.{extra}")).is_none());
        let mut back = Bag::read(&r);
        assert_eq!(back, bag);
        assert!(back.add(Item::plain(Base::Weapon(weapons::Id::Hammer))) > extra);
    }

    /// 거두는 값은 등급과 옵션의 세기를 따른다 — 보통이 가장 싸다.
    #[test]
    fn salvage_follows_rarity_and_tiers() {
        let base = Base::Weapon(weapons::Id::Sword);
        let plain = Item::plain(base).salvage();
        let sturdy = roll(base, Rarity::Sturdy, 1, 1).salvage();
        let rare_low = roll(base, Rarity::Rare, 1, 1).salvage();
        assert!(plain < sturdy && sturdy < rare_low);
        assert_eq!(plain, SALVAGE[0]);
        assert_eq!(rare_low, SALVAGE[2] + 2 * SALVAGE_PER_TIER);
    }
}
