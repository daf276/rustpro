// Herald of Mirage Lights
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 46935289;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        !c.is_type_full(duel, TYPE_TOKEN, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_same_property(duel, sg, &|c, duel| c.get_race_full(duel, Some(lc), sumtype, tp)).0
            && aux::check_same_property(duel, sg, &|c, duel| u64::from(c.get_attribute_full(duel, Some(lc), sumtype, tp))).0
    }
}

// In the Extra Deck, Link Summoned with LIGHT Fairies (Mystical Shine Ball, Happy Lover), a Fairy in hand to discard, a Ritual
// Monster in the Graveyard to add and the opponent's Raigeki to negate and to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (39552864, MZONE),
    (39552864, MZONE),
    (99030164, MZONE),
    (99030164, MZONE),
    (1929294, HAND),
    (64631466, GRAVE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, discost);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    if a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    a.re.is_some_and(|re| re.is_spell_trap_effect(duel)) && duel.is_chain_negatable(a.ev as u8)
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FAIRY) && c.is_able_to_grave_as_cost(duel)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(disfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, disfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp && a.e.get_handler(duel).is_previous_controler(duel, a.tp)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel) && (c.is_ritual_monster(duel) || c.is_ritual_spell(duel))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.get_matching_group(thfilter, tp, LOCATION_GRAVE, 0, None);
    let sg = aux::select_unselect_group(duel, g, 1, 2, &aux::dncheck, tp, HINTMSG_ATOHAND)?;
    if !sg.is_empty() {
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
