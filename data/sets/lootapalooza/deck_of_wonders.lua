local scroll = {
    id = "LOOT_106t",
    name = "Scroll of Wonder",
    text = "<b>Casts When Drawn</b>\nCast a random spell.",
    set = "LOOTAPALOOZA",
    type = "spell",
    class = "mage",
    collectible = false,
    cost = 5,
    keywords = { "casts_when_drawn" },
}

local function spell_pool(ctx)
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        if ctx:card_definition(id).type == "spell" and id ~= "LOOT_106" then
            pool[#pool + 1] = id
        end
    end
    return pool
end

function scroll.on_play(ctx, self)
    cardlib.random_spell.choose(
        ctx,
        ctx:controller(self),
        spell_pool(ctx),
        1,
        "scroll_spell_chosen"
    )
end

function scroll.scroll_spell_chosen(ctx, self, choice)
    cardlib.random_spell.cast(ctx, choice)
end

local card = {
    api_version = 1,
    id = "LOOT_106",
    name = "Deck of Wonders",
    text = "Shuffle 5 Scrolls into your deck. When drawn, cast a random spell.",
    set = "LOOTAPALOOZA",
    type = "spell",
    class = "mage",
    rarity = "epic",
    spell_school = "arcane",
    cost = 5,
    tokens = { scroll },
}

function card.on_play(ctx, self)
    for _ = 1, 5 do
        ctx:shuffle_card_into_deck(ctx:controller(self), "LOOT_106t")
    end
end

return card
