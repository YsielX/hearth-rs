return {
    api_version = 1,
    id = "RLK_048",
    name = "Anti-Magic Shell",
    text = "Give your minions +1/+1 and <b>Elusive</b>.",
    set = "CORE",
    type = "spell",
    class = "death_knight",
    rarity = "rare",
    spell_school = "shadow",
    cost = 3,
    rune_cost = { unholy = 1 },
    on_play = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            ctx:buff(minion, 1, 1)
            ctx:grant_keyword(minion, "elusive")
        end
    end,
}
