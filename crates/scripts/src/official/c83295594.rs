// Steam Synchron
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 83295594;

// On the field with Level 3 Normal Monsters (Nekogal #1) for Iron Chain Dragon (Level 6) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (1761063, MZONE),
    (1761063, MZONE),
    (19974580, EXTRA),
    (19974580, EXTRA),
    (19974580, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(19974580)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, sccon);
    e1.set_target(duel, sctarg);
    e1.set_operation(duel, scop);
    c.register_effect(duel, e1);
}

fn sccon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_CHAINING) && !duel.is_turn_player(a.tp) && duel.is_main_phase()
}

fn sctarg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), None),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_controler(duel, 1 - tp) || !c.is_relate_to_effect(duel, a.e) || c.is_facedown(duel) {
        return Ok(());
    }
    let g = duel.get_matching_group(
        |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), None),
        tp,
        LOCATION_EXTRA,
        0,
        None,
    );
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        if let Some(sc) = sg.get_first() {
            duel.synchro_summon(tp, sc, Some(c.into()), None)?;
        }
    }
    Ok(())
}
