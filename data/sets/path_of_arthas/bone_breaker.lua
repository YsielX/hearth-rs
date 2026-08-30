local card = {
    api_version = 1,
    id = "RLK_516",
    name = "Bone Breaker",
    text = "[x]After your hero attacks a\nminion, deal 2 damage\nto the enemy hero.",
    set = "PATH_OF_ARTHAS",
    type = "weapon",
    class = "death_knight",
    rarity = "common",
    cost = 1,
    attack = 2,
    health = 2,
    rune_cost = { frost = 1 },
}

card.triggers = {{
    event = "attack",
    timing = "after",
    active_zones = { "weapon", "graveyard" },
    condition = function(ctx, self, event)
        local player = ctx:controller(self)
        return event.attacker == ctx:player(player).hero
            and ctx:entity(event.defender).type == "minion"
    end,
    effect = function(ctx, self)
        local opponent = ctx:opponent(ctx:controller(self))
        cardlib.effects.damage(ctx, ctx:player(opponent).hero, 2)
    end,
}}

return card
