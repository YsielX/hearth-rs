local function has_keyword(ctx, entity, wanted)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == wanted then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "GVG_101",
    name = "Scarlet Purifier",
    text = "<b>Battlecry:</b> Deal 2 damage to all minions with <b>Deathrattle</b>.",
    set = "GVG",
    type = "minion",
    class = "paladin",
    rarity = "rare",
    cost = 3,
    attack = 4,
    health = 3,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local targets = {}
        for _, minion in ipairs(ctx:minions()) do
            if has_keyword(ctx, minion, "deathrattle") then
                targets[#targets + 1] = minion
            end
        end
        if #targets > 0 then cardlib.effects.damage_all(ctx, targets, 2) end
    end,
}
