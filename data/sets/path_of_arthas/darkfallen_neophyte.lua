local card = {
    api_version = 1,
    id = "RLK_731",
    name = "Darkfallen Neophyte",
    text = "[x]<b>Battlecry:</b> Spend 2 <b>Corpses</b>\nto give all minions in your\nhand +2 Attack.",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    rarity = "common",
    cost = 3,
    attack = 2,
    health = 5,
    tags = { "undead" },
    rune_cost = { blood = 1 },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    if not ctx:spend_corpses(player, 2) then return end
    for _, entity in ipairs(ctx:hand(player)) do
        if ctx:entity(entity).type == "minion" then ctx:buff(entity, 2, 0) end
    end
end

return card
