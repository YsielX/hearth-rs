return {
    api_version = 1, id = "CFM_756", name = "Alley Armorsmith",
    text = "[x]<b>Taunt</b>\nWhenever this minion\ndeals damage, gain\nthat much Armor.",
    set = "GANGS", type = "minion", class = "warrior", rarity = "rare",
    cost = 5, attack = 3, health = 7, keywords = { "taunt" },
    triggers = {{
        event = "damaged", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.source == self and event.amount > 0 end,
        effect = function(ctx, self, event) ctx:gain_armor(ctx:controller(self), event.amount) end,
    }},
}
