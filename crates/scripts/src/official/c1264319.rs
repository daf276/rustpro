// Gem-Knight Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 1264319;

pub const LISTED_SERIES: &[u16] = &[SET_GEM_KNIGHT];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_GEM_KNIGHT])
    }
}

// Gem-Knight Ruby in the Extra Deck, Fusion Summoned with Gem-Knight Garnet from the hand; a copy and Garnet in the GY to banish
// for the salvage.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (76614340, EXTRA),
    (76614340, EXTRA),
    (91731841, HAND),
    (91731841, HAND),
    (91731841, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(76614340), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::register_summon_eff::<Params>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_cost(duel, thcost);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GEM_KNIGHT])
        && c.is_monster(duel)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, thfilter, LOCATION_GRAVE | LOCATION_MZONE)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, c)?;
    }
    Ok(())
}
