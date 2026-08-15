# Frozen Throne deck corpus

This directory contains 354 runnable deck records sourced from the 2017 Knights
of the Frozen Throne period, covering all nine contemporary classes. Together
with `decks/quest_rogue.json`, the default RL pool contains 355 decks. The
importer reads all 356 links exposed by the two source indexes; two old pages no
longer expose a complete 30-card list and are reported as skipped.

The sources are the [Knights of the Frozen Throne deck-list index][kft] and the
[2017 HCT Americas Summer Playoffs deck lists][hct]. The broader archetype
selection was checked against [Vicious Syndicate Data Reaper Report #59][vs].

Each JSON file deliberately separates two representations:

- `source_cards` is the published 30-card list by card name and count.
- `cards` is the list of IDs that this repository can execute today.
- `substitutions` records every difference between them; `adapted` and
  `adaptation_ratio` summarize that difference.

Most published Standard lists use Basic/Classic cards whose Lua implementations
are not yet present in this repository. The importer therefore makes explicit,
class-legal substitutions from any set through Frozen Throne, including sets
that had rotated to Wild by 2017. `source_format` describes the historical list;
`format` describes the runnable adaptation. These adaptations are not presented
as historically exact deck lists. Once a missing card is implemented, removing
its entry from `SUBSTITUTIONS` and rerunning the importer automatically restores
the original card.

`bc_eligible` is intentionally conservative about strategy. Only explicitly
recognizable, relatively direct aggro/midrange/tempo decks feed demonstrations
from the simple heuristic bot. Combo, quest and control lists still participate
in DMC self-play and evaluation, where the learned policy can discover their
sequencing without treating weak heuristic play as expert data.

Regenerate the corpus from the source pages with:

```bash
.venv/bin/python scripts/import_frozen_throne_decks.py
```

[kft]: https://www.hearthstonetopdecks.com/knights-of-the-frozen-throne-deck-lists/
[hct]: https://www.hearthstonetopdecks.com/hct-americas-summer-playoffs-2017-deck-lists/
[vs]: https://www.vicioussyndicate.com/vs-data-reaper-report-59/
