return {
    api_version = 1, id = "CFM_753", name = "Grimestreet Outfitter",
    text = "<b>Battlecry:</b> Give all minions in your hand +1/+1.",
    set = "GANGS", type = "minion", class = "paladin", rarity = "common",
    cost = 2, attack = 2, health = 2, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
            if ctx:entity(entity).type == "minion" then cardlib.effects.buff(ctx, entity, 1, 1) end
        end
    end,
}
