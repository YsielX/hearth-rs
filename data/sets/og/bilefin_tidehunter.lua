return {
    api_version = 1,
    id = "OG_156",
    name = "Bilefin Tidehunter",
    text = "<b>Battlecry:</b> Summon a 1/1 Ooze with <b>Taunt</b>.",
    set = "OG",
    type = "minion",
    rarity = "common",
    cost = 2,
    attack = 2,
    health = 1,
    tags = { "murloc" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        ctx:summon_at(
            ctx:controller(self),
            "OG_156a",
            ctx:board_position(self) + 1
        )
    end,
    tokens = {
        {
            id = "OG_156a",
            name = "Ooze",
            text = "<b>Taunt</b>",
            set = "OG",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 1,
            keywords = { "taunt" },
        },
    },
}
