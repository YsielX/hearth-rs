local function legendary_minions(ctx)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.rarity == "legendary" then
            result[#result + 1] = card_id
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "LOE_079",
    name = "Elise Starseeker",
    text = "<b>Battlecry:</b> Shuffle the 'Map to the Golden Monkey'   into your deck.",
    set = "LOE",
    type = "minion",
    rarity = "legendary",
    cost = 4,
    attack = 3,
    health = 5,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    cardlib.effects.shuffle_card_into_deck(ctx, ctx:controller(self), "LOE_019t")
end

local golden_monkey = {
    id = "LOE_019t2",
    name = "Golden Monkey",
    text = "<b>Taunt</b>\n<b>Battlecry:</b> Replace your hand and deck with <b>Legendary</b> minions.",
    set = "LOE",
    type = "minion",
    cost = 4,
    attack = 6,
    health = 6,
    keywords = { "taunt", "battlecry" },
}

function golden_monkey.on_battlecry(ctx, self)
    ctx:set_data(self, "replace_zone", 1)
    ctx:set_data(self, "replace_index", 1)
    ctx:continue_with("replace_next_card")
end

function golden_monkey.replace_next_card(ctx, self)
    local player = ctx:controller(self)
    local zone = ctx:get_data(self, "replace_zone") or 1
    local index = ctx:get_data(self, "replace_index") or 1
    local entities = zone == 1 and ctx:hand(player) or ctx:deck(player)

    if index > #entities then
        if zone == 1 then
            ctx:set_data(self, "replace_zone", 2)
            ctx:set_data(self, "replace_index", 1)
            ctx:continue_with("replace_next_card")
        end
        return
    end

    ctx:set_data(self, "replace_target", entities[index])
    local candidates = legendary_minions(ctx)
    if #candidates > 0 then
        ctx:random_value(candidates, "replace_with_legendary")
    end
end

function golden_monkey.replace_with_legendary(ctx, self, card_id)
    local target = ctx:get_data(self, "replace_target")
    if target then cardlib.effects.transform(ctx, target, card_id) end
    local index = ctx:get_data(self, "replace_index") or 1
    ctx:set_data(self, "replace_index", index + 1)
    ctx:continue_with("replace_next_card")
end

card.tokens = {
    {
        id = "LOE_019t",
        name = "Map to the Golden Monkey",
        text = "Shuffle the Golden Monkey into your deck. Draw a card.",
        set = "LOE",
        type = "spell",
        cost = 2,
        on_play = function(ctx, self)
            local player = ctx:controller(self)
            cardlib.effects.shuffle_card_into_deck(ctx, player, "LOE_019t2")
            ctx:draw(player, 1)
        end,
    },
    golden_monkey,
}

return card
