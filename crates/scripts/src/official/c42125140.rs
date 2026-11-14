// Dracotail Shaulas
use super::c33760966::register_revive;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 42125140;

pub const LISTED_SERIES: &[u16] = &[SET_DRACOTAIL];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_DRACOTAIL], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, _, _, _| c.is_location(duel, LOCATION_HAND)),
    ];
}

// In the Extra Deck, with a Dracotail monster on the field and one in hand, and Dracotail cards in the Graveyard to shuffle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (75003700, MZONE),
    (1498449, HAND),
    (75003700, GRAVE),
    (1498449, GRAVE),
    (ID, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
    register_revive::<ID>(duel, c, |duel, _| !duel.is_damage_step());
}

fn tdfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    (c.is_set_card(duel, &[SET_DRACOTAIL]) || c.is_on_field(duel))
        && c.is_faceup(duel)
        && c.is_able_to_deck(duel)
        && c.is_can_be_effect_target(duel, e)
}

fn rescon(duel: &Duel, sg: CardSet) -> bool {
    let (gy_sg, field_sg) = sg.split(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
    gy_sg.len() == 2
        && field_sg.len() == 1
        && field_sg.get_first().is_some_and(|c| {
            c.is_type(
                duel,
                gy_sg.get_bitwise_or(duel, |c, duel| u64::from(c.get_main_card_type(duel))) as u32,
            )
        })
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(
        |c, duel| tdfilter(c, duel, e),
        tp,
        LOCATION_GRAVE | LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        None,
    );
    let rescon = |duel: &Duel, sg, _, _| (rescon(duel, sg), false);
    if chk == 0 {
        return Ok(g.len() >= 3 && aux::select_unselect_group_check(duel, g, 3, 3, &rescon));
    }
    let tg = aux::select_unselect_group(duel, g, 3, 3, &rescon, tp, HINTMSG_TODECK)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(tg), 3, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if tg.len() == 3 {
        duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
