local card = {
    api_version = 1,
    id = "RLK_012",
    name = "Soulbreaker",
    text = "After your hero attacks and kills a minion, gain 2 <b>Corpses</b>.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "weapon",
    class = "death_knight",
    rarity = "common",
    cost = 3,
    attack = 3,
    health = 2,
    rune_cost = { blood = 1 },
}

card.triggers = {{
    event = "attack",
    timing = "after",
    active_zones = { "weapon" },
    condition = function(ctx, self, event)
        local player = ctx:controller(self)
        local defender = ctx:entity(event.defender)
        return event.attacker == ctx:player(player).hero
            and defender.type == "minion"
            and defender.health <= 0
    end,
    effect = function(ctx, self)
        ctx:gain_corpses(ctx:controller(self), 2)
    end,
}}

return card
