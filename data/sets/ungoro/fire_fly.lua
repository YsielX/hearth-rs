return {
    api_version = 1,
    id = "UNG_809",
    name = "Fire Fly",
    text = "<b>Battlecry</b>: Add a 1/2 Elemental to your hand.",
    set = "UNGORO",
    type = "minion",
    rarity = "common",
    cost = 1,
    attack = 1,
    health = 2,
    tags = { "elemental" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        ctx:give_card(ctx:controller(self), "UNG_809t1")
    end,
    tokens = {
        {
            id = "UNG_809t1",
            name = "Flame Elemental",
            text = "",
            set = "UNGORO",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 2,
            tags = { "elemental" },
        },
    },
}
