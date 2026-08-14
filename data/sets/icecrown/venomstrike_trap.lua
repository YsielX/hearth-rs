local card = {
    api_version = 1, id = "ICC_200", name = "Venomstrike Trap",
    text = "<b>Secret:</b> When one of your minions is attacked, summon a 2/3 <b>Poisonous</b> Cobra.",
    set = "ICECROWN", type = "spell", class = "hunter", rarity = "rare", cost = 2,
    keywords = { "secret" },
}

card.triggers = {{
    event = "attack", timing = "before", active_zones = { "secret" },
    condition = function(ctx, self, event)
        local player = ctx:controller(self)
        return ctx:entity(event.defender).type == "minion"
            and ctx:controller(event.defender) == player
            and #ctx:board(player) < 7
    end,
    effect = function(ctx, self)
        local player = ctx:controller(self)
        ctx:reveal_secret(self)
        ctx:summon(player, "EX1_170")
    end,
}}

return card
