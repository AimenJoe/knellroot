//! 소리 첫 층(0064 M5): 효과음 + 성소 환경음 하나. 앱만 안다 — 월드는 소리를 모르고(`Event`·상태 변화만), 앱이 그것을 듣고 낸다.
//!
//! - **파일 소리**(`assets/sfx/<이름>_<n>.wav`, 바이너리에 넣는다): 첫 게임 nail의 CC0 소리를 그대로(출처는 `assets/LICENSES.md` —
//!   Kenney·OpenGameArt, 크레딧 의무 없음). 변형이 여럿이면 바로 전과 다른 것.
//! - **합성 소리**(시작할 때 만든다, 이 레포의 코드): 패링(맑은 쇳소리), 예고 넷(모양마다 다른 소리 — 소리만 듣고도 부채꼴·돌진·
//!   파동·둘레를 가르게, 0064 R4), 문지기 깨어남(낮은 울림), 쉼터(따뜻한 화륵 + 종), 등잔 한 모금, 성소 환경음(낮은 바람 + 먼 종 — Knell).
//! - 같은 소리는 `SAME_GAP`초 안에 겹쳐 내지 않는다(떼를 한 번에 맞혀도 귀가 아프지 않게).
use std::cell::{Cell, RefCell};

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

const SR: u32 = 44_100;
/// 전체 크기(설정 0..10에 곱한다).
const MASTER: f32 = 0.7;
const SAME_GAP: f64 = 0.045;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Id {
    Swing,
    Hit,
    Die,
    Hurt,
    Dodge,
    Ember,
    Chest,
    Buy,
    Deny,
    UiMove,
    UiConfirm,
    UiBack,
    Parry,
    /// 예고: 부채꼴(문지기 내려치기) — 낮게 부풀어 오르는 소리.
    TeleFan,
    /// 예고: 띠(돌진) — 높아지는 휘파람.
    TeleLane,
    /// 예고: 고리(뿌리 파동) — 깊게 떨리는 울림.
    TeleRing,
    /// 예고: 둘레 원판(잔재 내려찍기) — 짧은 두 번 딱딱.
    TeleDisk,
    Wake,
    Rest,
    Gulp,
    /// 맞힘의 둘째 층(2026-11-06, KB: 둔한 것 + 짧고 높은 것): 낮은 둥 + 4ms 딱. 파일 맞힘 소리와 함께 낸다.
    Thwack,
    /// 박자에 맞춘 평타(2026-11-07): 맑은 종 '팅' — 스태미나를 쓰지 않았다는 신호.
    /// 잇달아 맞추면 한 음씩 오른다(0076, KB 오르는 종소리 — nail 0180): Beat → Beat2 → Beat3.
    Beat,
    Beat2,
    Beat3,
    /// 지연 공격이 풀린다(0128): 짧게 들이쉬는 바람 + 낮은 딸깍 — 이 뒤 0.22초에 맞는다.
    TeleRelease,
}

/// 잇단 박자 수(1부터) → 종소리(셋째부터는 가장 높은 것).
pub fn beat_id(chain: u32) -> Id {
    match chain {
        0 | 1 => Id::Beat,
        2 => Id::Beat2,
        _ => Id::Beat3,
    }
}

/// 파일 소리 표: (이름, 변형 수, 크기).
const FILES: &[(Id, &str, &[&[u8]], f32)] = &[
    (
        Id::Swing,
        "swing",
        &[include_bytes!("../assets/sfx/swing_0.wav")],
        0.45,
    ),
    (
        Id::Hit,
        "enemy_hit",
        &[
            include_bytes!("../assets/sfx/enemy_hit_0.wav"),
            include_bytes!("../assets/sfx/enemy_hit_1.wav"),
        ],
        0.6,
    ),
    (
        Id::Die,
        "enemy_die",
        &[
            include_bytes!("../assets/sfx/enemy_die_0.wav"),
            include_bytes!("../assets/sfx/enemy_die_1.wav"),
            include_bytes!("../assets/sfx/enemy_die_2.wav"),
        ],
        0.6,
    ),
    (
        Id::Hurt,
        "hurt",
        &[
            include_bytes!("../assets/sfx/hurt_0.wav"),
            include_bytes!("../assets/sfx/hurt_1.wav"),
        ],
        0.8,
    ),
    (
        Id::Dodge,
        "dodge",
        &[include_bytes!("../assets/sfx/dodge_0.wav")],
        0.4,
    ),
    (
        Id::Ember,
        "pickup",
        &[include_bytes!("../assets/sfx/pickup_0.wav")],
        0.25,
    ),
    (
        Id::Chest,
        "chest",
        &[include_bytes!("../assets/sfx/chest_0.wav")],
        0.7,
    ),
    (
        Id::Buy,
        "buy",
        &[include_bytes!("../assets/sfx/buy_0.wav")],
        0.6,
    ),
    (
        Id::Deny,
        "deny",
        &[include_bytes!("../assets/sfx/deny_0.wav")],
        0.5,
    ),
    (
        Id::UiMove,
        "ui_move",
        &[include_bytes!("../assets/sfx/ui_move_0.wav")],
        0.3,
    ),
    (
        Id::UiConfirm,
        "ui_confirm",
        &[include_bytes!("../assets/sfx/ui_confirm_0.wav")],
        0.4,
    ),
    (
        Id::UiBack,
        "ui_back",
        &[include_bytes!("../assets/sfx/ui_back_0.wav")],
        0.35,
    ),
];

/// 합성 소리의 크기.
fn synth_gain(id: Id) -> f32 {
    match id {
        Id::Parry => 0.55,
        Id::TeleFan | Id::TeleRing => 0.5,
        Id::TeleLane => 0.4,
        Id::TeleDisk => 0.3,
        Id::TeleRelease => 0.5,
        Id::Wake => 0.7,
        Id::Rest => 0.55,
        Id::Gulp => 0.4,
        Id::Thwack => 0.5,
        // 휘두르는 소리(0.45)와 같은 틱에 나도 묻히지 않게(0076 — 0.35는 안 들렸다).
        Id::Beat | Id::Beat2 | Id::Beat3 => 0.6,
        _ => 0.5,
    }
}

const SYNTHS: [Id; 13] = [
    Id::TeleRelease,
    Id::Thwack,
    Id::Beat,
    Id::Beat2,
    Id::Beat3,
    Id::Parry,
    Id::TeleFan,
    Id::TeleLane,
    Id::TeleRing,
    Id::TeleDisk,
    Id::Wake,
    Id::Rest,
    Id::Gulp,
];

/// 환경음 크기(설정에 곱한다).
const AMBIENCE_GAIN: f32 = 0.35;

pub struct Sfx {
    files: Vec<(Id, Vec<Sound>, f32)>,
    synths: Vec<(Id, Sound)>,
    ambience: Option<Sound>,
    ambience_on: Cell<bool>,
    /// 설정의 크기 0..1.
    volume: Cell<f32>,
    last: RefCell<Vec<(Id, f64, usize)>>,
    /// 지금까지 낸 소리 수(앱의 메뉴 소리가 겹치지 않게 본다).
    played: Cell<u32>,
}

impl Sfx {
    /// 모두 올린다(웹은 브라우저가 뒤에서 푼다).
    pub async fn load() -> Sfx {
        let mut files = Vec::new();
        for (id, _, variants, gain) in FILES {
            let mut v = Vec::new();
            for bytes in variants.iter() {
                if let Ok(s) = load_sound_from_bytes(bytes).await {
                    v.push(s);
                }
            }
            files.push((*id, v, *gain));
        }
        let mut synths = Vec::new();
        for id in SYNTHS {
            if let Ok(s) = load_sound_from_bytes(&wav_bytes(&synth(id))).await {
                synths.push((id, s));
            }
        }
        let ambience = load_sound_from_bytes(&wav_bytes(&ambience())).await.ok();
        Sfx {
            files,
            synths,
            ambience,
            ambience_on: Cell::new(false),
            volume: Cell::new(0.8),
            last: RefCell::new(Vec::new()),
            played: Cell::new(0),
        }
    }

    pub fn set_volume(&self, v: f32) {
        self.volume.set(v.clamp(0.0, 1.0));
        // 환경음은 고리라 크기를 다시 맞추려면 다시 튼다.
        if self.ambience_on.get() {
            self.ambience(false);
            self.ambience(true);
        }
    }

    pub fn played(&self) -> u32 {
        self.played.get()
    }

    pub fn play(&self, id: Id) {
        self.played.set(self.played.get().wrapping_add(1));
        let vol = self.volume.get();
        if vol <= 0.0 {
            return;
        }
        let now = macroquad::time::get_time();
        let mut last = self.last.borrow_mut();
        let prev = last.iter().position(|(k, _, _)| *k == id);
        if let Some(i) = prev {
            if now - last[i].1 < SAME_GAP {
                return;
            }
        }
        let (sound, gain, pick) =
            if let Some((_, v, g)) = self.files.iter().find(|(k, _, _)| *k == id) {
                if v.is_empty() {
                    return;
                }
                // 바로 전과 다른 변형.
                let pick = prev.map_or(0, |i| (last[i].2 + 1) % v.len());
                (&v[pick], *g, pick)
            } else if let Some((_, s)) = self.synths.iter().find(|(k, _)| *k == id) {
                (s, synth_gain(id), 0)
            } else {
                return;
            };
        match prev {
            Some(i) => last[i] = (id, now, pick),
            None => last.push((id, now, pick)),
        }
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume: MASTER * vol * gain,
            },
        );
    }

    /// 성소 환경음을 켜고 끈다(고리).
    pub fn ambience(&self, on: bool) {
        let Some(s) = &self.ambience else { return };
        if on == self.ambience_on.get() {
            return;
        }
        self.ambience_on.set(on);
        if on && self.volume.get() > 0.0 {
            play_sound(
                s,
                PlaySoundParams {
                    looped: true,
                    volume: MASTER * self.volume.get() * AMBIENCE_GAIN,
                },
            );
        } else {
            stop_sound(s);
        }
    }
}

// ---------- 합성(순수 함수 — 시험은 여기서) ----------

fn rng(state: &mut u32) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    (*state as f32 / u32::MAX as f32) * 2.0 - 1.0
}

/// 소리 하나의 표본(−1..1 → i16, 봉우리 −1dBFS로 맞춘다).
fn normalize(v: &[f32]) -> Vec<i16> {
    let peak = v.iter().fold(0.0f32, |m, x| m.max(x.abs())).max(1e-6);
    let k = 0.89 / peak;
    v.iter().map(|x| (x * k * 32767.0) as i16).collect()
}

fn len(secs: f32) -> usize {
    (secs * SR as f32) as usize
}

/// 합성 소리(파일이 아닌 것).
pub fn synth(id: Id) -> Vec<i16> {
    use std::f32::consts::TAU;
    let mut seed = 0x9E37_79B9u32 ^ id as u32;
    let v: Vec<f32> = match id {
        // 패링: 맑은 쇳소리 — 어긋난 배음 셋이 빠르게 사라지고, 처음 5ms 쇠 긁힘.
        Id::Parry => (0..len(0.5))
            .map(|i| {
                let t = i as f32 / SR as f32;
                let e = (-t * 9.0).exp();
                let ring = (TAU * 1320.0 * t).sin()
                    + 0.6 * (TAU * 2051.0 * t).sin()
                    + 0.35 * (TAU * 3170.0 * t).sin();
                let scrape = if t < 0.005 { rng(&mut seed) * 0.8 } else { 0.0 };
                ring * e + scrape
            })
            .collect(),
        // 부채꼴: 낮게 부풀어 오른다(90 → 140Hz, 크기가 커지다 끊긴다).
        Id::TeleFan => {
            let n = len(0.45);
            let mut ph = 0.0f32;
            (0..n)
                .map(|i| {
                    let k = i as f32 / n as f32;
                    ph += (90.0 + 50.0 * k) / SR as f32;
                    let saw = (ph.fract() * 2.0 - 1.0) * 0.6 + (TAU * ph).sin() * 0.4;
                    saw * k.powf(1.5) * (1.0 - ((k - 0.92) / 0.08).max(0.0))
                })
                .collect()
        }
        // 띠(돌진): 높아지는 휘파람(600 → 1400Hz) + 바람 소음.
        Id::TeleLane => {
            let n = len(0.4);
            let mut ph = 0.0f32;
            (0..n)
                .map(|i| {
                    let k = i as f32 / n as f32;
                    ph += (600.0 + 800.0 * k * k) / SR as f32;
                    ((TAU * ph).sin() * 0.7 + rng(&mut seed) * 0.15)
                        * (k * 3.0).min(1.0)
                        * (1.0 - k * 0.3)
                })
                .collect()
        }
        // 고리(파동): 깊게 떨리는 울림(55Hz, 7Hz로 떨림), 길게.
        Id::TeleRing => (0..len(0.9))
            .map(|i| {
                let t = i as f32 / SR as f32;
                let am = 0.6 + 0.4 * (TAU * 7.0 * t).sin();
                let body = (TAU * 55.0 * t).sin() + 0.5 * (TAU * 110.0 * t).sin();
                body * am * (t / 0.15).min(1.0) * (1.0 - t / 0.9)
            })
            .collect(),
        // 지연이 풀린다(0128): 0.12초 들이쉬는 바람(오르는 잡음) + 끝에 낮은 딸깍 — 예고 소리들과 다른 결(숨 → 친다).
        Id::TeleRelease => (0..len(0.18))
            .map(|i| {
                let t = i as f32 / SR as f32;
                let breath = rng(&mut seed)
                    * (t / 0.12).min(1.0).powf(2.0)
                    * (1.0 - ((t - 0.12) / 0.06).max(0.0))
                    * 0.7;
                let d = t - 0.12;
                let click = if d < 0.0 {
                    0.0
                } else {
                    (TAU * 180.0 * d).sin() * (-d * 70.0).exp()
                };
                breath + click
            })
            .collect(),
        // 둘레 원판(잔재): 짧은 나무 딱 두 번.
        Id::TeleDisk => (0..len(0.22))
            .map(|i| {
                let t = i as f32 / SR as f32;
                let tick = |t0: f32| {
                    let d = t - t0;
                    if d < 0.0 {
                        0.0
                    } else {
                        (TAU * 520.0 * d).sin() * (-d * 60.0).exp()
                    }
                };
                tick(0.0) + 0.8 * tick(0.11)
            })
            .collect(),
        // 문지기 깨어남: 낮은 땅울림(낮은 소음 + 40Hz), 2초.
        Id::Wake => {
            let mut lp = 0.0f32;
            (0..len(2.0))
                .map(|i| {
                    let t = i as f32 / SR as f32;
                    lp += (rng(&mut seed) - lp) * 0.02;
                    let env = (t / 0.6).min(1.0) * (1.0 - (t - 1.4).max(0.0) / 0.6);
                    ((TAU * 40.0 * t).sin() * 0.6 + lp * 3.0) * env
                })
                .collect()
        }
        // 쉼터: 화륵(걸러 낸 소음이 커졌다 잦아듦) + 맑은 종 하나.
        Id::Rest => {
            let mut lp = 0.0f32;
            (0..len(1.6))
                .map(|i| {
                    let t = i as f32 / SR as f32;
                    lp += (rng(&mut seed) - lp) * 0.08;
                    let whoosh =
                        lp * 2.5 * (t / 0.25).min(1.0) * (-(t - 0.25).max(0.0) * 4.0).exp();
                    let bell = ((TAU * 660.0 * t).sin() + 0.4 * (TAU * 1650.0 * t).sin())
                        * (-t * 2.5).exp()
                        * 0.5;
                    whoosh + bell
                })
                .collect()
        }
        // 등잔 한 모금: 꿀꺽(낮아지는 짧은 사인 둘).
        Id::Gulp => (0..len(0.3))
            .map(|i| {
                let t = i as f32 / SR as f32;
                let g = |t0: f32, f: f32| {
                    let d = t - t0;
                    if d < 0.0 {
                        0.0
                    } else {
                        (TAU * (f - 300.0 * d) * d).sin() * (-d * 25.0).exp()
                    }
                };
                g(0.0, 380.0) + g(0.13, 320.0)
            })
            .collect(),
        // 맞힘 둘째 층: 120 → 70Hz로 떨어지는 둥(80ms) + 처음 4ms 딱(높은 소음).
        Id::Thwack => {
            let mut ph = 0.0f32;
            (0..len(0.12))
                .map(|i| {
                    let t = i as f32 / SR as f32;
                    ph += (120.0 - 400.0 * t).max(70.0) / SR as f32;
                    let thud = (TAU * ph).sin() * (-t * 30.0).exp();
                    let click = if t < 0.004 {
                        rng(&mut seed) * (1.0 - t / 0.004)
                    } else {
                        0.0
                    };
                    thud * 0.9 + click * 0.8
                })
                .collect()
        }
        // 박자: 종처럼 바탕음 + 2.76배 배음(작은 종의 맑은 겹)이 0.32초에 사라지는 '팅'. 잇달으면 G6 → B6 → D7.
        Id::Beat | Id::Beat2 | Id::Beat3 => {
            let f0 = match id {
                Id::Beat => 1568.0,
                Id::Beat2 => 1976.0,
                _ => 2349.0,
            };
            (0..len(0.32))
                .map(|i| {
                    let t = i as f32 / SR as f32;
                    let e = (-t * 13.0).exp();
                    let strike = (-t * 60.0).exp();
                    ((TAU * f0 * t).sin() + 0.35 * (TAU * f0 * 2.76 * t).sin() * strike) * e
                })
                .collect()
        }
        _ => vec![0.0; 64],
    };
    // 끝 5ms를 줄인다(딸깍 없이).
    let mut v = v;
    let fade = len(0.005).min(v.len());
    let n = v.len();
    for (j, x) in v[n - fade..].iter_mut().enumerate() {
        *x *= 1.0 - j as f32 / fade as f32;
    }
    normalize(&v)
}

/// 성소 환경음 12초 고리: 낮은 바람(걸러 낸 소음, 천천히 오르내림) + 먼 종 한 번(6초에, 길게 사라진다).
/// 처음과 끝을 이어 붙여도 튀지 않게 바람은 고리 길이로 감고, 종은 끝나기 전에 사라진다.
pub fn ambience() -> Vec<i16> {
    use std::f32::consts::TAU;
    let n = len(12.0);
    let mut seed = 0x1234_5678u32;
    // 바람: 두 번 거른 소음.
    let mut a = 0.0f32;
    let mut b = 0.0f32;
    let mut wind: Vec<f32> = (0..n)
        .map(|_| {
            a += (rng(&mut seed) - a) * 0.03;
            b += (a - b) * 0.03;
            b
        })
        .collect();
    // 고리 이음매: 끝 0.5초를 처음과 섞는다.
    let x = len(0.5);
    for j in 0..x {
        let k = j as f32 / x as f32;
        let tail = n - x + j;
        wind[tail] = wind[tail] * (1.0 - k) + wind[j] * k;
    }
    let v: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / SR as f32;
            let swell = 0.6 + 0.4 * (TAU * t / 12.0).sin();
            let w = wind[i] * 6.0 * swell;
            let d = t - 6.0;
            let bell = if d >= 0.0 {
                ((TAU * 196.0 * d).sin()
                    + 0.5 * (TAU * 392.7 * d).sin()
                    + 0.25 * (TAU * 589.0 * d).sin())
                    * (-d * 0.8).exp()
                    * 0.35
            } else {
                0.0
            };
            w + bell
        })
        .collect();
    normalize(&v)
}

/// 16비트 모노 PCM WAV.
pub fn wav_bytes(samples: &[i16]) -> Vec<u8> {
    let data = samples.len() as u32 * 2;
    let mut b = Vec::with_capacity(44 + data as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&SR.to_le_bytes());
    b.extend_from_slice(&(SR * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data.to_le_bytes());
    for s in samples {
        b.extend_from_slice(&s.to_le_bytes());
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sound_has_a_file_or_a_synth_and_synths_are_loud_but_not_clipped() {
        for id in [
            Id::Swing,
            Id::Hit,
            Id::Die,
            Id::Hurt,
            Id::Dodge,
            Id::Ember,
            Id::Chest,
            Id::Buy,
            Id::Deny,
            Id::UiMove,
            Id::UiConfirm,
            Id::UiBack,
        ] {
            let f = FILES.iter().find(|(k, ..)| *k == id).expect("a file row");
            for bytes in f.2 {
                assert_eq!(&bytes[0..4], b"RIFF", "{:?}", id);
            }
        }
        for id in SYNTHS {
            assert!(FILES.iter().all(|(k, ..)| *k != id), "{id:?} is not both");
            let s = synth(id);
            assert!(s.len() > 1000);
            let peak = s.iter().map(|x| x.unsigned_abs()).max().unwrap();
            assert!(peak > 20000 && peak < 32700, "{id:?} peak {peak}");
            assert!(
                s.last().unwrap().unsigned_abs() < 200,
                "{id:?} ends quietly"
            );
        }
    }

    #[test]
    fn beats_in_a_row_ring_higher() {
        let pitch = |id| {
            let s = synth(id);
            s.windows(2).filter(|w| (w[0] < 0) != (w[1] < 0)).count()
        };
        assert_eq!(beat_id(1), Id::Beat);
        assert_eq!(beat_id(5), Id::Beat3);
        assert!(pitch(Id::Beat) < pitch(Id::Beat2) && pitch(Id::Beat2) < pitch(Id::Beat3));
    }

    #[test]
    fn the_four_warnings_sound_different() {
        // 소리만 듣고 가르게(0064 R4): 길이·높이(영점 교차 수 — 대략의 음높이)가 서로 다르다.
        let feat = |id| {
            let s = synth(id);
            let crossings = s.windows(2).filter(|w| (w[0] < 0) != (w[1] < 0)).count() as f32;
            let secs = s.len() as f32 / SR as f32;
            (secs, crossings / secs)
        };
        let fan = feat(Id::TeleFan);
        let lane = feat(Id::TeleLane);
        let ring = feat(Id::TeleRing);
        let disk = feat(Id::TeleDisk);
        assert!(
            lane.1 > fan.1 * 3.0,
            "charge whistle is much higher than the fan swell"
        );
        assert!(
            ring.0 > fan.0 && ring.0 > lane.0,
            "the ripple rings longest"
        );
        assert!(ring.1 < lane.1, "the ripple is low");
        assert!(disk.0 < fan.0, "the husk ticks are short");
    }

    #[test]
    fn the_ambience_loops_without_a_seam() {
        let s = ambience();
        assert_eq!(s.len(), (12.0 * SR as f32) as usize);
        let jump = (s[s.len() - 1] as i32 - s[0] as i32).abs();
        let typical = s
            .windows(2)
            .map(|w| (w[1] as i32 - w[0] as i32).abs())
            .max()
            .unwrap();
        assert!(jump <= typical, "seam {jump} vs largest step {typical}");
    }

    #[test]
    fn a_wav_header_matches_its_data() {
        let b = wav_bytes(&[0, 1000, -1000]);
        assert_eq!(&b[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes([b[40], b[41], b[42], b[43]]), 6);
        assert_eq!(b.len(), 44 + 6);
    }
}
