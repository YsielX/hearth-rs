# hearth-client-bevy

Bevy 0.19 graphical client for hearth-rs. The current client is a playable
native desktop shell: it loads the real Lua card pack, shows the authoritative
player projection, and dispatches only commands returned by the engine's legal
action enumerator. It now includes a main menu, a repository-wide deck browser,
a simple collection/deck editor, match setup, results, and rematches.

The match scene uses original embedded artwork for a painterly tavern board and
identity-free opponent card backs. The executable carries the PNG data itself,
while solid-color layers remain as a safe loading fallback. No official game art
is redistributed; generation prompts and provenance live in
[`assets/ui/README.md`](assets/ui/README.md).

Open the main menu:

~~~bash
cargo run -p hearth-client-bevy
~~~

Available launch options:

~~~text
--data PATH
--deck-one PATH
--deck-two PATH
--seed N
--locale enUS|zhCN|zhTW
--human 1|2
--screenshot PATH
--quick-start
--hotseat
--bot-difficulty easy|normal|hard
--turn-seconds N
--fullscreen
--windowed
--ui-scale 80|100|120
--settings PATH | --no-settings
--resume PATH | --no-resume
~~~

`--locale` switches both card data and the complete client interface between
`enUS`, `zhCN`, and `zhTW`. This includes the menu, deck browser, collection,
match controls, resource/status labels, choices, previews, event toasts, Battle
log, common interaction errors, and game results. User-authored deck names and
canonical engine identifiers are preserved verbatim.

The Settings screen applies windowed or borderless-fullscreen mode and 80%,
100%, or 120% UI scaling immediately. `F11` toggles borderless fullscreen from
any scene. Display, locale, turn-timer, and AI-difficulty preferences share the
atomically written client settings file and survive restart; version-1 files
migrate with 100% windowed defaults, and version-1/version-2 files migrate with
Normal AI. Explicit `--fullscreen`, `--windowed`, `--ui-scale`, and
`--bot-difficulty` arguments override persisted values.

`--quick-start` skips the menu and starts the configured match immediately. It
is useful for focused testing and can be combined with `--screenshot PATH`.
Normal player turns use a visible 75-second timer. `--turn-seconds N` changes
that client default, while `0` disables only the default timer. Runtime card
rules still take precedence, so Nozdormu continues to enforce 15-second turns.
At timeout the client follows the existing CLI semantics: it ends the turn when
legal, or resolves forced paused input with the first non-concede legal option.

The deck selector offers Easy, Normal, and Hard practice AI. Easy is deliberately
naive, Normal uses lethal and Mana planning, and Hard also mulligans expensive
cards and prioritizes advantageous trades. All three use only the current player
view and authoritative legal actions, remain deterministic, and are recorded in
resumable match snapshots. The built-in AI advances by one legal action per visible playback interval rather
than resolving a whole turn synchronously. The action panel exposes the thinking
state, viewer-safe feedback renders between decisions, and each completed AI step
is atomically autosaved. Pausing or restarting during an AI turn continues from
the last completed action.

Every new graphical match derives its first player from the match seed without
consuming the card-effect RNG stream. The first player mulligans three cards; the
second mulligans four and receives The Coin only after both opening choices are
complete. Opening order is shown in the mulligan, hot-seat handoff, and match
status UI and is embedded in replay/snapshot proof data. Older files without the
field retain the historical Player 1 first behavior.

**Pause** or `Esc` opens an in-match menu that freezes the turn timer, AI playback,
targeting, and combat presentation. Its Settings route returns to the same paused
menu. Saving to the main menu keeps the resumable checkpoint; Concede uses a
separate confirmation and deterministically concedes the local player even during
an AI-owned input window.

The **Emotes** control opens all six localized hero emotes with automatic speech-
bubble dismissal and a short anti-spam cooldown. The built-in AI answers after a
deterministic delay. **Squelch Opponent** hides replies for the current match and
is tracked independently for each viewer in Local Two Player mode.

From the menu, **Play** opens the deck selector for both the human and built-in
AI, while **Quit** exits through Bevy's normal application shutdown path.
**My Collection** opens a simplified deck editor backed by all collectible
cards loaded from the Lua pack, with class, mana-cost, and card-type filters plus
a live mana curve and type breakdown. The deck selector can create an empty
custom deck; the editor provides native text input for its name, all eleven
constructed classes, and multi-term search across localized name/text, official
ID, set, class, and keywords. A non-empty deck must be cleared before changing
class, and a deck without an explicit Hero Power receives that class's canonical
basic power. Saved custom decks can be permanently deleted from the selector
after an explicit confirmation step; repository decks never expose deletion and
the persistence layer independently rejects paths outside `decks/custom/`.
Hovering a catalog entry, deck row, hand card, board entity, hero, or Hero Power
opens the same full-text card preview;
uncollectible tokens are resolved from the complete Lua definition catalog too.
Custom decks are validated against their card-driven required size and saved under `decks/custom/`;
normal decks enforce two copies per card and one copy per legendary. Prince
Renathal expands a draft to 40 cards and establishes 40 starting Health before
start-of-game effects; the editor updates every capacity label immediately.
The E.T.C., Band Manager row exposes a localized **Band** editor for its three
sideboard cards. Main-deck and sideboard copies share the normal copy limit;
Deck Code import/export uses Hearthstone's official sideboard footer, and the
opponent projection never exposes the band's contents. Death Knight drafts show
their current three-slot Blood/Frost/Unholy commitment, mark rune costs on
catalog rows, and dynamically hide incompatible candidates. The same rule
includes E.T.C. sideboards and is rechecked by persistence, Deckstring
import/export, and the game engine rather than trusting the UI. At game
time, Death Knight resource rows also show the public Corpse total in all three
locales. Friendly deaths update it in one simultaneous batch; gain/spend events
appear in the Battle log, and tokens explicitly marked as not leaving a Corpse
do not increment it. At game
over, **Rematch** creates a fresh deterministic session with a new seed and
**Main Menu** returns to the front end.

Common actions use board-first controls:

- click opening-hand cards to mark them for replacement, then confirm;
- click a green card or character to select it as an action source;
- drag a green card to a board or character to play, use, or attack;
- click or drop a playable Minion/Location into any highlighted board gap to
  choose its exact insertion position; targeted Battlecries keep that position
  selected while the target is chosen;
- follow the gold aiming arrow for targeted click or drag actions; it turns red
  and snaps to a legal visible target before release;
- click an orange character to use it as the target;
- use the dedicated Hero Power and End Turn buttons; open **Pause** or press `Esc`
  for Settings, save-to-menu, and confirmed Concede actions;
- open **Emotes** for Thanks, Well Played, Greetings, Wow, Oops, Threaten, and
  the per-viewer Squelch Opponent toggle;
- resolve Discover, Choose One, Dredge, Titan, and other paused inputs through
a centered option overlay; revealed card/entity options support full preview;
- read equipped Weapon attack/durability, friendly Secret identities, hidden
  opponent Secret slots, and public Quest/Questline/Sidequest badges directly
  beside each Hero; every known badge supports the full card preview;
- track the opponent's public hand size through identity-free card backs, and
  read both players' deck, next Fatigue, current/temporary mana, locked/pending
  Overload, and Hero Power; the opponent's known Hero Power is preview-only;
- watch viewer-safe public events as queued color-coded toasts and in the
  Battle log.

When a source/target/placement combination maps to one legal command it is
dispatched immediately. If multiple commands remain (for example, a card with
several actions or optional targets), the action panel shows only those
candidates. The panel remains an exhaustive fallback, so every engine mechanic
stays playable when a drag still needs one of several card actions or a choice
set larger than the centered overlay comfortably displays.
