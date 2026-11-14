// D/D/D Sky King Zeus Ragnarok
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 30998403;

pub const LISTED_SERIES: &[u16] = &[SET_DD, SET_DARK_CONTRACT];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_DD], Some(lc), sumtype, tp)
    }
}

struct ExtraPendulumSummon;

impl ExtraPendulum for ExtraPendulumSummon {
    const DESC: u64 = aux::stringid(ID, 3);
    const OWNER: u32 = ID;

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_DD])
    }
}

// D/D/D Stone King Darius and Dark Contract with the Witch to destroy and banish, with the opponent's Effect Veiler to negate from the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (51497409, MZONE),
    (51497409, GRAVE),
    (9765723, SZONE),
    (9765723, GRAVE),
    (97268402, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, negcost);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, |duel, a| {
        duel.negate_activation(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DD, SET_DARK_CONTRACT])
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_set_card(duel, &[SET_DD, SET_DARK_CONTRACT])
            && chkc.is_on_field(duel)
            && chkc.is_controler(duel, tp)
            && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(!duel.has_flag_effect(tp, ID) && duel.is_existing_target(desfilter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, desfilter, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Pendulum::grant_additional_pendulum_summon::<ExtraPendulumSummon>(duel, e.get_handler(duel), tp, aux::stringid(ID, 2));
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let trig_loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    a.re.is_some_and(|re| re.is_monster_effect(duel))
        && a.ep == 1 - a.tp
        && trig_loc == LOCATION_HAND
        && duel.is_chain_negatable(a.ev as u8)
        && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
}

fn negcostfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_DARK_CONTRACT]) || (c.is_set_card(duel, &[SET_DD]) && c.is_monster(duel)))
        && c.is_able_to_remove_as_cost(duel)
}

fn rescon(duel: &Duel, sg: CardSet) -> (bool, bool) {
    (
        sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_DARK_CONTRACT]), 1, None)
            && sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_DD]) && c.is_monster(duel), 1, None),
        false,
    )
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(negcostfilter, tp, LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(g.len() >= 2 && aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg)));
    }
    let rg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg), tp, HINTMSG_REMOVE)?;
    duel.remove(rg, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    Ok(false)
}
