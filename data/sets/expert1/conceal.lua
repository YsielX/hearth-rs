return {
    api_version = 1,
    id = "EX1_128",
    name = "Conceal",
    text = "Give your minions <b>Stealth</b> until your next turn.",
    set = "EXPERT1",
    type = "spell",
    class = "rogue",
    rarity = "common",
    spell_school = "shadow",
    cost = 1,
    on_play = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            ctx:grant_keyword_until_next_turn(minion, "stealth")
        end
    end,
}
