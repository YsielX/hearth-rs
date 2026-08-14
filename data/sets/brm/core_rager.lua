return {
    api_version = 1,
    id = "BRM_014",
    name = "Core Rager",
    text = "<b>Battlecry:</b> If your hand is empty, gain +3/+3.",
    set = "BRM",
    type = "minion",
    class = "hunter",
    rarity = "rare",
    cost = 4,
    attack = 4,
    health = 4,
    tags = { "elemental", "beast" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        if #ctx:hand(ctx:controller(self)) == 0 then ctx:buff(self, 3, 3) end
    end,
}
