return {
    api_version = 1, id = "AT_054", name = "The Mistcaller",
    text = "<b>Battlecry:</b> Give all minions in your hand and deck +1/+1.", set = "TGT", type = "minion",
    class = "shaman", rarity = "legendary", cost = 6, attack = 4, health = 4,
    tags = { "undead" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        for _, zone in ipairs({ ctx:hand(player), ctx:deck(player) }) do
            for _, entity in ipairs(zone) do
                if ctx:entity(entity).type == "minion" then ctx:buff(entity, 1, 1) end
            end
        end
    end,
}
