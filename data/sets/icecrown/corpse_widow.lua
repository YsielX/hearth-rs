local function has_keyword(entity, wanted)
    for _, keyword in ipairs(entity.keywords) do if keyword == wanted then return true end end
    return false
end

return {
    api_version = 1, id = "ICC_243", name = "Corpse Widow",
    text = "Your <b>Deathrattle</b> cards cost (2) less.",
    set = "ICECROWN", type = "minion", class = "hunter", rarity = "rare",
    cost = 5, attack = 4, health = 6, tags = { "undead", "beast" },
    auras = {{
        active_zones = { "board" }, cost = -2,
        targets = function(ctx, self)
            local result, player = {}, ctx:controller(self)
            for _, zone in ipairs({ ctx:hand(player), ctx:deck(player) }) do
                for _, entity in ipairs(zone) do
                    if has_keyword(ctx:entity(entity), "deathrattle") then result[#result + 1] = entity end
                end
            end
            return result
        end,
    }},
}
