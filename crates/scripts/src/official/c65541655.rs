// Red Nova Dragon - Burning Soul
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 65541655;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN1: usize = 2;
    const MAX1: usize = 2;
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const LISTED_NAMES: &[u32] = &[CARD_RED_DRAGON_ARCHFIEND];

// In the Extra Deck with two Tuners and Red Dragon Archfiend in the Graveyard to banish, and Archfiend in the Extra Deck to Synchro Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA),
    (CARD_RED_DRAGON_ARCHFIEND, GRAVE),
    (17266660, GRAVE),
    (92919429, GRAVE),
    (14558127, MZONE),
    (4035199, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    c.add_must_be_synchro_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_condition(duel, spcon);
    e0.set_target(duel, sptg);
    e0.set_operation(duel, spop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e1.set_condition(duel, |duel, a| duel.has_flag_effect(a.tp, ID));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
        ge1.set_operation(duel, regop);
        duel.register_effect(ge1, 0);
    });
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_code(duel, EFFECT_MULTIPLE_TUNERS);
    c.register_effect(duel, e3);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_type(duel, TYPE_TUNER) || c.is_code(duel, CARD_RED_DRAGON_ARCHFIEND))
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn rescon(tp: u8) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| {
        (
            duel.get_mzone_count(tp, Some(sg)) > 0
                && sg.is_exists(duel, |c, duel| c.is_type(duel, TYPE_TUNER), 2, None)
                && sg.is_exists(duel, |c, duel| c.is_code(duel, CARD_RED_DRAGON_ARCHFIEND), 1, None),
            false,
        )
    }
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(_) = a.c else { return true };
    let tp = a.e.get_handler_player(duel);
    let g = duel.get_matching_group(spcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    g.len() >= 2 && duel.get_mzone_count(tp, Some(g)) > 0 && aux::select_unselect_group_check(duel, g, 3, 3, &rescon(tp))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(spcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    let sg = aux::select_unselect_group_full(duel, g, 3, 3, &rescon(tp), tp, HINTMSG_REMOVE, None, true)?;
    if sg.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(sg));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(sg) = a.e.get_label_object_group(duel) {
        duel.remove(sg, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_hand, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(a.e.get_handler(duel).into()), 1, tp, 2000);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let Some(sc) = duel
        .select_matching_card(tp, CardId::is_able_to_hand, tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.hint_selection(sc);
    let c = a.e.get_handler(duel);
    if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_HAND)
        && c.is_relate_to_effect(duel, a.e)
        && c.is_faceup(duel)
    {
        c.update_attack(duel, 2000, None, None, None);
    }
    Ok(())
}

fn regopfilter(c: CardId, duel: &Duel, sp: u8) -> bool {
    c.is_code(duel, CARD_RED_DRAGON_ARCHFIEND) && c.is_summon_player(duel, sp) && c.is_synchro_summoned(duel) && c.is_faceup(duel)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.has_flag_effect(0, ID) && duel.has_flag_effect(1, ID) {
        return Ok(());
    }
    for sp in 0..2 {
        if a.eg
            .is_some_and(|eg| eg.is_exists(duel, |c, duel| regopfilter(c, duel, sp), 1, None))
        {
            duel.register_flag_effect(sp, ID, 0, 0, 1);
        }
    }
    Ok(())
}
