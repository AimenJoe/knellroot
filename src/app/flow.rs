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
    /// 조작 설정(0024). `from_pause`면 뒤로 = 멈춤, 아니면 타이틀.
    Controls {
        from_pause: bool,
    },
    /// 개발 시나리오(PoC 아레나). 개발판에만.
    #[cfg(feature = "dev")]
    Sandbox,
}

impl Scene {
    /// 메뉴 입력을 받는 장면(터치는 탭만, 패드 십자키는 커서).
    pub fn is_menu(self) -> bool {
        matches!(
            self,
            Scene::Title | Scene::Pause | Scene::Result { .. } | Scene::Controls { .. }
        )
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
    Start,
    Controls,
    Language,
    #[cfg(feature = "dev")]
    Sandbox,
    /// 개발: 문 앞에서 시작(보스만).
    #[cfg(feature = "dev")]
    SkipToGate,
}

pub fn title_items() -> Vec<TitleItem> {
    #[allow(unused_mut)]
    let mut v = vec![TitleItem::Start, TitleItem::Controls, TitleItem::Language];
    #[cfg(feature = "dev")]
    v.extend([TitleItem::SkipToGate, TitleItem::Sandbox]);
    v
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PauseItem {
    Resume,
    Controls,
    Restart,
    ToTitle,
}

pub const PAUSE_ITEMS: [PauseItem; 4] = [
    PauseItem::Resume,
    PauseItem::Controls,
    PauseItem::Restart,
    PauseItem::ToTitle,
];

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
        Scene::Controls { from_pause: true } => Scene::Pause,
        Scene::Controls { from_pause: false } => Scene::Title,
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
        PauseItem::Controls => (Scene::Controls { from_pause: true }, false),
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
        assert_eq!(title_items()[0], TitleItem::Start);
        assert_eq!(
            choose_pause(PauseItem::Controls),
            (Scene::Controls { from_pause: true }, false),
            "the run is kept"
        );
        assert_eq!(back(Scene::Controls { from_pause: true }), Scene::Pause);
        assert_eq!(back(Scene::Controls { from_pause: false }), Scene::Title);
        assert!(Scene::Controls { from_pause: true }.is_menu());
    }
}
