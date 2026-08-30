return {
    api_version = 1,
    id = "FP1_002", rarity = "common",
    name = "Haunted Creeper",
    text = "<b>Deathrattle:</b> Summon two 1/1 Spectral Spiders.",
    set = "NAXX",
    type = "minion",
    cost = 2,
    attack = 1,
    health = 2,
    tags = { "beast" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self, position)
        local player = ctx:controller(self)
        ctx:summon_at(player, "FP1_002t", position)
        ctx:summon_at(player, "FP1_002t", position)
    end,
    tokens = {
        {
            id = "FP1_002t",
            name = "Spectral Spider",
            text = "",
            set = "NAXX",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 1,
            tags = { "undead", "beast" },
        },
    },
}
