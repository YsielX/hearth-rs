local card = {
    api_version = 1,
    id = "BRM_010",
    name = "Druid of the Flame",
    text = "<b>Choose One -</b> Transform into a 5/2 minion; or a 2/5 minion.",
    set = "BRM",
    type = "minion",
    class = "druid",
    rarity = "common",
    cost = 3,
    attack = 2,
    health = 2,
    keywords = { "choose_one" },
}

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { label = "Transform into a 5/2 minion", value = 1 },
        { label = "Transform into a 2/5 minion", value = 2 },
    }, "chosen")
end

function card.chosen(ctx, self, choice)
    ctx:transform(self, choice == 1 and "BRM_010t" or "BRM_010t2")
end

function card.on_choose_multiple(ctx, self)
    ctx:transform(self, "OG_044b")
end

card.tokens = {
    {
        id = "BRM_010t", name = "Druid of the Flame", text = "",
        set = "BRM", type = "minion", class = "druid", rarity = "common",
        cost = 3, attack = 5, health = 2, tags = { "elemental", "beast" },
    },
    {
        id = "BRM_010t2", name = "Druid of the Flame", text = "",
        set = "BRM", type = "minion", class = "druid", rarity = "common",
        cost = 3, attack = 2, health = 5, tags = { "elemental", "beast" },
    },
}

return card
