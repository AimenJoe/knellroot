//! 남는 것(0064): 이어하기·기록 옮기기·장신구 얻기·사기·끼기·저장(0066 R3에서 옮김).
use super::*;

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
            self.down = at.down;
            self.rest_at = Some(at);
            if at.stage != self.stage || self.down > self.stage {
                self.enter_stage(at.stage);
            }
            self.run.resume(&mut self.world, at);
            self.cam = self.cam.renew(self.world.player.pos, &self.t);
            // 쉼터에서 잇는 판은 깨어나는 장면이 아니다(0171 — 그 장면은 시작 자리의 것).
            self.wake = None;
        }
    }

    /// 쓰러졌는데 이 지역에서 쉰 적이 없다(0118): 이 판에 다른 곳에서 쉬었으면 그 지역·화로에서 다시 선다. 쉰 적이 없으면 거짓(끝).
    pub(super) fn respawn_elsewhere(&mut self) -> bool {
        let Some(at) = self.rest_at else {
            return false;
        };
        if at.stage != self.stage {
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

    /// 판에서 기록할 것을 옮기고(쉰 쉼터·끝낸 판), 바뀌었으면 쓴다.
    pub(super) fn sync_progress(&mut self, out: crate::stage::StageOut) {
        // 마지막으로 쉰 곳(0118): 이 지역에서 쉬었으면 그곳, 아니면 이 판에 앞서 쉰 곳 그대로(지역을 바꿔도 남는다).
        if let Some(at) = self.run.run_at() {
            self.rest_at = Some(crate::save::RunAt {
                stage: self.stage,
                down: self.down,
                ..at
            });
        }
        if out == crate::stage::StageOut::CoreTaken {
            self.down = self.down.max(self.stage + 1);
        }
        if let Some(at) = self.rest_at.as_mut() {
            at.down = self.down;
        }
        self.progress.run = self.rest_at;
        // 상자(0064 M3 → 0117): 든 장신구·무기를 얻는다(이미 가졌으면 잔불).
        // 상자(0064 M3 → 0131): 열면 소리만 — 얻는 것은 바닥에 떨어진 것을 주울 때. 기록도 그때(줍지 않고 떠나면 다음에 다시 닫혀 있다).
        if self.run.take_opened().is_some() {
            self.snd(crate::sfx::Id::Chest);
        }
        if let Some(i) = self.run.take_looted() {
            use crate::stage::Loot;
            self.snd(crate::sfx::Id::Ember);
            self.progress
                .chests
                .push(self.stage as u32 * crate::stage::CHESTS_PER_STAGE + i as u32);
            match self.run.def.chest_loot.get(i).copied() {
                Some(Loot::Charm(id)) if !self.progress.owned.contains(&id) => self.gain_charm(id),
                Some(Loot::Weapon(w)) if !self.progress.owns_weapon(w) => self.gain_weapon(w),
                _ => {
                    self.world.embers += crate::stage::CHEST_EMBERS;
                    let n = crate::stage::CHEST_EMBERS.to_string();
                    self.toast = Some((fill(s().toast_embers, &[&n]), TOAST_TIME));
                }
            }
        }
        // 도감(0130): 처음 본 것을 적는다(기록은 정렬해 둔다 — 같은 기록이면 같은 글).
        let seen = self.run.take_codex();
        if !seen.is_empty() {
            for k in seen {
                if !self.progress.codex.contains(&k) {
                    self.progress.codex.push(k);
                }
            }
            self.progress.codex.sort();
        }
        // 주민(0122): 이 판에 처음 본 주민은 몇째 판에 봤는지 기록(재방문 규칙).
        for k in self.run.take_residents_seen() {
            let id = crate::residents::RESIDENTS[self.run.residents[k].0].id;
            if !self.progress.residents_seen.iter().any(|(s, _)| s == id) {
                self.progress
                    .residents_seen
                    .push((id.to_string(), self.progress.runs));
            }
        }
        // 숨은 물건(0105): 기록에 이름을 넣고 능력치에 — 등잔이면 지금 한 모금도 채운다.
        if let Some((kind, id)) = self.run.take_picked() {
            self.snd(crate::sfx::Id::Chest);
            // 덧옷(0176): 아무것도 안 걸쳤으면 바로 걸친다(장신구가 빈 칸에 끼워지는 것과 같은 꼴).
            let garb = crate::garb::of_find(&id);
            if !self.progress.finds.contains(&id) {
                self.progress.finds.push(id);
                self.progress.finds.sort();
            }
            let put_on = garb.is_some() && self.progress.garb.is_none();
            if put_on {
                self.progress.garb = garb;
            }
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
            self.toast = Some((text.to_string(), TOAST_TIME));
        }
        // 엘리트(0112): 처음 쓰러뜨린 것을 기록에(덤 잔불은 스테이지가 이미 흩었다).
        // 문 장치(0113): 처음 연 문을 기록에(판을 넘어 열려 있다).
        if self.run.take_gate_new() {
            let id = self.run.def.find_prefix.to_string();
            if !self.progress.gates.contains(&id) {
                self.progress.gates.push(id);
                self.progress.gates.sort();
            }
            self.snd(crate::sfx::Id::Chest);
        }
        let new = self.run.take_elites_new();
        if !new.is_empty() {
            let n = (crate::tuning::ELITE_FIRST_EMBERS * new.len() as u32).to_string();
            for id in new {
                if !self.progress.elites.contains(&id) {
                    self.progress.elites.push(id);
                }
            }
            self.progress.elites.sort();
            self.toast = Some((fill(s().toast_elite, &[&n]), TOAST_TIME));
        }
        let core = matches!(
            out,
            crate::stage::StageOut::Cleared | crate::stage::StageOut::CoreTaken
        );
        if out == crate::stage::StageOut::Cleared {
            self.progress.run = None;
            self.rest_at = None;
        }
        // 둘째 지역 문지기(깨진 종지기, 0125): 핵은 따로 셋까지 — 기술 점수는 같이 센다(0161).
        if core && self.stage == 1 {
            let s = s();
            let first = self.progress.bell_cores == 0;
            if self.progress.bell_cores < crate::progression::CORE_CAP {
                self.progress.bell_cores += 1;
                let (n, m) = (
                    self.progress.bell_cores.to_string(),
                    crate::progression::CORE_CAP.to_string(),
                );
                self.toast = Some((fill(s.toast_core, &[&n, &m]), TOAST_TIME));
            } else {
                self.toast = Some((s.toast_core_full.to_string(), TOAST_TIME));
            }
            if first {
                self.run.card = Some((|s| s.core_title, crate::stage::CARD_TIME));
                if let Some(id) = self.run.def.boss_charm {
                    if !self.progress.owned.contains(&id) {
                        self.gain_charm(id);
                    }
                }
            }
        }
        // 첫 지역 문지기의 핵(0105).
        if core && self.stage == 0 {
            // 핵(0105 → 0161): 이길 때마다 기술 점수 1, 문지기마다 셋까지. 처음 이긴 판은 "핵이 몸에 스민다"(lore CANON — 핵이 다가왔다).
            let s = s();
            if self.progress.cores < crate::progression::CORE_CAP {
                self.progress.cores += 1;
                let (n, m) = (
                    self.progress.cores.to_string(),
                    crate::progression::CORE_CAP.to_string(),
                );
                self.toast = Some((fill(s.toast_core, &[&n, &m]), TOAST_TIME));
            } else {
                self.toast = Some((s.toast_core_full.to_string(), TOAST_TIME));
            }
            if self.progress.clears == 0 {
                self.run.card = Some((|s| s.core_title, crate::stage::CARD_TIME));
            }
            // 문지기를 처음 이기면 장신구 하나.
            if self.progress.clears == 0 {
                if let Some(id) = self.run.def.boss_charm {
                    if !self.progress.owned.contains(&id) {
                        self.gain_charm(id);
                    }
                }
            }
            self.progress.clears += 1;
        }
        self.progress.embers = self.world.embers;
        self.persist();
    }

    /// 장신구를 얻는다: 가진 것에 넣고, 빈 칸이 있으면 바로 끼운다(얻자마자 효과가 보이게). 알림.
    pub(super) fn gain_charm(&mut self, id: crate::charms::Id) {
        self.progress.owned.push(id);
        self.progress.owned.sort();
        if let Some(slot) = self.progress.worn.iter_mut().find(|w| w.is_none()) {
            *slot = Some(id);
            self.world.apply_stats(self.progress.stats(), false);
        }
        let name = tr(crate::charms::def(id).name);
        self.toast = Some((fill(s().toast_charm, &[name]), TOAST_TIME));
    }

    /// 무기를 얻는다(0117): 가진 것에 넣는다 — 드는 것은 장비 창에서(싸우는 손을 마음대로 바꾸지 않는다). 알림.
    pub(super) fn gain_weapon(&mut self, w: weapons::Id) {
        if !self.progress.weapons.contains(&w) {
            self.progress.weapons.push(w);
        }
        self.toast = Some((fill(s().toast_weapon, &[weapon_name(w)]), TOAST_TIME));
    }

    /// 화로 가게에서 산다(0064 M4): 잔불이 모자라면 알림만.
    pub(super) fn buy(&mut self, (ware, price): (crate::growth::Ware, u32)) {
        let s = s();
        if self.world.embers < price {
            self.toast = Some((s.shop_poor.to_string(), 1.6));
            self.snd(crate::sfx::Id::Deny);
            return;
        }
        self.snd(crate::sfx::Id::Buy);
        self.world.embers -= price;
        self.progress.embers = self.world.embers;
        match ware {
            crate::growth::Ware::Charm(id) => self.gain_charm(id),
            crate::growth::Ware::Oil => {
                self.progress.growth.oil = true;
                // 쉼터에서 막 쉬었으니 늘어난 등잔도 가득.
                self.world.apply_stats(self.progress.stats(), true);
                self.toast = Some((fill(s.toast_bought, &[s.shop_oil]), TOAST_TIME));
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
        // 개발 판(0075, nail 0269 B3의 꼴): 메모리의 기록만 바뀌고 슬롯은 그대로 — 슬롯을 다시 고르면 슬롯 것을 읽는다.
        #[cfg(feature = "dev")]
        if self.dev_run {
            return;
        }
        if self.progress != self.saved {
            let ok = crate::save::save(self.slot, &self.progress);
            self.slots[self.slot] = self.progress.summary();
            // 실패가 시작될 때 한 번만 알린다(저장할 때마다 알리면 화면을 덮는다).
            if !ok && !self.save_failed {
                warn!("save failed");
                self.toast = Some((s().save_failed.to_string(), TOAST_TIME));
            }
            self.save_failed = !ok;
            self.saved = self.progress.clone();
        }
    }
}
