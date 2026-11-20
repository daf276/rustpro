// The messages EDOPro's server sends each player: a port of the switch of GenericDuel::Sending in gframe/generic_duel.cpp
// (https://github.com/edo9300/edopro), which hides codes per player. Its MSG_UPDATE_DATA refreshes are what
// [engine::Duel::observe] shows. MSG_TAG_SWAP and MSG_MATCH_KILL are left out, as this engine writes neither.
use engine::constants::*;

const MSG_ROCK_PAPER_SCISSORS: u8 = 132;
const LOCATION_INFO: usize = 10;

// The framing of OCG_DuelGetMessage: each message prefixed by its u32 length in native byte order.
pub fn frames(mut buffer: &[u8]) -> impl Iterator<Item = &[u8]> {
    std::iter::from_fn(move || {
        let (size, rest) = buffer.split_first_chunk::<4>()?;
        let (message, rest) = rest.split_at_checked(u32::from_ne_bytes(*size) as usize)?;
        buffer = rest;
        Some(message)
    })
}

// Appends the messages of `messages` that `player` receives, framed the same, with the codes it may not see zeroed.
pub fn mask(messages: &[u8], player: u8, out: &mut Vec<u8>) {
    for message in frames(messages).filter(|message| receives(message, player)) {
        out.extend_from_slice(&(message.len() as u32).to_ne_bytes());
        let start = out.len();
        out.extend_from_slice(message);
        if let Some(copy) = out.get_mut(start..) {
            hide(copy, player);
        }
    }
}

pub(crate) fn u32_at(message: &[u8], at: usize) -> u32 {
    message
        .get(at..at + 4)
        .and_then(|bytes| bytes.try_into().ok())
        .map_or(0, u32::from_ne_bytes)
}

fn byte(message: &[u8], at: usize) -> u8 {
    message.get(at).copied().unwrap_or(0)
}

fn receives(message: &[u8], player: u8) -> bool {
    let [id, first] = [byte(message, 0), byte(message, 1)];
    match id {
        // Hints 1-3 and 5 go to their player alone, 4, 6-9 and 11 to everyone else.
        MSG_HINT => match first {
            1..=3 | 5 => byte(message, 2) == player,
            4 | 6..=9 | 11 => byte(message, 2) != player,
            _ => true,
        },
        MSG_SELECT_BATTLECMD
        | MSG_SELECT_IDLECMD
        | MSG_SELECT_EFFECTYN
        | MSG_SELECT_YESNO
        | MSG_SELECT_OPTION
        | MSG_SELECT_CHAIN
        | MSG_SELECT_PLACE
        | MSG_SELECT_DISFIELD
        | MSG_SELECT_POSITION
        | MSG_SELECT_COUNTER
        | MSG_SELECT_SUM
        | MSG_SORT_CARD
        | MSG_SORT_CHAIN
        | MSG_ROCK_PAPER_SCISSORS
        | MSG_ANNOUNCE_RACE
        | MSG_ANNOUNCE_ATTRIB
        | MSG_ANNOUNCE_CARD
        | MSG_ANNOUNCE_NUMBER
        | MSG_SELECT_CARD
        | MSG_SELECT_TRIBUTE
        | MSG_SELECT_UNSELECT_CARD
        | MSG_MISSED_EFFECT => first == player,
        // Cards confirmed in the Deck or Extra Deck go to the confirming player alone, as the first card's location decides.
        MSG_CONFIRM_CARDS => u32_at(message, 2) == 0 || !matches!(byte(message, 11), LOCATION_DECK | LOCATION_EXTRA) || first == player,
        _ => true,
    }
}

fn zero(message: &mut [u8], at: usize) {
    if let Some(code) = message.get_mut(at..at + 4) {
        code.fill(0);
    }
}

// Zeroes the codes of the cards in a list of `size`-byte entries (code first) from `at` whose controller (at `controller` in
// the entry) is not `player`; returns the end of the list.
fn hide_others(message: &mut [u8], at: usize, size: usize, controller: usize, player: u8) -> usize {
    let count = u32_at(message, at) as usize;
    let start = at + 4;
    for entry in (start..).step_by(size).take(count) {
        if byte(message, entry + controller) != player {
            zero(message, entry);
        }
    }
    start + count * size
}

fn hide(message: &mut [u8], player: u8) {
    let first = byte(message, 1);
    match byte(message, 0) {
        MSG_SELECT_CARD => {
            hide_others(message, 11, 4 + LOCATION_INFO, 4, player);
        }
        MSG_SELECT_TRIBUTE => {
            hide_others(message, 11, 11, 4, player);
        }
        MSG_SELECT_UNSELECT_CARD => {
            let end = hide_others(message, 12, 4 + LOCATION_INFO, 4, player);
            hide_others(message, end, 4 + LOCATION_INFO, 4, player);
        }
        MSG_SHUFFLE_HAND | MSG_SHUFFLE_EXTRA if first != player => {
            (0..u32_at(message, 2) as usize).for_each(|index| zero(message, 6 + 4 * index));
        }
        MSG_MOVE => {
            let [controller, location] = [byte(message, 15), byte(message, 16)];
            let face_down = u32_at(message, 21) & POS_FACEDOWN != 0;
            let hidden =
                location & (LOCATION_GRAVE | LOCATION_OVERLAY) == 0 && (location & (LOCATION_DECK | LOCATION_HAND) != 0 || face_down);
            if controller != player && hidden {
                zero(message, 1);
            }
        }
        MSG_SET => zero(message, 1),
        MSG_SPSUMMONING if byte(message, 5) != player && u32_at(message, 11) & POS_FACEDOWN != 0 => zero(message, 1),
        MSG_DRAW if first != player => {
            for entry in (6..).step_by(8).take(u32_at(message, 2) as usize) {
                if u32_at(message, entry + 4) & POS_FACEUP == 0 {
                    zero(message, entry);
                }
            }
        }
        _ => {}
    }
}
