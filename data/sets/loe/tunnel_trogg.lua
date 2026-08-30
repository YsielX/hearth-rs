return {
    api_version = 1,
    id = "LOE_018",
    name = "Tunnel Trogg",
    text = "Whenever you <b>Overload</b>, gain +1 Attack per locked Mana Crystal.",
    set = "LOE",
    type = "minion",
    class = "shaman",
    rarity = "common",
    cost = 1,
    attack = 1,
    health = 3,
    triggers = {
        {
            event = "overload_queued",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self) and event.amount > 0
            end,
            effect = function(ctx, self, event)
                cardlib.effects.buff(ctx, self, event.amount, 0)
            end,
        },
    },
}
