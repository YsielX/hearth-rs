local card = {
    api_version = 1, id = "ICC_082", name = "Frozen Clone",
    text = "<b>Secret:</b> After your opponent plays a minion, add two copies of it to your hand.",
    set = "ICECROWN", type = "spell", class = "mage", rarity = "common",
    spell_school = "frost", cost = 3, keywords = { "secret" },
}

card.triggers = {{
    event = "minion_played", timing = "after", active_zones = { "secret" },
    condition = function(ctx, self, event)
        return event.player == ctx:opponent(ctx:controller(self))
    end,
    effect = function(ctx, self, event)
        local player = ctx:controller(self)
        ctx:reveal_secret(self)
        ctx:give_copy(player, event.entity)
        ctx:give_copy(player, event.entity)
    end,
}}

return card
