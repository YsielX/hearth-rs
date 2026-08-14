return {
    api_version = 1,
    id = "BRM_024",
    name = "Drakonid Crusher",
    text = "<b>Battlecry:</b> If your opponent has 15 or less Health, gain +3/+3.",
    set = "BRM",
    type = "minion",
    rarity = "common",
    cost = 6,
    attack = 6,
    health = 6,
    tags = { "dragon" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local opponent = ctx:opponent(ctx:controller(self))
        if ctx:entity(ctx:player(opponent).hero).health <= 15 then ctx:buff(self, 3, 3) end
    end,
}
