// Light and Darkness Ritual
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 33599853;
const CARD_BLACK_LUSTER_SOLDIER_LIGHT_AND_DARKNESS: u32 = 70405001;
const CARD_BLACK_CHAOS: u32 = 44001993;

pub const LISTED_NAMES: &[u32] = &[
    CARD_BLACK_CHAOS,
    CARD_BLACK_LUSTER_SOLDIER_LIGHT_AND_DARKNESS,
    CARD_LIGHT_AND_DARKNESS_RITUAL,
];

struct Params;

impl RitualParams for Params {
    const DESC: u64 = aux::stringid(ID, 0);

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_code(duel, CARD_BLACK_CHAOS) || c.is_code(duel, CARD_BLACK_LUSTER_SOLDIER_LIGHT_AND_DARKNESS)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8) -> Option<CardSet> {
        Some(duel.get_matching_group(
            |c, duel| c.has_level(duel) && c.is_able_to_remove(duel),
            tp,
            LOCATION_GRAVE,
            0,
            None,
        ))
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, a.tp, LOCATION_GRAVE as i32);
        }
        Ok(true)
    }
}

// In the hand, Ritual Summoning Black Chaos with Stealth Angecradle, and in the GY with Black Chaos, which mentions it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_BLACK_CHAOS, HAND),
    (64413994, HAND),
    (ID, GRAVE),
    (CARD_BLACK_CHAOS, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_greater::<Params>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel) && duel.is_existing_matching_card(thfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let mut g = duel.select_matching_card(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    if !g.is_empty() {
        g.insert(c);
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
