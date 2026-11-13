// Cyberse Witch
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 11674673;
const CYNET_RITUAL: u32 = 34767865;

pub const LISTED_NAMES: &[u32] = &[CYNET_RITUAL];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

// Cyberse monsters (Protron, Bitron) to Link Summon with and to revive, a Spell to banish, and Cynet Ritual and Paladin of
// Storm Dragon in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (24154052, MZONE),
    (24154052, MZONE),
    (36211150, MZONE),
    (36211150, GRAVE),
    (12580477, GRAVE),
    (CYNET_RITUAL, DECK),
    (CYNET_RITUAL, DECK),
    (86401517, DECK),
    (86401517, DECK),
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
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        aux::zptgroupcon(duel, a.eg.unwrap_or_default(), aux::TRUE, a.e.get_handler(duel))
    });
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.get_flag_effect(a.tp, ID) > 0);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_able_to_remove_as_cost(duel)
}

fn thfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_RITUAL) && c.is_monster(duel) && c.is_race(duel, RACE_CYBERSE) && c.is_able_to_hand(duel)
}

fn thfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, CYNET_RITUAL)
        && c.is_able_to_hand(duel)
        && duel.is_existing_matching_card(thfilter2, tp, LOCATION_DECK, 0, 1, Some(c.into()))
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, cfilter, LOCATION_GRAVE)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| thfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_DECK as i32);
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g1 = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g1.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g2 = duel.select_matching_card(tp, thfilter2, tp, LOCATION_DECK, 0, 1, 1, None)?;
        let g1 = g1.union(g2);
        duel.send_to_hand(g1, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g1)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_CYBERSE) && c.is_level_below(duel, 4) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(&filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, &filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
