local card = {
    api_version = 1,
    id = "AT_113",
    name = "Recruiter",
    text = "<b>Inspire:</b> Add a 2/2 Squire to your hand.",
    set = "TGT",
    type = "minion",
    rarity = "epic",
    cost = 5,
    attack = 5,
    health = 4,
    keywords = { "inspire" },
    on_inspire = function(ctx, self) cardlib.effects.give_card(ctx, ctx:controller(self), "CS2_152") end,
}

card.tokens = {
    {
        id = "CS2_152",
        name = "Squire",
        text = "",
        set = "EXPERT1",
        type = "minion",
        cost = 1,
        attack = 2,
        health = 2,
    },
}

return card
