// Clock Lizard
use crate::prelude::*;
use crate::proc_fusion::Preselect;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 51476410;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

struct Params;

impl FusionSummonParams for Params {
    const DESC: u64 = aux::stringid(ID, 0);
    const LOCATION: u32 = LOCATION_GRAVE;
    const CHKF: Option<u64> = Some(PLAYER_NONE as u64);
    const NOSUMMONCHECK: bool = true;
    const PRESELECT: Option<Preselect> = Some(preop);
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        lizardcheck(c, duel, c.get_controler(duel))
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let location = if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some() {
            LOCATION_MZONE
        } else {
            LOCATION_GRAVE
        };
        let group = duel.get_matching_group(
            |c, duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove),
            tp,
            location,
            0,
            None,
        );
        (Some(group), Some((checkextra, e.get_handler(duel))))
    }
}

// In the Extra Deck, Link Summoned from Cyberse Normal Monsters or on the field, with Fusion Monsters and their materials in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (17881964, GRAVE),
    (53293545, GRAVE),
    (75356564, GRAVE),
    (70681994, GRAVE),
    (53153481, GRAVE),
    (33064647, GRAVE),
    (47226949, GRAVE),
    (24154052, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_category(
        duel,
        CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON + CATEGORY_REMOVE,
    );
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_target(duel, fustg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_condition(duel, atkcon);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

// The Lua also passes the card's original Archetypes, which the core already loads with the code.
fn lizardcheck(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_extra(duel)
        && c.is_has_effect(duel, CARD_CLOCK_LIZARD).is_none()
        && duel.check_mut(|duel| {
            let stats = MonsterStats {
                setcode: None,
                card_type: Some(c.get_original_type(duel)),
                attack: Some(c.get_base_attack(duel)),
                defense: Some(c.get_base_defense(duel)),
                level: Some(c.get_original_level(duel)),
                race: Some(c.get_original_race(duel)),
                attribute: Some(c.get_original_attribute(duel)),
            };
            duel.is_player_can_special_summon_monster(tp, c.get_original_code(duel), stats)
        })
}

fn checkextra(duel: &Duel, c: CardId, tp: u8, sg: CardSet, fc: CardId) -> bool {
    if fc.is_location(duel, LOCATION_EXTRA) {
        duel.get_location_count_from_ex(tp, tp, Some(sg), fc) > 0
    } else {
        duel.get_location_count_from_ex(tp, tp, Some(sg.union(c.into())), TYPE_FUSION) > 0
    }
}

fn preop(duel: &mut Duel, _: EffectId, tc: CardId) -> Script<bool> {
    Ok(duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? != 0)
}

fn fustg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_remove(tp, None)
            && aux::check_summon_gate(duel, tp).is_none_or(|limit| limit >= 2)
            && Fusion::summon_eff_tg::<Params>(duel, a, chk, chkc)?);
    }
    duel.set_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_faceup(duel) && c.is_previous_location(duel, LOCATION_GRAVE)
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_special_summoned(duel)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(|c, duel| c.is_race(duel, RACE_CYBERSE), tp, LOCATION_GRAVE, 0, 1, None)
                && duel.is_existing_matching_card(atkfilter, tp, 0, LOCATION_MZONE, 1, None),
        );
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g = duel.get_matching_group(atkfilter, tp, 0, LOCATION_MZONE, None);
    if g.is_empty() {
        return Ok(());
    }
    let atk = duel.get_matching_group_count(|c, duel| c.is_race(duel, RACE_CYBERSE), tp, LOCATION_GRAVE, 0, None) as i32 * 400;
    if atk == 0 {
        return Ok(());
    }
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -atk);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
