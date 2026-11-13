// Protector of The Agents - Moon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 90290572;

pub const LISTED_NAMES: &[u32] = &[CARD_SANCTUARY_SKY, 91188343];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_FAIRY, Some(lc), sumtype, tp)
    }
}

// Copies in the Extra Deck, Link Summoned from Key Mace (Level 1 Fairy Normal Monsters), with The Agent of Mystery - Earth and the Sanctuary in the Deck, and
// the opponent's Ash Blossom to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (1929294, MZONE),
    (1929294, MZONE),
    (1929294, MZONE),
    (91188343, DECK),
    (91188343, DECK),
    (56433456, DECK),
    (14558127, MZONE | OPPONENT),
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
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, ghtg);
    e1.set_operation(duel, ghop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

fn sanct(duel: &Duel) -> bool {
    duel.is_environment(CARD_SANCTUARY_SKY)
        || duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_SANCTUARY_SKY)),
            0,
            LOCATION_ONFIELD | LOCATION_GRAVE,
            LOCATION_ONFIELD | LOCATION_GRAVE,
            1,
            None,
        )
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_SANCTUARY_SKY) || c.lists_code(duel, CARD_SANCTUARY_SKY)) && c.is_able_to_grave(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, 91188343) && c.is_able_to_hand(duel)
}

fn ghtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None)
            || (sanct(duel) && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None)));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn ghop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let gg = duel.get_matching_group(tgfilter, tp, LOCATION_DECK, 0, None);
    let hg = duel.get_matching_group(aux::necro_valley_filter(thfilter), tp, LOCATION_DECK | LOCATION_GRAVE, 0, None);
    if !hg.is_empty() && sanct(duel) && (gg.is_empty() || duel.select_yes_no(tp, aux::stringid(ID, 2))?) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = hg.select(duel, tp, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    } else if !gg.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g = gg.select(duel, tp, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_grave(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn fairy(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FAIRY)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let dg = duel.get_matching_group(|c, duel| c.is_can_be_effect_target(duel, e), tp, 0, LOCATION_ONFIELD, None);
    let check = aux::release_check_target(dg);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, fairy, (1, 1), false, Some(&check), None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let g = duel.select_release_group_cost(tp, fairy, (1, 1), false, Some(&check), None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}
