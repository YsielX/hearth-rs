# Official keyword examples

The JSON manifests in this directory map every public keyword module to at least one implemented card from Blizzard's official Hearthstone Card Library. Together they cover all 68 functional Constructed keywords tracked by this project.

Each mapping is locked by `keyword_official_coverage.rs`: the keyword must exist, the referenced card or token must load from Lua, the URL must use Blizzard's official card-library domain, and no public keyword may be missing or duplicated. `conditional_charge` is an internal reusable rule for Southsea Deckhand and is intentionally excluded from the official glossary.
