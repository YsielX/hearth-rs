local function has_keyword(definition, wanted)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == wanted then return true end
    end
    return false
end

local function eligible_for_player(ctx, player, definition)
    local classes = definition.classes or {}
    if #classes > 0 then
        for _, class in ipairs(classes) do
            if class == ctx:player(player).class then return true end
        end
        return false
    end
    return definition.class == "neutral" or definition.class == ctx:player(player).class
end

local card = {
    api_version = 1,
    id = "ICC_702",
    name = "Shallow Gravedigger",
    text = "<b>Deathrattle:</b> Add a random <b>Deathrattle</b> minion to your hand.",
    set = "ICECROWN",
    type = "minion",
    rarity = "rare",
    cost = 3,
    attack = 3,
    health = 1,
    tags = { "undead" },
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local player = ctx:controller(self)
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion"
            and has_keyword(definition, "deathrattle")
            and eligible_for_player(ctx, player, definition)
        then
            candidates[#candidates + 1] = card_id
        end
    end
    if #candidates > 0 then ctx:random_value(candidates, "receive_random_deathrattle_minion") end
end

function card.receive_random_deathrattle_minion(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

return card
