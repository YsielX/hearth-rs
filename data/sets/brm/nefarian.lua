local function opponent_spells(ctx, self)
    local opponent = ctx:opponent(ctx:controller(self))
    local opponent_class = ctx:player(opponent).class
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "spell" and definition.class == opponent_class then
            candidates[#candidates + 1] = card_id
        end
    end
    return candidates
end

local card = {
    api_version = 1,
    id = "BRM_030",
    name = "Nefarian",
    text = "<b>Battlecry:</b> Add 2 random spells to your hand <i>(from your opponent's class)</i>.",
    set = "BRM",
    type = "minion",
    rarity = "legendary",
    cost = 9,
    attack = 8,
    health = 8,
    tags = { "dragon" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    ctx:set_data(self, "spells_added", 0)
    local candidates = opponent_spells(ctx, self)
    if #candidates > 0 then ctx:random_value(candidates, "add_opponent_spell") end
end

function card.add_opponent_spell(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
    local added = ctx:get_data(self, "spells_added") + 1
    ctx:set_data(self, "spells_added", added)
    if added < 2 then
        local candidates = opponent_spells(ctx, self)
        if #candidates > 0 then ctx:random_value(candidates, "add_opponent_spell") end
    end
end

card.tokens = {
    {
        id = "BRM_030t",
        name = "Tail Swipe",
        text = "Deal $4 damage.",
        set = "BRM",
        type = "spell",
        cost = 4,
        target_mode = "required",
        targets = function(ctx) return ctx:characters() end,
        on_play = function(ctx, self, target) cardlib.effects.damage(ctx, target, 4) end,
    },
}

return card
