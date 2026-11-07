// Darklord Desire
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 55690251;

struct Procedure;

impl NormalProcedure for Procedure {
    const NS: bool = true;
    const OPT: bool = true;
    const MIN: i32 = 1;
    const MAX: i32 = 1;
    const DESC: Option<u64> = Some(aux::stringid(ID, 0));

    fn f(c: CardId, duel: &Duel, tp: u8) -> bool {
        c.is_race(duel, RACE_FAIRY) && (c.is_controler(duel, tp) || c.is_faceup(duel))
    }
}

// In the hand twice, to be Tribute Summoned or Set with a Fairy on the field: Medius the Pure.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (97556336, MZONE)];
pub const CHECKS: &[Check] = &[Check::TributeSummoned(1), Check::TributeSet(1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    Normal::add_summon_procedure::<Procedure>(duel, c);
    Normal::add_set_procedure::<Procedure>(duel, c);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TOGRAVE);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_count_limit(duel, 1, 0);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target(duel, target);
    e4.set_operation(duel, operation);
    c.register_effect(duel, e4);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.get_controler(duel) != tp && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_able_to_grave(duel));
    }
    let filter = |c: CardId, duel: &Duel| c.is_able_to_grave(duel);
    if chk == 0 {
        return Ok(c.get_attack(duel) >= 1000 && duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, count, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    if c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && c.update_attack(duel, -1000, None, None, None) == -1000
        && let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
    {
        duel.send_to_grave(tc, REASON_EFFECT)?;
    }
    Ok(())
}
