//! 남는 것(0064): 이어하기·기록 옮기기·장신구 얻기·사기·끼기·저장(0066 R3에서 옮김).
use super::*;

/// 아이템의 빛(등급 — 바닥의 빛기둥·떨어질 때와 주울 때의 이펙트가 같은 빛).
pub(super) fn glow_of(item: &crate::items::Item) -> Color {
    let [r, g, b] = item.rarity.glow();
    Color::new(r, g, b, 1.0)
}

impl App {
    /// 이어하기(0064 M2): 새 판을 세우고, 기록에 쉼터가 있으면 거기서.
    pub(super) fn continue_run(&mut self) {
        // 쉰 곳이 있으면 그 판을 잇는다(판 수 그대로), 없으면 새 판.
        if self.progress.run.is_some() {
            self.build_run();
        } else {
            self.restart();
        }
        if let Some(at) = self.progress.run {
            // 지역 잇기(0118): 쉰 곳의 지역에서, 이 판에 이긴 문지기는 그대로.
            self.this_run.down = at.down;
            self.this_run.fields_down = at.fields;
            self.this_run.rest_at = Some(at);
            if at.stage != self.this_run.stage
                || self.this_run.down > self.this_run.stage
                || self.this_run.fields_down != 0
            {
                self.enter_stage(at.stage);
            }
            self.run.resume(&mut self.world, at);
            self.cam = self.cam.renew(self.world.player.pos, &self.t);
            // 쉼터에서 잇는 판은 깨어나는 장면이 아니다(0171 — 그 장면은 시작 자리의 것).
            self.end_wake();
        }
    }

    /// 쓰러졌는데 이 지역에서 쉰 적이 없다(0118): 이 판에 다른 곳에서 쉬었으면 그 지역·화로에서 다시 선다. 쉰 적이 없으면 거짓(끝).
    pub(super) fn respawn_elsewhere(&mut self) -> bool {
        let Some(at) = self.this_run.rest_at else {
            return false;
        };
        if at.stage != self.this_run.stage {
            self.enter_stage(at.stage);
        }
        self.run.resume(&mut self.world, at);
        self.run.card = Some((|s| s.respawned, crate::stage::CARD_TIME));
        self.cam = self.cam.renew(self.world.player.pos, &self.t);
        true
    }

    /// 지역 사이 전환이 지역을 바꿀 때(0118): 다음(내려감)·앞(올라감) 지역을 세우고 바닥 길·오르는 길 곁에 선다.
    pub(super) fn cross_to(&mut self, to: usize, up: bool) {
        self.enter_stage(to);
        if up {
            self.run.arrive_from_below(&mut self.world);
        } else {
            self.run.arrive_from_above(&mut self.world);
        }
        self.cam = self.cam.renew(self.world.player.pos, &self.t);
    }

    /// 서막 숲에서 성소로(0262 "성소에 들다"의 다 어두운 순간): 성소를 세우고 입구 폐허의 남쪽 가장자리에 선다. 숲을 지났다고 적는다
    /// (개발 창의 판은 기록에 쓰지 않는다 — `persist`가 거른다).
    pub(super) fn arrive_sanctuary(&mut self) {
        self.enter_stage(FIRST_STAGE);
        self.run.arrive_from_prologue(&mut self.world);
        self.cam = self.cam.renew(self.world.player.pos, &self.t);
        self.progress.prologue = true;
        self.persist();
    }

    /// 판에서 기록할 것을 옮기고(쉰 쉼터·끝낸 판), 바뀌었으면 쓴다.
    pub(super) fn sync_progress(&mut self, out: crate::stage::StageOut) {
        self.note_rest(out);
        self.on_chests();
        self.on_item_taken();
        self.on_seen();
        self.on_find();
        self.on_gates_and_elites();
        self.on_drops();
        self.on_core(out);
        self.progress.embers = self.world.embers();
        self.journey_watch();
        self.note_ground();
        self.persist();
    }

    /// 쉰 곳과 이 판에 이긴 문지기 수.
    fn note_rest(&mut self, out: crate::stage::StageOut) {
        // 마지막으로 쉰 곳(0118): 이 지역에서 쉬었으면 그곳, 아니면 이 판에 앞서 쉰 곳 그대로(지역을 바꿔도 남는다).
        if let Some(at) = self.run.run_at() {
            self.this_run.rest_at = Some(crate::save::RunAt {
                stage: self.this_run.stage,
                down: self.this_run.down,
                ..at
            });
        }
        if out == crate::stage::StageOut::CoreTaken {
            self.this_run.down = self.this_run.down.max(self.this_run.stage + 1);
        }
        if let Some(at) = self.this_run.rest_at.as_mut() {
            at.down = self.this_run.down;
            at.fields = self.this_run.fields_down;
        }
        self.progress.run = self.this_run.rest_at;
    }

    /// 상자를 열었다 · 바닥의 것을 주웠다(상자의 것).
    fn on_chests(&mut self) {
        // 상자(0064 M3 → 0117): 든 장신구·무기를 얻는다(이미 가졌으면 잔불).
        // 상자(0064 M3 → 0131): 열면 소리만 — 얻는 것은 바닥에 떨어진 것을 주울 때. 기록도 그때(줍지 않고 떠나면 다음에 다시 닫혀 있다).
        if self.run.take_opened().is_some() {
            self.snd(crate::sfx::Id::Chest);
            self.gesture = Some(("pickup", 0.0));
        }
        if let Some(i) = self.run.take_looted() {
            use crate::stage::Loot;
            self.gesture = Some(("pickup", 0.0));
            self.snd(crate::sfx::Id::Ember);
            // 깨어난 자리 곁의 검(0220): 얻고 바로 든다(맨손이었으니). 상자 기록은 없다.
            if i == crate::stage::START_SWORD {
                self.gain_weapon(weapons::Id::Sword, None);
                if self.weapon == weapons::Id::Fists {
                    self.wield(weapons::Id::Sword);
                }
            } else {
                self.progress
                    .chests
                    .push(self.this_run.stage as u32 * crate::stage::CHESTS_PER_STAGE + i as u32);
                match self.run.def.chest_loot.get(i).copied() {
                    // 상자의 것은 등급·옵션을 굴린다(0223 V4c — 잡몹·상자의 급).
                    Some(Loot::Charm(id)) if !self.progress.owned.contains(&id) => {
                        self.gain_charm(id, Some((0, self.loot_seed(1, i))))
                    }
                    Some(Loot::Weapon(w)) if !self.progress.owns_weapon(w) => {
                        self.gain_weapon(w, Some((0, self.loot_seed(1, i))))
                    }
                    // 숫돌(0274 ⑦): 숨은 물건의 숫돌과 같은 길로 기록에 — 상자는 한 번만 열리지만 같은 이름이 있으면 잔불.
                    Some(Loot::Stone)
                        if !self.progress.finds.contains(&crate::stage::chest_stone_id(
                            self.run.def.find_prefix,
                            i,
                        )) =>
                    {
                        let id = crate::stage::chest_stone_id(self.run.def.find_prefix, i);
                        self.progress.note_find(id);
                        self.notice.toast = Some((s().toast_find_stone.to_string(), TOAST_TIME));
                    }
                    // 여벌(0274 ⑦): 가진 바탕의 고급(`Sturdy` — 옵션 한 줄) — 주운 것이니 바로 가방에(바닥의 여벌을 주울 때와 같은 길).
                    // 가진 것이 없으면(맨손뿐) 잔불.
                    Some(Loot::Spare) => {
                        let rare = Some(crate::items::Rarity::Sturdy);
                        match self.roll_spare_of(rare, 0, self.loot_seed(1, i)) {
                            Some(item) => self.take_item(item),
                            None => self.chest_embers(),
                        }
                    }
                    _ => self.chest_embers(),
                }
            }
        }
    }

    /// 상자의 것을 이미 가졌다(또는 든 것이 없다): 잔불 `CHEST_EMBERS`.
    fn chest_embers(&mut self) {
        self.world.earn(crate::stage::CHEST_EMBERS);
        let n = crate::stage::CHEST_EMBERS.to_string();
        self.notice.toast = Some((fill(s().toast_embers, &[&n]), TOAST_TIME));
    }

    /// 떨어진 아이템을 주웠다.
    fn on_item_taken(&mut self) {
        // 떨어진 아이템(0226): 주워야 가방에 든다 — 가득이면 스테이지가 줍게 두지 않는다(`tell_bag`).
        if let Some((item, at)) = self.run.take_item() {
            self.gesture = Some(("pickup", 0.0));
            self.snd(crate::sfx::Id::Ember);
            self.fx.loot_take(at, glow_of(&item));
            self.take_item(item);
        }
    }

    /// 처음 본 것(도감·주민).
    fn on_seen(&mut self) {
        // 도감(0130): 처음 본 것을 적는다(기록은 정렬해 둔다 — 같은 기록이면 같은 글).
        self.progress.note_codex(self.run.take_codex());
        // 주민(0122): 이 판에 처음 본 주민은 몇째 판에 봤는지 기록(재방문 규칙).
        for k in self.run.take_residents_seen() {
            let id = crate::residents::RESIDENTS[self.run.folk.list[k].0].id;
            self.progress.note_resident_seen(id);
        }
    }

    /// 숨은 물건을 주웠다.
    fn on_find(&mut self) {
        // 숨은 물건(0105): 기록에 이름을 넣고 능력치에 — 등잔이면 지금 한 모금도 채운다.
        if let Some((kind, id)) = self.run.take_picked() {
            self.snd(crate::sfx::Id::Chest);
            self.gesture = Some(("pickup", 0.0));
            // 덧옷(0176): 아무것도 안 걸쳤으면 바로 걸친다(장신구가 빈 칸에 끼워지는 것과 같은 꼴).
            let crate::progress_notes::Found { garb, put_on } = self.progress.note_find(id);
            self.world.apply_stats(self.progress.stats(), false);
            let s = s();
            let named;
            let text = match kind {
                crate::stage::Find::Lamp => {
                    let pl = &mut self.world.player;
                    pl.flask = (pl.flask + 1).min(pl.flask_max);
                    s.toast_find_lamp
                }
                crate::stage::Find::Sip => s.toast_find_sip,
                crate::stage::Find::Shard => s.toast_find_shard,
                crate::stage::Find::Stone => s.toast_find_stone,
                crate::stage::Find::Garb => {
                    let name = garb.map_or("", |g| tr(crate::garb::def(g).name));
                    let line = if put_on {
                        s.toast_find_garb
                    } else {
                        s.toast_find_garb_kept
                    };
                    named = fill(line, &[name]);
                    &named
                }
            };
            self.notice.toast = Some((text.to_string(), TOAST_TIME));
        }
    }

    /// 장치를 당겼다 · 문을 처음 열었다 · 엘리트를 처음 쓰러뜨렸다.
    fn on_gates_and_elites(&mut self) {
        // 엘리트(0112): 처음 쓰러뜨린 것을 기록에(덤 잔불은 스테이지가 이미 흩었다).
        // 문 장치(0113): 처음 연 문을 기록에(판을 넘어 열려 있다).
        // 장치 당기기 몸짓(0197).
        if self.run.take_pulled() {
            self.gesture = Some(("pull", 0.0));
        }
        if self.run.take_gate_new() {
            self.progress
                .note_gate(self.run.def.find_prefix.to_string());
            self.snd(crate::sfx::Id::Chest);
        }
        let new = self.run.take_elites_new();
        if !new.is_empty() {
            let n = (crate::tuning::ELITE_FIRST_EMBERS * new.len() as u32).to_string();
            self.progress.note_elites(new);
            self.notice.toast = Some((fill(s().toast_elite, &[&n]), TOAST_TIME));
        }
    }

    /// 엘리트·문지기가 쓰러진 자리에 떨어지는 것.
    fn on_drops(&mut self) {
        // 엘리트의 여벌(0223 V4c — "엘리트 확정 하나"): 쓰러진 자리에 떨어진다(0226 — 주워야 얻는다).
        // 0244(사장 설문 k16 "게임내 강제도 생각해봐야함" → "첫 처치만 확정"): 기록에서 처음 쓰러뜨린 엘리트만 확정,
        // 그 뒤로는 `ELITE_REPEAT_SPARE` 확률 — 새 판마다 다시 서는 엘리트를 되풀이해 가방을 채우는 길을 느리게.
        for (k, at, first) in self.run.take_elites_down() {
            let seed = self.loot_seed(3, k);
            if crate::items::elite_drops(first, seed) {
                self.drop_spare(1, seed, at);
            }
        }
        // 문지기도(쓰러질 때마다 — 더 드문 쪽으로): 쓰러진 자리 곁에 떨어진다(0226 — 사장 "아이템은 바로 획득하지 마, 바닥에
        // 떨어지고 비주얼적인 효과가 있어야 해"). 핵은 쓰러진 자리에 남으니 그 옆으로 비킨다. 아직 없는 문지기의 장신구도
        // 여기서 떨어진다(가졌거나 이미 바닥에 놓여 있으면 아니다 — 줍지 않고 두면 그 자리에 남는다).
        if let Some(at) = self.run.take_boss_down() {
            self.drop_spare_from(2, self.loot_seed(4, 0), at, at + super::bag::BESIDE);
            if let Some(id) = self.run.def.boss_charm {
                let base = crate::items::Base::Charm(id);
                let lying = self.run.item_drops.iter().any(|d| d.item.base == base);
                if !self.progress.owned.contains(&id) && !lying {
                    let seed = self.loot_seed(2, 0);
                    let item = crate::items::roll(
                        base,
                        crate::items::roll_rarity(2, seed),
                        crate::items::tier_cap(self.this_run.stage),
                        seed,
                    );
                    self.lay_drop(at, at - super::bag::BESIDE, item);
                }
            }
        }
        self.on_field_down();
    }

    /// 필드 보스가 쓰러졌다(0270 D): 그 판에는 다시 서지 않는다. 처음이면 기록에 적고 **가장 높은 등급의 여벌 하나를 확정**으로 +
    /// 그 보스의 장신구(아직 없고 바닥에도 없으면), 그 뒤로는 문지기의 급으로 굴린 여벌 하나 — 모두 쓰러진 자리 곁의 바닥에(0226:
    /// 주워야 든다, 가방이 가득이면 남는다). 처음의 덤 잔불은 스테이지가 이미 흩었다.
    fn on_field_down(&mut self) {
        let Some((at, first)) = self.run.take_field_down() else {
            return;
        };
        self.this_run.fields_down |= 1 << self.this_run.stage;
        let Some(field) = &self.run.def.field else {
            return;
        };
        let seed = self.loot_seed(5, 0);
        let loot = crate::items::field_loot(first);
        if first {
            self.progress.note_field(self.run.def.find_prefix);
            let n = crate::tuning::FIELD_FIRST_EMBERS.to_string();
            self.notice.toast = Some((fill(s().toast_field, &[&n]), TOAST_TIME));
        }
        if let Some(item) = self.roll_spare_of(loot.rarity, loot.luck, seed) {
            self.lay_drop(at, at + super::bag::BESIDE, item);
        }
        if let Some(id) = field.charm.filter(|_| loot.charm) {
            let base = crate::items::Base::Charm(id);
            let lying = self.run.item_drops.iter().any(|d| d.item.base == base);
            if !self.progress.owned.contains(&id) && !lying {
                let seed = self.loot_seed(6, 0);
                let item = crate::items::roll(
                    base,
                    crate::items::roll_rarity(loot.luck, seed),
                    crate::items::tier_cap(self.this_run.stage),
                    seed,
                );
                self.lay_drop(at, at - super::bag::BESIDE, item);
            }
        }
    }

    /// 판을 끝냈다 · 핵을 쥐었다.
    fn on_core(&mut self, out: crate::stage::StageOut) {
        let core = matches!(
            out,
            crate::stage::StageOut::Cleared | crate::stage::StageOut::CoreTaken
        );
        if out == crate::stage::StageOut::Cleared {
            self.progress.run = None;
            self.this_run.rest_at = None;
        }
        // 문지기의 핵(0105 · 0125 · 0185 · 0208 → 0161): 이길 때마다 기술 점수 1, 문지기마다 따로 셋까지. 처음 이긴 판은
        // "핵이 몸에 스민다"(lore CANON — 핵이 다가왔다). 셈은 `Progress::take_core`(0251 R7).
        if core {
            let s = s();
            let taken = self.progress.take_core(self.this_run.stage);
            self.notice.toast = Some(match taken.count {
                Some(n) => {
                    let (n, m) = (n.to_string(), crate::progression::CORE_CAP.to_string());
                    (fill(s.toast_core, &[&n, &m]), TOAST_TIME)
                }
                None => (s.toast_core_full.to_string(), TOAST_TIME),
            });
            if taken.first {
                self.run.card = Some((|s| s.core_title, crate::stage::CARD_TIME));
            }
        }
    }

    /// 바닥에 둔 아이템을 기록에 옮긴다(0226): 이 지역의 것을 지금 바닥에 있는 것들로 — 게임을 꺼도, 지역을 떠났다 와도
    /// 그 자리에 있다(`lay_ground`). 개발 창의 판은 적지 않는다.
    pub(super) fn note_ground(&mut self) {
        // 서막 숲(0262)은 한 번 지나는 곳 — 거기 내려놓은 것은 기록하지 않는다(숲의 자리 번호는 지역이 늘면 뒤로 밀린다).
        if self.dev_run_now() || self.run.def.prologue {
            return;
        }
        let drops = self
            .run
            .item_drops
            .iter()
            .map(|d| (d.pos.to_array(), d.item));
        self.progress.ground.set(self.this_run.stage, drops);
    }

    /// 지역을 세운 뒤 기록의 바닥 것들을 되놓는다(0226).
    pub(super) fn lay_ground(&mut self) {
        if self.dev_run_now() {
            return;
        }
        let here: Vec<_> = self.progress.ground.of(self.this_run.stage).collect();
        for (p, item) in here {
            self.run.lay_item(Vec3::from_array(p), item);
        }
    }

    /// 굴린 아이템을 가방에 넣고 그 이름("단단한 낡은 검")을 돌려준다(0223 V4c). `roll` = (떨군 것의 급, 씨앗) — 없으면 보통 것.
    /// 세기의 상한은 지금 지역(`items::tier_cap`).
    fn bag_in(&mut self, base: crate::items::Base, roll: Option<(u8, u64)>, name: &str) -> String {
        use crate::items;
        let item = match roll {
            Some((luck, seed)) => items::roll(
                base,
                items::roll_rarity(luck, seed),
                items::tier_cap(self.this_run.stage),
                seed,
            ),
            None => items::Item::plain(base),
        };
        self.progress.bag.add(item);
        crate::i18n::item_label(item.rarity, name)
    }

    /// 이 판의 떨굼에 쓰는 씨앗: 지역 · 자리 · 몇째 판 — 같은 기록의 같은 자리라도 판이 다르면 다른 것이 나온다.
    pub(super) fn loot_seed(&self, kind: u64, i: usize) -> u64 {
        // 가방의 다음 번호도 섞는다: 지역을 오가거나 이어하면 그 판의 엘리트가 다시 서는데(판 수는 그대로), 같은 씨앗이면
        // 다시 쓰러뜨릴 때마다 똑같은 아이템이 나온다.
        (kind << 48)
            ^ ((self.this_run.stage as u64) << 32)
            ^ ((self.progress.bag.serial() as u64) << 20)
            // 바닥에 떨어뜨린 수도(0226): 줍지 않고 둔 채 다시 쓰러뜨리면 가방의 번호는 그대로다.
            ^ (((self.run.item_serial() & 0xff) as u64) << 40)
            ^ ((self.progress.runs as u64) << 12)
            ^ i as u64
    }

    /// 장신구를 얻는다: 가진 것에 넣고, 빈 칸이 있으면 바로 끼운다(얻자마자 효과가 보이게). 알림. `roll`이 있으면 등급·옵션을 굴린다.
    pub(super) fn gain_charm(&mut self, id: crate::charms::Id, roll: Option<(u8, u64)>) {
        let name = tr(crate::charms::def(id).name);
        let label = self.bag_in(crate::items::Base::Charm(id), roll, name);
        self.progress.owned.push(id);
        self.progress.owned.sort();
        // 제 종류의 칸(0224)이 비어 있으면.
        let slot = &mut self.progress.worn[id.kind().slot()];
        if slot.is_none() {
            *slot = Some(id);
        }
        self.progress.settle_items();
        self.world.apply_stats(self.progress.stats(), false);
        self.notice.toast = Some((fill(s().toast_charm, &[&label]), TOAST_TIME));
    }

    /// 무기를 얻는다(0117): 가진 것에 넣는다 — 드는 것은 장비 창에서(싸우는 손을 마음대로 바꾸지 않는다). 알림.
    pub(super) fn gain_weapon(&mut self, w: weapons::Id, roll: Option<(u8, u64)>) {
        let label = self.bag_in(crate::items::Base::Weapon(w), roll, weapon_name(w));
        if !self.progress.weapons.contains(&w) {
            self.progress.weapons.push(w);
        }
        self.progress.settle_items();
        self.notice.toast = Some((fill(s().toast_weapon, &[&label]), TOAST_TIME));
    }

    /// 여벌을 굴린다(0223 V4c — 엘리트·문지기가 떨군다): 이미 가진 바탕 가운데 하나의 **다른 것**(등급·옵션을 굴린다) — 더 나은 것을
    /// 찾는 재미. 가진 것이 없으면(맨손뿐) 아무것도 없다.
    fn roll_spare(&self, luck: u8, seed: u64) -> Option<crate::items::Item> {
        self.roll_spare_of(None, luck, seed)
    }

    /// 여벌을 굴리되 등급을 정해 줄 수 있다(0270 D — 필드 보스의 첫 떨굼은 굴리는 등급 가운데 가장 높은 것으로 확정).
    /// `rarity`가 없으면 떨군 것의 급(`luck`)으로 굴린다.
    fn roll_spare_of(
        &self,
        rarity: Option<crate::items::Rarity>,
        luck: u8,
        seed: u64,
    ) -> Option<crate::items::Item> {
        use crate::items::{self, Base};
        let p = &self.progress;
        let mut pool: Vec<Base> = p.weapons.iter().map(|w| Base::Weapon(*w)).collect();
        pool.extend(p.shields.iter().map(|s| Base::Shield(*s)));
        pool.extend(p.owned.iter().map(|c| Base::Charm(*c)));
        let mut worn: Vec<Base> = [p.weapon, p.sub]
            .into_iter()
            .flatten()
            .map(Base::Weapon)
            .collect();
        worn.extend(p.shield.map(Base::Shield));
        worn.extend(p.worn.iter().flatten().map(|c| Base::Charm(*c)));
        let base = items::pick_base(&pool, &worn, seed)?;
        Some(items::roll(
            base,
            rarity.unwrap_or_else(|| items::roll_rarity(luck, seed)),
            items::tier_cap(self.this_run.stage),
            seed,
        ))
    }

    /// 아이템의 이름("단단한 낡은 검").
    pub(super) fn item_label(item: &crate::items::Item) -> String {
        crate::i18n::item_label(item.rarity, Self::base_name(item))
    }

    /// 바탕의 이름(등급 없이 — 큰 창의 머리, 0275).
    pub(super) fn base_name(item: &crate::items::Item) -> &'static str {
        use crate::items::Base;
        match item.base {
            Base::Weapon(w) => weapon_name(w),
            Base::Shield(s) => tr(s.def().name),
            Base::Charm(c) => tr(crate::charms::def(c).name),
        }
    }

    /// 여벌이 바닥에 떨어진다(0226 — 사장 "가득이면 못 주워야지, 바닥에 떨어져 있어야지"): 주워야 가방에 든다.
    pub(super) fn drop_spare(&mut self, luck: u8, seed: u64, at: Vec3) {
        self.drop_spare_from(luck, seed, at, at);
    }

    /// `from`(쓰러진 자리)에서 `at`(옆으로 비킨 자리)으로 — `lay_drop`.
    fn drop_spare_from(&mut self, luck: u8, seed: u64, from: Vec3, at: Vec3) {
        if let Some(item) = self.roll_spare(luck, seed) {
            self.lay_drop(from, at, item);
        }
    }

    /// 아이템 하나를 그 자리의 땅에 떨군다(공중에서 쓰러졌어도 — 쟁반 뒤집기): 등급의 빛으로 번쩍이고 소리가 난다.
    /// `from` = 떨군 것의 자리(쓰러진 적) — `at`이 벽·소품 칸이면 그쪽으로 물러나 걸을 수 있는 땅에 놓는다(0226 — 줍지 못할 자리에
    /// 떨어지지 않게).
    fn lay_drop(&mut self, from: Vec3, at: Vec3, item: crate::items::Item) {
        let at = self.run.level.floor_toward(from, at);
        let at = vec3(at.x, self.world.ground_y(at), at.z);
        self.run.drop_item(at, item, false);
        self.fx.loot_drop(at, glow_of(&item), item.rarity.rank());
        self.snd(crate::sfx::Id::Chest);
    }

    /// 바닥의 아이템을 주웠다(0226): 가방에. 처음 얻는 바탕(문지기의 장신구)이면 가진 것에 넣고, 장신구는 제 종류의 칸이
    /// 비어 있으면 바로 낀다(얻자마자 효과가 보이게 — `gain_charm`과 같은 규칙).
    pub(super) fn take_item(&mut self, item: crate::items::Item) {
        use crate::items::Base;
        let label = Self::item_label(&item);
        self.progress.bag.add(item);
        match item.base {
            Base::Weapon(w) if !self.progress.weapons.contains(&w) => self.progress.weapons.push(w),
            Base::Shield(s) if !self.progress.shields.contains(&s) => self.progress.shields.push(s),
            Base::Charm(c) if !self.progress.owned.contains(&c) => {
                self.progress.owned.push(c);
                self.progress.owned.sort();
                let slot = &mut self.progress.worn[c.kind().slot()];
                if slot.is_none() {
                    *slot = Some(c);
                }
            }
            _ => {}
        }
        self.progress.settle_items();
        self.world.apply_stats(self.progress.stats(), false);
        // 같은 틱에 핵·엘리트의 알림도 뜬다 — 덮어쓰지 않고 그 뒤에 잇는다.
        self.notice.toast_wait.push(fill(s().toast_item, &[&label]));
    }

    /// 가방이 가득인지를 스테이지에 알린다(0226 — 틱마다, 스테이지의 틱 앞에): 가득이면 떨어진 아이템을 못 줍고, 상자의 것도
    /// 아직 없는 바탕(칸이 든다)이면 못 줍는다 — 이미 가진 것(잔불이 된다)은 줍는다.
    pub(super) fn tell_bag(&mut self) {
        use crate::stage::Loot;
        let full = self.progress.bag.full();
        self.run.bag_full = full;
        // 깨어난 자리의 검(0220)도 처음 얻는 바탕 — 칸이 든다.
        self.run.start_block = full && !self.progress.owns_weapon(weapons::Id::Sword);
        self.run.chest_block = if full {
            self.run
                .def
                .chest_loot
                .iter()
                .enumerate()
                .filter(|(i, l)| {
                    *i < 64
                        && match l {
                            Loot::Charm(id) => !self.progress.owned.contains(id),
                            Loot::Weapon(w) => !self.progress.owns_weapon(*w),
                            // 숫돌은 가방 밖 · 여벌은 늘 칸이 든다(0274 ⑦).
                            Loot::Stone => false,
                            Loot::Spare => true,
                        }
                })
                .fold(0u64, |m, (i, _)| m | 1 << i)
        } else {
            0
        };
    }

    /// 화로 가게에서 산다(0064 M4): 잔불이 모자라면 알림만.
    pub(super) fn buy(&mut self, (ware, price): (crate::growth::Ware, u32)) {
        let s = s();
        if self.world.embers() < price {
            self.notice.toast = Some((s.shop_poor.to_string(), 1.6));
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        // 가방이 가득이면(0226) 칸이 드는 것(장신구·방패)은 못 산다.
        if self.progress.bag.full() && crate::growth::needs_room(ware) {
            self.notice.toast = Some((s.bag_full.to_string(), 1.6));
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        self.snd(crate::sfx::Id::Buy);
        // 값은 위에서 견줬다.
        let paid = self.world.spend(price);
        debug_assert!(paid);
        self.progress.embers = self.world.embers();
        match ware {
            crate::growth::Ware::Charm(id) => self.gain_charm(id, None),
            crate::growth::Ware::Oil => {
                self.progress.growth.oil = true;
                // 쉼터에서 막 쉬었으니 늘어난 등잔도 가득.
                self.world.apply_stats(self.progress.stats(), true);
                self.notice.toast = Some((fill(s.toast_bought, &[s.shop_oil]), TOAST_TIME));
            }
            crate::growth::Ware::Shield(id) => {
                // 방패(0223): 사면 바로 든다(보조무기 칸이 비었으면 — 0224) — 한손 무기를 들고 있어야 쓰인다.
                if !self.progress.shields.contains(&id) {
                    // 가방에 먼저 넣는다 — 산 장신구처럼 "새 것"으로 보인다(0226).
                    self.bag_in(crate::items::Base::Shield(id), None, "");
                    self.progress.shields.push(id);
                }
                if self.progress.shield.is_none() && self.progress.sub.is_none() {
                    self.progress.shield = Some(id);
                }
                self.arm_world();
                self.notice.toast = Some((fill(s.toast_bought, &[tr(id.def().name)]), TOAST_TIME));
            }
        }
        self.persist();
    }

    /// 장비 창에서 칸을 바꾼다.
    pub(super) fn step_charm(&mut self, slot: usize, step: i32) {
        self.progress.worn[slot] =
            next_charm(&self.progress.owned, &self.progress.worn, slot, step);
        self.world.apply_stats(self.progress.stats(), false);
        self.persist();
    }

    /// 방패를 든다·내린다(0223 — 가진 것만). 방패는 보조무기 칸에 든다(0224) — 들면 그 칸의 무기는 내린다.
    pub(super) fn set_shield(&mut self, s: Option<crate::shield::Id>) {
        self.progress.shield = s.filter(|id| self.progress.shields.contains(id));
        if self.progress.shield.is_some() {
            self.progress.sub = None;
        }
        self.arm_world();
        self.persist();
    }

    /// 보조무기 칸 빨리 바꾸기(0223 → 0224): 비우기 → 맨손 → 가진 무기(표 순서, 주무기 빼고) → 가진 방패 → 비우기로 돈다.
    pub(super) fn step_sub(&mut self, step: i32) {
        #[derive(Clone, Copy, PartialEq)]
        enum Off {
            None,
            Arm(weapons::Id),
            Guard(crate::shield::Id),
        }
        let main = self.weapon;
        let mut opts = vec![Off::None];
        opts.extend(
            weapons::IDS
                .into_iter()
                .filter(|&w| w != main && (w == weapons::Id::Fists || self.arms().contains(&w)))
                .map(Off::Arm),
        );
        opts.extend(
            crate::shield::TABLE
                .iter()
                .filter(|d| self.progress.shields.contains(&d.id))
                .map(|d| Off::Guard(d.id)),
        );
        let now = match (self.progress.sub, self.progress.shield) {
            (Some(w), _) => Off::Arm(w),
            (None, Some(s)) => Off::Guard(s),
            (None, None) => Off::None,
        };
        let at = opts.iter().position(|o| *o == now).unwrap_or(0) as i32;
        match opts[(at + step).rem_euclid(opts.len() as i32) as usize] {
            Off::Guard(s) => self.set_shield(Some(s)),
            Off::Arm(w) => self.set_sub_weapon(Some(w)),
            Off::None => {
                self.set_shield(None);
                self.set_sub_weapon(None);
            }
        }
    }

    /// 덧옷 칸 빨리 바꾸기(0176): 벗기 → 가진 덧옷(표 순서) → 벗기로 돈다.
    pub(super) fn step_garb(&mut self, step: i32) {
        let mut opts: Vec<Option<crate::garb::Id>> = vec![None];
        opts.extend(self.garbs().into_iter().map(Some));
        let at = opts
            .iter()
            .position(|o| *o == self.progress.garb)
            .unwrap_or(0) as i32;
        self.progress.garb = opts[(at + step).rem_euclid(opts.len() as i32) as usize];
        self.world.apply_stats(self.progress.stats(), false);
        self.persist();
    }

    /// 바뀐 것이 있으면 쓴다. 못 쓰면(사생활 보호 모드 등) 다음에 또 바뀔 때까지 다시 시도하지 않는다.
    pub(super) fn persist(&mut self) {
        // 가방을 가진 것·낀 것에 맞춘 뒤에 쓴다(0223 V4b — 얻거나 바꿔 낀 뒤의 자리).
        self.progress.settle_items();
        // 개발 판(0075, nail 0269 B3의 꼴): 메모리의 기록만 바뀌고 슬롯은 그대로 — 슬롯을 다시 고르면 슬롯 것을 읽는다.
        #[cfg(feature = "dev")]
        if self.dev.run {
            return;
        }
        if self.progress != self.book.saved {
            let ok = crate::save::save(self.book.slot, &self.progress);
            self.book.slots[self.book.slot] = self.progress.summary();
            // 실패가 시작될 때 한 번만 알린다(저장할 때마다 알리면 화면을 덮는다).
            if !ok && !self.book.save_failed {
                warn!("save failed");
                self.notice.toast = Some((s().save_failed.to_string(), TOAST_TIME));
            }
            self.book.save_failed = !ok;
            self.book.saved = self.progress.clone();
        }
    }
}
