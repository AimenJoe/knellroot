//! 기록(0064 M2): 판을 넘어 남는 것 + 판 안의 자리(마지막으로 쉰 쉼터). 설정(`prefs`)과 따로.
//!
//! - 형식: `키=값` 줄(첫 게임 nail `profile.rs` — 키 이름은 바꾸지 않는다, 모르는 키는 지우지 않고 그대로 들고 다닌다,
//!   깨진 줄은 건너뛴다). 그래서 뒤에 칸을 더해도 옛 기록이 그대로 읽힌다.
//! - 자리(슬롯 셋, 0066 — KB `roguelite-save-slots-and-continue`): 웹은 localStorage `knell.save.0`·`.1`·`.2` + 마지막 슬롯
//!   `knell.slot`(web/index.html `knell_rec_*`, 칸 번호 인자), 데스크톱·Android는 설정과 같은 폴더의 `save0.txt`·`save1.txt`·
//!   `save2.txt`·`slot.txt`(임시 파일에 쓰고 이름 바꾸기 — 쓰다 꺼져도 옛 기록이 남는다). 슬롯 1 = 옛 한 칸 그대로(옮기는 코드 없음).
//! - 든 무기도 기록에(`weapon`, 0066 — 설정 칸에 두면 슬롯 사이에 샜다).
//! - 쓰는 때는 앱이 정한다: 바뀐 것이 있을 때만(`Progress` 비교).
use std::collections::BTreeMap;

/// 키 이름(바꾸지 않는다).
pub mod key {
    /// 형식 판(지금 1).
    pub const VERSION: &str = "v";
    /// 잔불(M1, 쓰러져도 잃지 않는다).
    pub const EMBERS: &str = "embers";
    /// 스테이지를 끝낸 횟수.
    pub const CLEARS: &str = "clears";
    /// 판 안의 자리: 마지막으로 쉰 쉼터 번호(없으면 판 밖 — 다음은 입구부터).
    pub const RUN_REST: &str = "run.rest";
    /// 그 쉼터에서 다시 서면 문이 열린 뒤부터인가(1) 입구부터인가(0) — 0032의 되돌아갈 단계.
    pub const RUN_GATE: &str = "run.gate";
    /// 장신구(M3): 가진 것("shell,ear")·낀 것(세 칸 "shell,,ear") — `charms.rs`의 키.
    pub const CHARMS_OWNED: &str = "charm.owned";
    pub const CHARMS_WORN: &str = "charm.worn";
    /// 연 상자 번호들("0,2") — 판을 넘어 남는다(한 번 연 상자는 다시 비어 있다).
    pub const CHESTS: &str = "chests";
    /// 성장(M4): 핵으로 고른 횟수(갈래마다), 가게에서 산 등잔 기름.
    pub const GROW_HP: &str = "grow.hp";
    pub const GROW_FLASK: &str = "grow.flask";
    pub const GROW_HEAL: &str = "grow.heal";
    pub const SHOP_OIL: &str = "shop.oil";
    /// 든 무기(0066: prefs 칸 21에서 옮김 — 슬롯 사이에 새지 않게).
    pub const WEAPON: &str = "weapon";
    /// 쌓인 플레이 시간(초, 0081 — 이어하기 목록의 "시간"). 개발 판·메뉴·타이틀에 있는 동안은 세지 않는다.
    pub const PLAY: &str = "play";
}

/// 글 기록(키 → 값). 모르는 키도 지키며 들고 다닌다.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Record {
    map: BTreeMap<String, String>,
}

impl Record {
    pub fn parse(text: &str) -> Record {
        let mut map = BTreeMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim();
                if !k.is_empty() {
                    map.insert(k.to_string(), v.trim().to_string());
                }
            }
        }
        Record { map }
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        for (k, v) in &self.map {
            s.push_str(k);
            s.push('=');
            s.push_str(v);
            s.push('\n');
        }
        s
    }

    pub fn get(&self, k: &str) -> Option<&str> {
        self.map.get(k).map(String::as_str)
    }

    pub fn num(&self, k: &str) -> Option<u32> {
        self.get(k)?.parse().ok()
    }

    pub fn set(&mut self, k: &str, v: impl ToString) {
        self.map.insert(k.to_string(), v.to_string());
    }

    pub fn remove(&mut self, k: &str) {
        self.map.remove(k);
    }
}

/// 판 안의 자리: 쉼터 번호와 문이 열린 뒤인지.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunAt {
    pub rest: usize,
    pub gate_open: bool,
}

/// 기록에서 게임이 쓰는 것(앱이 들고 있다가 바뀌면 쓴다).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Progress {
    pub embers: u32,
    pub clears: u32,
    pub run: Option<RunAt>,
    pub owned: Vec<crate::charms::Id>,
    pub worn: [Option<crate::charms::Id>; crate::charms::SLOTS],
    pub chests: Vec<u32>,
    pub growth: crate::growth::Growth,
    /// 든 무기(없으면 옛 prefs 칸의 값을 쓴다).
    pub weapon: Option<crate::weapons::Id>,
    /// 쌓인 플레이 시간(초, 0081).
    pub play: u32,
    /// 원래 기록(모르는 키를 지키려고 — 쓸 때 이 위에 덮는다).
    base: Record,
}

impl Progress {
    pub fn from_record(r: Record) -> Progress {
        let run = r.num(key::RUN_REST).map(|i| RunAt {
            rest: i as usize,
            gate_open: r.num(key::RUN_GATE) == Some(1),
        });
        let owned = crate::charms::decode_owned(r.get(key::CHARMS_OWNED).unwrap_or(""));
        let worn = crate::charms::decode_worn(r.get(key::CHARMS_WORN).unwrap_or(""), &owned);
        let mut chests: Vec<u32> = r
            .get(key::CHESTS)
            .unwrap_or("")
            .split(',')
            .filter_map(|c| c.trim().parse().ok())
            .collect();
        chests.sort();
        chests.dedup();
        let small = |k: &str| r.num(k).unwrap_or(0).min(u8::MAX as u32) as u8;
        let growth = crate::growth::Growth {
            hp: small(key::GROW_HP),
            flask: small(key::GROW_FLASK),
            heal: small(key::GROW_HEAL),
            oil: r.num(key::SHOP_OIL) == Some(1),
        };
        Progress {
            growth,
            weapon: r.get(key::WEAPON).and_then(crate::weapons::Id::by_key),
            play: r.num(key::PLAY).unwrap_or(0),
            embers: r.num(key::EMBERS).unwrap_or(0),
            clears: r.num(key::CLEARS).unwrap_or(0),
            run,
            owned,
            worn,
            chests,
            base: r,
        }
    }

    pub fn to_record(&self) -> Record {
        let mut r = self.base.clone();
        r.set(key::VERSION, 1);
        r.set(key::EMBERS, self.embers);
        r.set(key::CLEARS, self.clears);
        r.set(key::CHARMS_OWNED, crate::charms::encode_owned(&self.owned));
        r.set(key::CHARMS_WORN, crate::charms::encode_worn(&self.worn));
        let chests: Vec<String> = self.chests.iter().map(|c| c.to_string()).collect();
        r.set(key::CHESTS, chests.join(","));
        r.set(key::GROW_HP, self.growth.hp);
        r.set(key::GROW_FLASK, self.growth.flask);
        r.set(key::GROW_HEAL, self.growth.heal);
        r.set(key::SHOP_OIL, self.growth.oil as u8);
        r.set(key::PLAY, self.play);
        match self.weapon {
            Some(w) => r.set(key::WEAPON, w.key()),
            None => r.remove(key::WEAPON),
        }
        match self.run {
            Some(at) => {
                r.set(key::RUN_REST, at.rest);
                r.set(key::RUN_GATE, at.gate_open as u8);
            }
            None => {
                r.remove(key::RUN_REST);
                r.remove(key::RUN_GATE);
            }
        }
        r
    }

    /// 이어할 것이 있나(타이틀 "이어하기"): 판 밖의 것이 하나라도 쌓였거나 판 안의 자리가 있으면.
    pub fn has_any(&self) -> bool {
        self.run.is_some()
            || self.play > 0
            || self.embers > 0
            || self.clears > 0
            || !self.owned.is_empty()
            || self.growth != crate::growth::Growth::default()
    }

    /// 낀 장신구 + 성장으로 합친 능력치.
    pub fn stats(&self) -> crate::stats::Stats {
        let mut mods = crate::charms::mods(&self.worn);
        mods.push(self.growth.mods());
        crate::stats::Stats::compute(&mods)
    }
}

/// 기록 슬롯 수(0066, KB `roguelite-save-slots-and-continue`, nail 0078).
pub const SLOTS: usize = 3;
/// 저장 칸: 0..SLOTS = 슬롯 기록, `LAST` = 마지막으로 쓴 슬롯 번호.
const LAST: u32 = SLOTS as u32;

#[cfg(target_arch = "wasm32")]
mod store {
    extern "C" {
        fn knell_rec_len(cell: u32) -> u32;
        fn knell_rec_read(cell: u32, ptr: *mut u8, len: u32);
        fn knell_rec_write(cell: u32, ptr: *const u8, len: u32) -> u32;
    }

    pub fn load(cell: u32) -> String {
        // SAFETY: 칸의 길이를 돌려주는 JS 함수, 그다음 그 길이만큼 이 버퍼에 바이트를 쓰는 JS 함수(web/index.html).
        let n = unsafe { knell_rec_len(cell) } as usize;
        let mut buf = vec![0u8; n];
        if n > 0 {
            unsafe { knell_rec_read(cell, buf.as_mut_ptr(), n as u32) };
        }
        String::from_utf8(buf).unwrap_or_default()
    }

    pub fn save(cell: u32, text: &str) -> bool {
        // SAFETY: 이 바이트를 읽기만 하는 JS 함수(web/index.html). 길이 0이면 지운다.
        unsafe { knell_rec_write(cell, text.as_ptr(), text.len() as u32) == 1 }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod store {
    use std::path::PathBuf;

    /// 슬롯 1 = 옛 한 칸 `save0.txt` 그대로(옮기는 코드 없음), 마지막 슬롯 = `slot.txt`.
    fn path(cell: u32) -> Option<PathBuf> {
        let name = if cell == super::LAST {
            "slot.txt".to_string()
        } else {
            format!("save{cell}.txt")
        };
        crate::prefs::data_dir().map(|d| d.join(name))
    }

    pub fn load(cell: u32) -> String {
        path(cell)
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_default()
    }

    pub fn save(cell: u32, text: &str) -> bool {
        let Some(p) = path(cell) else { return false };
        if text.is_empty() {
            return std::fs::remove_file(&p).is_ok() || !p.exists();
        }
        let result = (|| {
            if let Some(d) = p.parent() {
                std::fs::create_dir_all(d)?;
            }
            let tmp = p.with_extension("tmp");
            std::fs::write(&tmp, text)?;
            std::fs::rename(&tmp, &p)
        })();
        result.is_ok()
    }
}

pub fn load(slot: usize) -> Progress {
    Progress::from_record(Record::parse(&store::load(slot as u32)))
}

/// 쓴다(실패하면 거짓 — 사생활 보호 모드·디스크 등).
pub fn save(slot: usize, p: &Progress) -> bool {
    store::save(slot as u32, &p.to_record().to_text())
}

/// 슬롯의 기록을 지운다(슬롯 창 "지우기"·"덮어쓰기").
pub fn wipe(slot: usize) -> bool {
    store::save(slot as u32, "")
}

/// 마지막으로 쓴 슬롯(없거나 깨졌으면 0 = 슬롯 1 — 옛 한 칸 기록이 있는 곳).
pub fn last_slot() -> usize {
    parse_slot(&store::load(LAST))
}

pub fn set_last_slot(slot: usize) -> bool {
    store::save(LAST, &slot.to_string())
}

fn parse_slot(text: &str) -> usize {
    text.trim()
        .parse::<usize>()
        .ok()
        .filter(|&s| s < SLOTS)
        .unwrap_or(0)
}

/// 슬롯 창 한 줄의 요약(기록에서 바로 계산되는 것, KB).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Summary {
    /// 장(챕터) 번호, 1부터 — 지금은 스테이지 하나라 늘 1(v0.4.0 구역 잇기 0069 T1에서 `run.stage`로).
    pub chapter: u32,
    /// 쌓인 플레이 시간(초).
    pub play: u32,
}

impl Progress {
    /// 기록이 있는 슬롯이면 요약(장·지역·시간, 사장 2026-11-07 — 그 밖은 뺌), 비었으면 없음.
    pub fn summary(&self) -> Option<Summary> {
        self.has_any().then_some(Summary {
            chapter: 1,
            play: self.play,
        })
    }
}

/// 플레이 시간 글 "h:mm"(초는 뺀다 — 슬롯 줄은 대강이면 된다).
pub fn play_text(secs: u32) -> String {
    format!("{}:{:02}", secs / 3600, secs / 60 % 60)
}

/// 슬롯 셋의 요약(타이틀 "이어하기"가 보이나, 슬롯 창의 줄).
pub fn summaries() -> [Option<Summary>; SLOTS] {
    std::array::from_fn(|i| load(i).summary())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_round_trips_keeps_unknown_keys_and_skips_broken_lines() {
        let text =
            "embers=12\nfuture.key=hello = world\n\nbroken line\n=nokey\n# comment\nclears=x\n";
        let r = Record::parse(text);
        assert_eq!(r.num("embers"), Some(12));
        assert_eq!(r.get("future.key"), Some("hello = world"));
        assert_eq!(r.num("clears"), None, "not a number");
        let again = Record::parse(&r.to_text());
        assert_eq!(again, r);
        // 게임이 쓰는 칸을 바꿔 써도 모르는 키는 남는다.
        let mut p = Progress::from_record(r);
        p.embers = 30;
        let out = p.to_record();
        assert_eq!(out.get("future.key"), Some("hello = world"));
        assert_eq!(out.num(key::EMBERS), Some(30));
        assert_eq!(out.num(key::VERSION), Some(1));
    }

    #[test]
    fn progress_keeps_the_rest_site_and_drops_it_when_the_run_ends() {
        let mut p = Progress::default();
        assert!(!p.has_any(), "nothing yet");
        p.run = Some(RunAt {
            rest: 1,
            gate_open: true,
        });
        let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
        assert_eq!(back.run, p.run);
        assert!(back.has_any());
        // 판이 끝나면 자리는 지우고 남는 것만.
        let mut done = back.clone();
        done.run = None;
        done.clears = 1;
        let r = done.to_record();
        assert_eq!(r.get(key::RUN_REST), None);
        assert_eq!(r.get(key::RUN_GATE), None);
        let back = Progress::from_record(r);
        assert_eq!(back.run, None);
        assert!(back.has_any(), "a clear stays");
    }

    #[test]
    fn charms_and_opened_chests_survive_the_record() {
        use crate::charms::Id;
        let mut p = Progress::default();
        p.owned = vec![Id::Shell, Id::Ear];
        p.worn = [None, Some(Id::Ear), None];
        p.chests = vec![0, 2];
        p.growth = crate::growth::Growth {
            hp: 2,
            flask: 1,
            heal: 0,
            oil: true,
        };
        let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
        assert_eq!(
            (back.owned.clone(), back.worn, back.chests.clone()),
            (p.owned, p.worn, p.chests)
        );
        assert!(back.has_any());
        assert_eq!(back.stats().parry, 0.06, "the worn charm counts");
        assert_eq!(back.growth, p.growth);
        let base = crate::stats::Stats::default();
        assert_eq!(back.stats().max_hp, base.max_hp + 2);
        assert_eq!(
            back.stats().flask,
            base.flask + 2,
            "a core pick and the oil"
        );
    }

    #[test]
    fn an_empty_or_garbage_store_reads_as_a_fresh_start() {
        assert_eq!(
            Progress::from_record(Record::parse("")),
            Progress::default()
        );
        let p = Progress::from_record(Record::parse("\u{0}\u{1}garbage"));
        assert_eq!((p.embers, p.clears, p.run), (0, 0, None));
        assert!(p.owned.is_empty() && p.chests.is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn the_native_store_writes_reads_and_wipes_the_file() {
        let dir = std::env::temp_dir().join(format!("knell-save-test-{}", std::process::id()));
        std::env::set_var("KNELL_SAVE_DIR", &dir);
        let mut p = Progress::default();
        p.embers = 7;
        p.run = Some(RunAt {
            rest: 0,
            gate_open: false,
        });
        assert!(save(0, &p));
        let got = load(0);
        assert_eq!((got.embers, got.run), (7, p.run));
        // 슬롯 1 = 옛 한 칸 파일 그대로(옮기는 코드 없음, 0066).
        assert!(dir.join("save0.txt").exists());
        // 슬롯마다 따로: 슬롯 3에 쓴 것이 슬롯 1·2에 새지 않는다.
        let other = Progress {
            clears: 2,
            ..Progress::default()
        };
        assert!(save(2, &other));
        assert!(dir.join("save2.txt").exists());
        assert_eq!(load(2).clears, 2);
        assert_eq!(load(0).clears, 0);
        assert_eq!(load(1).summary(), None, "untouched slot is empty");
        let sums = summaries();
        assert!(sums[0].is_some() && sums[1].is_none() && sums[2].is_some());
        // 마지막 슬롯 칸.
        assert_eq!(last_slot(), 0);
        assert!(set_last_slot(2));
        assert_eq!(last_slot(), 2);
        assert!(wipe(0));
        assert!(!load(0).has_any());
        assert_eq!(load(2).clears, 2, "wiping one slot keeps the others");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_last_slot_cell_falls_back_to_slot_one_and_weapons_round_trip() {
        assert_eq!(parse_slot(""), 0, "no cell yet = the old single record");
        assert_eq!(parse_slot("2"), 2);
        assert_eq!(parse_slot("7"), 0, "out of range");
        assert_eq!(parse_slot(" 1\n"), 1);
        // 가장 큰 값까지 왕복(KB: 범위를 자르는 코드는 새 값에서 조용히 틀린다).
        for w in crate::weapons::IDS {
            let p = Progress {
                weapon: Some(w),
                ..Progress::default()
            };
            let back = Progress::from_record(Record::parse(&p.to_record().to_text()));
            assert_eq!(back.weapon, Some(w));
        }
        // 무기만 바꾼 슬롯은 "기록 있음"이 아니다(빈 슬롯 그대로).
        let only_weapon = Progress {
            weapon: Some(crate::weapons::Id::Staff),
            ..Progress::default()
        };
        assert_eq!(only_weapon.summary(), None);
        let some = Progress {
            clears: 1,
            embers: 9,
            owned: vec![crate::charms::Id::Ear],
            ..Progress::default()
        };
        // 요약은 장 · 시간만(0081 — 지역은 장에서 정해진다), 완주·잔불·장신구 수는 뺐다.
        assert_eq!(
            some.summary(),
            Some(Summary {
                chapter: 1,
                play: 0
            })
        );
        // 플레이 시간은 기록에 남고(키 `play`), 시간만 있어도 "기록 있음"이다.
        let played = Progress {
            play: 3725,
            ..Progress::default()
        };
        let back = Progress::from_record(Record::parse(&played.to_record().to_text()));
        assert_eq!(back.play, 3725);
        assert_eq!(back.summary().map(|m| m.play), Some(3725));
        assert_eq!(play_text(3725), "1:02");
        assert_eq!(play_text(59), "0:00");
    }
}
