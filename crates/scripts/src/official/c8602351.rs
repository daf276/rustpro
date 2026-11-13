// Reptilianne Echidna
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 8602351;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_race_full(duel, RACE_REPTILE, Some(lc), sumtype, tp), 1, None)
    }
}

// Link Summoned from two Reptilianne monsters (Reptiles) with Reptiles in the Deck to add and monsters to weaken on the opponent's field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (42303365, MZONE),
    (57647597, MZONE),
    (42303365, MZONE),
    (57647597, MZONE),
    (53776525, DECK),
    (82742611, DECK),
    (53776525, DECK),
    (6103114, MZONE | OPPONENT),
    (6103114, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_faceup(duel) && chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::has_non_zero_attack, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::has_non_zero_attack, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, 0);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn ctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_attack(duel) == 0
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_REPTILE) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(ctfilter, tp, 0, LOCATION_MZONE, None);
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if chk == 0 {
        return Ok(ct > 0 && aux::select_unselect_group_check(duel, g, 1, ct, &aux::dncheck));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_race(duel, RACE_REPTILE)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_race(duel, RACE_REPTILE));
    let ct = duel.get_matching_group_count(ctfilter, tp, 0, LOCATION_MZONE, None);
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if ct == 0 || g.is_empty() {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 1, ct, &aux::dncheck, tp, HINTMSG_ATOHAND)?;
    if !sg.is_empty() {
        duel.send_to_hand(sg, tp, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
