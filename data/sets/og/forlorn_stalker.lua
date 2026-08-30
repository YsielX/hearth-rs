local function deathrattle(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "deathrattle" then return true end
    end
    return false
end
return {
    api_version = 1, id = "OG_292", name = "Forlorn Stalker",
    text = "<b>Battlecry:</b> Give all <b>Deathrattle</b> minions in your hand +1/+1.",
    set = "OG", type = "minion", class = "hunter", rarity = "rare",
    cost = 3, attack = 4, health = 2, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
            if ctx:entity(entity).type == "minion" and deathrattle(ctx, entity) then
                cardlib.effects.buff(ctx, entity, 1, 1)
            end
        end
    end,
}
