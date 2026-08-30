local card = {
    api_version = 1,
    id = "RLK_711",
    name = "Vicious Bloodworm",
    text = "[x]<b>Battlecry:</b> Give a minion in\nyour hand Attack equal to\nthis minion's Attack.",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    rarity = "rare",
    cost = 2,
    attack = 3,
    health = 2,
    tags = { "beast" },
    rune_cost = { blood = 1 },
    keywords = { "battlecry" },
    target_mode = "required_if_available",
}

function card.targets(ctx, self)
    local targets = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if entity ~= self and ctx:entity(entity).type == "minion" then
            targets[#targets + 1] = entity
        end
    end
    return targets
end

function card.on_battlecry(ctx, self, target)
    if target then cardlib.effects.buff(ctx, target, ctx:entity(self).attack, 0) end
end

return card
