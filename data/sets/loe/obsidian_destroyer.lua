local card = {
    api_version = 1,
    id = "LOE_009",
    name = "Obsidian Destroyer",
    text = "At the end of your turn, summon a 1/1 Scarab with <b>Taunt</b>.",
    set = "LOE",
    type = "minion",
    class = "warrior",
    rarity = "common",
    cost = 7,
    attack = 7,
    health = 7,
    triggers = {
        {
            event = "turn_ended",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                ctx:summon(ctx:controller(self), "LOE_009t")
            end,
        },
    },
}

card.tokens = {
    {
        id = "LOE_009t",
        name = "Scarab",
        text = "<b>Taunt</b>",
        set = "LOE",
        type = "minion",
        class = "warrior",
        cost = 1,
        attack = 1,
        health = 1,
        tags = { "beast" },
        keywords = { "taunt" },
    },
}

return card
