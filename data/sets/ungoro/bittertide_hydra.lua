return {
    api_version = 1, id = "UNG_087", name = "Bittertide Hydra",
    text = "Whenever this minion takes damage, deal 3 damage to your hero.",
    set = "UNGORO", type = "minion", rarity = "epic", cost = 5, attack = 8, health = 8,
    tags = { "beast" },
    triggers = {{
        event = "damaged", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.target == self and event.amount > 0 end,
        effect = function(ctx, self) cardlib.effects.damage(ctx, ctx:player(ctx:controller(self)).hero, 3) end,
    }},
}
