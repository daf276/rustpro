// Debris Dragon
use crate::prelude::*;
use crate::testing::{GRAVE, HAND};

const ID: u32 = 14943837;

// Psychic Kappa, a Normal Monster with 400 ATK, in the Graveyard to summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (7892180, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e4 = duel.create_effect(c);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_SYNCHRO_MAT_RESTRICTION);
    e4.set_value(duel, Value::Card(|duel, _, c| i32::from(c.get_level(duel) != 4)));
    c.register_effect(duel, e4);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_race(duel, RACE_DRAGON))));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_target(duel, sumtg);
    e3.set_operation(duel, sumop);
    c.register_effect(duel, e3);
}

fn filter2(c: CardId, duel: &Duel, e: EffectId, sp: u8) -> bool {
    c.is_attack_below(duel, 500) && c.is_can_be_special_summoned(duel, e, 0, sp, false, false)
}

fn sumtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter2(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter2(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter2(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_ATTACK)?
    {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect_full(duel, e1, true);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
