//! 적: 잔불 떨구기, 종의 버릇(등딱지·꼬투리·청소자), 보스 2단계·종·울림 고리·조각, 알아차림과 끈, 등급·엘리트, 돌진 띠.
use super::*;

#[test]
fn a_fallen_enemy_drops_its_embers_once_and_they_come_to_the_player() {
    let mut w = World::new("t", 20.0);
    w.player.pos = Vec3::ZERO;
    let i = w.spawn(crate::species::Kind::Thorn, vec3(3.0, 0.0, 0.0));
    w.enemies[i].hp = 0;
    w.enemies[i].state = crate::enemy::EState::Dead;
    let t = crate::tuning::Tuning::default();
    let a = Actions::default();
    let mut picked = 0;
    for _ in 0..180 {
        for e in w.step(&a, &t) {
            if let Event::Ember { value } = e {
                picked += value;
            }
        }
    }
    let want = crate::species::def(crate::species::Kind::Thorn).embers;
    assert_eq!(picked, want, "every piece reaches the player");
    assert_eq!(w.embers, want);
    assert!(w.drops.is_empty());
    assert!(w.enemies[i].dropped, "dropped once, not every tick");
}

/// 0120: 등딱지짐승 — 앞에서 친 평타는 튕긴다(피해 0, 방랑자는 짧게 튄다), 뒤에서 친 평타·앞에서 친 강타는 들어간다.
#[test]
fn a_shellback_deflects_light_hits_from_the_front_but_not_heavies_or_from_behind() {
    let t = Tuning::default();
    let hit = |facing: f32, heavy: bool| {
        let mut w = World::new("test", 20.0);
        w.player.facing = std::f32::consts::PI; // -z = 앞
        let i = w.spawn(Kind::Shell, vec3(0.0, 0.0, -1.5));
        w.enemies[i].facing = facing;
        idle(&mut w);
        let hp = w.enemies[i].hp;
        let mut ev = w.step(
            &Actions {
                attack: true,
                attack_held: heavy,
                ..Default::default()
            },
            &t,
        );
        let mut recoiled = false;
        for _ in 0..90 {
            w.player.stamina = 100.0;
            let hold = Actions {
                attack_held: heavy
                    && !matches!(w.player.state, PState::Charging { t: e } if e > 0.8),
                ..Default::default()
            };
            ev.extend(w.step(&hold, &t));
            w.enemies[i].facing = facing;
            recoiled |= matches!(w.player.state, PState::Recoil { .. });
            if w.enemies[i].hp < hp {
                break;
            }
        }
        let deflected = ev.iter().any(|e| matches!(e, Event::Deflect { .. }));
        (w.enemies[i].hp < hp, deflected, recoiled)
    };
    assert_eq!(
        hit(0.0, false),
        (false, true, true),
        "front light: deflected, recoil"
    );
    assert_eq!(
        hit(std::f32::consts::PI, false).0,
        true,
        "light from behind lands"
    );
    assert_eq!(hit(0.0, true).0, true, "a heavy from the front lands");
}

/// 0120: 꼬투리는 가까이 오면 깡충 물러나고(다시 뛰기까지 쉼), 청소자는 보여도 먹기만 하다가(체력이 찬다) 맞으면 싸운다.
#[test]
fn pods_hop_away_and_scavengers_ignore_you_until_struck() {
    let t = Tuning::default();
    let mut w = World::new("test", 30.0);
    let i = w.spawn(Kind::Pod, vec3(0.0, 0.0, -2.0));
    w.step(&Actions::default(), &t);
    assert!(w.enemies[i].hop.is_some(), "too close: hop");
    for _ in 0..40 {
        w.step(&Actions::default(), &t);
    }
    assert!(
        w.enemies[i].pos.z < -4.0,
        "landed farther: {:?}",
        w.enemies[i].pos
    );
    assert!(w.enemies[i].hop_cd > 0.0);

    let mut w = World::new("test", 30.0);
    let s = w.spawn(Kind::Husk, vec3(0.0, 0.0, -4.0));
    w.enemies[s].place(0.0);
    w.enemies[s].docile = true;
    w.enemies[s].hp = 10;
    let steps = (crate::tuning::EAT_TIME * 2.2 / TICK_DT) as usize;
    for _ in 0..steps {
        w.step(&Actions::default(), &t);
    }
    assert_eq!(w.enemies[s].state, EState::Idle, "seen but not chased");
    assert_eq!(
        w.enemies[s].hp, 30,
        "ate back to full, one old point a bite"
    );
    w.enemies[s].hurt = 0.15;
    w.step(&Actions::default(), &t);
    assert_eq!(w.enemies[s].state, EState::Chase, "struck: fights");
}

/// 0133: 2단계 전환 — 맞아도 피해가 없고, 터지는 틱에 가까운 방랑자를 밀어낸다(피해 없이 튕긴 꼴로).
#[test]
fn a_boss_rise_takes_no_damage_and_its_burst_pushes_you_back_unhurt() {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    let b = w.spawn(Kind::Warden, vec3(0.0, 0.0, -2.6));
    w.enemies[b].hp = w.enemies[b].max_hp / 2;
    w.player.hp = 90;
    w.step(&Actions::default(), &t);
    assert!(w.enemies[b].rising());
    let hp = w.enemies[b].hp;
    let start = w.player.pos;
    let mut pushed = false;
    for _ in 0..(crate::tuning::RISE_TIME / TICK_DT) as usize {
        // 줄곧 쳐도(맞는 자리) 깎이지 않는다.
        w.step(
            &Actions {
                attack: true,
                ..Default::default()
            },
            &t,
        );
        if w.enemies[b].rising() {
            assert!(w.enemies[b].hp >= hp, "no damage during the rise");
        }
        pushed |=
            matches!(w.player.state, PState::Recoil { .. }) && w.player.pos.distance(start) > 0.05;
    }
    assert!(pushed, "the burst pushes the wanderer");
    assert_eq!(w.player.hp, 90, "the burst does not hurt");
    // 튕길 때마다의 히트스톱만큼 늦게 끝난다.
    for _ in 0..90 {
        w.step(&Actions::default(), &t);
    }
    assert!(!w.enemies[b].rising() && w.enemies[b].roused);
}

/// 0133: 종이 떨어진 자리의 고리는 뛰면 넘는다(떨어지는 순간의 원판은 아니다).
#[test]
fn the_hurled_bells_ring_is_cleared_by_a_jump() {
    let t = Tuning::default();
    let run = |jump: bool| -> i32 {
        let mut w = World::new("test", 40.0);
        let b = w.spawn(Kind::Bellkeeper, vec3(0.0, 0.0, -9.0));
        w.enemies[b].roused = true;
        w.enemies[b].atk = Atk::Hurl;
        w.enemies[b].plant_at = Some(vec3(0.0, 0.0, -4.0));
        w.enemies[b].state = EState::Wave { t: 0.0 };
        w.player.hp = 990;
        let hp = w.player.hp;
        for _ in 0..200 {
            w.player.hurt = 0.0;
            w.player.state = PState::Idle;
            if jump {
                w.player.air = 2.0;
            }
            w.step(&Actions::default(), &t);
            if !matches!(w.enemies[b].state, EState::Wave { .. }) {
                w.enemies[b].state = EState::Recover { t: -1000.0 };
            }
        }
        hp - w.player.hp
    };
    assert_eq!(run(false), 10, "the ring from the landing hits once");
    assert_eq!(run(true), 0, "jumped over");
}

/// 0125: 울림 고리는 고리마다 한 번씩 친다(반 아래면 하나 더) — 뛰면 넘고, 고리가 닿지 않는 멀리에서는 맞지 않는다.
#[test]
fn toll_rings_hit_once_each_and_a_jump_clears_them() {
    let t = Tuning::default();
    let run = |dist: f32, jump: bool, roused: bool| -> i32 {
        let mut w = World::new("test", 40.0);
        let b = w.spawn(Kind::Bellkeeper, vec3(0.0, 0.0, -dist));
        w.enemies[b].roused = roused;
        w.enemies[b].atk = Atk::Toll;
        w.enemies[b].state = EState::Wave { t: 0.0 };
        w.player.hp = 990;
        let hp = w.player.hp;
        for _ in 0..240 {
            w.player.hurt = 0.0;
            w.player.state = PState::Idle;
            if jump {
                w.player.air = 2.0;
            }
            w.step(&Actions::default(), &t);
            if !matches!(w.enemies[b].state, EState::Wave { .. }) {
                w.enemies[b].state = EState::Recover { t: -1000.0 };
            }
        }
        hp - w.player.hp
    };
    assert_eq!(run(3.0, false, false), 20, "two rings, one hit each");
    assert_eq!(run(3.0, false, true), 30, "roused: a third ring");
    assert_eq!(run(3.0, true, false), 0, "jumping clears every ring");
    assert_eq!(run(12.0, false, false), 0, "beyond the rings' reach");
}

/// 0125: 종 조각은 부채꼴(셋, 반 아래 다섯) — 앞에서 패링하면 종지기에게 되돌아가 깎고, 그냥 맞으면 아프다.
#[test]
fn a_parried_shard_flies_back_and_cuts_the_bellkeeper() {
    let t = Tuning::default();
    let shot = |w: &mut World, b: usize| {
        w.shards.push(Shard {
            pos: vec3(0.0, 1.1, -1.6),
            dir: vec3(0.0, 0.0, 1.0),
            went: 0.0,
            reflected: false,
            from: b,
        });
    };
    // 부채꼴: 휘두르는 첫 틱에 조각 수만큼.
    let mut w = World::new("test", 40.0);
    let b = w.spawn(Kind::Bellkeeper, vec3(0.0, 0.0, -8.0));
    w.enemies[b].atk = Atk::Shards;
    w.enemies[b].state = EState::Swing { t: 0.0 };
    w.step(&Actions::default(), &t);
    assert_eq!(w.shards.len(), 3);
    let mut w = World::new("test", 40.0);
    let b = w.spawn(Kind::Bellkeeper, vec3(0.0, 0.0, -8.0));
    w.enemies[b].roused = true;
    w.enemies[b].atk = Atk::Shards;
    w.enemies[b].state = EState::Swing { t: 0.0 };
    w.step(&Actions::default(), &t);
    assert_eq!(w.shards.len(), 5, "roused: two more");

    // 패링 → 되받아치기.
    let mut w = World::new("test", 40.0);
    let b = w.spawn(Kind::Bellkeeper, vec3(0.0, 0.0, -8.0));
    idle(&mut w);
    let hp = w.enemies[b].hp;
    shot(&mut w, b);
    w.player.state = PState::Parry { t: 0.0 };
    let mut parried = false;
    for _ in 0..90 {
        let ev = w.step(&Actions::default(), &t);
        parried |= ev.iter().any(|e| matches!(e, Event::Parry { .. }));
        idle(&mut w);
    }
    assert!(parried, "the parry turned it");
    assert_eq!(w.player.hp, w.player.max_hp, "unhurt");
    assert_eq!(w.enemies[b].hp, hp - crate::tuning::SHARD_REFLECT_DAMAGE);
    assert!(w.shards.is_empty());

    // 패링 없이: 맞는다.
    let mut w = World::new("test", 40.0);
    let b = w.spawn(Kind::Bellkeeper, vec3(0.0, 0.0, -8.0));
    idle(&mut w);
    let hp = w.player.hp;
    shot(&mut w, b);
    for _ in 0..30 {
        w.step(&Actions::default(), &t);
        idle(&mut w);
    }
    assert_eq!(w.player.hp, hp - 10);
    assert!(w.shards.is_empty());
}

/// 0125(KB ranged-boss-for-melee-player): 종지기가 쓰러지면 날던 조각도 사라진다 — 이긴 뒤에 맞지 않게.
#[test]
fn shards_vanish_when_the_bellkeeper_falls() {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    let b = w.spawn(Kind::Bellkeeper, vec3(0.0, 0.0, -8.0));
    w.enemies[b].atk = Atk::Shards;
    w.enemies[b].state = EState::Swing { t: 0.0 };
    w.step(&Actions::default(), &t);
    assert!(!w.shards.is_empty());
    w.enemies[b].state = EState::Dead;
    let hp = w.player.hp;
    for _ in 0..60 {
        w.step(&Actions::default(), &t);
    }
    assert!(w.shards.is_empty());
    assert_eq!(w.player.hp, hp);
}

/// 0113 알아차림: 제자리의 적은 시야 안(벽 너머 빼고)에 들면 쫓고, 벽 너머라도 아주 가까우면(발소리) 쫓는다.
#[test]
fn placed_enemies_notice_by_sight_but_not_through_walls_unless_close() {
    let t = Tuning::default();
    let notices = |at: Vec3, wall: bool| {
        let mut w = World::new("test", 30.0);
        if wall {
            w.obstacles.push(Aabb {
                min: vec3(-3.0, 0.0, -1.2),
                max: vec3(3.0, 3.0, -0.8),
            });
        }
        let i = w.spawn(Kind::Husk, at);
        w.enemies[i].place(0.0);
        w.step(&Actions::default(), &t);
        w.enemies[i].state != EState::Idle
    };
    assert!(notices(vec3(0.0, 0.0, -7.0), false), "in sight");
    assert!(!notices(vec3(0.0, 0.0, -7.0), true), "behind a wall");
    assert!(notices(vec3(0.0, 0.0, -2.5), true), "heard up close");
    assert!(
        !notices(vec3(0.0, 0.0, -SIGHT_RANGE - 2.0), false),
        "too far"
    );
}

/// 0113: 맞으면 깨고, 깨면 곁(`ALERT_RADIUS`)의 동료도 깬다 — 먼 것은 그대로.
#[test]
fn a_hit_wakes_the_enemy_and_its_close_friends() {
    let t = Tuning::default();
    let mut w = World::new("test", 40.0);
    let spots = [
        vec3(0.0, 0.0, -20.0),
        vec3(2.0, 0.0, -21.0),
        vec3(0.0, 0.0, -32.0),
    ];
    for at in spots {
        let i = w.spawn(Kind::Husk, at);
        w.enemies[i].place(0.0);
    }
    w.enemies[0].hurt = 0.15;
    w.step(&Actions::default(), &t);
    let st: Vec<_> = w.enemies.iter().map(|e| e.state).collect();
    assert_ne!(st[0], EState::Idle);
    assert_ne!(st[1], EState::Idle, "the friend next to it");
    assert_eq!(st[2], EState::Idle, "the far one sleeps on");
}

/// 0113: 제자리에서 끈보다 멀어지고 플레이어도 멀면 돌아가고, 닿으면 가득 찬 채 제 일로.
#[test]
fn a_leashed_enemy_walks_home_and_heals() {
    let t = Tuning::default();
    let mut w = World::new("test", 60.0);
    let home = vec3(0.0, 0.0, -30.0);
    let i = w.spawn(Kind::Husk, home);
    w.enemies[i].place(0.0);
    w.enemies[i].notice();
    w.enemies[i].pos = vec3(0.0, 0.0, -30.0 + LEASH_RANGE + 2.0);
    w.enemies[i].hp = 10;
    w.player.pos = vec3(0.0, 0.0, 20.0);
    w.step(&Actions::default(), &t);
    assert_eq!(w.enemies[i].state, EState::Return);
    for _ in 0..60 * 15 {
        w.step(&Actions::default(), &t);
        if w.enemies[i].state == EState::Idle {
            break;
        }
    }
    let e = &w.enemies[i];
    assert_eq!(e.state, EState::Idle);
    assert!(e.pos.distance(home) < 0.5);
    assert_eq!(e.hp, e.max_hp, "full again at home");
}

/// 0112 남는: 쓰러진 자리에 고리가 남고 `ELITE_LINGER_DELAY` 뒤 둘레를 친다 — 안이면 맞고, 밖이면 안 맞는다.
/// 0112·0116 T3: 등급 2 잡몹은 체력 ×1.5(옛 눈금으로 올림 — 잔재 30 → 50, 가시짐승 20 → 30, 0158 N0)·자세 ×1.3(커진 체력 몫에 더해)·
/// 잔불 ×1.3(올림, 0123), 한 대는 10 그대로 — 등급 3부터 20. 예고 길이는 같다.
#[test]
fn tier_two_mobs_are_sturdier_and_richer_but_hit_and_tell_the_same() {
    use crate::species::Kind;
    let t = Tuning::default();
    let make = |kind: Kind, tier: u8| {
        let mut w = World::new("test", 20.0);
        let i = w.spawn(kind, vec3(0.0, 0.0, -3.0));
        w.enemies[i].set_tier(tier);
        w
    };
    let (husk1, husk2) = (make(Kind::Husk, 1), make(Kind::Husk, 2));
    assert_eq!((husk1.enemies[0].max_hp, husk2.enemies[0].max_hp), (30, 50));
    assert_eq!(make(Kind::Thorn, 2).enemies[0].max_hp, 30);
    // 자세는 눈금을 옮기지 않았다: 체력 50 = 옛 다섯 칸 몫.
    let base = t[P::PoiseBase] + 5.0 * t[P::PoisePerHp];
    assert!((poise_max(&husk2.enemies[0], &t) - base * 1.3).abs() < 1e-3);
    // 잔불: 쓰러뜨려 떨군 값.
    let drop = |mut w: World| {
        w.enemies[0].hp = 0;
        w.enemies[0].state = EState::Dead;
        w.step(&Actions::default(), &t);
        w.drops.iter().map(|d| d.value).sum::<u32>()
    };
    // 0123: 잔재 1·가시짐승 2 × 1.3(올림) = 2·3.
    assert_eq!(drop(make(Kind::Husk, 2)), 2);
    assert_eq!(drop(make(Kind::Thorn, 2)), 3);
    // 보스는 등급을 받지 않는다.
    let mut w = World::new("test", 20.0);
    let b = w.spawn(Kind::Warden, vec3(0.0, 0.0, -3.0));
    let hp = w.enemies[b].max_hp;
    w.enemies[b].set_tier(2);
    assert_eq!((w.enemies[b].max_hp, w.enemies[b].tier), (hp, 1));
    // 한 대: 등급 2까지 10, 등급 3부터 20(0112).
    for (tier, want) in [(2u8, 10), (crate::tuning::TIER_HEAVY_HITS, 20)] {
        let mut w = World::new("test", 20.0);
        let i = w.spawn(Kind::Husk, vec3(0.6, 0.0, 0.0));
        w.enemies[i].set_tier(tier);
        let before = w.player.hp;
        for _ in 0..240 {
            w.step(&Actions::default(), &t);
            if w.player.hp < before {
                break;
            }
        }
        assert_eq!(before - w.player.hp, want, "tier {tier}");
    }
}

/// 엘리트는 잔불을 × `ELITE_EMBERS`, 굳은 엘리트의 자세 막대는 × `ELITE_HARD_POISE`.
#[test]
fn a_lingering_elite_leaves_a_ring_that_bursts_after_a_beat() {
    let t = Tuning::default();
    for (dist, hurt) in [(1.5, true), (ELITE_LINGER_RADIUS + 1.5, false)] {
        let mut w = World::new("test", 20.0);
        let i = w.spawn(Kind::Husk, vec3(dist, 0.0, 0.0));
        w.enemies[i].make_elite(Affix::Lingering);
        w.enemies[i].state = EState::Dead;
        w.step(&Actions::default(), &t);
        assert_eq!(w.lingers.len(), 1);
        let hp = w.player.hp;
        let mut burst = false;
        for _ in 0..((ELITE_LINGER_DELAY / TICK_DT) as usize + 5) {
            burst |= w
                .step(&Actions::default(), &t)
                .iter()
                .any(|e| matches!(e, Event::Boom { .. }));
        }
        assert!(burst && w.lingers.is_empty());
        assert_eq!(w.player.hp < hp, hurt, "at {dist} m");
    }
}

#[test]
fn elites_drop_more_embers_and_hardened_ones_hold_their_poise() {
    let t = Tuning::default();
    let drop = |elite: Option<Affix>| {
        let mut w = World::new("test", 20.0);
        let i = w.spawn(Kind::Husk, vec3(6.0, 0.0, 0.0));
        if let Some(a) = elite {
            w.enemies[i].make_elite(a);
        }
        w.enemies[i].state = EState::Dead;
        w.step(&Actions::default(), &t);
        w.drops.iter().map(|d| d.value).sum::<u32>()
    };
    assert_eq!(drop(Some(Affix::Repeating)), drop(None) * ELITE_EMBERS);
    let mut plain = Enemy::new(Kind::Husk, Vec3::ZERO);
    plain.make_elite(Affix::Repeating);
    let mut hard = Enemy::new(Kind::Husk, Vec3::ZERO);
    hard.make_elite(Affix::Hardened);
    assert!((poise_max(&hard, &t) - poise_max(&plain, &t) * ELITE_HARD_POISE).abs() < 1e-4);
}

/// 0076: 돌진 예고 띠 = 맞는 곳. 띠 가장자리 바로 안은 맞고 바로 밖은 안 맞는다(폭도 끝도).
#[test]
fn the_charge_lane_is_exactly_where_the_charge_hits() {
    let t = Tuning::default();
    // (옆 가장자리에서 바깥으로 m, 끝에서 앞으로 m — 둘 중 하나만, 둘 다 None이면 띠 가운데).
    for (side, along, hit) in [
        (None, None, true),
        (Some(-0.05), None, true),
        (Some(0.05), None, false),
        (None, Some(-0.05), true),
        (None, Some(0.05), false),
    ] {
        let mut w = World::new("test", 20.0);
        w.player.pos = vec3(0.0, 0.0, 0.0);
        let i = w.spawn(Kind::Thorn, vec3(0.0, 0.0, -6.5));
        // 예고가 굳을 때까지 돌리고 띠를 읽는다.
        let mut lane = None;
        for _ in 0..60 {
            w.step(&Actions::default(), &t);
            if let Some((_, TeleShape::Lane { len, w: lw })) = w.enemies[i].telegraph() {
                lane = Some((len, lw));
            }
            if matches!(w.enemies[i].state, EState::Charge { .. }) {
                break;
            }
        }
        let (len, lw) = lane.expect("a lane");
        // 플레이어를 띠 가장자리(옆) 또는 끝(앞)으로 옮겨 놓고 돌진을 맞는다.
        let en = &w.enemies[i];
        let f = en.facing_dir();
        let right = vec3(-f.z, 0.0, f.x);
        let at = match (side, along) {
            (Some(s), _) => en.pos + f * (len * 0.5) + right * (lw * 0.5 + s),
            (_, Some(a)) => en.pos + f * (len + a),
            _ => en.pos + f * (len * 0.5),
        };
        w.player.pos = at;
        w.player.prev = at;
        let mut got = false;
        for _ in 0..60 {
            let before = w.player.hp;
            w.step(&Actions::default(), &t);
            got |= w.player.hp < before;
            w.player.pos = at;
        }
        assert_eq!(got, hit, "side {side:?} along {along:?}: lane {len}×{lw}");
    }
}

/// 0076: 돌진은 기둥 앞에서 멈추고 예고 띠도 거기까지(띠가 기둥을 뚫고 그려지던 것).
#[test]
fn a_charge_stops_at_a_pillar_and_its_lane_says_so() {
    let t = Tuning::default();
    let mut w = World::new("test", 20.0);
    let i = w.spawn(Kind::Thorn, vec3(0.0, 0.0, -6.0));
    w.step(&Actions::default(), &t);
    assert!(matches!(w.enemies[i].state, EState::Tele { .. }));
    // 예고가 시작된 뒤 사이에 기둥이 생겼다(플레이어가 기둥 뒤로 숨은 것과 같다).
    w.obstacles.push(Aabb {
        min: vec3(-1.0, 0.0, -3.0),
        max: vec3(1.0, 3.0, -2.0),
    });
    w.step(&Actions::default(), &t);
    let Some((_, TeleShape::Lane { len, .. })) = w.enemies[i].telegraph() else {
        panic!("lane")
    };
    let room = 6.0 - 3.0 - w.enemies[i].radius;
    assert!(
        (len - (room + w.enemies[i].touch())).abs() < 0.1,
        "lane {len}"
    );
    for _ in 0..120 {
        w.step(&Actions::default(), &t);
    }
    assert!(
        w.enemies[i].pos.z < -3.0 + 0.01,
        "stopped before the pillar: {:?}",
        w.enemies[i].pos
    );
    assert_eq!(w.player.hp, w.player.max_hp, "never reached you");
}
