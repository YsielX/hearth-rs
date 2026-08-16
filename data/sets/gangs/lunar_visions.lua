local card = {
    api_version = 1,
    id = "CFM_811",
    name = "Lunar Visions",
    text = "Draw 2 cards. Minions drawn cost (2) less.",
    set = "GANGS",
    type = "spell",
    class = "druid",
    rarity = "epic",
    spell_school = "arcane",
    cost = 5,
}

function card.on_play(ctx, self)
    ctx:set_data(self, "draws_remaining", 2)
    ctx:draw(ctx:controller(self), 2)
end

card.triggers = {
    {
        event = "card_drawn", timing = "after", active_zones = { "graveyard" },
        condition = function(ctx, self, event)
            return event.source == self and event.player == ctx:controller(self)
                and ctx:get_data(self, "draws_remaining") > 0
        end,
        effect = function(ctx, self, event)
            ctx:set_data(self, "draws_remaining", ctx:get_data(self, "draws_remaining") - 1)
            if ctx:entity(event.entity).type == "minion" then
                cardlib.effects.modify(ctx, event.entity, { stat = "cost", operation = "add", value = -2 })
            end
        end,
    },
    {
        event = "fatigue", timing = "after", active_zones = { "graveyard" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and ctx:get_data(self, "draws_remaining") > 0
        end,
        effect = function(ctx, self)
            ctx:set_data(self, "draws_remaining", ctx:get_data(self, "draws_remaining") - 1)
        end,
    },
}

return card
