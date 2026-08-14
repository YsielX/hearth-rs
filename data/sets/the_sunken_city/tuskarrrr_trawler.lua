local card = {
    api_version = 1,
    id = "TSC_909",
    name = "Tuskarrrr Trawler",
    text = "<b>Battlecry:</b> <b>Dredge</b>.",
    set = "THE_SUNKEN_CITY",
    type = "minion",
    class = "neutral",
    cost = 2,
    attack = 2,
    health = 3,
    tags = { "pirate" },
    keywords = { "battlecry", "dredge" },
}

function card.on_battlecry(ctx, self) end

function card.on_dredge(ctx, self)
    local deck = ctx:deck(ctx:controller(self))
    local bottom = {}
    for index = math.max(1, #deck - 2), #deck do
        bottom[#bottom + 1] = deck[index]
    end
    if #bottom > 0 then
        ctx:choose_entities(ctx:controller(self), "Dredge", bottom, "dredged")
    end
end

function card.dredged(ctx, self, chosen) ctx:move(chosen, "deck_top") end

return card
