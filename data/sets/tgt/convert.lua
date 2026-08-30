local card = {
    api_version = 1,
    id = "AT_015",
    name = "Convert",
    text = "Put a copy of an enemy minion into your hand. It costs (1).",
    set = "TGT",
    type = "spell",
    class = "priest",
    rarity = "rare",
    cost = 3,
    target_mode = "required",
    targets = function(ctx, self)
        local result = {}
        for _, entity in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
        end
        return result
    end,
    triggers = {
        {
            event = "card_created",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return event.source == self and ctx:get_data(self, "waiting_for_copy") == 1
            end,
            effect = function(ctx, self, event)
                ctx:set_data(self, "waiting_for_copy", 0)
                cardlib.effects.modify(ctx, event.entity, { stat = "cost", operation = "set", value = 1 })
            end,
        },
    },
}

function card.on_play(ctx, self, target)
    ctx:set_data(self, "waiting_for_copy", 1)
    cardlib.effects.give_card(ctx, ctx:controller(self), ctx:entity(target).card_id)
end

return card
