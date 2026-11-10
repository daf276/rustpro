// Ancient Fairy Life Dragon
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, HAND, MZONE};

const ID: u32 = 43321985;
const WORLD_OF_SPIRITS: u32 = 5414777;
const ETERNAL_SUNSHINE: u32 = 28903523;

pub const LISTED_NAMES: &[u32] = &[WORLD_OF_SPIRITS, ETERNAL_SUNSHINE, CARD_ANCIENT_FAIRY_DRAGON];
// Two copies in the Extra Deck, Synchro Summoned with Angel Trumpeter (Tuner) and Prevent Rat on the field; World of Spirits and its Eternal Sunshine search target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (87979586, MZONE),
    (549481, MZONE),
    (5414777, HAND),
    (ETERNAL_SUNSHINE, DECK),
    (ETERNAL_SUNSHINE, DECK),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, drthtg);
    e1.set_operation(duel, drthop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_DEFENSE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| {
        c.is_code(duel, CARD_ANCIENT_FAIRY_DRAGON) || c.lists_code(duel, CARD_ANCIENT_FAIRY_DRAGON)
    });
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_BEAST | RACE_PLANT | RACE_FAIRY)) || c.is_code(duel, ETERNAL_SUNSHINE))
        && c.is_able_to_hand(duel)
}

fn world_of_spirits_on_field(duel: &Duel, tp: u8) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_code(duel, WORLD_OF_SPIRITS)),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        None,
    )
}

fn drthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let world_of_spirits_chk = world_of_spirits_on_field(duel, tp);
    let b1 = !world_of_spirits_chk && duel.is_player_can_draw(tp, 1);
    let b2 = world_of_spirits_chk && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    if b1 {
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    } else {
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    }
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn drthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !world_of_spirits_on_field(duel, tp) {
        duel.draw(tp, 1, REASON_EFFECT)?;
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
