// The Dark Magicians
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 50237654;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN, CARD_DARK_MAGICIAN_GIRL];
pub const MATERIAL_SETCODE: &[u16] = &[SET_DARK_MAGICIAN, SET_MAGICIAN_GIRL, SET_DARK_MAGICIAN_GIRL];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Table(&[
            FusionMaterial::Code(CARD_DARK_MAGICIAN),
            FusionMaterial::Code(CARD_DARK_MAGICIAN_GIRL),
        ]),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Dark Magician and Dark Magician Girl, with Spells (Twin Twisters,
// Magician's Rod) in the Deck to draw into and Raigeki of the opponent's to destroy it, and Dark Magician and Dark Magician Girl
// in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_DARK_MAGICIAN, HAND),
    (CARD_DARK_MAGICIAN_GIRL, HAND),
    (CARD_DARK_MAGICIAN, DECK),
    (CARD_DARK_MAGICIAN_GIRL, DECK),
    (43898403, DECK),
    (43898403, DECK),
    (7084129, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_SET);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| a.re.is_some_and(|re| re.is_spell_trap_effect(duel)));
    e1.set_target(duel, stock::draw_target::<1>);
    e1.set_operation(duel, drop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? != 0
        && let Some(tc) = duel.get_operated_group().get_first()
        && tc.is_spell_trap(duel)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.sset_full(tp, tc, tp, false)?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        if tc.is_quick_play_spell(duel) {
            e1.set_code(duel, EFFECT_QP_ACT_IN_SET_TURN);
        } else if tc.is_trap(duel) {
            e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
        }
        e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_description(duel, aux::stringid(ID, 3));
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

const LOCATIONS: u32 = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE;

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, CARD_DARK_MAGICIAN), tp, LOCATIONS, 0, 1, None)
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp, CARD_DARK_MAGICIAN_GIRL),
                tp,
                LOCATIONS,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATIONS as i32);
    Ok(false)
}

fn rescon(duel: &Duel, sg: CardSet) -> (bool, bool) {
    (
        sg.is_exists(duel, |c, duel| c.is_code(duel, CARD_DARK_MAGICIAN), 1, None)
            && sg.is_exists(duel, |c, duel| c.is_code(duel, CARD_DARK_MAGICIAN_GIRL), 1, None),
        false,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let g1 = duel.get_matching_group(
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, CARD_DARK_MAGICIAN)),
        tp,
        LOCATIONS,
        0,
        None,
    );
    let g2 = duel.get_matching_group(
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, CARD_DARK_MAGICIAN_GIRL)),
        tp,
        LOCATIONS,
        0,
        None,
    );
    if !g1.is_empty() && !g2.is_empty() {
        let mut g = g1;
        g.merge(g2);
        let sg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg), tp, HINTMSG_SPSUMMON)?;
        if !sg.is_empty() {
            duel.special_summon(sg, 0, tp, tp, true, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
