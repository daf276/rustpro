// Crimson Blade Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 3294539;
const CARD_CRIMSON_BLADER: u32 = 80321197;

pub const LISTED_NAMES: &[u32] = &[CARD_CRIMSON_BLADER];
// In the Extra Deck, Synchro Summoned with Soul Resonator (Level 3 Tuner) and Luster Dragon (Level 4), with Dark Necrofear
// (Level 8, Special Summon only) to take and Summoned Skull (Level 6) of the opponent to destroy in battle, for which copies
// stand on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (62991792, MZONE),
    (62991792, MZONE),
    (62991792, MZONE),
    (11091375, MZONE),
    (11091375, MZONE),
    (11091375, MZONE),
    (31829185, DECK),
    (31829185, DECK),
    (31829185, GRAVE),
    (70781052, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_RESONATOR], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, thsptg);
    e1.set_operation(duel, thspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    c.is_level_above(duel, 8)
        && !c.is_summonable_card(duel)
        && (c.is_able_to_hand(duel) || (mmz_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        return Ok(duel.is_existing_matching_card(
            |c, duel| thspfilter(c, duel, e, tp, mmz_chk),
            tp,
            LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| thspfilter(c, duel, e, tp, mmz_chk)),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    aux::to_hand_or_else(
        duel,
        sc,
        tp,
        |_, duel| mmz_chk && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| {
            if duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)? {
                sc.negate_effects(duel, e.get_handler(duel), 0, false);
            }
            duel.special_summon_complete()?;
            Ok(())
        },
        aux::stringid(ID, 3),
    )
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    if chk == 0 {
        return Ok(bc.is_some_and(|bc| bc.is_faceup(duel) && bc.is_level_above(duel, 5)));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, bc.map(Into::into), 1, a.tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(bc) = a.e.get_handler(duel).get_battle_target(duel)
        && bc.is_relate_to_battle(duel)
    {
        duel.destroy(bc, REASON_EFFECT)?;
    }
    Ok(())
}
