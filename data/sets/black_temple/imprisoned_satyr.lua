local card = {
    api_version = 1,
    id = "BT_127",
    name = "Imprisoned Satyr",
    text = "[x]<b>Dormant</b> for 2 turns.\nWhen this awakens, reduce\nthe Cost of a random minion\nin your hand by (5).",
    set = "BLACK_TEMPLE",
    type = "minion",
    class = "druid",
    cost = 3,
    attack = 3,
    health = 3,
    tags = { "demon" },
    keywords = { "dormant" },
}

card.triggers = {
    {
        event = "turn_started", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
                and ctx:get_data(self, "awakened") ~= 1
        end,
        effect = function(ctx, self, event)
            local turns = (ctx:get_data(self, "dormant_turns") or 0) + 1
            ctx:set_data(self, "dormant_turns", turns)
            if turns == 2 then ctx:continue_with("awaken") end
        end,
    },
}

function card.awaken(ctx, self)
    ctx:set_data(self, "awakened", 1)
    ctx:disable_keyword(self, "dormant")
    local candidates = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "reduce_minion") end
end

function card.reduce_minion(ctx, self, target)
    ctx:modify(target, { stat = "cost", operation = "add", value = -5 })
end

return card
