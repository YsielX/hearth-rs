return {
    api_version = 1, id = "CFM_020", name = "Raza the Chained",
    text = "[x]  <b>Battlecry:</b> If your deck has  \nno duplicates, your Hero\n Power costs (0) this game.",
    set = "GANGS", type = "minion", class = "priest", rarity = "legendary",
    cost = 5, attack = 5, health = 5, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player, seen = ctx:controller(self), {}
        for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
            local id = ctx:entity(entity).card_id
            if seen[id] then return end
            seen[id] = true
        end
        ctx:grant_player_keyword(player, "raza_hero_power_zero")
    end,
}
