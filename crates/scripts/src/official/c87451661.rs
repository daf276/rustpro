// Scarred Dragon Archfiend
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 87451661;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(sc), sumtype, tp)
    }
}

pub const LISTED_NAMES: &[u32] = &[CARD_RED_DRAGON_ARCHFIEND];
// On the field with Mono Synchron and Hot Red Dragon Archfiend Abyss in the Extra Deck, a DARK Dragon Synchro Monster it
// is a material for, Red Dragon Archfiend in the Extra Deck, and the opponent's Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (56897896, MZONE),
    (9753964, EXTRA),
    (70902743, EXTRA),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = super::c80666118::name_change(duel, c);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_MZONE));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_operation(duel, matcheck);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let pg = aux::get_must_be_material_group(duel, tp, Some(CardSet::default()), tp, Some(c), None, REASON_SYNCHRO);
    pg.is_empty()
        && c.is_code(duel, CARD_RED_DRAGON_ARCHFIEND)
        && c.is_type(duel, TYPE_SYNCHRO)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_SYNCHRO, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    if e.get_handler(duel).has_flag_effect(duel, ID) {
        e.set_label(duel, 1);
        duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - tp, LOCATION_MZONE as i32);
    } else {
        e.set_label(duel, 0);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let Some(tc) = duel
        .select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    tc.set_material(duel, None);
    if duel.special_summon(tc, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    tc.complete_procedure(duel);
    if e.get_label(duel) == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(CardId::is_attack_pos, tp, 0, LOCATION_MZONE, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.break_effect()?;
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn matcheck(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if a.r & REASON_SYNCHRO == REASON_SYNCHRO
        && c.get_reason_card(duel)
            .is_some_and(|rc| rc.is_attribute(duel, ATTRIBUTE_DARK) && rc.is_race(duel, RACE_DRAGON))
    {
        c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
    }
    Ok(())
}
