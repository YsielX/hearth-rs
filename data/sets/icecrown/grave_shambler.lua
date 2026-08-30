return {
    api_version = 1,
    id = "ICC_097",
    name = "Grave Shambler",
    text = "Whenever your weapon is destroyed, gain +1/+1.",
    set = "ICECROWN",
    type = "minion",
    rarity = "common",
    cost = 4,
    attack = 4,
    health = 4,
    tags = { "undead", "elemental" },
    triggers = {
        {
            event = "weapon_destroyed",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                cardlib.effects.buff(ctx, self, 1, 1)
            end,
        },
    },
}
