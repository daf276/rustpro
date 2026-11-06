use std::error::Error;
use std::ops::Range;
use std::sync::LazyLock;

use carddata::{Card, CardDatabase, Deck, TYPE_MONSTER, TYPE_NORMAL, TYPE_SPELL, TYPE_TRAP};
use engine::constants::{
    EFFECT_FLAG_SINGLE_RANGE, EFFECT_TYPE_CONTINUOUS, EFFECT_TYPE_FIELD, EFFECT_TYPE_SINGLE, EFFECT_UPDATE_DEFENSE, EVENT_DRAW,
    LOCATION_MZONE,
};
use engine::{
    Action, CardId, DUEL_MODE_MR5, Duel, DuelOptions, InitialEffect, Library, PHASE_DAMAGE, PHASE_DAMAGE_CAL, PlayerOptions, Prompt, Status,
};

const CODES: std::ops::RangeInclusive<u32> = 1..=40;
const TYPE_EFFECT: u32 = 0x20;
const TYPE_QUICKPLAY: u32 = 0x1_0000;
const TYPE_FLIP: u32 = 0x20_0000;

// The cards phase 2a ports, with their type, level, attack and defense.
const EFFECT_CARDS: [(u32, u32, u32, i32, i32); 12] = [
    (55144522, TYPE_SPELL, 0, 0, 0),
    (84257639, TYPE_SPELL, 0, 0, 0),
    (46130346, TYPE_SPELL, 0, 0, 0),
    (12580477, TYPE_SPELL, 0, 0, 0),
    (53129443, TYPE_SPELL, 0, 0, 0),
    (83764718, TYPE_SPELL, 0, 0, 0),
    (5318639, TYPE_SPELL | TYPE_QUICKPLAY, 0, 0, 0),
    (4206964, TYPE_TRAP, 0, 0, 0),
    (44095762, TYPE_TRAP, 0, 0, 0),
    (54652250, TYPE_MONSTER | TYPE_EFFECT | TYPE_FLIP, 2, 450, 600),
    (26202165, TYPE_MONSTER | TYPE_EFFECT, 3, 1000, 600),
    (14558127, TYPE_MONSTER | TYPE_EFFECT, 3, 0, 1800),
];

static CARDS: LazyLock<CardDatabase> = LazyLock::new(|| {
    let monster = |code: u32| {
        let (attack, defense) = ((code * 100) as i32, (code % 7 * 300) as i32);
        Card {
            code,
            card_type: TYPE_MONSTER | TYPE_NORMAL,
            level: code % 8 + 1,
            attack,
            defense,
            ..Card::default()
        }
    };
    let effect = |(code, card_type, level, attack, defense)| Card {
        code,
        card_type,
        level,
        attack,
        defense,
        ..Card::default()
    };
    CODES.map(monster).chain(EFFECT_CARDS.map(effect)).collect()
});

fn started_duel(main: &[u32], library: &'static Library) -> Result<Duel, Box<dyn Error>> {
    let player = PlayerOptions::default();
    let mut duel = Duel::new(DuelOptions {
        seed: [1, 2, 3, 4],
        flags: DUEL_MODE_MR5,
        team1: player,
        team2: player,
    })?;
    duel.load_library(library);
    let deck = Deck {
        main: main.to_vec(),
        extra: Vec::new(),
    };
    for team in 0..2 {
        duel.add_deck(team, &deck, &CARDS)?;
    }
    duel.set_trace(true);
    duel.start();
    Ok(duel)
}

// The first decision taken in each situation a clone has to carry over.
#[derive(Debug, Default)]
struct Situations {
    mid_tribute: Option<usize>,
    damage_step: Option<usize>,
    mid_chain: Option<usize>,
    // A prompt of a cost, target or operation function, which reruns from its choice log once answered.
    script_prompt: Option<usize>,
}

// Picks a legal action by a hash of the decision number, so the choice depends on nothing else.
fn play(duel: &mut Duel, decisions: Range<usize>) -> Result<Situations, Box<dyn Error>> {
    let mut status = duel.process();
    let mut situations = Situations::default();
    for decision in decisions {
        if status == Status::End {
            break;
        }
        let legal = duel.legal_actions();
        let action = *legal
            .get((decision.wrapping_mul(0x9e37_79b9) >> 8) % legal.len().max(1))
            .ok_or("no legal action")?;
        if matches!(action, Action::Card(_)) && matches!(duel.prompt(), Some(Prompt::Tribute { max: 2.., .. })) {
            situations.mid_tribute.get_or_insert(decision + 1);
        }
        if [PHASE_DAMAGE, PHASE_DAMAGE_CAL].contains(&duel.phase()) {
            situations.damage_step.get_or_insert(decision);
        }
        if duel.get_current_chain() > 0 {
            situations.mid_chain.get_or_insert(decision);
            // Outside a chain only attack targets are chosen with this prompt.
            if matches!(duel.prompt(), Some(Prompt::Card { .. })) {
                situations.script_prompt.get_or_insert(decision);
            }
        }
        status = duel.apply(action)?;
    }
    Ok(situations)
}

const DECISIONS: usize = 20_000;

// The situations a deck must reach and be cloned in.
type ClonePoints = fn(Situations) -> [Option<usize>; 2];

#[test]
fn should_finish_identically_when_cloned_at_a_prompt() {
    let effects = EFFECT_CARDS.iter().flat_map(|card| [card.0; 2]).chain(1..=16).collect();
    let decks: [(Vec<u32>, ClonePoints); 2] = [
        (CODES.collect(), |s| [s.mid_tribute, s.damage_step]),
        (effects, |s| [s.mid_chain, s.script_prompt]),
    ];
    for (main, required) in decks {
        let mut full = started_duel(&main, &scripts::LIBRARY).unwrap();
        let situations = play(&mut full, 0..DECISIONS).unwrap();
        assert!(full.result().is_some());
        for prompt in [0, 1, 17, 100, 333].into_iter().chain(required(situations).map(Option::unwrap)) {
            let mut duel = started_duel(&main, &scripts::LIBRARY).unwrap();
            play(&mut duel, 0..prompt).unwrap();
            let mut clone = duel.clone();
            play(&mut clone, prompt..DECISIONS).unwrap();
            assert_eq!(
                (clone.take_trace(), clone.result()),
                (full.clone().take_trace(), full.result()),
                "cloned at prompt {prompt}"
            );
        }
    }
}

// Every card gains DEF from an effect code no ported card registers, which get_defense does not apply yet.
fn unported_defense_boost(_: u32) -> Option<InitialEffect> {
    Some(|duel: &mut Duel, card: CardId| {
        let effect = duel.create_effect(card);
        effect.set_type(duel, EFFECT_TYPE_SINGLE.into());
        effect.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
        effect.set_range(duel, LOCATION_MZONE.into());
        effect.set_code(duel, EFFECT_UPDATE_DEFENSE);
        effect.set_value(duel, 500);
        card.register_effect(duel, effect);
    })
}

#[test]
#[cfg_attr(debug_assertions, should_panic(expected = "unported call site"))]
fn should_mark_the_duel_when_an_effect_reaches_an_unported_call_site() {
    static LIBRARY: Library = Library {
        load: |_| {},
        card_script: unported_defense_boost,
    };
    let mut duel = started_duel(&CODES.collect::<Vec<_>>(), &LIBRARY).unwrap();
    assert_eq!(duel.unported_site(), None);
    play(&mut duel, 0..DECISIONS).unwrap();
    assert!(duel.unported_site().is_some_and(|site| site.contains("card.rs")));
}

// A global effect declared promptless whose operation asks a question on every draw.
fn load_prompting_promptless(duel: &mut Duel) {
    let effect = duel.global_effect();
    effect.set_type(duel, (EFFECT_TYPE_FIELD | EFFECT_TYPE_CONTINUOUS).into());
    effect.set_code(duel, EVENT_DRAW);
    effect.set_promptless_operation(duel, |duel, a| duel.select_yes_no(a.tp, 0).map(drop));
    duel.register_effect_core(effect, 0);
}

#[test]
#[cfg_attr(debug_assertions, should_panic(expected = "a promptless function prompted"))]
fn should_mark_the_duel_when_a_promptless_function_prompts() {
    static LIBRARY: Library = Library {
        load: load_prompting_promptless,
        card_script: |_| None,
    };
    let mut duel = started_duel(&CODES.collect::<Vec<_>>(), &LIBRARY).unwrap();
    play(&mut duel, 0..DECISIONS).unwrap();
    assert!(duel.unported_site().is_some_and(|site| site.contains("script.rs")));
}
