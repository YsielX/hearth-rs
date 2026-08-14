local card = {
    api_version = 1,
    id = "LOOT_375",
    name = "Guild Recruiter",
    text = "<b>Battlecry:</b> <b>Recruit</b> a minion that costs (4) or less.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "common",
    cost = 5,
    attack = 2,
    health = 4,
    tags = { "draenei" },
    keywords = { "battlecry", "recruit" },
}

function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        local minion = ctx:entity(entity)
        if minion.type == "minion" and minion.cost <= 4 then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_value(candidates, "guild_recruiter_chosen") end
end

function card.guild_recruiter_chosen(ctx, self, entity)
    ctx:recruit(ctx:controller(self), entity)
end

return card
