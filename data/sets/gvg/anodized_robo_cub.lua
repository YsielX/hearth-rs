local card = {
    api_version = 1,
    id = "GVG_030",
    name = "Anodized Robo Cub",
    text = "<b>Taunt</b>. <b>Choose One -</b>\n+1 Attack; or +1 Health.",
    set = "GVG",
    type = "minion",
    class = "druid",
    rarity = "common",
    cost = 2,
    attack = 2,
    health = 2,
    tags = { "mech", "beast" },
    keywords = { "taunt", "choose_one" },
}

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { card_id = "GVG_030a", label = "Attack Mode: +1 Attack" },
        { card_id = "GVG_030b", label = "Tank Mode: +1 Health" },
    }, "chosen")
end

function card.chosen(ctx, self, choice)
    if choice == "GVG_030a" then cardlib.effects.buff(ctx, self, 1, 0)
    else cardlib.effects.buff(ctx, self, 0, 1) end
end

function card.on_choose_multiple(ctx, self)
    cardlib.effects.buff(ctx, self, 1, 1)
end

card.tokens = {
    { id = "GVG_030a", name = "Attack Mode", text = "+1 Attack.", set = "GVG", type = "spell", class = "druid", collectible = false, cost = 2 },
    { id = "GVG_030b", name = "Tank Mode", text = "+1 Health.", set = "GVG", type = "spell", class = "druid", collectible = false, cost = 2 },
}

return card
