// Sorcerer of the Swamp
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, HAND};

const ID: u32 = 45223540;

// In hand with Cyber End Dragon in the Extra Deck to reveal and Cyber Dragons in the Deck to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (1546123, EXTRA), (CARD_CYBER_DRAGON, DECK), (CARD_CYBER_DRAGON, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn materials(c: CardId, duel: &Duel) -> Vec<u32> {
    Fusion::material(duel, c.get_original_code(duel))
}

fn banfilter(c: CardId, duel: &Duel, codes: &[u32]) -> bool {
    codes.iter().any(|&code| c.is_code(duel, code)) && c.is_able_to_remove(duel)
}

fn revealfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let codes = materials(c, duel);
    c.is_fusion_monster(duel)
        && !codes.is_empty()
        && duel.is_existing_matching_card(|c, duel| banfilter(c, duel, &codes), tp, LOCATION_DECK, 0, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(|c, duel| revealfilter(c, duel, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let Some(reveal_card) = duel
        .select_matching_card(tp, |c, duel| revealfilter(c, duel, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.confirm_cards(1 - tp, reveal_card)?;
    duel.shuffle_extra(tp)?;
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let codes = materials(reveal_card, duel);
    let banish_card = duel
        .select_matching_card(tp, |c, duel| banfilter(c, duel, &codes), tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    if let Some(banish_card) = banish_card
        && duel.remove(banish_card, POS_FACEUP, REASON_EFFECT)? > 0
        && c.is_relate_to_effect(duel, e)
    {
        duel.break_effect()?;
        if duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, aux::stringid(ID, 1));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_ADD_CODE);
            e1.set_value(duel, banish_card.get_original_code_rule(duel) as i32);
            e1.set_code_condition(duel, |_, _, sumtype, _| {
                sumtype & MATERIAL_FUSION > 0 || sumtype & u64::from(SUMMON_TYPE_FUSION) > 0
            });
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            c.register_effect(duel, e1);
        }
    }
    Ok(())
}
