// Witness of the Ancient
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 54577949;
const TOKEN_ARC: u32 = ID + 1;
pub const LISTED_NAMES: &[u32] = &[TOKEN_ARC];

// Two in hand with Synchro Monsters in the Graveyard and Extra Deck (Diabell, Dragunity Knight - Luin, Centur-Ion Legatia,
// Lightstorm Dragon) to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (14307929, MZONE),
    (14307929, GRAVE),
    (12496261, EXTRA),
    (15982593, EXTRA),
    (10515412, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(TOKEN_ARC),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, pltg);
    e2.set_operation(duel, plop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(CardId::is_synchro_monster),
        a.tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        1,
        None,
    )
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_synchro_monster(duel) && !c.is_forbidden(duel)
}

fn rescon(
    tokenlv1chk: bool,
    tokenlv2chk: bool,
    tokenlv3chk: bool,
    mzone_chk: bool,
) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| {
        let distinct = sg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) == sg.len();
        if !mzone_chk {
            return (distinct, false);
        }
        let token_ok = match sg.len() {
            1 => tokenlv1chk,
            2 => tokenlv2chk,
            3 => tokenlv3chk,
            _ => false,
        };
        (token_ok, !distinct)
    }
}

fn token_checks(duel: &mut Duel, tp: u8, stzone_count: i32) -> [bool; 3] {
    [1, 2, 3].map(|level| {
        let stats = MonsterStats::new(0, TYPES_TOKEN, 0, 0, level as u32, RACE_MACHINE, ATTRIBUTE_LIGHT);
        stzone_count >= level && duel.is_player_can_special_summon_monster(tp, TOKEN_ARC, stats)
    })
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let stzone_count = duel.get_location_count(tp, LOCATION_SZONE);
        if stzone_count <= 0 || duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            return Ok(false);
        }
        let g = duel.get_matching_group(plfilter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, None);
        if g.is_empty() {
            return Ok(false);
        }
        let [lv1, lv2, lv3] = token_checks(duel, tp, stzone_count);
        return Ok(aux::select_unselect_group_check(
            duel,
            g,
            1,
            stzone_count.min(3) as usize,
            &rescon(lv1, lv2, lv3, true),
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, tp, 0);
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let stzone_count = duel.get_location_count(tp, LOCATION_SZONE);
    let g = duel.get_matching_group(aux::necro_valley_filter(plfilter), tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, None);
    if stzone_count > 0 && !g.is_empty() {
        let mut place_success_chk = false;
        let mzone_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        let [lv1, lv2, lv3] = token_checks(duel, tp, stzone_count);
        let rescon = rescon(lv1, lv2, lv3, mzone_chk);
        let max = g.len().min(stzone_count as usize).min(3);
        let sg = aux::select_unselect_group_full(duel, g, 1, max, &rescon, tp, HINTMSG_TOFIELD, Some(&rescon), false)?;
        for sc in sg {
            if duel.move_to_field(sc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)? {
                place_success_chk = true;
                let e1 = duel.create_effect(c);
                e1.set_code(duel, EFFECT_CHANGE_TYPE);
                e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
                e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
                sc.register_effect(duel, e1);
            }
        }
        if place_success_chk
            && mzone_chk
            && let Some(token) = duel.create_token(tp, TOKEN_ARC)
        {
            token.set_level(duel, sg.len() as u32);
            duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_target_range(duel, 1, 0);
    e2.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_synchro_monster(duel)
    });
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    Ok(())
}
