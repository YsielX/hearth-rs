local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "KAR_026",
    name = "Protect the King!",
    text = "For each enemy minion, summon a 1/1 Pawn with <b>Taunt</b>.",
    set = "KARA",
    type = "spell",
    class = "warrior",
    rarity = "rare",
    cost = 3,
    tokens = {{
        id = "KAR_026t",
        name = "Pawn",
        text = "<b>Taunt</b>",
        set = "KARA",
        type = "minion",
        class = "warrior",
        cost = 1,
        attack = 1,
        health = 1,
        keywords = { "taunt" },
    }},
}

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local count = 0
    for _, minion in ipairs(ctx:board(ctx:opponent(player))) do
        if ctx:entity(minion).type == "minion" and not is_dormant(ctx, minion) then count = count + 1 end
    end
    for _ = 1, count do ctx:summon(player, "KAR_026t") end
end

return card
