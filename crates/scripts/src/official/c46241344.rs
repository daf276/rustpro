// Brotherhood of the Fire Fist - Eagle
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE, SZONE};

const ID: u32 = 46241344;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_BEASTWARRIOR, Some(lc), sumtype, tp)
    }
}

// Link Summoned from Brotherhood of the Fire Fist - Buffalo, with Fire Formation - Tenki on the field and in the Graveyard to return and Spirit in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (92572371, MZONE),
    (92572371, MZONE),
    (92572371, MZONE),
    (1662004, DECK),
    (1662004, DECK),
    (1662004, DECK),
    (1662004, DECK),
    (57103969, SZONE),
    (57103969, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.set_sp_summon_once(duel, ID);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_COST_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::CostReplace(repval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn repval(duel: &Duel, base: EffectId, call: CostReplaceCall) -> bool {
    let a = call.args;
    let c = a.e.get_handler(duel);
    c.is_set_card(duel, &[SET_FIRE_FIST]) && c.is_monster(duel) && (call.extracon)(duel, base, a)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel)
        && c.is_set_card(duel, &[SET_FIRE_FORMATION])
        && c.is_able_to_hand(duel)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_GRAVE))
}

fn gyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_FIRE_FIST]) && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_ONFIELD | LOCATION_GRAVE) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, None)
            && duel.is_existing_matching_card(gyfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? != 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g = duel.select_matching_card(tp, gyfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_grave(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
