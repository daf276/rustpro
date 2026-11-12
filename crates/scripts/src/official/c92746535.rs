// Luster Pendulum, the Dracoslayer
use crate::prelude::*;
use crate::testing::{Check, DECK, PZONE};

const ID: u32 = 92746535;

// Two copies in the Pendulum Zones, and copies in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (ID, DECK), (ID, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, thcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_FUSION_MATERIAL);
    e2.set_value(duel, Value::Card(splimit));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_code(duel, EFFECT_CANNOT_BE_XYZ_MATERIAL);
    c.register_effect(duel, e4);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(aux::TRUE, a.tp, LOCATION_PZONE, 0, 1, Some(a.e.get_handler(duel).into()))
}

fn thfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_code(duel, code) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(sc) = duel.get_first_matching_card(aux::TRUE, tp, LOCATION_PZONE, 0, Some(a.e.get_handler(duel).into())) else {
        return Ok(false);
    };
    let code = sc.get_code(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| thfilter(c, duel, code), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_target_card(sc);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sc.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.destroy(tc, REASON_EFFECT)? != 0
    {
        let code = tc.get_code(duel);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}

fn splimit(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    i32::from(!c.is_set_card(duel, &[SET_DRACOSLAYER]))
}
