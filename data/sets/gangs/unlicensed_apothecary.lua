return {
    api_version = 1, id = "CFM_900", name = "Unlicensed Apothecary",
    text = "After you summon a minion, deal 5 damage to your hero.", set = "GANGS",
    type = "minion", class = "warlock", rarity = "epic", cost = 3,
    attack = 5, health = 5, tags = { "demon" },
    triggers = {{
        event = "minion_summoned", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.entity ~= self
        end,
        effect = function(ctx, self)
            ctx:damage(ctx:player(ctx:controller(self)).hero, 5)
        end,
    }},
}
