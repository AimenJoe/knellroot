//! 서막 "이름 없는 숲"(0262 — 사장 2026-11-20 "숲에서 일어나서 저 침묵의 성소에 들어가는 컷씬"): 새 기록의 방랑자가 깨어나는 곳.
//! 성소 바깥, 땅 위. 남쪽 깨어난 빈터(`P`) → 쓰러진 둥치를 넘는 턱(뛰기 0.5m) → 좁은 숲길 → 잔재 둘(`a` — 공격·피하기·받아치기를
//! 해 보는 상대, 끈이 짧다) → 북쪽 숲 끝(`E` — 옛 길의 기둥 둘이 표지, 밟으면 "성소에 들다" `arrive`).
//!
//! 지역 표(`STAGES`)의 **끝**에 붙고 판의 차례 밖이다(`StageDef::prologue`) — 기록의 지역 번호·상자 번호가 그대로다.
//! 문지기·핵·문·화로·상자·숨은 물건이 없다(기록 키를 늘리지 않는다). 쓰러지면 깨어난 자리에서 다시 선다. 종소리는 숲 끝에서.
//! 숲의 가장자리는 벽이 아니라 바위 단 + 나무(`flora`, 지역 환경 `backdrop::FOREST`의 `woods`).
//! 지도는 생성기(`tools/maps/forest.py`)가 MAP·HEIGHTS를 함께 쓴다. 한 칸 = 2m, 위 = 북(-z).
use super::StageDef;
use crate::species::Kind;

pub static DEF: StageDef = StageDef {
    name: |s| s.forest_name,
    map: MAP,
    heights: Some(HEIGHTS),
    zones: &[(|s| s.zone_woodpath, usize::MAX)],
    waves: &[],
    horde: (&[], 0),
    spawns: &[('a', Kind::Husk)],
    docile: &[],
    obj_explore: |s| s.obj_follow_bell,
    // 문지기 표지(`W`)가 지도에 없다 — 문지기는 서지 않는다.
    boss: ('W', Kind::Warden),
    intro: (|s| s.intro_1, |s| s.intro_2),
    obj_waves: |s| s.obj_follow_bell,
    obj_gate: |s| s.obj_follow_bell,
    obj_core: |s| s.obj_follow_bell,
    boss_wakes: |s| s.boss_wakes,
    chest_loot: &[],
    boss_charm: None,
    stones: &[],
    find_prefix: "forest",
    card_lever: |s| s.card_gate_opened,
    depth_m: 0,
    tier: 1,
    env: &crate::backdrop::FOREST,
    elites: &[],
    shortcut: false,
    prologue: true,
};

const MAP: &str = "
#                       #
            .            
          ..E..          
         .i...i.         
          .....          
           .....         
            ....         
             ....        
             ....        
           ....a.        
          .......        
         .......         
        ......           
       .....             
      .....              
     .......             
     .......             
     .....a.             
      .....              
       .....             
        .......          
         ......          
          .....          
          .....          
          .....          
          u...u          
          .....          
          .....          
         .......         
        .........        
       ..u........       
      .............      
      .............      
      .............      
       .....P...u.       
        .........        
         .......         
#                       #
#                       #
#                       #
";

/// 높낮이(0035·0043): `MAP`과 같은 칸. 둥치 북쪽의 땅은 +0.5m(뛰어 넘는 턱), 걷는 땅에서 멀어질수록 높아지는 바위 단(+0.5 → +2.5m) 위에 나무.
const HEIGHTS: &str = "
bbbbbbaa888666888aabbbbbb
bbbbbba88666566688abbbbbb
bbbbbba86655555668abbbbbb
bbbbbba86555555568abbbbbb
bbbbbba86655555668aabbbbb
bbbbbba886655555688abbbbb
bbbbbbaa88665555668abbbbb
bbbbbbbaa8866555568abbbbb
bbbbbbaa88666555568abbbbb
bbbbbaa886655555568abbbbb
bbbbaa8866555555568abbbbb
bbbaa88665555555668abbbbb
bbaa886655555566688abbbbb
bba886655555666888aabbbbb
bba8665555566888aaabbbbbb
bba86555555568aaabbbbbbbb
bba86555555568abbbbbbbbbb
bba86555555568aaaabbbbbbb
bba86655555668888abbbbbbb
bba88665555566668abbbbbbb
bbaa8866555555568abbbbbbb
bbbaa886655555568abbbbbbb
bbbbaa88665555568abbbbbbb
bbbbbaa8865555568abbbbbbb
bbbbbbaa865555568abbbbbbb
bbbbbb9975.....5799bbbbbb
bbbbb99775.....57799bbbbb
bbbb997755.....557799bbbb
bbb997755.......557799bbb
bbb97755.........55779bbb
bbb9755...........5579bbb
bbb975.............579bbb
bbb975.............579bbb
bbb975.............579bbb
bbb9755...........5579bbb
bbb97755.........55779bbb
bbb997755.......557799bbb
bbbb99775555555557799bbbb
bbbbb997777777777799bbbbb
bbbbbb9999999999999bbbbbb
";
