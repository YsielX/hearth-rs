local function has_deathrattle(definition)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == "deathrattle" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "LOE_006",
    name = "Museum Curator",
    text = "<b>Battlecry: Discover</b> a <b>Deathrattle</b> card.\nIt costs (1) less.",
    set = "LOE",
    type = "minion",
    class = "priest",
    rarity = "common",
    cost = 2,
    attack = 1,
    health = 2,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local player_class = ctx:player(player).class
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if has_deathrattle(definition)
            and (definition.class == "neutral" or definition.class == player_class) then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then
        ctx:discover_cards(player, "Choose a Deathrattle card", pool, 3, "receive_deathrattle_card")
    end
end

function card.receive_deathrattle_card(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

card.triggers = {{
    event = "card_created",
    timing = "after",
    active_zones = { "board", "graveyard" },
    condition = function(ctx, self, event) return event.source == self end,
    effect = function(ctx, self, event)
        ctx:modify(event.entity, { stat = "cost", operation = "add", value = -1 })
    end,
}}

return card
