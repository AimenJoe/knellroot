//! 다국어(첫 게임 규칙 — KB `guides/second-game-project-day-one.md` 7번, 0019).
//!
//! - 플레이어가 보는 글은 전부 언어별 표(`ko.rs`·`en.rs`) 한 곳에. 키는 `Strings`의 필드 — 한 언어에 빠지면 컴파일이 안 된다.
//! - 숫자가 들어가는 글은 `{0}`·`{1}` + `fill()`. 문장을 조각으로 잇지 않는다.
//! - 데이터 표(스테이지·적·구역)는 글 대신 `Tr`(표에서 글을 고르는 함수)을 들고, 그릴 때 `tr()`로 꺼낸다. 월드는 언어를 모른다.
//! - 키 이름(J·Space·X …)은 글에 박지 않는다 — 동사만 여기, 키 그림은 `input::prompt`가 지금 장치에 맞춰 붙인다.
//! - 글꼴은 고운바탕(OFL) 서브셋 — 표 글자만 들어 있다(0021). 새 글자를 쓰면 `tools/fonts/subset_ui.py`를 다시 돌린다(시험이 잡는다).
//! - 지금 두 언어(한국어·영어). 구조는 첫 게임처럼 열한 언어까지 그대로 늘어난다(파일 하나 + `Lang` 한 줄).
use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(feature = "dev")]
pub mod dev;
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
    /// 묻는 창(KB — 하는 일 이름 / 취소, 무엇을 잃는지 한 줄)·저장 실패 알림.
    pub ask_restart: &'static str,
    pub ask_to_title: &'static str,
    pub ask_buy: &'static str,
    pub ask_buy_yes: &'static str,
    pub ask_cancel: &'static str,
    pub save_failed: &'static str,

    /// 성장(0064 M4): 핵 고르기·쉼터 화로 가게.
    pub core_title: &'static str,
    pub tree_title: &'static str,
    pub tree_points: &'static str,
    pub hearth_tree_d: &'static str,
    pub hearth_points: &'static str,
    pub tree_beat: &'static str,
    pub tree_beat_d: &'static str,
    pub tree_riposte: &'static str,
    pub tree_riposte_d: &'static str,
    pub tree_footing: &'static str,
    pub tree_footing_d: &'static str,
    pub tree_open: &'static str,
    pub tree_ready: &'static str,
    pub tree_short: &'static str,
    pub tree_far: &'static str,
    pub tree_goal: &'static str,
    pub tree_view_only: &'static str,
    pub toast_pin: &'static str,
    pub toast_unpin: &'static str,
    pub ask_tree: &'static str,
    pub ask_tree_yes: &'static str,
    pub toast_tree: &'static str,
    pub toast_find_shard: &'static str,
    /// 엘리트를 처음 쓰러뜨렸다(0112): {0} = 더 흩은 잔불.
    pub toast_elite: &'static str,
    pub verb_pin: &'static str,
    pub cell_even_breath: &'static str,
    pub cell_even_breath_d: &'static str,
    pub cell_flowing_beat: &'static str,
    pub cell_flowing_beat_d: &'static str,
    pub cell_closing_blow: &'static str,
    pub cell_closing_blow_d: &'static str,
    pub cell_gap_beat: &'static str,
    pub cell_gap_beat_d: &'static str,
    pub cell_final_measure: &'static str,
    pub cell_final_measure_d: &'static str,
    pub cell_catch: &'static str,
    pub cell_catch_d: &'static str,
    pub cell_turn_it_back: &'static str,
    pub cell_turn_it_back_d: &'static str,
    pub cell_gathered: &'static str,
    pub cell_gathered_d: &'static str,
    pub cell_open_breach: &'static str,
    pub cell_open_breach_d: &'static str,
    pub cell_one_who_stands: &'static str,
    pub cell_one_who_stands_d: &'static str,
    pub cell_stride: &'static str,
    pub cell_stride_d: &'static str,
    pub cell_sidestep: &'static str,
    pub cell_sidestep_d: &'static str,
    pub cell_tilting: &'static str,
    pub cell_tilting_d: &'static str,
    pub cell_landing: &'static str,
    pub cell_landing_d: &'static str,
    pub cell_path: &'static str,
    pub cell_path_d: &'static str,
    pub find_verb: &'static str,
    pub toast_find_lamp: &'static str,
    pub toast_find_sip: &'static str,
    pub toast_core: &'static str,
    pub toast_core_full: &'static str,
    pub shop_title: &'static str,
    pub shop_embers: &'static str,
    pub shop_item: &'static str,
    pub shop_oil: &'static str,
    pub shop_oil_d: &'static str,
    pub shop_leave: &'static str,
    pub shop_poor: &'static str,
    pub shop_empty: &'static str,
    pub toast_bought: &'static str,

    /// 장비(0064 M3): 멈춤 줄·창 제목·칸 줄·빈 칸·없을 때 안내·얻었을 때 알림·빈 상자·상자 곁 동사.
    pub menu_equip: &'static str,
    pub equip_title: &'static str,
    pub menu_charm_slot: &'static str,
    pub charm_empty: &'static str,
    pub equip_none_yet: &'static str,
    pub toast_charm: &'static str,
    pub toast_weapon: &'static str,
    pub toast_embers: &'static str,
    pub chest_verb: &'static str,

    /// 기록이 있을 때의 시작(0064 M2)·새로 하기·그 확인.
    pub menu_continue: &'static str,
    /// 슬롯 창(0066, KB `roguelite-save-slots-and-continue`): 제목 둘, 줄 이름 "{0}" = 번호, 요약 "{0}" 완주 · "{1}" 잔불 · "{2}" 장신구,
    /// 빈 슬롯, 단추(시작·덮어쓰기·이어하기·지우기)와 두 번째 누름을 묻는 글.
    /// 장비 창(0067): 상태 머리글, 장비 중인 무기 표시, 바뀌는 것 없음, 빈 칸 설명,
    /// 상태 줄 이름 열과 등잔 횟수 "{0}", 무기 설명 넷.
    pub equip_status: &'static str,
    /// 무기 숫자 줄(2026-11-07).
    pub wstat_damage: &'static str,
    pub wstat_speed: &'static str,
    pub wstat_reach: &'static str,
    pub wstat_heavy: &'static str,
    pub wstat_shot: &'static str,
    pub wstat_mag: &'static str,
    pub wstat_reload: &'static str,
    pub wstat_grenade: &'static str,
    pub equip_no_change: &'static str,
    pub equip_empty_d: &'static str,
    pub stat_max_hp: &'static str,
    pub stat_flask: &'static str,
    pub stat_sips: &'static str,
    pub stat_heal: &'static str,
    pub stat_dmg: &'static str,
    pub stat_taken: &'static str,
    pub stat_atk_speed: &'static str,
    pub stat_dodge: &'static str,
    pub stat_parry: &'static str,
    pub stat_kill: &'static str,
    pub stat_ember: &'static str,
    pub stat_stamina: &'static str,
    pub stat_regen: &'static str,
    pub stat_poise: &'static str,
    pub stat_heavy: &'static str,
    pub stat_light: &'static str,
    pub stat_opened: &'static str,
    pub hearth_level: &'static str,
    pub hearth_level_d: &'static str,
    pub hearth_shop: &'static str,
    pub hearth_shop_d: &'static str,
    pub level_title: &'static str,
    pub lv_life: &'static str,
    pub lv_stamina: &'static str,
    pub lv_might: &'static str,
    pub lv_rank: &'static str,
    pub lv_full: &'static str,
    pub lv_full_d: &'static str,
    pub ask_level: &'static str,
    pub ask_level_yes: &'static str,
    pub toast_level: &'static str,
    pub weapon_sword_d: &'static str,
    pub weapon_hammer_d: &'static str,
    pub weapon_staff_d: &'static str,
    pub weapon_crossbow_d: &'static str,
    pub weapon_fists_d: &'static str,
    /// 장비 창 무기 후보에서 지금 든 것의 표시(0076).
    pub equip_worn: &'static str,
    pub slots_new: &'static str,
    pub slots_load: &'static str,
    pub slot_name: &'static str,
    pub slot_summary: &'static str,
    pub slot_empty: &'static str,
    pub slot_start: &'static str,
    pub slot_overwrite: &'static str,
    pub slot_overwrite_sure: &'static str,
    pub slot_continue: &'static str,
    pub slot_erase: &'static str,
    pub slot_erase_sure: &'static str,
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
    /// 조작 설정의 회피 줄(0076 — 쥐면 달리기).
    pub verb_dodge_hint: &'static str,
    pub verb_turn_left: &'static str,
    pub verb_turn_right: &'static str,
    pub menu_controls: &'static str,
    /// 연구소(랩, 0070 → 0079) — 개발판 타이틀·멈춤 메뉴의 설정 바로 아래 줄.
    pub menu_lab: &'static str,
    /// 설정(0052): 타이틀·멈춤의 줄, 화면 제목, 화면 흔들림 줄("{0}" = 백분율).
    pub menu_settings: &'static str,
    pub settings_title: &'static str,
    pub menu_shake: &'static str,
    /// 소리 크기(0064 M5).
    pub menu_volume: &'static str,
    pub menu_tells: &'static str,
    pub menu_tell_timing: &'static str,
    /// 멈춤의 무기 줄(0051, "{0}" = 무기 이름)과 무기 이름.
    pub menu_weapon: &'static str,
    pub weapon_sword: &'static str,
    pub weapon_hammer: &'static str,
    pub weapon_staff: &'static str,
    pub weapon_crossbow: &'static str,
    pub weapon_fists: &'static str,
    /// 타이틀의 전체 화면 줄(0046, 지금 상태를 보인다).
    /// 창 안 줄의 이름(값은 오른쪽 칸 — 0066).
    pub menu_fullscreen: &'static str,
    pub value_on: &'static str,
    pub value_off: &'static str,
    /// 초 값(`{0}`).
    pub value_secs: &'static str,
    /// 설정의 마우스 조준 줄(0058): 켬/끔, 커서가 쉰 뒤 자동 조준까지 "{0}초", 조준 표시 크기 "{0}" + 크기 셋.
    pub menu_mouse_aim: &'static str,
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
    pub verb_swap: &'static str,
    // 첫 스테이지 "침묵의 성소"(docs/phase-1-first-stage.md).
    pub stage1_name: &'static str,
    pub zone_entrance: &'static str,
    pub zone_courtyard: &'static str,
    /// 0113: 입구와 뜰 사이 무너진 길(뿌리 굴 포함).
    pub zone_road: &'static str,
    /// 0113: 문이 닫혀 있을 때의 목표, 문 장치 곁의 동사, 처음 열었을 때의 카드.
    pub obj_find_gate: &'static str,
    pub lever_verb: &'static str,
    pub card_gate_opened: &'static str,
    pub zone_sanctum: &'static str,
    pub stage2_name: &'static str,
    pub zone_village_edge: &'static str,
    pub zone_bell_site: &'static str,
    pub zone_root_alley: &'static str,
    pub zone_lower_square: &'static str,
    pub talk_verb: &'static str,
    pub v01_a: &'static str,
    pub v01_b: &'static str,
    pub v01_c: &'static str,
    pub v01_after: &'static str,
    pub v02_a: &'static str,
    pub v02_b: &'static str,
    pub v02_c: &'static str,
    pub v02_after: &'static str,
    pub v03_a: &'static str,
    pub v03_b: &'static str,
    pub v03_c: &'static str,
    pub v03_after: &'static str,
    pub obj_bell_site: &'static str,
    pub obj_descend: &'static str,
    pub card_way_down: &'static str,
    pub depth_under: &'static str,
    pub depth_surface: &'static str,
    pub intro_1: &'static str,
    pub intro_2: &'static str,
    pub obj_courtyard: &'static str,
    pub obj_gate_open: &'static str,
    pub obj_core: &'static str,
    pub boss_warden: &'static str,
    pub boss_wakes: &'static str,
    /// 장신구(0064 M3): 이름·설명(`charms.rs` 표).
    pub charm_shell: &'static str,
    pub charm_shell_d: &'static str,
    pub charm_feet: &'static str,
    pub charm_feet_d: &'static str,
    pub charm_ear: &'static str,
    pub charm_ear_d: &'static str,
    pub charm_hunger: &'static str,
    pub charm_hunger_d: &'static str,
    pub charm_hand: &'static str,
    pub charm_hand_d: &'static str,
    pub charm_cloth: &'static str,
    pub charm_cloth_d: &'static str,
    pub charm_jar: &'static str,
    pub charm_jar_d: &'static str,
    pub charm_bellshard: &'static str,
    pub charm_bellshard_d: &'static str,
    pub charm_pouch: &'static str,
    pub charm_pouch_d: &'static str,
    pub enemy_husk: &'static str,
    pub enemy_thorn: &'static str,
    pub enemy_shell: &'static str,
    pub enemy_pod: &'static str,
    pub boss_bellkeeper: &'static str,
    pub bellkeeper_wakes: &'static str,
    pub clear_title: &'static str,
    /// "{0} · {1}" — 스테이지 이름, 걸린 시간.
    pub clear_sub: &'static str,
    pub defeat_title: &'static str,
    pub defeat_sub: &'static str,
}

/// 개발 화면 글(0080 — 랩 값 이름·묶음·시나리오·쌍). 글꼴 시험과 글자 미리 굽기(`ui::all_text`)가 같이 쓴다 —
/// 미리 굽지 않으면 개발 창의 글자가 그 프레임에 아틀라스를 키워 앞서 그린 글이 지운 텍스처를 본다(0106, `text.rs`).
#[cfg(feature = "dev")]
pub fn dev_texts() -> Vec<&'static str> {
    dev::SPEC_KO
        .iter()
        .map(|(_, v)| *v)
        .chain(dev::GROUP_KO)
        .chain(dev::SCENARIO_KO.iter().flat_map(|(_, n, c)| [*n, *c]))
        .chain(dev::PAIRS.iter().map(|p| p.0))
        .collect()
}

#[cfg(not(feature = "dev"))]
pub fn dev_texts() -> Vec<&'static str> {
    Vec::new()
}

/// 표의 모든 글(글자 굽기·글꼴 시험). 필드를 더하면 여기에도 — 빠뜨리면 그 글자가 안 구워진다.
pub fn all(s: &Strings) -> Vec<&'static str> {
    vec![
        s.lang_name,
        s.game_title,
        s.title_subtitle,
        s.menu_start,
        s.ask_restart,
        s.ask_to_title,
        s.ask_buy,
        s.ask_buy_yes,
        s.ask_cancel,
        s.save_failed,
        s.core_title,
        s.tree_title,
        s.tree_points,
        s.hearth_tree_d,
        s.hearth_points,
        s.tree_beat,
        s.tree_beat_d,
        s.tree_riposte,
        s.tree_riposte_d,
        s.tree_footing,
        s.tree_footing_d,
        s.tree_open,
        s.tree_ready,
        s.tree_short,
        s.tree_far,
        s.tree_goal,
        s.tree_view_only,
        s.toast_pin,
        s.toast_unpin,
        s.ask_tree,
        s.ask_tree_yes,
        s.toast_tree,
        s.toast_find_shard,
        s.toast_elite,
        s.verb_pin,
        s.cell_even_breath,
        s.cell_even_breath_d,
        s.cell_flowing_beat,
        s.cell_flowing_beat_d,
        s.cell_closing_blow,
        s.cell_closing_blow_d,
        s.cell_gap_beat,
        s.cell_gap_beat_d,
        s.cell_final_measure,
        s.cell_final_measure_d,
        s.cell_catch,
        s.cell_catch_d,
        s.cell_turn_it_back,
        s.cell_turn_it_back_d,
        s.cell_gathered,
        s.cell_gathered_d,
        s.cell_open_breach,
        s.cell_open_breach_d,
        s.cell_one_who_stands,
        s.cell_one_who_stands_d,
        s.cell_stride,
        s.cell_stride_d,
        s.cell_sidestep,
        s.cell_sidestep_d,
        s.cell_tilting,
        s.cell_tilting_d,
        s.cell_landing,
        s.cell_landing_d,
        s.cell_path,
        s.cell_path_d,
        s.find_verb,
        s.toast_find_lamp,
        s.toast_find_sip,
        s.toast_core,
        s.toast_core_full,
        s.shop_title,
        s.shop_embers,
        s.shop_item,
        s.shop_oil,
        s.shop_oil_d,
        s.shop_leave,
        s.shop_poor,
        s.shop_empty,
        s.toast_bought,
        s.menu_equip,
        s.equip_title,
        s.menu_charm_slot,
        s.charm_empty,
        s.equip_none_yet,
        s.toast_charm,
        s.toast_weapon,
        s.toast_embers,
        s.chest_verb,
        s.menu_continue,
        s.equip_status,
        s.wstat_damage,
        s.wstat_speed,
        s.wstat_reach,
        s.wstat_heavy,
        s.wstat_shot,
        s.wstat_mag,
        s.wstat_reload,
        s.wstat_grenade,
        s.equip_no_change,
        s.equip_empty_d,
        s.stat_max_hp,
        s.stat_flask,
        s.stat_sips,
        s.stat_heal,
        s.stat_dmg,
        s.stat_taken,
        s.stat_atk_speed,
        s.stat_dodge,
        s.stat_parry,
        s.stat_kill,
        s.stat_ember,
        s.stat_stamina,
        s.stat_regen,
        s.stat_poise,
        s.stat_heavy,
        s.stat_light,
        s.stat_opened,
        s.hearth_level,
        s.hearth_level_d,
        s.hearth_shop,
        s.hearth_shop_d,
        s.level_title,
        s.lv_life,
        s.lv_stamina,
        s.lv_might,
        s.lv_rank,
        s.lv_full,
        s.lv_full_d,
        s.ask_level,
        s.ask_level_yes,
        s.toast_level,
        s.weapon_sword_d,
        s.weapon_hammer_d,
        s.weapon_staff_d,
        s.weapon_crossbow_d,
        s.weapon_fists_d,
        s.equip_worn,
        s.slots_new,
        s.slots_load,
        s.slot_name,
        s.slot_summary,
        s.slot_empty,
        s.slot_start,
        s.slot_overwrite,
        s.slot_overwrite_sure,
        s.slot_continue,
        s.slot_erase,
        s.slot_erase_sure,
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
        s.verb_dodge_hint,
        s.verb_turn_left,
        s.verb_turn_right,
        s.menu_controls,
        s.menu_lab,
        s.menu_settings,
        s.settings_title,
        s.menu_shake,
        s.menu_volume,
        s.menu_tells,
        s.menu_tell_timing,
        s.menu_weapon,
        s.weapon_sword,
        s.weapon_hammer,
        s.weapon_staff,
        s.weapon_crossbow,
        s.weapon_fists,
        s.menu_fullscreen,
        s.value_on,
        s.value_off,
        s.value_secs,
        s.controls_title,
        s.controls_keyboard,
        s.controls_pad,
        s.controls_preset_mouse,
        s.controls_preset_keys,
        s.menu_mouse_aim,
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
        s.verb_swap,
        s.stage1_name,
        s.zone_entrance,
        s.zone_courtyard,
        s.zone_road,
        s.obj_find_gate,
        s.lever_verb,
        s.card_gate_opened,
        s.zone_sanctum,
        s.stage2_name,
        s.zone_village_edge,
        s.zone_bell_site,
        s.zone_root_alley,
        s.zone_lower_square,
        s.talk_verb,
        s.v01_a,
        s.v01_b,
        s.v01_c,
        s.v01_after,
        s.v02_a,
        s.v02_b,
        s.v02_c,
        s.v02_after,
        s.v03_a,
        s.v03_b,
        s.v03_c,
        s.v03_after,
        s.obj_bell_site,
        s.obj_descend,
        s.card_way_down,
        s.depth_under,
        s.depth_surface,
        s.intro_1,
        s.intro_2,
        s.obj_courtyard,
        s.obj_gate_open,
        s.obj_core,
        s.boss_warden,
        s.boss_wakes,
        s.charm_shell,
        s.charm_shell_d,
        s.charm_feet,
        s.charm_feet_d,
        s.charm_ear,
        s.charm_ear_d,
        s.charm_hunger,
        s.charm_hunger_d,
        s.charm_hand,
        s.charm_hand_d,
        s.charm_cloth,
        s.charm_cloth_d,
        s.charm_jar,
        s.charm_jar_d,
        s.charm_bellshard,
        s.charm_bellshard_d,
        s.charm_pouch,
        s.charm_pouch_d,
        s.enemy_husk,
        s.enemy_thorn,
        s.enemy_shell,
        s.enemy_pod,
        s.boss_bellkeeper,
        s.bellkeeper_wakes,
        s.clear_title,
        s.clear_sub,
        s.defeat_title,
        s.defeat_sub,
    ]
}

/// 표 밖에서 화면에 찍는 기호(메뉴 표지 등). 글꼴 시험이 같이 본다.
pub const UI_GLYPHS: &str = "◆◇…·▲▼▶◀→";

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
                // 개발 화면 글(0080)도 같은 글꼴로 그린다.
                let dev_texts = dev_texts();
                for text in all(l.strings())
                    .into_iter()
                    .chain([UI_GLYPHS])
                    .chain(dev_texts)
                {
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
