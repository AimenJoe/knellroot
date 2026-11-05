//! 다국어(첫 게임 규칙 — KB `guides/second-game-project-day-one.md` 7번, 0019).
//!
//! - 플레이어가 보는 글은 전부 언어별 표(`ko.rs`·`en.rs`) 한 곳에. 키는 `Strings`의 필드 — 한 언어에 빠지면 컴파일이 안 된다.
//! - 숫자가 들어가는 글은 `{0}`·`{1}` + `fill()`. 문장을 조각으로 잇지 않는다.
//! - 데이터 표(스테이지·적·구역)는 글 대신 `Tr`(표에서 글을 고르는 함수)을 들고, 그릴 때 `tr()`로 꺼낸다. 월드는 언어를 모른다.
//! - 키 이름(J·Space·X …)은 글에 박지 않는다 — 동사만 여기, 키 그림은 `input::prompt`가 지금 장치에 맞춰 붙인다.
//! - 글꼴은 고운바탕(OFL) 서브셋 — 표 글자만 들어 있다(0021). 새 글자를 쓰면 `tools/fonts/subset_ui.py`를 다시 돌린다(시험이 잡는다).
//! - 지금 두 언어(한국어·영어). 구조는 첫 게임처럼 열한 언어까지 그대로 늘어난다(파일 하나 + `Lang` 한 줄).
use std::sync::atomic::{AtomicU8, Ordering};

mod en;
mod ko;

pub use en::EN;
pub use ko::KO;

/// 표에서 글 하나를 고르는 함수.
pub type Tr = fn(&Strings) -> &'static str;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Ko,
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Ko, Lang::En];

    pub fn strings(self) -> &'static Strings {
        match self {
            Lang::Ko => &KO,
            Lang::En => &EN,
        }
    }

    pub fn next(self) -> Lang {
        let i = Lang::ALL.iter().position(|l| *l == self).unwrap_or(0);
        Lang::ALL[(i + 1) % Lang::ALL.len()]
    }
}

static LANG: AtomicU8 = AtomicU8::new(0);

pub fn lang() -> Lang {
    Lang::ALL[LANG.load(Ordering::Relaxed) as usize % Lang::ALL.len()]
}

pub fn set_lang(l: Lang) {
    let i = Lang::ALL.iter().position(|x| *x == l).unwrap_or(0);
    LANG.store(i as u8, Ordering::Relaxed);
}

/// 지금 언어의 표.
pub fn s() -> &'static Strings {
    lang().strings()
}

pub fn tr(f: Tr) -> &'static str {
    f(s())
}

/// `{0}`, `{1}` … 자리에 값을 채운다.
pub fn fill(template: &str, args: &[&str]) -> String {
    let mut out = template.to_string();
    for (i, a) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), a);
    }
    out
}

/// 언어마다 같은 키. 필드를 더하면 모든 언어 파일에 더해야 컴파일된다.
pub struct Strings {
    /// 이 언어의 이름(언어 고르기 줄).
    pub lang_name: &'static str,
    /// 게임 이름(고유명사 — 언어마다 같아도 표에 둔다).
    pub game_title: &'static str,
    pub title_subtitle: &'static str,
    pub menu_start: &'static str,
    /// "언어: {0}"
    pub menu_language: &'static str,
    pub menu_resume: &'static str,
    pub menu_restart: &'static str,
    pub menu_to_title: &'static str,
    pub menu_retry: &'static str,
    pub pause_title: &'static str,
    pub verb_attack: &'static str,
    pub verb_dodge: &'static str,
    pub verb_turn: &'static str,
    pub verb_lock: &'static str,
    pub verb_parry: &'static str,
    pub verb_jump: &'static str,
    /// 등잔 마시기(0031).
    pub verb_flask: &'static str,
    /// 상호작용(0032).
    /// 마법(0048, RB).
    pub verb_cast: &'static str,
    /// 조작 설정의 점프 줄 — 점프 버튼이 상호작용도 한다(0048).
    pub verb_jump_interact: &'static str,
    /// 쉼터(0032): 곁에서 뜨는 동사, 적이 가까울 때, 쉰 뒤, 다시 일어선 뒤.
    pub rest_verb: &'static str,
    pub rest_blocked: &'static str,
    pub rested: &'static str,
    pub respawned: &'static str,
    /// 싸움 안내 줄의 공격(0030: 꾹 쥐면 강공격).
    pub verb_attack_hint: &'static str,
    pub verb_turn_left: &'static str,
    pub verb_turn_right: &'static str,
    pub menu_controls: &'static str,
    /// 설정(0052): 타이틀·멈춤의 줄, 화면 제목, 화면 흔들림 줄("{0}" = 백분율).
    pub menu_settings: &'static str,
    pub settings_title: &'static str,
    pub menu_shake: &'static str,
    /// 멈춤의 무기 줄(0051, "{0}" = 무기 이름)과 무기 이름.
    pub menu_weapon: &'static str,
    pub weapon_sword: &'static str,
    pub weapon_hammer: &'static str,
    pub weapon_staff: &'static str,
    pub weapon_crossbow: &'static str,
    /// 타이틀의 전체 화면 줄(0046, 지금 상태를 보인다).
    pub menu_fullscreen_on: &'static str,
    pub menu_fullscreen_off: &'static str,
    /// 설정의 마우스 조준 줄(0058): 켬/끔, 커서가 쉰 뒤 자동 조준까지 "{0}초", 조준 표시 크기 "{0}" + 크기 셋.
    pub menu_mouse_aim_on: &'static str,
    pub menu_mouse_aim_off: &'static str,
    pub menu_cursor_rest: &'static str,
    pub menu_reticle: &'static str,
    pub reticle_small: &'static str,
    pub reticle_medium: &'static str,
    pub reticle_large: &'static str,
    pub controls_title: &'static str,
    pub controls_keyboard: &'static str,
    pub controls_pad: &'static str,
    /// 조작 설정의 배치 둘(0058): 키보드+마우스(기본)·키보드만.
    pub controls_preset_mouse: &'static str,
    pub controls_preset_keys: &'static str,
    pub controls_back: &'static str,
    pub controls_press: &'static str,
    pub controls_fixed: &'static str,
    pub verb_aim: &'static str,
    pub verb_choose: &'static str,
    pub verb_confirm: &'static str,
    // 첫 스테이지 "침묵의 성소"(docs/phase-1-first-stage.md).
    pub stage1_name: &'static str,
    pub zone_entrance: &'static str,
    pub zone_courtyard: &'static str,
    pub zone_sanctum: &'static str,
    pub intro_1: &'static str,
    pub intro_2: &'static str,
    pub obj_courtyard: &'static str,
    pub obj_gate_open: &'static str,
    pub obj_core: &'static str,
    pub boss_warden: &'static str,
    pub boss_wakes: &'static str,
    pub enemy_husk: &'static str,
    pub enemy_thorn: &'static str,
    pub clear_title: &'static str,
    /// "{0} · {1}" — 스테이지 이름, 걸린 시간.
    pub clear_sub: &'static str,
    pub defeat_title: &'static str,
    pub defeat_sub: &'static str,
}

/// 표의 모든 글(글자 굽기·글꼴 시험). 필드를 더하면 여기에도 — 빠뜨리면 그 글자가 안 구워진다.
pub fn all(s: &Strings) -> Vec<&'static str> {
    vec![
        s.lang_name,
        s.game_title,
        s.title_subtitle,
        s.menu_start,
        s.menu_language,
        s.menu_resume,
        s.menu_restart,
        s.menu_to_title,
        s.menu_retry,
        s.pause_title,
        s.verb_attack,
        s.verb_dodge,
        s.verb_turn,
        s.verb_lock,
        s.verb_parry,
        s.verb_jump,
        s.verb_flask,
        s.verb_cast,
        s.verb_jump_interact,
        s.rest_verb,
        s.rest_blocked,
        s.rested,
        s.respawned,
        s.verb_attack_hint,
        s.verb_turn_left,
        s.verb_turn_right,
        s.menu_controls,
        s.menu_settings,
        s.settings_title,
        s.menu_shake,
        s.menu_weapon,
        s.weapon_sword,
        s.weapon_hammer,
        s.weapon_staff,
        s.weapon_crossbow,
        s.menu_fullscreen_on,
        s.menu_fullscreen_off,
        s.controls_title,
        s.controls_keyboard,
        s.controls_pad,
        s.controls_preset_mouse,
        s.controls_preset_keys,
        s.menu_mouse_aim_on,
        s.menu_mouse_aim_off,
        s.menu_cursor_rest,
        s.menu_reticle,
        s.reticle_small,
        s.reticle_medium,
        s.reticle_large,
        s.controls_back,
        s.controls_press,
        s.controls_fixed,
        s.verb_aim,
        s.verb_choose,
        s.verb_confirm,
        s.stage1_name,
        s.zone_entrance,
        s.zone_courtyard,
        s.zone_sanctum,
        s.intro_1,
        s.intro_2,
        s.obj_courtyard,
        s.obj_gate_open,
        s.obj_core,
        s.boss_warden,
        s.boss_wakes,
        s.enemy_husk,
        s.enemy_thorn,
        s.clear_title,
        s.clear_sub,
        s.defeat_title,
        s.defeat_sub,
    ]
}

/// 표 밖에서 화면에 찍는 기호(메뉴 표지 등). 글꼴 시험이 같이 본다.
pub const UI_GLYPHS: &str = "◆◇…·▲▼▶";

#[cfg(test)]
mod tests {
    use super::*;

    /// 모든 언어의 모든 글자가 글꼴에 있다(없으면 빈 칸으로 찍힌다).
    #[test]
    fn every_string_is_drawable_with_the_font() {
        for font in [
            &include_bytes!("../../assets/fonts/GowunBatang-UI-Regular.ttf")[..],
            include_bytes!("../../assets/fonts/GowunBatang-UI-Bold.ttf"),
        ] {
            let face = ttf_parser::Face::parse(font, 0).unwrap();
            for l in Lang::ALL {
                for text in all(l.strings()).into_iter().chain([UI_GLYPHS]) {
                    for c in text.chars() {
                        assert!(
                        c.is_whitespace() || face.glyph_index(c).is_some(),
                        "{l:?}: {c:?} in {text:?} not in the UI font — tools/fonts/subset_ui.py"
                    );
                    }
                }
            }
        }
    }

    /// 자리표시자({0}…)가 언어끼리 같다.
    #[test]
    fn placeholders_match_across_languages() {
        let count = |t: &str| (0..4).filter(|i| t.contains(&format!("{{{i}}}"))).count();
        let ko = all(&KO);
        for l in Lang::ALL {
            for (a, b) in ko.iter().zip(all(l.strings())) {
                assert_eq!(count(a), count(b), "{l:?}: {b:?}");
            }
        }
        assert_eq!(fill("{0} · {1}", &["a", "b"]), "a · b");
    }
}
