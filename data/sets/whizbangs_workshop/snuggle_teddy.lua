local card = {
    api_version = 1,
    id = "MIS_300",
    name = "Snuggle Teddy",
    text = "<b>Gigantify</b>\n<b>Elusive</b>, <b>Lifesteal</b>, <b>Taunt</b>",
    set = "WHIZBANGS_WORKSHOP",
    type = "minion",
    class = "druid",
    cost = 3,
    attack = 2,
    health = 4,
    tags = { "beast" },
    keywords = { "gigantify", "elusive", "lifesteal", "taunt" },
}

function card.on_gigantify(ctx, self)
    ctx:give_card(ctx:controller(self), "MIS_300t")
end

card.tokens = {
    {
        id = "MIS_300t", name = "Snuggle Teddy",
        text = "<b>Gigantic</b>\n<b>Elusive</b>, <b>Lifesteal</b>, <b>Taunt</b>",
        set = "WHIZBANGS_WORKSHOP", type = "minion", class = "druid",
        cost = 8, attack = 8, health = 8, tags = { "beast" },
        keywords = { "elusive", "lifesteal", "taunt" },
    },
}

return card
