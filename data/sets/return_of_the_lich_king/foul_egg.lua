return {
    api_version = 1,
    id = "RLK_833", rarity = "common",
    name = "Foul Egg",
    text = "<b>Deathrattle:</b> Summon a 3/3 Undead Chicken.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    cost = 1,
    attack = 0,
    health = 2,
    tags = { "undead" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self, position)
        ctx:summon_at(ctx:controller(self), "RLK_833t", position)
    end,
    tokens = {
        {
            id = "RLK_833t",
            name = "Foul Fowl",
            text = "",
            set = "RETURN_OF_THE_LICH_KING",
            type = "minion",
            cost = 3,
            attack = 3,
            health = 3,
            tags = { "undead", "beast" },
        },
    },
}
