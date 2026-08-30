return {
    api_version = 1,
    id = "RLK_503",
    name = "Body Bagger",
    text = "<b>Battlecry</b>: Gain a <b>Corpse</b>.",
    set = "CORE",
    type = "minion",
    class = "death_knight",
    rarity = "common",
    cost = 1,
    attack = 1,
    health = 3,
    tags = { "undead" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        ctx:gain_corpses(ctx:controller(self), 1)
    end,
}
