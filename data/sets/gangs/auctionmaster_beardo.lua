return {
    api_version = 1, id = "CFM_807", name = "Auctionmaster Beardo",
    text = "After you cast a spell, refresh your Hero Power.", set = "GANGS",
    type = "minion", rarity = "legendary", cost = 3, attack = 3, health = 4,
    triggers = {{
        event = "spell_cast", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.player_cast
        end,
        effect = function(ctx, self) ctx:refresh_hero_power(ctx:controller(self)) end,
    }},
}
