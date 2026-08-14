local function has_keyword(ctx, entity, wanted)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do if keyword == wanted then return true end end
    return false
end

return {
    api_version = 1, id = "CFM_656", name = "Streetwise Investigator",
    text = "<b>Battlecry:</b> Enemy minions lose <b>Stealth</b>.",
    set = "GANGS", type = "minion", rarity = "common", cost = 5, attack = 4, health = 6,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, entity in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(entity).type == "minion" and has_keyword(ctx, entity, "stealth") then
                ctx:disable_keyword(entity, "stealth")
            end
        end
    end,
}
