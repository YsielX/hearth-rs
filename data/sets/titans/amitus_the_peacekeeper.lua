local card = {
    api_version = 1,
    id = "TTN_858",
    name = "Amitus, the Peacekeeper",
    text = "<b>Titan</b>\n<b>Taunt</b>. Your minions\ncan't take more than 2 damage at a time.",
    set = "TITANS",
    type = "minion",
    class = "paladin",
    rarity = "legendary",
    cost = 7,
    attack = 1,
    health = 8,
    keywords = { "titan", "taunt" },
    triggers = {
        {
            event = "damaged",
            timing = "before",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                local target = ctx:entity(event.target)
                return target.type == "minion"
                    and target.controller == ctx:controller(self)
                    and event.amount > 2
            end,
            effect = function(ctx, self, event)
                cardlib.effects.set_event_amount(ctx, event, 2)
            end,
        },
    },
}

local function set_to_two(ctx, entity)
    cardlib.effects.modify(ctx, entity, { stat = "attack", operation = "set", value = 2 })
    cardlib.effects.modify(ctx, entity, { stat = "health", operation = "set", value = 2 })
end

card.action_effects = {
    titan_1 = function(ctx, self)
        local player = ctx:controller(self)
        local selected = {}
        for _, entity in ipairs(ctx:deck(player)) do
            if ctx:entity(entity).type == "minion" then
                selected[#selected + 1] = entity
                if #selected == 2 then break end
            end
        end
        for index = #selected, 1, -1 do
            ctx:move(selected[index], "deck_top")
        end
        if #selected > 0 then ctx:draw(player, #selected) end
        for _, entity in ipairs(selected) do
            set_to_two(ctx, entity)
            cardlib.effects.modify(ctx, entity, { stat = "cost", operation = "set", value = 2 })
        end
    end,
    titan_2 = function(ctx, self)
        for _, entity in ipairs(ctx:friendly_minions(self)) do
            if entity ~= self then ctx:buff(entity, 2, 2) end
        end
    end,
    titan_3 = function(ctx, self)
        local enemy = ctx:opponent(ctx:controller(self))
        for _, entity in ipairs(ctx:board(enemy)) do
            if ctx:entity(entity).type == "minion" then set_to_two(ctx, entity) end
        end
    end,
}

return card
