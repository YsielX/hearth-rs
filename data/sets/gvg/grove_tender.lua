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
        { card_id = "GVG_032a", label = "Give each player a Mana Crystal" },
        { card_id = "GVG_032b", label = "Each player draws a card" },
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
    if choice == "GVG_032a" then give_mana(ctx, self)
    else draw_cards(ctx, self) end
end

function card.on_choose_multiple(ctx, self)
    give_mana(ctx, self)
    draw_cards(ctx, self)
end

card.tokens = {
    { id = "GVG_032a", name = "Gift of Mana", text = "Give each player a Mana Crystal.", set = "GVG", type = "spell", class = "druid", collectible = false, cost = 3 },
    { id = "GVG_032b", name = "Gift of Cards", text = "Each player draws a card.", set = "GVG", type = "spell", class = "druid", collectible = false, cost = 3 },
}

return card
