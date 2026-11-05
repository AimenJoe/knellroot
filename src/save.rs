//! 기록(0064 M2): 판을 넘어 남는 것 + 판 안의 자리(마지막으로 쉰 쉼터). 설정(`prefs`)과 따로.
//!
//! - 형식: `키=값` 줄(첫 게임 nail `profile.rs` — 키 이름은 바꾸지 않는다, 모르는 키는 지우지 않고 그대로 들고 다닌다,
//!   깨진 줄은 건너뛴다). 그래서 뒤에 칸을 더해도 옛 기록이 그대로 읽힌다.
//! - 자리: 웹은 localStorage 글 한 칸 `knell.save.0`(web/index.html `knell_save_*`), 데스크톱·Android는 설정과 같은 폴더의
//!   `save0.txt`(임시 파일에 쓰고 이름 바꾸기 — 쓰다 꺼져도 옛 기록이 남는다).
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
    /// 원래 기록(모르는 키를 지키려고 — 쓸 때 이 위에 덮는다).
    base: Record,
}

impl Progress {
    pub fn from_record(r: Record) -> Progress {
        let run = r.num(key::RUN_REST).map(|i| RunAt {
            rest: i as usize,
            gate_open: r.num(key::RUN_GATE) == Some(1),
        });
        Progress {
            embers: r.num(key::EMBERS).unwrap_or(0),
            clears: r.num(key::CLEARS).unwrap_or(0),
            run,
            base: r,
        }
    }

    pub fn to_record(&self) -> Record {
        let mut r = self.base.clone();
        r.set(key::VERSION, 1);
        r.set(key::EMBERS, self.embers);
        r.set(key::CLEARS, self.clears);
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
        self.run.is_some() || self.embers > 0 || self.clears > 0
    }
}

#[cfg(target_arch = "wasm32")]
mod store {
    extern "C" {
        fn knell_save_len() -> u32;
        fn knell_save_read(ptr: *mut u8, len: u32);
        fn knell_save_write(ptr: *const u8, len: u32) -> u32;
    }

    pub fn load() -> String {
        // SAFETY: 길이를 돌려주는 JS 함수, 그다음 그 길이만큼 이 버퍼에 바이트를 쓰는 JS 함수(web/index.html).
        let n = unsafe { knell_save_len() } as usize;
        let mut buf = vec![0u8; n];
        if n > 0 {
            unsafe { knell_save_read(buf.as_mut_ptr(), n as u32) };
        }
        String::from_utf8(buf).unwrap_or_default()
    }

    pub fn save(text: &str) -> bool {
        // SAFETY: 이 바이트를 읽기만 하는 JS 함수(web/index.html). 길이 0이면 지운다.
        unsafe { knell_save_write(text.as_ptr(), text.len() as u32) == 1 }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod store {
    use std::path::PathBuf;

    fn path() -> Option<PathBuf> {
        crate::prefs::data_dir().map(|d| d.join("save0.txt"))
    }

    pub fn load() -> String {
        path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_default()
    }

    pub fn save(text: &str) -> bool {
        let Some(p) = path() else { return false };
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

pub fn load() -> Progress {
    Progress::from_record(Record::parse(&store::load()))
}

/// 쓴다(실패하면 거짓 — 사생활 보호 모드·디스크 등).
pub fn save(p: &Progress) -> bool {
    store::save(&p.to_record().to_text())
}

/// 기록을 지운다(타이틀 "새로 하기").
pub fn wipe() -> bool {
    store::save("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_round_trips_keeps_unknown_keys_and_skips_broken_lines() {
        let text = "embers=12\nfuture.key=hello = world\n\nbroken line\n=nokey\n# comment\nclears=x\n";
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
        p.run = Some(RunAt { rest: 1, gate_open: true });
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
    fn an_empty_or_garbage_store_reads_as_a_fresh_start() {
        assert_eq!(Progress::from_record(Record::parse("")), Progress::default());
        let p = Progress::from_record(Record::parse("\u{0}\u{1}garbage"));
        assert_eq!((p.embers, p.clears, p.run), (0, 0, None));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn the_native_store_writes_reads_and_wipes_the_file() {
        let dir = std::env::temp_dir().join(format!("knell-save-test-{}", std::process::id()));
        std::env::set_var("KNELL_SAVE_DIR", &dir);
        let mut p = Progress::default();
        p.embers = 7;
        p.run = Some(RunAt { rest: 0, gate_open: false });
        assert!(save(&p));
        let got = load();
        assert_eq!((got.embers, got.run), (7, p.run));
        assert!(wipe());
        assert!(!load().has_any());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
