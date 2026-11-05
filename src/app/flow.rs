//! 장면 흐름(0019) — 순수 함수. 앱은 여기서 정한 대로만 바꾼다(첫 게임 `app/flow.rs`, nail 0230 R15).
//! 흐름을 바꾸면 여기 시험을 더한다.
use crate::stage::StageOut;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scene {
    Title,
    Play,
    Pause,
    /// 스테이지가 끝나고 장면이 멈춰 보이는 동안(쓰러짐·종소리) — 이어서 `Result`.
    Ending {
        won: bool,
    },
    Result {
        won: bool,
    },
    /// 조작 설정(0024). 뒤로 = 설정(0052).
    Controls {
        from_pause: bool,
    },
    /// 장비(0064 M3): 무기 + 장신구 세 칸. 멈춤 "장비"에서, 뒤로 = 멈춤.
    Equip,
    /// 설정(0052, nail 0269의 틀): 언어·전체 화면·화면 흔들림·조작. `from_pause`면 뒤로 = 멈춤, 아니면 타이틀.
    Settings {
        from_pause: bool,
    },
    /// 개발 메뉴(0052, nail `Scene::Dev`): 문 앞에서 시작·PoC 시나리오 목록. 개발판에만.
    #[cfg(feature = "dev")]
    Dev,
    /// 개발 시나리오(PoC 아레나). 개발판에만.
    #[cfg(feature = "dev")]
    Sandbox,
}

impl Scene {
    /// 메뉴 입력을 받는 장면(터치는 탭만, 패드 십자키는 커서).
    pub fn is_menu(self) -> bool {
        matches!(
            self,
            Scene::Title
                | Scene::Pause
                | Scene::Equip
                | Scene::Result { .. }
                | Scene::Controls { .. }
                | Scene::Settings { .. }
        ) || self.is_dev()
    }

    #[cfg(feature = "dev")]
    fn is_dev(self) -> bool {
        self == Scene::Dev
    }
    #[cfg(not(feature = "dev"))]
    fn is_dev(self) -> bool {
        false
    }

    /// 월드가 도는 장면.
    pub fn ticks_world(self) -> bool {
        match self {
            Scene::Play => true,
            #[cfg(feature = "dev")]
            Scene::Sandbox => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TitleItem {
    /// 시작 — 기록이 있으면 "이어하기"(마지막으로 쉰 쉼터에서, 없으면 입구부터 + 남는 것 그대로, 0064 M2).
    Start,
    /// 새로 하기(기록이 있을 때만): 한 번 더 눌러야 기록을 지운다.
    NewGame,
    /// 설정(0052): 언어·전체 화면·흔들림·조작이 여기로 모였다(타이틀에 흩어져 있었다).
    Settings,
    /// 개발 메뉴(0052). 개발판에만.
    #[cfg(feature = "dev")]
    Dev,
}

/// 타이틀 줄. 설정은 늘 둘째·DEV는 셋째(헤드리스 시험의 길), 새로 하기는 기록이 있을 때 맨 끝에.
pub fn title_items(has_save: bool) -> Vec<TitleItem> {
    #[allow(unused_mut)]
    let mut v = vec![TitleItem::Start, TitleItem::Settings];
    #[cfg(feature = "dev")]
    v.push(TitleItem::Dev);
    if has_save {
        v.push(TitleItem::NewGame);
    }
    v
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PauseItem {
    Resume,
    /// 장비(0064 M3, 0051의 무기 줄): 좌우 = 다음 무기(전처럼), 확인 = 장비 창(무기 + 장신구 세 칸).
    Weapon,
    Settings,
    Restart,
    ToTitle,
}

pub const PAUSE_ITEMS: [PauseItem; 5] = [
    PauseItem::Resume,
    PauseItem::Weapon,
    PauseItem::Settings,
    PauseItem::Restart,
    PauseItem::ToTitle,
];

/// 설정 줄(0052). 값 줄은 좌우·확인으로 바꾸고, 누르는 줄은 확인으로 들어간다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsRow {
    Language,
    /// 쓸 수 있는 기기에서만(웹·데스크톱, 0046).
    Fullscreen,
    /// 화면 흔들림 0..=10.
    Shake,
    /// 마우스 조준 켬/끔·커서 쉼 시간·조준 표시 크기(0058) — 마우스가 있을 법한 기기에서만.
    MouseAim,
    CursorRest,
    Reticle,
    Controls,
    Back,
}

/// 조작 설정은 늘 끝에서 둘째(`tools/test/shot.js`가 위로 두 번 간다).
pub fn settings_rows(fullscreen: bool, mouse: bool) -> Vec<SettingsRow> {
    let mut v = vec![SettingsRow::Language];
    if fullscreen {
        v.push(SettingsRow::Fullscreen);
    }
    v.push(SettingsRow::Shake);
    if mouse {
        v.extend([
            SettingsRow::MouseAim,
            SettingsRow::CursorRest,
            SettingsRow::Reticle,
        ]);
    }
    v.extend([SettingsRow::Controls, SettingsRow::Back]);
    v
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResultItem {
    Retry,
    ToTitle,
}

pub const RESULT_ITEMS: [ResultItem; 2] = [ResultItem::Retry, ResultItem::ToTitle];

/// 스테이지가 이렇게 끝나면 어디로. 이어지는 중이면 그대로.
pub fn after_stage(out: StageOut) -> Option<Scene> {
    match out {
        StageOut::Playing | StageOut::Respawned => None,
        StageOut::Cleared => Some(Scene::Ending { won: true }),
        StageOut::Defeated => Some(Scene::Ending { won: false }),
    }
}

/// 끝 장면을 이만큼(초) 보여 준 뒤 결과 창.
pub const ENDING_TIME: f32 = 1.6;

/// 뒤로(B·Esc): 멈춤 → 계속, 조작 설정 → 온 곳, 결과·타이틀은 그대로.
pub fn back(scene: Scene) -> Scene {
    match scene {
        Scene::Pause => Scene::Play,
        Scene::Equip => Scene::Pause,
        Scene::Controls { from_pause } => Scene::Settings { from_pause },
        Scene::Settings { from_pause: true } => Scene::Pause,
        Scene::Settings { from_pause: false } => Scene::Title,
        #[cfg(feature = "dev")]
        Scene::Dev => Scene::Title,
        s => s,
    }
}

/// 멈춤 버튼: 싸움 중이면 멈춤, 멈춤이면 계속.
pub fn pause(scene: Scene) -> Scene {
    match scene {
        Scene::Play => Scene::Pause,
        Scene::Pause => Scene::Play,
        s => s,
    }
}

/// 이 장면에서 고르면: (다음 장면, 스테이지를 새로 세우나).
pub fn choose_pause(item: PauseItem) -> (Scene, bool) {
    match item {
        PauseItem::Resume => (Scene::Play, false),
        PauseItem::Weapon => (Scene::Equip, false),
        PauseItem::Settings => (Scene::Settings { from_pause: true }, false),
        PauseItem::Restart => (Scene::Play, true),
        PauseItem::ToTitle => (Scene::Title, true),
    }
}

pub fn choose_result(item: ResultItem) -> (Scene, bool) {
    match item {
        ResultItem::Retry => (Scene::Play, true),
        ResultItem::ToTitle => (Scene::Title, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finished_stage_shows_its_ending_before_the_result() {
        assert_eq!(after_stage(StageOut::Playing), None);
        assert_eq!(
            after_stage(StageOut::Cleared),
            Some(Scene::Ending { won: true })
        );
        assert_eq!(
            after_stage(StageOut::Defeated),
            Some(Scene::Ending { won: false })
        );
        assert!(!Scene::Ending { won: true }.ticks_world());
        assert!(!Scene::Ending { won: true }.is_menu());
    }

    #[test]
    fn pause_toggles_and_back_resumes() {
        assert_eq!(pause(Scene::Play), Scene::Pause);
        assert_eq!(pause(Scene::Pause), Scene::Play);
        assert_eq!(back(Scene::Pause), Scene::Play);
        assert_eq!(pause(Scene::Title), Scene::Title);
        assert_eq!(
            back(Scene::Result { won: false }),
            Scene::Result { won: false }
        );
    }

    #[test]
    fn menus_rebuild_the_stage_except_resume() {
        assert_eq!(choose_pause(PauseItem::Resume), (Scene::Play, false));
        assert_eq!(choose_pause(PauseItem::Restart), (Scene::Play, true));
        assert_eq!(choose_pause(PauseItem::ToTitle), (Scene::Title, true));
        assert_eq!(choose_result(ResultItem::Retry), (Scene::Play, true));
        assert_eq!(title_items(false)[0], TitleItem::Start);
        assert_eq!(title_items(false)[1], TitleItem::Settings);
        assert!(
            !title_items(false).contains(&TitleItem::NewGame),
            "nothing to erase"
        );
        // 기록이 있어도 시작(이어하기)·설정 자리는 그대로, 새로 하기는 그 뒤.
        assert_eq!(
            title_items(true)[..2],
            [TitleItem::Start, TitleItem::Settings]
        );
        assert_eq!(title_items(true).last(), Some(&TitleItem::NewGame));
        assert_eq!(
            choose_pause(PauseItem::Settings),
            (Scene::Settings { from_pause: true }, false),
            "the run is kept"
        );
        assert_eq!(choose_pause(PauseItem::Weapon), (Scene::Equip, false));
        assert_eq!(back(Scene::Equip), Scene::Pause);
        // 0052: 조작 → 설정 → 온 곳.
        assert_eq!(
            back(Scene::Controls { from_pause: true }),
            Scene::Settings { from_pause: true }
        );
        assert_eq!(back(Scene::Settings { from_pause: true }), Scene::Pause);
        assert_eq!(back(Scene::Settings { from_pause: false }), Scene::Title);
        assert!(Scene::Controls { from_pause: true }.is_menu());
        assert!(Scene::Settings { from_pause: false }.is_menu());
        assert!(!settings_rows(false, true).contains(&SettingsRow::Fullscreen));
        assert!(!settings_rows(true, false).contains(&SettingsRow::MouseAim));
        for (f, m) in [(false, false), (true, true), (true, false)] {
            let r = settings_rows(f, m);
            assert_eq!(r[r.len() - 2..], [SettingsRow::Controls, SettingsRow::Back]);
        }
    }
}
