//! 첫 스테이지 "침묵의 성소"(docs/phase-1-first-stage.md). 남쪽(아래) 무너진 입구에서 출발 → 좁은 길 → 뿌리의 뜰(물결 셋) →
//! 문이 열리면 북쪽 닫힌 성소 → 잠든 문지기가 깨어난다 → 쓰러뜨리면 남는 핵을 거둔다.
//!
//! 지도 글자는 `Level::parse` 주석. 스폰 표지: `a` 첫 물결 잔재 · `h` 둘째 물결 잔재 · `T` 둘째 물결 가시짐승 ·
//! `c` 셋째 물결 잔재 · `U` 셋째 물결 가시짐승 · `W` 문지기. 한 칸 = 2m, 위 = 북(-z).
use super::StageDef;
use crate::species::Kind;

pub static DEF: StageDef = StageDef {
    name: |s| s.stage1_name,
    map: MAP,
    zones: &[
        (|s| s.zone_entrance, usize::MAX),
        (|s| s.zone_courtyard, 27),
        (|s| s.zone_sanctum, 12),
    ],
    waves: &[
        &[('a', Kind::Husk)],
        &[('h', Kind::Husk), ('T', Kind::Thorn)],
        &[('c', Kind::Husk), ('U', Kind::Thorn)],
    ],
    boss: ('W', Kind::Warden),
    intro: (|s| s.intro_1, |s| s.intro_2),
    obj_waves: |s| s.obj_courtyard,
    obj_gate: |s| s.obj_gate_open,
    obj_core: |s| s.obj_core,
    boss_wakes: |s| s.boss_wakes,
};

const MAP: &str = "
     ###############     
     #r.....L.....r#     
     #.b.........b.#     
     #.............#     
     #..i.......i..#     
     #.............#     
     #......W......#     
     #.............#     
     #..i.......i..#     
     #u...........u#     
     #.b.........b.#     
     #.............#     
##########GGGGG##########
#r.b......a...a......b.r#
#....................u..#
#..a...i.......i...c....#
#......w....h......T....#
#.u.r.............r.....#
#...........U...........#
#..h..............h..c..#
#.......c..T.....a......#
#..i....r.......r...i...#
#....u.....h.....u......#
#.c.................U...#
#.......i.......i....w..#
#..r..h.......c......r..#
#.b...................b.#
##########.....##########
         #b...b#         
         #.....#         
         #u...r#         
      ####.....####      
      #r.........u#      
      #...w...b...#      
      #b....P.....#      
      #..u......r.#      
      #############      
";
