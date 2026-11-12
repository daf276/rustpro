// Overmind Archfiend
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 24221808;

// In the Extra Deck, Synchro Summoned with Krebons (Level 2 Psychic Tuner), PSY-Frame Driver (Level 6) and Chosen by the World
// Chalice (Level 3 Psychic), with Psychic monsters in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (59575539, MZONE),
    (22916281, MZONE),
    (22916281, MZONE),
    (22916281, GRAVE),
    (84754430, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_PSYCHIC, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_PSYCHIC, Some(sc), sumtype, tp))
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    // The Lua shares one group between both effects; here it lives on e1 and e2 reaches it through its label object.
    e1.set_label_object_group(duel, Some(CardSet::default()));
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, LOCATION_REMOVED>);
    e2.set_operation(duel, spop);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_PSYCHIC) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && chkc.is_controler(duel, tp) && rmfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? != 0
        && tc.is_location(duel, LOCATION_REMOVED)
        && c.is_relate_to_effect(duel, e)
    {
        let mut sg = e.get_label_object_group(duel).unwrap_or_default();
        if c.get_flag_effect(duel, ID) == 0 {
            sg.clear();
            c.register_flag_effect(
                duel,
                ID,
                RESET_EVENT | (RESETS_STANDARD & !(RESET_LEAVE | RESET_TEMP_REMOVE | RESET_TOGRAVE | RESET_TURN_SET)),
                0,
                1,
            );
        }
        sg.insert(tc);
        e.set_label_object_group(duel, Some(sg));
        tc.create_relation(duel, c, RESET_EVENT | RESETS_STANDARD);
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_ONFIELD) && c.get_flag_effect(duel, ID) != 0
}

fn spfilter(c: CardId, duel: &Duel, rc: CardId, e: EffectId, tp: u8) -> bool {
    c.is_relate_to_card(duel, rc) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = e
        .get_label_object(duel)
        .and_then(|e1| e1.get_label_object_group(duel))
        .unwrap_or_default();
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    let tg = g.filter(duel, |c, duel| spfilter(c, duel, e.get_handler(duel), e, tp), None);
    if ft <= 0 || tg.is_empty() {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = tg.select(duel, tp, ft as u8, ft as u8, None)?;
    duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}
