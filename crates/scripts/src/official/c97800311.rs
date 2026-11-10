// Divine Serpent Apophis
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 97800311;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, _, _, _| c.lists_code(duel, CARD_TEMPLE_OF_THE_KINGS)),
        FusionMaterial::Filter(|c, duel, _, _, _| c.lists_code(duel, CARD_TEMPLE_OF_THE_KINGS)),
    ];
}

pub const LISTED_NAMES: &[u32] = &[CARD_TEMPLE_OF_THE_KINGS];
// In the Extra Deck, Special Summoned by Tributing two Apophis monsters (Apophis the Serpent and Embodiment of Apophis, Set
// and Special Summoned as Trap Monsters) and with Apophis Traps in the Graveyard to Set. No ported card mentions Temple of
// the Kings, so the Fusion Summon is unreachable.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (95561146, SZONE),
    (95561146, SZONE),
    (28649820, SZONE),
    (28649820, SZONE),
    (85888377, SZONE),
    (95561146, GRAVE),
    (28649820, GRAVE),
    (85888377, GRAVE),
    (44095762, SZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
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
    e1.set_category(duel, CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, settg);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_trap_effect(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
    });
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn release_group(duel: &Duel, tp: u8) -> CardSet {
    duel.get_release_group(tp)
        .filter(duel, |c, duel| c.is_set_card(duel, &[SET_APOPHIS]), None)
}

fn spcheck(duel: &Duel, e: EffectId, tp: u8, sg: CardSet) -> (bool, bool) {
    (duel.get_location_count_from_ex(tp, tp, Some(sg), e.get_handler(duel)) > 0, false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let (e, tp) = (a.e, c.get_controler(duel));
    aux::select_unselect_group_check(duel, release_group(duel, tp), 2, 2, &|duel, sg, _, _| spcheck(duel, e, tp, sg))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = release_group(duel, tp);
    let g = aux::select_unselect_group_full(
        duel,
        rg,
        2,
        2,
        &|duel, sg, _, _| spcheck(duel, e, tp, sg),
        tp,
        HINTMSG_RELEASE,
        None,
        true,
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.release(g, REASON_COST)?;
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_set_card(duel, &[SET_APOPHIS]) && c.is_trap(duel) && c.is_ssetable(duel) && c.is_can_be_effect_target(duel, e)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && setfilter(chkc, duel, e));
    }
    let g = duel.get_matching_group(|c, duel| setfilter(c, duel, e), tp, LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    let tg = aux::select_unselect_group(duel, g, 1, ft.clamp(0, 3) as usize, &aux::dncheck, tp, HINTMSG_SET)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(tg), tg.len() as u8, tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mut g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        let ft = duel.get_location_count(tp, LOCATION_SZONE);
        if g.len() as i32 > ft {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
            let ft = ft.max(0) as u8;
            g = g.select(duel, tp, ft, ft, None)?;
        }
        duel.sset(tp, g)?;
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
