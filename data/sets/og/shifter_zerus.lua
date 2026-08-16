local function minion_pool(ctx)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        if ctx:card_definition(card_id).type == "minion" then
            result[#result + 1] = card_id
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "OG_123",
    name = "Shifter Zerus",
    text = "Each turn this is in your hand, transform it into a random minion.",
    set = "OG",
    type = "minion",
    rarity = "legendary",
    cost = 1,
    attack = 1,
    health = 1,
    triggers = {{
        event = "turn_started",
        timing = "after",
        active_zones = { "hand" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
        end,
        effect = function(ctx, self)
            local pool = minion_pool(ctx)
            if #pool > 0 then
                ctx:attach_script(self, "OG_123")
                ctx:random_value(pool, "shift_into_minion")
            end
        end,
    }},
}

function card.shift_into_minion(ctx, self, card_id)
    cardlib.effects.transform_preserving_scripts(ctx, self, card_id)
end

return card
