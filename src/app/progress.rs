//! 남는 것(0064): 이어하기·기록 옮기기·장신구 얻기·사기·끼기·저장(0066 R3에서 옮김).
use super::*;

impl App {
    /// 이어하기(0064 M2): 새 판을 세우고, 기록에 쉼터가 있으면 거기서.
    pub(super) fn continue_run(&mut self) {
        self.restart();
        if let Some(at) = self.progress.run {
            self.run.resume(&mut self.world, at);
            self.cam = Camera::new(self.world.player.pos, &self.t);
        }
    }

    /// 판에서 기록할 것을 옮기고(쉰 쉼터·끝낸 판), 바뀌었으면 쓴다.
    pub(super) fn sync_progress(&mut self, out: crate::stage::StageOut) {
        self.progress.run = self.run.run_at();
        // 상자(0064 M3): 든 장신구를 얻는다(이미 가졌으면 잔불).
        if let Some(i) = self.run.take_opened() {
            self.snd(crate::sfx::Id::Chest);
            self.progress.chests.push(i as u32);
            match self.run.def.chest_charms.get(i).copied() {
                Some(id) if !self.progress.owned.contains(&id) => self.gain_charm(id),
                _ => {
                    self.world.embers += crate::stage::CHEST_EMBERS;
                    let n = crate::stage::CHEST_EMBERS.to_string();
                    self.toast = Some((fill(s().toast_embers, &[&n]), TOAST_TIME));
                }
            }
        }
        // 숨은 물건(0105): 기록에 이름을 넣고 능력치에 — 등잔이면 지금 한 모금도 채운다.
        if let Some((kind, id)) = self.run.take_picked() {
            self.snd(crate::sfx::Id::Chest);
            if !self.progress.finds.contains(&id) {
                self.progress.finds.push(id);
                self.progress.finds.sort();
            }
            self.world.apply_stats(self.progress.stats(), false);
            let s = s();
            let text = match kind {
                crate::stage::Find::Lamp => {
                    let pl = &mut self.world.player;
                    pl.flask = (pl.flask + 1).min(pl.flask_max);
                    s.toast_find_lamp
                }
                crate::stage::Find::Sip => s.toast_find_sip,
            };
            self.toast = Some((text.to_string(), TOAST_TIME));
        }
        if out == crate::stage::StageOut::Cleared {
            self.progress.run = None;
            // 핵(0105): 이길 때마다 나무 점수 1, 문지기마다 셋까지. 처음 이긴 판은 "핵이 몸에 스민다"(lore CANON — 핵이 다가왔다).
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
