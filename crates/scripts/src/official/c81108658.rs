// Ursarctic Megabilis
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 81108658;

pub const LISTED_SERIES: &[u16] = &[SET_URSARCTIC];
pub const LISTED_NAMES: &[u32] = &[ID];
// Blue-Eyes White Dragon in the hand to Tribute, Ursarctic Megapolar and Megatanus on the field, and cards in the opponent's
// Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (89631139, HAND),
    (89631139, HAND),
    (28715905, MZONE),
    (54700519, MZONE),
    (89631140, GRAVE | OPPONENT),
    (89631140, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ursarctic::add_sp_summon_quick_effect::<ID>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, bancon);
    e2.set_target(duel, bantg);
    e2.set_operation(duel, banop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_URSARCTIC])
}

fn bfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel)
}

fn bancon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, Some(a.e.get_handler(duel).into()))
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(bfilter(chkc, duel) && chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(bfilter, tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, bfilter, tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
