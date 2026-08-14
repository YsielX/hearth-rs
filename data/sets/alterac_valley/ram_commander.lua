return {
    api_version = 1,
    id = "AV_219",
    name = "Ram Commander",
    text = "[x]<b>Battlecry:</b> Add two\n1/1 Rams with <b>Rush</b>\nto your hand.",
    set = "ALTERAC_VALLEY",
    type = "minion",
    cost = 2,
    attack = 2,
    health = 2,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        ctx:give_card(player, "AV_219t")
        ctx:give_card(player, "AV_219t")
    end,
    tokens = {
        {
            id = "AV_219t",
            name = "Battle Ram",
            text = "<b>Rush</b>",
            set = "ALTERAC_VALLEY",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 1,
            tags = { "beast" },
            keywords = { "rush" },
        },
    },
}
