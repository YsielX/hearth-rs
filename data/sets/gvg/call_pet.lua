local function has_tag(definition, wanted)
    for _, tag in ipairs(definition.tags) do
        if tag == wanted then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "GVG_017",
    name = "Call Pet",
    text = "Draw a card.\nIf it's a Beast, it costs (4) less.",
    set = "GVG",
    type = "spell",
    class = "hunter",
    rarity = "rare",
    cost = 2,
}

function card.on_play(ctx, self)
    ctx:set_data(self, "awaiting_draw", 1)
    ctx:draw(ctx:controller(self), 1)
end

card.triggers = {
    {
        event = "card_drawn",
        timing = "after",
        active_zones = { "graveyard" },
        condition = function(ctx, self, event)
            return ctx:get_data(self, "awaiting_draw") == 1
                and event.player == ctx:controller(self)
        end,
        effect = function(ctx, self, event)
            ctx:set_data(self, "awaiting_draw", 0)
            local definition = ctx:card_definition(ctx:entity(event.entity).card_id)
            if has_tag(definition, "beast") then
                ctx:modify(event.entity, { stat = "cost", operation = "add", value = -4 })
            end
        end,
    },
    {
        event = "fatigue",
        timing = "after",
        active_zones = { "graveyard" },
        condition = function(ctx, self, event)
            return ctx:get_data(self, "awaiting_draw") == 1
                and event.player == ctx:controller(self)
        end,
        effect = function(ctx, self)
            ctx:set_data(self, "awaiting_draw", 0)
        end,
    },
}

return card
