// Mitsurugi Ritual
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 81560239;

struct Params1;

impl RitualParams for Params1 {
    const LOCATION: Option<u32> = Some(LOCATION_DECK);

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_REPTILE)
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_race(duel, RACE_REPTILE)
    }
}

struct Params2;

impl RitualParams for Params2 {
    const FORCEDSELECTION: Option<ForcedSelection> = Some(|_, _, _, g, _| (g.len() <= 2, g.len() > 2));

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_REPTILE)
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_race(duel, RACE_REPTILE)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8) -> Option<CardSet> {
        Some(duel.get_matching_group(deckmatfilter, tp, LOCATION_DECK, 0, None))
    }

    fn extraop(duel: &mut Duel, mat: CardSet, _: EffectId, _: u8, _: CardId) -> Script<()> {
        duel.release_ritual_material_full(mat, true)
    }
}

// Ritual Monsters in the hand and Deck, with Aramasa on the field and Gogiga Gagagigo (Level 8 Reptile Normal Monster) in
// the Deck and on the field to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (19899073, HAND),
    (13332685, HAND),
    (19899073, DECK),
    (40543231, MZONE),
    (40543231, MZONE),
    (39674352, DECK),
    (39674352, MZONE),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(19899073)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn deckmatfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_REPTILE) && c.is_releasable_by_effect(duel)
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    a.e.set_label(duel, -100);
    let b1 = !duel.has_flag_effect(tp, ID) && Ritual::target::<Params1, RITPROC_EQUAL>(duel, a, 0, None)?;
    let b2 = !duel.has_flag_effect(tp, ID + 1) && Ritual::target::<Params2, RITPROC_EQUAL>(duel, a, 0, None)?;
    Ok(chk == 0 && (b1 || b2))
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let cost_skip = e.get_label(duel) != -100;
    let b1 = (cost_skip || !duel.has_flag_effect(tp, ID)) && Ritual::target::<Params1, RITPROC_EQUAL>(duel, a, 0, None)?;
    let b2 = (cost_skip || !duel.has_flag_effect(tp, ID + 1)) && Ritual::target::<Params2, RITPROC_EQUAL>(duel, a, 0, None)?;
    if chk == 0 {
        e.set_label(duel, 0);
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        if !cost_skip {
            duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        }
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
        duel.set_operation_info(0, CATEGORY_RELEASE, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    } else if op == Some(2) {
        if !cost_skip {
            duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        }
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
        duel.set_operation_info(
            0,
            CATEGORY_RELEASE,
            None,
            1,
            tp,
            (LOCATION_HAND | LOCATION_MZONE | LOCATION_DECK) as i32,
        );
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    match a.e.get_label(duel) {
        1 => Ritual::operation::<Params1, RITPROC_EQUAL>(duel, a),
        2 => Ritual::operation::<Params2, RITPROC_EQUAL>(duel, a),
        _ => Ok(()),
    }
}
