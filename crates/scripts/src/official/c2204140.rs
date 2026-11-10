// Book of Life
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 2204140;

// In the hand with Zombino (Zombie) in the Graveyard to Special Summon, and the opponent's Sangan on the field and in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (14575467, GRAVE),
    (14575467, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_ZOMBIE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let spfilter = |c: CardId, duel: &Duel| spfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(rmfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, None)
            && duel.is_existing_target(spfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g1 = duel.select_target(tp, spfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g1, 1, 0, 0);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g2 = duel.select_target(tp, rmfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g2, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg1 = duel.get_operation_info(0, CATEGORY_SPECIAL_SUMMON).and_then(|info| info.op_cards);
    let tg2 = duel.get_operation_info(0, CATEGORY_REMOVE).and_then(|info| info.op_cards);
    let (Some(tg1), Some(tg2)) = (tg1, tg2) else {
        unported!(duel, true);
        return Ok(());
    };
    if let Some(tc) = tg1.get_first()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_race(duel, RACE_ZOMBIE)
    {
        duel.special_summon(tg1, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    if tg2.get_first().is_some_and(|tc| tc.is_relate_to_effect(duel, e)) {
        duel.remove(tg2, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
