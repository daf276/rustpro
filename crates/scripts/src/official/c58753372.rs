// Super Quantal Fairy Alphan
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 58753372;

// Super Quant monsters in the Deck for the Special Summon: Blue Layer and White Layer, and another Alphan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, DECK),
    (12369277, MZONE),
    (12369277, DECK),
    (12369277, DECK),
    (73422829, DECK),
    (73422829, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn filter1(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.has_level(duel)
}

fn filter2(c: CardId, duel: &Duel, tp: u8) -> bool {
    filter1(c, duel)
        && c.is_set_card(duel, &[SET_SUPER_QUANT])
        && duel.is_existing_matching_card(filter1, tp, LOCATION_MZONE, 0, 1, Some(c.into()))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g1 = duel.get_matching_group(filter1, tp, LOCATION_MZONE, 0, None);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter2(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| filter2(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None)
                && g1.get_class_count(duel, |c, duel| i64::from(c.get_level(duel))) > 1,
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| filter2(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
        let g = duel.get_matching_group(filter1, tp, LOCATION_MZONE, 0, Some(tc.into()));
        let lv = tc.get_level(duel) as i32;
        for lc in g {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_CHANGE_LEVEL_FINAL);
            e1.set_value(duel, lv);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            lc.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SUPER_QUANT]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SUPER_QUANT]) && c.is_monster(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    if chk == 0 {
        let g = duel.get_matching_group(|c, duel| spfilter1(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
        return Ok(ft > 0 && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut g = duel.get_matching_group(spfilter2, tp, LOCATION_DECK, 0, None);
    if g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3 {
        let mut cg = CardSet::default();
        for _ in 0..3 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = g.select(duel, tp, 1, 1, None)?;
            if let Some(code) = sg.get_first().map(|sc| sc.get_code(duel)) {
                g = g.filter(duel, |c, duel| !c.is_code(duel, code), None);
            }
            cg.merge(sg);
        }
        duel.confirm_cards(1 - tp, cg)?;
        duel.shuffle_deck(tp)?;
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_SPSUMMON);
        let tg = cg.select(duel, 1 - tp, 1, 1, None)?;
        if let Some(tc) = tg.get_first()
            && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        {
            duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
            cg.remove(tc);
        }
        duel.send_to_grave(cg, REASON_EFFECT)?;
    }
    Ok(())
}
