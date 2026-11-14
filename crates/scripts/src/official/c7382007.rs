// Gouki Sheik Ogre
use super::c22510667::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 7382007;

pub const LISTED_SERIES: &[u16] = &[SET_GOUKI];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_GOUKI])
    }
}

// Link Summoned from Gouki Guts and Gouki Machine Suprex, with Gouki monsters in the Deck to add and the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (7540107, MZONE),
    (29099860, MZONE),
    (7540107, MZONE),
    (7540107, HAND),
    (29099860, HAND),
    (7540107, DECK),
    (29099860, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure<2, 2>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_HAND);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GOUKI])
        && c.is_monster(duel)
        && c.is_previous_location(duel, LOCATION_DECK)
        && c.is_controler(duel, tp)
        && !c.is_reason(duel, REASON_DRAW)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| spconfilter(c, duel, a.tp), 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GOUKI]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    ft = ft.min(g.len() as i32);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let rescon = aux::dpcheck(|c, duel| i64::from(c.get_level(duel)));
    let sg = aux::select_unselect_group(duel, g, 1, ft as usize, &rescon, tp, HINTMSG_SPSUMMON)?;
    if !sg.is_empty() && duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        for sc in sg {
            sc.register_flag_effect_full(
                duel,
                ID,
                RESET_EVENT | RESETS_STANDARD,
                EFFECT_FLAG_CLIENT_HINT,
                1,
                0,
                aux::stringid(ID, 2),
            );
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_property(
                duel,
                EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE,
            );
            e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
            e1.set_range(duel, LOCATION_MZONE);
            e1.set_absolute_range(duel, tp, 1, 0);
            e1.set_condition(duel, |duel, a| {
                let e = a.e;
                e.get_handler(duel).is_controler(duel, e.get_owner_player(duel))
            });
            e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_set_card(duel, &[SET_GOUKI]));
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            sc.register_effect_full(duel, e1, true);
        }
    }
    Ok(())
}
