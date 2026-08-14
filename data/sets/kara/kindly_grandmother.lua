return {
    api_version = 1,
    id = "KAR_005",
    name = "Kindly Grandmother",
    text = "<b>Deathrattle:</b> Summon a 3/2 Big Bad Wolf.",
    set = "KARA",
    type = "minion",
    class = "hunter",
    cost = 2,
    attack = 1,
    health = 1,
    tags = { "beast" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self, position)
        ctx:summon_at(ctx:controller(self), "KAR_005a", position)
    end,
    tokens = {
        {
            id = "KAR_005a",
            name = "Big Bad Wolf",
            text = "",
            set = "KARA",
            type = "minion",
            class = "hunter",
            cost = 2,
            attack = 3,
            health = 2,
            tags = { "beast" },
        },
    },
}
