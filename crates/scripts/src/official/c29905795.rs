// The Fabled Chawa
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 29905795;

// Copies in the hand with Fabled Lurrie to discard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (97651498, HAND), (97651498, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FABLED]) && c.is_monster(duel) && c.is_discardable(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(disfilter, tp, LOCATION_HAND, 0, 1, Some(c.into()))
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g = duel.get_matching_group(disfilter, tp, LOCATION_HAND, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    if g.len() == 1 {
        duel.send_to_grave(g, REASON_EFFECT | REASON_DISCARD)?;
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
        let sg = g.select(duel, tp, 1, 1, Some(c.into()))?;
        duel.send_to_grave(sg, REASON_EFFECT | REASON_DISCARD)?;
    }
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}
