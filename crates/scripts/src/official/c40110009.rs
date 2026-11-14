// Dragonmaid Changeover
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 40110009;

pub const LISTED_SERIES: &[u16] = &[SET_DRAGONMAID];

const HOUSE_DRAGONMAID: u32 = 41232647;
const KITCHEN_DRAGONMAID: u32 = 16960120;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_DRAGON)
    }
}

// In the hand to Fusion Summon House Dragonmaid from Kitchen Dragonmaids, and in the GY with Kitchen Dragonmaid on the
// field to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (HOUSE_DRAGONMAID, EXTRA),
    (KITCHEN_DRAGONMAID, HAND),
    (KITCHEN_DRAGONMAID, HAND),
    (KITCHEN_DRAGONMAID, MZONE),
    (KITCHEN_DRAGONMAID, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummonedCode(HOUSE_DRAGONMAID),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::register_summon_eff::<Params>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DRAGONMAID]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel) && duel.is_existing_target(thfilter, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e)
        && duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_HAND)
        && let Some(tc) = tc
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
