// Uzuhime the Manifested Mikanko
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 57566760;

pub const LISTED_SERIES: &[u16] = &[SET_MIKANKO];
// In the Extra Deck, Xyz Summoned with two of four Ha-Re (Level 3 Mikanko), with Equip Spells in the Deck to add and the opponent's
// monsters to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (18377261, MZONE),
    (18377261, MZONE),
    (18377261, MZONE),
    (18377261, MZONE),
    (83746708, DECK),
    (83746708, DECK),
    (40619825, DECK),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_set_card_full(duel, &[SET_MIKANKO], Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_REFLECT_BATTLE_DAMAGE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_DAMAGE_STEP_END);
    e4.set_condition(duel, atcon);
    e4.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e4.set_operation(duel, atop);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_equip_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.get_attacker() == Some(c) && duel.check_mut(|duel| c.can_chain_attack_full(duel, 0, false))
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.e.get_handler(duel).is_relate_to_battle(duel) {
        duel.chain_attack();
    }
    Ok(())
}
