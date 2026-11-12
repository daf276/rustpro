// Antidote Nurse
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 31539614;

// In the Extra Deck, Xyz Summoned with Level 3 Normal Monsters (Nekogal #1, Mountain Warrior, Swordsman of Landstar), with Megamorph in
// hand to change a monster's ATK, and the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (1761063, MZONE),
    (1761063, MZONE),
    (4931562, MZONE),
    (4931562, MZONE),
    (3573512, MZONE),
    (22046459, HAND),
    (22046459, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_condition(duel, |duel, _| !(duel.is_phase(PHASE_DAMAGE) && duel.is_damage_calculated()));
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, changeatkdeftg);
    e1.set_operation(duel, changeatkdefop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| {
        let eg = a.eg.unwrap_or_default();
        !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, mine_faceup(a.tp), 1, None)
    });
    e2.set_target(duel, gainatktg);
    e2.set_operation(duel, gainatkop);
    c.register_effect(duel, e2);
}

fn changeatkdeffilter(c: CardId, duel: &Duel) -> bool {
    !(c.is_attack(duel, &[c.get_base_attack(duel)]) && c.is_defense(duel, &[c.get_base_defense(duel)]))
}

fn changeatkdeftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && changeatkdeffilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(changeatkdeffilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    let g = duel.select_target(tp, changeatkdeffilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    let mine = g.and_then(|g| g.get_first()).is_some_and(|tc| tc.is_controler(duel, tp));
    e.set_label(duel, i64::from(mine));
    Ok(false)
}

fn changeatkdefop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel)) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e1.set_value(duel, tc.get_base_attack(duel));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
    e2.set_value(duel, tc.get_base_defense(duel));
    tc.register_effect(duel, e2);
    duel.adjust_instantly(Some(tc));
    if !(tc.is_attack(duel, &[tc.get_base_attack(duel)]) && tc.is_defense(duel, &[tc.get_base_defense(duel)]) && e.get_label(duel) == 1) {
        return Ok(());
    }
    duel.break_effect()?;
    let e3 = duel.create_effect(c);
    e3.set_description(duel, 3008);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_value(duel, 1);
    e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    tc.register_effect(duel, e4);
    Ok(())
}

fn mine_faceup(tp: u8) -> impl Fn(CardId, &Duel) -> bool {
    aux::faceup_filter(move |c, duel| c.is_controler(duel, tp))
}

fn gainatktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g =
        a.eg.unwrap_or_default()
            .filter(duel, mine_faceup(a.tp), None)
            .filter(duel, |c, duel| c.is_location(duel, LOCATION_MZONE), None);
    if chk == 0 {
        return Ok(a.e.get_handler(duel).get_overlay_count(duel) >= 3 && !g.is_empty());
    }
    duel.set_target_card(g);
    Ok(false)
}

fn gainatkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default().filter(duel, CardId::is_faceup, None);
    if g.is_empty() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 900);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
