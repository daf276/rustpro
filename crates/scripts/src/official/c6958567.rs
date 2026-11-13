// Witchcrafter Celebration
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 6958567;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_WITCHCRAFTER])
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (Some(fextra(duel, tp)), None)
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::shuffle_material);
}

// Witchcrafter Genni and Schmietta in the Graveyard as material for Witchcrafter Vice-Madame in the Extra Deck, or to destroy with an
// opponent's monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (64756282, MZONE),
    (64756282, GRAVE),
    (21744288, GRAVE),
    (9603252, EXTRA),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(9603252),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        duel.is_turn_player(a.tp)
            && duel.is_existing_matching_card(
                aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_WITCHCRAFTER])),
                a.tp,
                LOCATION_MZONE,
                0,
                1,
                None,
            )
    });
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel, opp: u8) -> bool {
    (c.is_set_card(duel, &[SET_WITCHCRAFTER]) && c.is_monster(duel) && c.is_faceup(duel)) || c.is_controler(duel, opp)
}

fn matfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_able_to_deck(duel) && c.is_faceup(duel)
}

fn fextra(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(aux::necro_valley_filter(matfilter), tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None)
}

fn controler(c: CardId, duel: &Duel) -> i64 {
    i64::from(c.get_controler(duel))
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, 1 - tp), tp, LOCATION_MZONE, LOCATION_ONFIELD, None);
    let b1 = g.len() >= 2 && aux::select_unselect_group_check(duel, g, 2, 2, &aux::dpcheck(controler));
    Fusion::new_summon_eff_closure(duel, e, false);
    let b2 = Fusion::summon_eff_tg::<Params>(duel, a, 0, None)?;
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_DESTROY);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 2, tp, 0);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
        Fusion::new_summon_eff_closure(duel, e, false);
        Fusion::summon_eff_tg::<Params>(duel, a, 1, None)?;
        duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        let g = duel.get_matching_group(|c, duel| desfilter(c, duel, 1 - tp), tp, LOCATION_MZONE, LOCATION_ONFIELD, None);
        if g.len() < 2 || g.get_class_count(duel, controler) < 2 {
            return Ok(());
        }
        let sg = aux::select_unselect_group(duel, g, 2, 2, &aux::dpcheck(controler), tp, HINTMSG_DESTROY)?;
        if sg.len() == 2 {
            duel.hint_selection(sg);
            duel.destroy(sg, REASON_EFFECT)?;
        }
    } else if op == 2 {
        Fusion::new_summon_eff_closure(duel, e, true);
        Fusion::summon_eff_op::<Params>(duel, a)?;
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::self_to_hand(duel, a)
}
