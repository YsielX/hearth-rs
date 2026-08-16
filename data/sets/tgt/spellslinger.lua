local card = {
    api_version = 1, id = "AT_007", name = "Spellslinger",
    text = "<b>Battlecry:</b> Both players\nget a random spell.\nYours costs (2) less.",
    set = "TGT", type = "minion", class = "mage", rarity = "common",
    cost = 3, attack = 3, health = 4, keywords = { "battlecry" },
}

local function spell_pool(ctx)
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        if definition.type == "spell" then
            pool[#pool + 1] = id
        end
    end
    return pool
end

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local own = spell_pool(ctx)
    if #own > 0 then ctx:random_value(own, "give_own_spell") end
    local opponent = ctx:opponent(player)
    local enemy = spell_pool(ctx)
    if #enemy > 0 then ctx:random_value(enemy, "give_enemy_spell") end
end

function card.give_own_spell(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

function card.give_enemy_spell(ctx, self, card_id)
    ctx:give_card(ctx:opponent(ctx:controller(self)), card_id)
end

card.triggers = {
    {
        event = "card_created", timing = "after",
        active_zones = { "board", "graveyard" },
        condition = function(ctx, self, event)
            return event.source == self and event.player == ctx:controller(self)
        end,
        effect = function(ctx, self, event)
            cardlib.effects.modify(ctx, event.entity, { stat = "cost", operation = "add", value = -2 })
        end,
    },
}

return card
