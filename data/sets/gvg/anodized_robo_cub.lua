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
        { label = "Attack Mode: +1 Attack", value = 1 },
        { label = "Tank Mode: +1 Health", value = 2 },
    }, "chosen")
end

function card.chosen(ctx, self, choice)
    if choice == 1 then cardlib.effects.buff(ctx, self, 1, 0)
    else cardlib.effects.buff(ctx, self, 0, 1) end
end

function card.on_choose_multiple(ctx, self)
    cardlib.effects.buff(ctx, self, 1, 1)
end

return card
