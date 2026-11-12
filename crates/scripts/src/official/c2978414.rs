// Number 46: Dragluon
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 2978414;
pub const XYZ_NUMBER: u32 = 46;

// A copy on the field with its own materials, since the effect needs it to be the only monster; Luster Dragon in the hand to Special
// Summon and one on the opponent's side to take.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (11091375, HAND), (17658803, MZONE | OPPONENT)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_DRAGON, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        !duel.is_existing_matching_card(aux::TRUE, a.tp, LOCATION_MZONE, 0, 1, Some(a.e.get_handler(duel).into()))
    });
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn ctrlfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_faceup(duel) && c.is_controler_can_be_changed(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(e.get_label(duel) == 2
            && chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, 1 - tp)
            && ctrlfilter(chkc, duel));
    }
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None);
    let b2 = duel.is_existing_target(ctrlfilter, tp, 0, LOCATION_MZONE, 1, None);
    let b3 = !duel.has_flag_effect(1 - tp, ID);
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let Some(op) = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?
    else {
        return Ok(false);
    };
    e.set_label(duel, op as i64);
    e.set_category(duel, 0);
    e.set_property(duel, 0);
    if op == 1 {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    } else if op == 2 {
        e.set_category(duel, CATEGORY_CONTROL);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
        let g = duel.select_target(tp, ctrlfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        2 => {
            let tc = duel.get_first_target();
            unported!(duel, tc.is_none());
            if let Some(tc) = tc
                && tc.is_relate_to_effect(duel, e)
                && tc.is_race(duel, RACE_DRAGON)
                && tc.is_faceup(duel)
            {
                duel.get_control(tc, tp)?;
            }
        }
        3 => {
            let opp = 1 - tp;
            if duel.has_flag_effect(opp, ID) {
                return Ok(());
            }
            duel.register_flag_effect(opp, ID, RESET_PHASE | PHASE_END | RESET_OPPO_TURN, 0, 1);
            let c = e.get_handler(duel);
            aux::register_client_hint_full(duel, c, 0, tp, 0, 1, Some(aux::stringid(ID, 4)), RESET_OPPO_TURN, 1);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
            e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
            e1.set_target_range(duel, 0, LOCATION_MZONE);
            e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_DRAGON));
            e1.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN, 1);
            duel.register_effect(e1, tp);
        }
        _ => {}
    }
    Ok(())
}
