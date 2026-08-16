local card = {
    api_version = 1, id = "GVG_092", name = "Gnomish Experimenter",
    text = "<b>Battlecry:</b> Draw a card. If it's a minion, transform it into a Chicken.",
    set = "GVG", type = "minion", rarity = "rare", cost = 3, attack = 3, health = 2,
    keywords = { "battlecry" },
    tokens = {{ id = "GVG_092t", name = "Chicken", text = "", set = "GVG", type = "minion", cost = 1, attack = 1, health = 1, tags = { "beast" } }},
}
function card.on_battlecry(ctx, self)
    ctx:set_data(self, "waiting_for_draw", 1)
    ctx:draw(ctx:controller(self), 1)
end
card.triggers = {
    {
        event = "card_drawn", active_zones = { "board" },
        condition = function(ctx, self, event)
            return ctx:get_data(self, "waiting_for_draw") == 1
                and event.player == ctx:controller(self)
        end,
        effect = function(ctx, self, event)
            ctx:set_data(self, "waiting_for_draw", 0)
            if ctx:entity(event.entity).type == "minion" then cardlib.effects.transform(ctx, event.entity, "GVG_092t") end
        end,
    },
    {
        event = "fatigue", active_zones = { "board" },
        condition = function(ctx, self, event)
            return ctx:get_data(self, "waiting_for_draw") == 1
                and event.player == ctx:controller(self)
        end,
        effect = function(ctx, self)
            ctx:set_data(self, "waiting_for_draw", 0)
        end,
    },
}
return card
