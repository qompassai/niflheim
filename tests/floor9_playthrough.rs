use niflheim::game::{El, Game, Screen, Verb, geom, hit_test, layout};
use niflheim::{puzzles, state::Expedition};
fn click(g: &mut Game, el: El) {
    let r = layout(g).into_iter().find(|(e, _)| *e == el).unwrap().1;
    assert_eq!(hit_test(g, r.cx(), r.cy()), Some(el));
    g.pointer_down(r.cx(), r.cy());
    g.pointer_up(r.cx(), r.cy());
}
fn repair(g: &mut Game) {
    click(g, El::Chip(Verb::Prove));
    click(g, El::Seal);
    assert!(g.seal_armed());
    click(g, El::SealConfirm);
}
pub fn earn(g: &mut Game) {
    click(g, El::Floor9Door(0));
    click(g, El::F9Watch);
    for (i, pair) in puzzles::hoard_shelf_trace().0.iter().enumerate() {
        for (f, n) in pair.iter().enumerate() {
            for _ in 0..*n {
                click(g, El::F9Prediction(i as u8, f as u8));
            }
        }
    }
    click(g, El::F9Judge);
    repair(g);
    click(g, El::F9Return);
    click(g, El::Floor9Door(1));
    click(g, El::F9Watch);
    let (slots, nexts) = puzzles::hoard_chain_indices();
    for i in 0..6 {
        for _ in 0..slots[i] {
            click(g, El::F9Slot(i as u8));
        }
        while g.expedition.floor9.chambers[1].work.nexts[i] != nexts[i] {
            click(g, El::F9Next(i as u8));
        }
    }
    repair(g);
    click(g, El::F9Return);
    click(g, El::Floor9Door(2));
    click(g, El::F9Place);
    let (probes, flood) = puzzles::hoard_cellar_trace();
    for (i, n) in probes.into_iter().enumerate() {
        for _ in 0..n {
            click(g, El::F9Probe(i as u8));
        }
    }
    for _ in 0..flood {
        click(g, El::F9FloodCost);
    }
    click(g, El::F9Flood);
    click(g, El::F9Judge);
    repair(g);
    click(g, El::F9Return);
    click(g, El::Floor9Door(3));
    // Earn a miss, prove that a wrong name keeps the lock, load it back,
    // then attribute the true growth operation and re-predict exactly.
    click(g, El::F9StewardRun);
    let operation_label = |g: &Game| {
        niflheim::game::floor9_controls(g, 3)
            .into_iter()
            .find(|(el, _, _)| *el == El::F9StewardCycle)
            .unwrap()
            .2
    };
    let true_label = format!(
        "Name operation: {} (cycle)",
        niflheim::content::STEWARD_TRAP_OP + 1
    );
    if operation_label(g) == true_label {
        click(g, El::F9StewardCycle);
    }
    click(g, El::F9StewardName);
    assert!(g.expedition.floor9.chambers[3].work.steward_missed);
    let before = g.expedition.floor9.chambers[3].work.clone();
    click(g, El::F9StewardPrediction);
    assert_eq!(before, g.expedition.floor9.chambers[3].work);
    g.expedition = Expedition::load_json(&g.expedition.save_json().unwrap()).unwrap();
    while operation_label(g) != true_label {
        click(g, El::F9StewardCycle);
    }
    click(g, El::F9StewardName);
    for _ in 0..puzzles::hoard_steward_run().1 {
        click(g, El::F9StewardPrediction);
    }
    click(g, El::F9StewardRun);
    repair(g);
    click(g, El::F9Return);
    click(g, El::Floor9Door(4));
    click(g, El::F9Watch);
    click(g, El::F9Suite);
    for (i, h) in puzzles::hoard_intern_transcript(false)
        .0
        .into_iter()
        .enumerate()
    {
        while g.expedition.floor9.chambers[4].work.blocks[i] != h {
            click(g, El::F9Block(i as u8));
        }
    }
    while usize::from(g.expedition.floor9.chambers[4].work.last_mile) != puzzles::hoard_last_mile()
    {
        click(g, El::F9LastMile);
    }
    click(g, El::F9Find);
    repair(g);
    click(g, El::F9Return);
    click(g, El::DoorFoe9);
    use niflheim::game::HoardAction::*;
    click(g, El::F9Foe(Exhibit(0)));
    for (i, c) in puzzles::hoard_census().0.into_iter().enumerate() {
        for _ in 0..c {
            click(g, El::F9Foe(Class(i as u8)));
        }
    }
    click(g, El::F9Foe(Exhibit(1)));
    click(g, El::F9Foe(Exhibit(2)));
    click(g, El::Chip(Verb::Prove));
    click(g, El::F9Foe(Page(1)));
    for i in 0..3 {
        click(g, El::F9Foe(Trial(i)));
        click(g, El::F9Foe(Adversary(i)));
    }
    click(g, El::F9Foe(Page(2)));
    for i in 0..3 {
        click(g, El::F9Foe(Reissue(i)));
        assert!(g.expedition.floor9.foe.reissued[usize::from(i)].is_none());
        click(g, El::F9Foe(Stamp(i)));
        click(g, El::F9Foe(Reissue(i)));
    }
    click(g, El::Seal);
    assert!(g.seal_armed());
    click(g, El::SealConfirm);
    assert!(g.expedition.floor9_complete());
}

#[test]
fn whole_floor_pointer_playthrough_earns_v9_and_roundtrips() {
    let mut g = Game::new();
    g.begin();
    g.io = None;
    g.expedition = Expedition::load_json(include_str!("fixtures/floor7-earned.json")).unwrap();
    g.screen = Screen::Floor5Hub;
    let r = geom::door_floor9();
    g.pointer_down(r.cx(), r.cy());
    g.pointer_up(r.cx(), r.cy());
    assert_eq!(g.screen, Screen::Floor9Hub);
    earn(&mut g);
    assert_eq!(g.expedition.floor9_chambers_repaired(), 5);
    let save = g.expedition.save_json().unwrap();
    let loaded = Expedition::load_json(&save).unwrap();
    assert_eq!(loaded, g.expedition);
    assert_eq!(loaded.save_json().unwrap(), save);
    assert!(g.expedition.floor9.chambers.iter().all(|p| p.facet));
}

#[test]
fn premature_repair_and_dependent_reset_cannot_preserve_completion() {
    let mut g = Game::new();
    g.begin();
    g.io = None;
    g.expedition = Expedition::load_json(include_str!("fixtures/floor7-earned.json")).unwrap();
    g.screen = Screen::Floor5Hub;
    click(&mut g, El::DoorFloor9);
    click(&mut g, El::Floor9Door(0));
    click(&mut g, El::Chip(Verb::Prove));
    click(&mut g, El::Seal);
    assert!(!g.seal_armed());
    assert_eq!(g.expedition.floor9_chambers_repaired(), 0);
    click(&mut g, El::F9Return);
    earn(&mut g);
    click(&mut g, El::F9Return);
    click(&mut g, El::Floor9Door(0));
    click(&mut g, El::ResetChamber);
    assert_eq!(g.expedition.floor9_chambers_repaired(), 0);
    assert!(!g.expedition.floor9_complete());
    assert_eq!(g.expedition.floor9.foe, Default::default());
    let saved = g.expedition.save_json().unwrap();
    g.expedition = Expedition::load_json(&saved).unwrap();
    click(&mut g, El::F9Return);
    earn(&mut g);
    assert!(g.expedition.floor9_complete());
}
