local function has_keyword(entity, wanted)
    for _, keyword in ipairs(entity.keywords) do
        if keyword == wanted then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "FP1_017",
    name = "Nerub'ar Weblord",
    text = "Minions with <b>Battlecry</b> cost (2) more.",
    set = "NAXX",
    type = "minion",
    rarity = "common",
    cost = 2,
    attack = 1,
    health = 4,
    tags = { "undead" },
    auras = {
        {
            cost = 2,
            targets = function(ctx, self)
                local targets = {}
                for player = 0, 1 do
                    for _, entity in ipairs(ctx:hand(player)) do
                        local snapshot = ctx:entity(entity)
                        if snapshot.type == "minion" and has_keyword(snapshot, "battlecry") then
                            targets[#targets + 1] = entity
                        end
                    end
                    for _, entity in ipairs(ctx:deck(player)) do
                        local snapshot = ctx:entity(entity)
                        if snapshot.type == "minion" and has_keyword(snapshot, "battlecry") then
                            targets[#targets + 1] = entity
                        end
                    end
                end
                return targets
            end,
        },
    },
}
