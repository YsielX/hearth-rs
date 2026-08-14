local card = {
    api_version = 1, id = "AT_060", name = "Bear Trap",
    text = "<b>Secret:</b> After your hero is attacked, summon a 3/3 Bear with <b>Taunt</b>.",
    set = "TGT", type = "spell", class = "hunter", rarity = "common",
    cost = 2, keywords = { "secret" },
}

card.triggers = {
    {
        event = "attack", timing = "after", active_zones = { "secret" },
        condition = function(ctx, self, event)
            local player = ctx:controller(self)
            return event.defender == ctx:player(player).hero and #ctx:board(player) < 7
        end,
        effect = function(ctx, self)
            ctx:reveal_secret(self)
            ctx:summon(ctx:controller(self), "CS2_125")
        end,
    },
}

return card
