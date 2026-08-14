return {
    api_version = 1, id = "OG_026", name = "Eternal Sentinel",
    text = "<b>Battlecry:</b> Unlock your <b>Overloaded</b> Mana Crystals.", set = "OG",
    type = "minion", class = "shaman", rarity = "epic", cost = 2, attack = 3, health = 2,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        local amount = ctx:player(player).overloaded_mana
        if amount > 0 then ctx:unlock_mana(player, amount) end
    end,
}
