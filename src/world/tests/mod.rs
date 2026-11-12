//! 월드 시험(0159 R3): `world.rs` 끝의 한 모듈(5,000줄)을 주제마다 파일로 나눴다 — 시험과 도우미의 몸은 그대로다.
//! 둘 이상의 주제가 쓰는 도우미만 여기에, 한 주제만 쓰는 도우미는 그 파일에 있다. 새 시험은 맞는 주제 파일에(없으면 파일 하나 + 아래 한 줄).
use super::*;

mod attack;
mod basis_lock;
mod dodge_stamina;
mod enemies;
mod flask;
mod growth;
mod heavy;
mod hurt;
mod jump_ledge;
mod parry;
mod ranged;
mod turn;

fn world_with_boss() -> World {
    let mut w = World::new("test", 20.0);
    w.spawn_boss(vec3(0.0, 0.0, -6.0));
    w
}

/// 패링 시험용: 잡몹 하나를 플레이어 바로 앞(-z)에 두고 내려치기를 막 휘두르게.
fn mob_swinging_in_front(t: &Tuning) -> World {
    let mut w = World::new("test", 20.0);
    w.spawn_mob(vec3(0.0, 0.0, -1.2));
    let en = &mut w.enemies[0];
    en.facing = 0.0; // +z = 플레이어 쪽
    en.atk = Atk::Smash;
    en.state = EState::Tele { t: en.tele_dur() };
    w.player.facing = std::f32::consts::PI; // -z = 적 쪽
    let _ = t;
    w
}

/// 기본 자세(화면 위를 봄)에서 카메라가 `cw`(요 +90°) 쪽으로 돌게 하는 버튼 — 위를 보면 왼쪽 버튼이 요 +90°다(0023).
fn turn(cw: bool) -> Actions {
    Actions {
        turn_left: cw,
        turn_right: !cw,
        ..Default::default()
    }
}

/// 시험용: 적이 쫓지도 치지도 않게 세워 둔다.
fn idle(w: &mut World) {
    for e in &mut w.enemies {
        e.state = EState::Recover { t: -1000.0 };
    }
}

/// 돌리고 멈추는(쏟는) 틱까지.
fn turn_until_spill(w: &mut World, cw: bool, t: &Tuning) -> Vec<Event> {
    let mut ev = w.step(&turn(cw), t);
    // 시작 휘청(0085)이 세워 둔 적을 깨우지 않게 — 이 시험들은 멈춘 적으로 쏟기만 본다.
    idle(w);
    for _ in 0..600 {
        if w.spill_at.is_none() {
            break;
        }
        ev.extend(w.step(&Actions::default(), t));
    }
    ev
}

/// 강공격 시험용: 누름 + 쥐기를 `ticks`틱, 그 뒤 놓고 `after`틱. 지나간 상태·사건을 모은다.
fn hold_attack(w: &mut World, t: &Tuning, ticks: usize, after: usize) -> (Vec<PState>, Vec<Event>) {
    let (mut st, mut ev) = (Vec::new(), Vec::new());
    for i in 0..ticks + after {
        let held = i < ticks;
        ev.extend(w.step(
            &Actions {
                attack: i == 0,
                attack_held: held,
                ..Default::default()
            },
            t,
        ));
        st.push(w.player.state);
    }
    (st, ev)
}

fn press(t: &Tuning, w: &mut World, a: Actions, ticks: usize) -> Vec<Event> {
    let mut ev = w.step(&a, t);
    for _ in 1..ticks {
        ev.extend(w.step(&Actions::default(), t));
    }
    ev
}

fn mob_with(w: &mut World, at: Vec3, hp: i32) -> usize {
    w.spawn_mob(at);
    let i = w.enemies.len() - 1;
    w.enemies[i].hp = hp;
    w.enemies[i].max_hp = hp;
    i
}

/// 0057 시험: (누름, 바라는 상태).
type Check = (Actions, fn(&PState) -> bool);

/// 0057: 누른 틱부터 몇 틱 뒤에 상태가 바뀌었나(못 바뀌면 None).
fn ticks_until(
    w: &mut World,
    t: &Tuning,
    a: Actions,
    want: fn(&PState) -> bool,
    max: usize,
) -> Option<usize> {
    w.step(&a, t);
    if want(&w.player.state) {
        return Some(0);
    }
    for i in 1..max {
        w.step(&Actions::default(), t);
        if want(&w.player.state) {
            return Some(i);
        }
    }
    None
}

fn hit_by_swing(w: &mut World, t: &Tuning) -> Vec<Event> {
    let i = w.enemies.len();
    w.spawn_mob(w.player.pos + vec3(0.0, 0.0, -1.0));
    w.enemies[i].facing = 0.0; // +z = 플레이어 쪽
    w.enemies[i].state = EState::Swing { t: 0.0 };
    let ev = w.step(&Actions::default(), t);
    w.enemies[i].state = EState::Dead;
    ev
}

fn up() -> Actions {
    Actions {
        move_dir: vec2(0.0, 1.0),
        ..Default::default()
    }
}
