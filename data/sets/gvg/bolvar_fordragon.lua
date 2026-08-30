return {
    api_version = 1,
    id = "GVG_063",
    name = "Bolvar Fordragon",
    text = "Whenever a friendly minion dies while this is in your hand, gain +1 Attack.",
    set = "GVG",
    type = "minion",
    class = "paladin",
    rarity = "legendary",
    cost = 5,
    attack = 1,
    health = 7,
    triggers = {
        {
            event = "entity_died",
            active_zones = { "hand" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and ctx:entity(event.entity).type == "minion"
            end,
            effect = function(ctx, self)
                cardlib.effects.buff(ctx, self, 1, 0)
            end,
        },
    },
}
