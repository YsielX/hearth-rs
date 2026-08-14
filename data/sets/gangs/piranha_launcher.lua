local card = {
    api_version = 1,
    id = "CFM_337",
    name = "Piranha Launcher",
    text = "[x]After your hero attacks,\nsummon a 1/1 Piranha.",
    set = "GANGS",
    type = "weapon",
    class = "hunter",
    rarity = "epic",
    cost = 5,
    attack = 2,
    health = 4,
    triggers = {{
        event = "attack", timing = "after", active_zones = { "weapon" },
        condition = function(ctx, self, event)
            return event.attacker == ctx:player(ctx:controller(self)).hero
        end,
        effect = function(ctx, self) ctx:summon(ctx:controller(self), "CFM_337t") end,
    }},
}

card.tokens = {{
    id = "CFM_337t", name = "Piranha", text = "", set = "GANGS",
    type = "minion", class = "hunter", cost = 1, attack = 1, health = 1,
    tags = { "beast" },
}}

return card
