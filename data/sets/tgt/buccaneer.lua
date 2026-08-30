return {
    api_version = 1,
    id = "AT_029",
    name = "Buccaneer",
    text = "Whenever you equip a weapon, give it +1 Attack.",
    set = "TGT",
    type = "minion",
    class = "rogue",
    rarity = "common",
    cost = 1,
    attack = 2,
    health = 1,
    tags = { "pirate" },
    triggers = {
        {
            event = "weapon_equipped",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self, event) cardlib.effects.buff(ctx, event.entity, 1, 0) end,
        },
    },
}
