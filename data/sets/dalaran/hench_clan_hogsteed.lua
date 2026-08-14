return {
    api_version = 1,
    id = "DAL_743",
    name = "Hench-Clan Hogsteed",
    text = "<b>Rush</b>\n<b>Deathrattle:</b> Summon a 1/1 Murloc.",
    set = "DALARAN",
    type = "minion",
    cost = 2,
    attack = 2,
    health = 1,
    tags = { "beast" },
    keywords = { "rush", "deathrattle" },
    on_deathrattle = function(ctx, self, position)
        ctx:summon_at(ctx:controller(self), "DAL_743t", position)
    end,
    tokens = {
        {
            id = "DAL_743t",
            name = "Hench-Clan Squire",
            text = "",
            set = "DALARAN",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 1,
            tags = { "murloc" },
        },
    },
}
