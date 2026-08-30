return {
    api_version = 1,
    id = "BRM_020",
    name = "Dragonkin Sorcerer",
    text = "Whenever <b>you</b> target this minion with a spell, gain +1/+1.",
    set = "BRM",
    type = "minion",
    rarity = "common",
    cost = 4,
    attack = 3,
    health = 5,
    tags = { "dragon" },
    triggers = {
        {
            event = "spell_targeted",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self) and event.target == self
            end,
            effect = function(ctx, self)
                cardlib.effects.buff(ctx, self, 1, 1)
            end,
        },
    },
}
