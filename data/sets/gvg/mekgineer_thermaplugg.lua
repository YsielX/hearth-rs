return {
    api_version = 1, id = "GVG_116", name = "Mekgineer Thermaplugg",
    text = "Whenever an enemy minion dies, summon a Leper Gnome.", set = "GVG", type = "minion",
    rarity = "legendary", cost = 9, attack = 9, health = 7, tags = { "mech" },
    triggers = {{
        event = "entity_died", active_zones = { "board" },
        condition = function(ctx, self, event) return event.player ~= ctx:controller(self) end,
        effect = function(ctx, self) ctx:summon(ctx:controller(self), "EX1_029") end,
    }},
}
