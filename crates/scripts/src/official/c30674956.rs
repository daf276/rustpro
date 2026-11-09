// Wynn the Wind Charmer, Verdant
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 30674956;

// Flying Kamakiri #2 (WIND) x2 for the Link Summon, one more in the Deck to add and one in the opponent's Graveyard to Special
// Summon; Raigeki on the opponent's side destroys it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (3134241, MZONE),
    (3134241, MZONE),
    (3134241, DECK),
    (3134241, GRAVE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub(super) struct Procedure<const ATTRIBUTE: u32>;

impl<const ATTRIBUTE: u32> LinkProcedure for Procedure<ATTRIBUTE> {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_attribute_full(duel, ATTRIBUTE, Some(lc), sumtype, tp), 1, None)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register::<ID, ATTRIBUTE_WIND>(duel, c);
}

// The Charmers share their script apart from the code and Attribute.
pub(super) fn register<const CODE: u32, const ATTRIBUTE: u32>(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure<ATTRIBUTE>>(duel, c);
    c.enable_revive_limit(duel);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(CODE, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, CODE);
    e2.set_target(duel, sptg::<ATTRIBUTE>);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (CODE, 1));
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg::<ATTRIBUTE>);
    e3.set_operation(duel, thop::<ATTRIBUTE>);
    c.register_effect(duel, e3);
}

fn spfilter<const ATTRIBUTE: u32>(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_attribute(duel, ATTRIBUTE) && zone != 0 && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)
}

fn sptg<const ATTRIBUTE: u32>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let zone = e.get_handler(duel).get_free_linked_zone(duel) & ZONES_MMZ;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE)
            && chkc.is_controler(duel, 1 - tp)
            && spfilter::<ATTRIBUTE>(chkc, duel, e, tp, zone));
    }
    let filter = |c, duel: &Duel| spfilter::<ATTRIBUTE>(c, duel, e, tp, zone);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let zone = e.get_handler(duel).get_free_linked_zone(duel) & ZONES_MMZ;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && zone != 0
    {
        duel.special_summon_full(tc, 0, tp, tp, false, false, POS_FACEUP, zone)?;
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && a.rp == 1 - a.tp
        && c.is_link_summoned(duel)
        && c.is_previous_location(duel, LOCATION_MZONE)
}

fn thfilter<const ATTRIBUTE: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_defense_below(duel, 1500) && c.is_monster(duel) && c.is_attribute(duel, ATTRIBUTE) && c.is_able_to_hand(duel)
}

fn thtg<const ATTRIBUTE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter::<ATTRIBUTE>, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop<const ATTRIBUTE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter::<ATTRIBUTE>, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
