local card = {
    api_version = 1,
    id = "GVG_032",
    name = "Grove Tender",
    text = "<b>Choose One -</b> Give each player a Mana Crystal; or Each player draws a card.",
    set = "GVG",
    type = "minion",
    class = "druid",
    rarity = "rare",
    cost = 3,
    attack = 2,
    health = 4,
    keywords = { "choose_one" },
}

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { label = "Give each player a Mana Crystal", value = 1 },
        { label = "Each player draws a card", value = 2 },
    }, "chosen")
end

local function give_mana(ctx, self)
    local player = ctx:controller(self)
    ctx:gain_mana_crystals(player, 1, true)
    ctx:gain_mana_crystals(ctx:opponent(player), 1, true)
end

local function draw_cards(ctx, self)
    local player = ctx:controller(self)
    ctx:draw(player, 1)
    ctx:draw(ctx:opponent(player), 1)
end

function card.chosen(ctx, self, choice)
    if choice == 1 then give_mana(ctx, self)
    else draw_cards(ctx, self) end
end

function card.on_choose_multiple(ctx, self)
    give_mana(ctx, self)
    draw_cards(ctx, self)
end

return card
