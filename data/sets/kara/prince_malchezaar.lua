local card = {
    api_version = 1,
    id = "KAR_096",
    name = "Prince Malchezaar",
    text = "[x]<b>Start of Game:</b>\nAdd 5 extra <b>Legendary</b>\nminions to your deck.",
    set = "KARA",
    type = "minion",
    rarity = "legendary",
    class = "neutral",
    tags = { "demon" },
    cost = 5,
    attack = 5,
    health = 6,
    keywords = { "start_of_game" },
}

local function choose_legendary(ctx, self, just_selected)
    local candidates = {}
    local own_class = ctx:player(ctx:controller(self)).class
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion"
            and definition.rarity == "legendary"
            and (definition.class == "neutral" or definition.class == own_class)
            and card_id ~= "KAR_096"
            and card_id ~= just_selected
            and ctx:get_data(self, "excluded:" .. card_id) == 0 then
            candidates[#candidates + 1] = card_id
        end
    end
    if #candidates > 0 then ctx:random_value(candidates, "on_legendary") end
end

function card.on_start_of_game(ctx, self)
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        local card_id = ctx:entity(entity).card_id
        local definition = ctx:card_definition(card_id)
        if definition.rarity == "legendary" then
            ctx:set_data(self, "excluded:" .. card_id, 1)
        end
    end
    ctx:set_data(self, "added", 0)
    ctx:continue_with("begin_legendary_generation")
end

function card.begin_legendary_generation(ctx, self)
    choose_legendary(ctx, self)
end

function card.on_legendary(ctx, self, card_id)
    ctx:set_data(self, "excluded:" .. card_id, 1)
    ctx:shuffle_card_into_deck(ctx:controller(self), card_id)
    local added = ctx:get_data(self, "added") + 1
    ctx:set_data(self, "added", added)
    if added < 5 then choose_legendary(ctx, self, card_id) end
end

return card
