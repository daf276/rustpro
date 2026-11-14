// Number 104: Masquerade Vain
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 16908882;

pub const LISTED_SERIES: &[u16] = &[SET_BARIANS];
pub const XYZ_NUMBER: u32 = 104;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
}

// Xyz Summoned with Mystical Elf, Flying Kamakiri #2 and Maiden of the Moonlight, Seventh Barian's in the Deck to add, and
// V Umbral Horror in the Graveyard to Xyz Summon Number C104: Umbral Horror Masquerade (Rank 5) with it as material.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (15025844, MZONE),
    (3134241, MZONE),
    (79629370, MZONE),
    (39513225, DECK),
    (39513225, DECK),
    (39053882, GRAVE),
    (49456901, EXTRA),
    (4206964, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::MaterialFrom(MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |_, a| a.r == REASON_XYZ);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BARIANS]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(rc) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_xyz_summoned(duel) && a.eg.is_some_and(|eg| eg.is_contains(c))
    });
    e1.set_target(duel, bantg);
    e1.set_operation(duel, banop);
    e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
    rc.register_effect_full(duel, e1, true);
    if !rc.is_type(duel, TYPE_EFFECT) {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_ADD_TYPE);
        e2.set_value(duel, TYPE_EFFECT as i32);
        e2.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
        rc.register_effect_full(duel, e2, true);
    }
    Ok(())
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn banfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_monster(duel) || c.is_spell(duel)) && c.is_able_to_remove(duel)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, opp) = (a.tp, 1 - a.tp);
    let g = duel.get_matching_group(banfilter, tp, 0, LOCATION_DECK, None);
    if g.is_exists(duel, CardId::is_spell, 1, None)
        && g.get_class_count(duel, |c, duel| i64::from(c.get_attribute(duel))) > 4
        && duel.select_yes_no(opp, aux::stringid(ID, 2))?
    {
        let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| {
            (
                sg.len() == 5 && sg.get_class_count(duel, |c, duel| i64::from(c.get_attribute(duel))) == 5,
                sg.filter_count(duel, CardId::is_spell, None) != 1,
            )
        };
        let sg = aux::select_unselect_group(duel, g, 5, 5, &rescon, opp, HINTMSG_REMOVE)?;
        if sg.len() == 5 {
            duel.remove_full(sg, POS_FACEUP, REASON_EFFECT, PLAYER_NONE, Some(opp))?;
        }
    } else {
        let dg = duel.get_decktop_group(opp, 10);
        duel.disable_shuffle_check(true);
        duel.remove(dg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
