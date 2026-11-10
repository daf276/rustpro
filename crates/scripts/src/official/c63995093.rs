// Machine Duplication
use crate::prelude::*;
use crate::testing::{DECK, MZONE};

const ID: u32 = 63995093;

// Unicycular (Normal Machine, 100 ATK) on the field and in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(57308711, MZONE), (57308711, DECK), (57308711, DECK), (57308711, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_attack_below(duel, 500)
        && c.is_race(duel, RACE_MACHINE)
        && duel.is_existing_matching_card(|sc, duel| filter2(sc, duel, c.get_code(duel), e, tp), tp, LOCATION_DECK, 0, 1, None)
}

fn filter2(c: CardId, duel: &Duel, code: u32, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 0));
    duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 {
        return Ok(());
    }
    ft = ft.min(2);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let code = tc.get_code(duel);
        let sg = duel.select_matching_card(tp, |c, duel| filter2(c, duel, code, e, tp), tp, LOCATION_DECK, 0, 1, ft as u8, None)?;
        if !sg.is_empty() {
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
