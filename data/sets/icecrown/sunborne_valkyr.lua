return {
    api_version = 1, id = "ICC_028", name = "Sunborne Val'kyr",
    text = "<b>Battlecry:</b> Give adjacent minions +2 Health.",
    set = "ICECROWN", type = "minion", rarity = "common",
    cost = 5, attack = 5, health = 4, tags = { "undead" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:adjacent_minions(self)) do ctx:buff(minion, 0, 2) end
    end,
}
